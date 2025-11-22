//! # SEMBR (Sentence Break) - Markdown文章整形ツール
//!
//! このツールは、Markdownファイルの文章を適切に分割・結合し、
//! 読みやすさとGit差分の品質を向上させます。
//!
//! ## 主な機能
//! 1. 長い文を句読点（. ! ? ;）で分割
//! 2. 短い文を結合して行長を最適化
//! 3. コードブロックやテーブルを保護
//!
//! ## 使用方法
//! ```bash
//! sembr <path> [--overwrite] [--show-diff] [--line-length-limit 100]
//! ```
//!
//! ## オプション
//! - `--overwrite`: ファイルを直接変更
//! - `--show-diff`: 変更内容を diff 形式で表示
//! - `--line-length-limit`: 行長制限（デフォルト: 100）

use std::path::PathBuf;
use std::sync::LazyLock;
use std::{fs, process};

use anyhow::Result;
use clap::Parser;
use ignore::Walk;
use imara_diff::{Algorithm, BasicLineDiffPrinter, Diff, InternedInput, UnifiedDiffConfig};
use regex::Regex;

/// コマンドライン引数の定義
///
/// clap クレートを使用して、コマンドライン引数を自動的にパースします。
///
/// # フィールド
/// - `path`: チェック対象のファイルまたはディレクトリ
/// - `overwrite`: 適合しないファイルを自動修正するかどうか
/// - `line_length_limit`: 行を結合する際の最大長
/// - `show_diff`: 変更内容をdiff形式で表示するかどうか
#[derive(Parser)]
struct Cli {
    /// チェック対象のファイルまたはディレクトリのパス
    ///
    /// ファイルを指定した場合はそのファイルのみ処理。
    /// ディレクトリを指定した場合は再帰的に .md ファイルを検索。
    path: PathBuf,

    /// 適合しないファイルを自動的に修正する
    ///
    /// このフラグが指定されている場合、ルール違反のファイルを
    /// 自動的に修正して上書き保存します。
    #[arg(long)]
    overwrite: bool,

    /// 行を結合する際の最大行長
    ///
    /// この長さ以下の場合、次の行と結合を試みます。
    /// デフォルトは100文字。
    #[arg(long, default_value_t = 100)]
    line_length_limit: usize,

    /// 変更内容をdiff形式で表示する
    ///
    /// このフラグが指定されている場合、変更前後の差分を
    /// Unified Diff 形式で表示します。
    #[arg(long)]
    show_diff: bool,
}

/// 文末の句読点パターンにマッチする正規表現
///
/// 行末が以下の文字で終わる場合、その行で文が終わっていると判断します：
/// - `.` (ピリオド)
/// - `?` (疑問符)
/// - `;` (セミコロン)
/// - `!` (感嘆符)
/// - `,` (カンマ)
/// - `-` (ハイフン)
///
/// LazyLock により、初回アクセス時に一度だけコンパイルされます。
static REGEX_IGNORE_END: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\.|\?|;|!|,|\-)$").unwrap());

/// リンクターゲット定義を検出する正規表現
///
/// Markdown のリンク参照定義（`[label]: URL`）を検出します。
/// この形式の行は文章ではないため、分割・結合の対象外とします。
///
/// # マッチ例
/// ```markdown
/// [RFC 2119]: https://www.rfc-editor.org/rfc/rfc2119
/// [another link]: /path/to/doc
/// ```
static REGEX_IGNORE_LINK_TARGETS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\[.+\]: ").unwrap());

/// 文の分割位置を検出する正規表現
///
/// この正規表現は、文を分割すべき位置（句読点の後）を検出します。
///
/// # パターンの詳細
/// - `[^\.\d\-\*]\.`: ドット以外 + ドット（小数点や省略記号を除外）
/// - `[^r]\?`: r以外 + ?（r? はGitコマンドなので除外）
/// - `;`: セミコロン
/// - `!`: 感嘆符
/// - `\s`: 後続の空白
///
/// # 除外されるケース
/// - 小数点（1.5）
/// - 省略記号（...）
/// - リスト記号（- や *）
/// - Gitコマンド（r? @reviewer）
static REGEX_SPLIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([^\.\d\-\*]\.|[^r]\?|;|!)\s").unwrap());

/// リスト項目を検出する正規表現
///
/// Markdown のリスト項目（番号付き/記号）を検出します。
///
/// # マッチするパターン
/// - 番号付きリスト: `1. `, `2. `, ...
/// - ダッシュリスト: `- `
/// - アスタリスクリスト: `* `
///
/// インデントがあっても正しく検出します。
///
/// # マッチ例
/// ```markdown
/// 1. First item
///   - Nested item
///     * More nested
/// ```
static REGEX_LIST_ENTRY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(\d\.|\-|\*)\s+").unwrap());

/// メイン関数
///
/// コマンドライン引数を解析し、指定されたパスの
/// Markdownファイルを処理します。
///
/// # 処理の流れ
/// 1. コマンドライン引数をパース
/// 2. ファイル/ディレクトリを走査
/// 3. 各 .md ファイルを処理
/// 4. 結果を集計して表示
/// 5. 非適合ファイルがあれば終了コード1で終了
///
/// # 終了コード
/// - 0: 全てのファイルが適合（または修正完了）
/// - 1: 非適合ファイルが存在
fn main() -> Result<()> {
    // コマンドライン引数をパース
    let cli = Cli::parse();

    // 結果を格納するベクタ
    // 適合しているファイル
    let mut compliant = Vec::new();
    // 適合していないファイル
    let mut not_compliant = Vec::new();
    // 自動修正したファイル
    let mut made_compliant = Vec::new();

    // ignore クレートを使用してファイルを走査
    // .gitignore などの設定を自動的に尊重
    for result in Walk::new(cli.path) {
        let entry = result?;

        // ディレクトリはスキップ
        if entry.file_type().expect("no stdin").is_dir() {
            continue;
        }

        // パスを取得
        let path = entry.into_path();

        // 拡張子をチェック
        if let Some(extension) = path.extension() {
            // .md ファイルのみ処理
            if extension != "md" {
                continue;
            }

            // ファイルを読み込み
            let old = fs::read_to_string(&path)?;

            // 文章を整形
            // 1. comply: 文を分割
            // 2. lengthen_lines: 短い文を結合
            let new = lengthen_lines(&comply(&old), cli.line_length_limit);

            // 元の内容と比較
            if new == old {
                // 変更なし = 適合している
                compliant.push(path.clone());
            } else if cli.overwrite {
                // 自動修正モード: ファイルを上書き
                fs::write(&path, new)?;
                made_compliant.push(path.clone());
            } else if cli.show_diff {
                // diff表示モード: 変更内容を表示
                println!("{}:", path.display());
                show_diff(&old, &new);
                println!("---");
            } else {
                // デフォルト: 非適合として記録
                not_compliant.push(path.clone());
            }
        }
    }

    // 結果を表示
    if !compliant.is_empty() {
        display("compliant", &compliant);
    }
    if !made_compliant.is_empty() {
        display("made compliant", &made_compliant);
    }
    if !not_compliant.is_empty() {
        // 非適合ファイルがある場合、表示して終了コード1で終了
        display("not compliant", &not_compliant);
        process::exit(1);
    }

    Ok(())
}

/// 2つのテキストの差分を表示
///
/// Unified Diff 形式で差分を表示します。
/// Gitなどで使われる標準的な差分表示形式です。
///
/// # 引数
/// - `old`: 変更前のテキスト
/// - `new`: 変更後のテキスト
///
/// # 出力例
/// ```diff
/// @@ -1,3 +1,2 @@
/// -old line 1. old line 2.
/// +old line 1.
/// +old line 2.
/// ```
fn show_diff(old: &str, new: &str) {
    // 文字列をインターン（効率的な比較のため）
    let input = InternedInput::new(old, new);

    // Histogram アルゴリズムで差分を計算
    // これは Git のデフォルトアルゴリズムで、人間が読みやすい差分を生成
    let mut diff = Diff::compute(Algorithm::Histogram, &input);

    // 行単位の差分処理を追加
    diff.postprocess_lines(&input);

    // Unified Diff 形式に変換
    let diff = diff
        .unified_diff(&BasicLineDiffPrinter(&input.interner), UnifiedDiffConfig::default(), &input)
        .to_string();

    // 差分を出力
    print!("{diff}");
}

/// ファイルリストを整形して表示
///
/// 指定されたヘッダーとファイルパスのリストを表示します。
///
/// # 引数
/// - `header`: セクションのヘッダー文字列
/// - `paths`: ファイルパスのスライス
///
/// # 出力例
/// ```text
/// compliant:
/// - path/to/file1.md
/// - path/to/file2.md
/// ```
fn display(header: &str, paths: &[PathBuf]) {
    println!("{header}:");
    for element in paths {
        println!("- {}", element.display());
    }
}

/// 行を無視すべきかどうかを判定
///
/// 以下の条件のいずれかに該当する行は、
/// 文の分割・結合の対象外とします。
///
/// # 無視する行
/// - コードブロック内（\`\`\`で囲まれた部分）
/// - "e.g." を含む行（例示の略語）
/// - "i.e." を含む行（言い換えの略語）
/// - パイプ文字を含む行（テーブル）
/// - 引用ブロック（> で始まる行）
/// - 見出し（# で始まる行）
/// - 空行
/// - リンクターゲット定義（[label]: URL）
///
/// # 引数
/// - `line`: チェック対象の行
/// - `in_code_block`: 現在コードブロック内かどうか
///
/// # 戻り値
/// 無視すべき場合は `true`
///
/// # 使用例
/// ```rust,ignore
/// assert!(ignore("```rust", false));
/// assert!(ignore("| col1 | col2 |", false));
/// assert!(!ignore("This is a normal sentence.", false));
/// ```
fn ignore(line: &str, in_code_block: bool) -> bool {
    // コードブロック内は常に無視
    in_code_block
        // e.g. (exempli gratia: ラテン語で「例えば」)
        || line.to_lowercase().contains("e.g.")
        // i.e. (id est: ラテン語で「すなわち」)
        || line.contains("i.e.")
        // パイプはMarkdownテーブルの区切り文字
        || line.contains('|')
        // 引用ブロック（> で始まる、前に空白があってもよい）
        || line.trim_start().starts_with('>')
        // 見出し（# で始まる）
        || line.starts_with('#')
        // 空行（空白のみの行も含む）
        || line.trim().is_empty()
        // リンクターゲット定義（[label]: URL 形式）
        || REGEX_IGNORE_LINK_TARGETS.is_match(line)
}

/// 文章を規則に適合させる（文を分割する）
///
/// 長い文を句読点の位置で適切に分割します。
/// これにより、Git差分が見やすくなり、レビューが容易になります。
///
/// # 引数
/// - `content`: 処理対象のMarkdownテキスト
///
/// # 戻り値
/// 分割後のテキスト
///
/// # 処理の詳細
/// 1. 各行を走査
/// 2. 無視すべき行（コードブロック、テーブルなど）をスキップ
/// 3. 句読点パターン（REGEX_SPLIT）にマッチする位置で分割
/// 4. 分割後の各部分に適切なインデントを付与
/// 5. 元の行を分割後の複数行で置き換え
///
/// # インデント処理
/// - リスト項目の場合: リストマーカーの長さ分インデント
/// - 通常の行の場合: 行頭の空白の数だけインデント
///
/// # 使用例
/// ```rust,ignore
/// let text = "First sentence. Second sentence.\n";
/// let result = comply(text);
/// // "First sentence.\nSecond sentence.\n"
/// ```
fn comply(content: &str) -> String {
    // 行をベクタに変換（所有権を持つ）
    let content: Vec<_> = content.lines().map(std::borrow::ToOwned::to_owned).collect();

    // 新しい内容を格納するベクタ（初期値は元の内容）
    let mut new_content = content.clone();

    // 新しいベクタ内の現在位置
    let mut new_n = 0;

    // コードブロック内かどうかのフラグ
    let mut in_code_block = false;

    // 各行を処理
    for (n, line) in content.into_iter().enumerate() {
        // 最初の行以外では、インデックスを進める
        if n != 0 {
            new_n += 1;
        }

        // コードブロックの開始/終了を検出
        // ``` で始まる行はコードブロックの境界
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // 無視すべき行はスキップ
        if ignore(&line, in_code_block) {
            continue;
        }

        // 分割パターンにマッチするかチェック
        if REGEX_SPLIT.is_match(&line) {
            // インデント幅を計算
            let indent = if let Some(regex_match) = REGEX_LIST_ENTRY.find(&line) {
                // リスト項目の場合: マーカーの長さ
                // 例: "1. " の長さは 3
                // 例: "  - " の長さは 4
                regex_match.len()
            } else {
                // 通常の行: 最初の非空白文字の位置
                line.find(|ch: char| !ch.is_whitespace()).unwrap()
            };

            // 正規表現で分割（区切り文字を含む）
            let mut newly_split_lines = line.split_inclusive(&*REGEX_SPLIT);

            // 最初の部分を取得（末尾の空白を削除）
            let first = newly_split_lines.next().unwrap().trim_end().to_owned();

            // 残りの部分を処理（各部分にインデントを追加）
            let mut remaining: Vec<_> = newly_split_lines
                .map(|portion| {
                    // format! でインデントを追加
                    // {:indent$} は indent 個の空白を生成
                    format!("{:indent$}{}", "", portion.trim_end())
                })
                .collect();

            // 分割後の行を結合
            let mut new_lines = Vec::new();
            new_lines.push(first);
            new_lines.append(&mut remaining);

            // 元の行を分割後の行で置き換え
            // splice は範囲を指定して要素を置き換える
            new_content.splice(new_n..=new_n, new_lines.clone());

            // インデックスを分割後の行数分進める
            new_n += new_lines.len() - 1;
        }
    }

    // 行を改行で結合し、末尾に改行を追加
    new_content.join("\n") + "\n"
}

/// 短い行を結合する
///
/// 分割された短い行を結合して、行長を最適化します。
/// これにより、読みやすさを保ちながらファイルサイズを削減できます。
///
/// # 引数
/// - `content`: 処理対象のテキスト
/// - `limit`: 行長の上限（これ以下なら結合を試みる）
///
/// # 戻り値
/// 結合後のテキスト
///
/// # 結合しない条件
/// - コードブロック内
/// - HTMLの div タグ内
/// - 分割パターンにマッチする行
/// - 次の行が無視すべき行
/// - 次の行がリスト項目
/// - 現在の行が句読点で終わる
///
/// # 処理の詳細
/// 1. 各行をチェック
/// 2. 結合可能かを判定
/// 3. 可能なら次の行と結合
/// 4. 次の行は削除
///
/// # 使用例
/// ```rust,ignore
/// let text = "Short line.\nAnother short line.\n";
/// let result = lengthen_lines(text, 100);
/// // "Short line. Another short line.\n"
/// ```
fn lengthen_lines(content: &str, limit: usize) -> String {
    // 行をベクタに変換
    let content: Vec<_> = content.lines().map(std::borrow::ToOwned::to_owned).collect();

    // 新しい内容を格納するベクタ
    let mut new_content = content.clone();

    // 現在の位置
    let mut new_n = 0;

    // 状態フラグ
    let mut in_code_block = false;
    let mut in_html_div = false;
    let mut skip_next = false;

    // 各行を処理
    for (n, line) in content.iter().enumerate() {
        // 前回の処理で結合済みの行はスキップ
        if skip_next {
            skip_next = false;
            continue;
        }

        // インデックスを進める
        if n != 0 {
            new_n += 1;
        }

        // コードブロックの検出
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // HTMLの div 開始タグを検出
        if line.trim_start().starts_with("<div") {
            in_html_div = true;
            continue;
        }

        // HTMLの div 終了タグを検出
        if line.trim_start().starts_with("</div") {
            in_html_div = false;
            continue;
        }

        // div 内はスキップ
        if in_html_div {
            continue;
        }

        // 無視すべき行、または分割パターンにマッチする行はスキップ
        if ignore(line, in_code_block) || REGEX_SPLIT.is_match(line) {
            continue;
        }

        // 次の行を取得
        let Some(next_line) = content.get(n + 1) else {
            // 次の行がなければ終了
            continue;
        };

        // 次の行が結合不可能な条件をチェック
        if ignore(next_line, in_code_block)
            || REGEX_LIST_ENTRY.is_match(next_line)
            || REGEX_IGNORE_END.is_match(line)
        {
            continue;
        }

        // 結合後の長さをチェック
        if line.len() + next_line.len() < limit {
            // 2つの行を結合（間にスペースを挿入）
            // next_line.trim_start() で次の行の先頭空白を除去
            new_content[new_n] = format!("{line} {}", next_line.trim_start());

            // 次の行を削除
            new_content.remove(new_n + 1);

            // 次のイテレーションで削除した行をスキップ
            skip_next = true;
        }
    }

    // 行を改行で結合し、末尾に改行を追加
    new_content.join("\n") + "\n"
}

// テストモジュール
#[cfg(test)]
mod tests {
    /// sembr（文の分割）機能のテスト
    ///
    /// 様々な形式の文を正しく分割できることを検証します。
    #[test]
    fn test_sembr() {
        // テスト用の入力テキスト
        let original = "
# some. heading
must! be; split?
1. ignore a dot after number. but no further
ignore | tables
ignore e.g. and
ignore i.e. and
ignore E.g. too
- list. entry
 * list. entry
```
some code. block
```
sentence with *italics* should not be ignored. truly.
git log main.. compiler
 foo.   bar.  baz
";

        // 期待される出力
        let expected = "
# some. heading
must!
be;
split?
1. ignore a dot after number.
   but no further
ignore | tables
ignore e.g. and
ignore i.e. and
ignore E.g. too
- list.
  entry
 * list.
   entry
```
some code. block
```
sentence with *italics* should not be ignored.
truly.
git log main.. compiler
 foo.
   bar.
  baz
";

        // comply 関数をテスト
        assert_eq!(expected, super::comply(original));
    }

    /// prettify（文の結合）機能のテスト
    ///
    /// 短い文を適切に結合し、特殊な構造（リスト、div）を
    /// 保護できることを検証します。
    #[test]
    fn test_prettify() {
        let original = "\
do not split
short sentences
<div class='warning'>
a bit of text inside
</div>
preserve next line
1. one

preserve next line
- two

preserve next line
* three
";

        let expected = "\
do not split short sentences
<div class='warning'>
a bit of text inside
</div>
preserve next line
1. one

preserve next line
- two

preserve next line
* three
";

        assert_eq!(expected, super::lengthen_lines(original, 50));
    }

    /// インデント付き行の結合テスト
    ///
    /// インデントがある行も正しく結合できることを検証します。
    #[test]
    fn test_prettify_prefix_spaces() {
        let original = "\
 do not split
 short sentences
";

        let expected = "\
 do not split short sentences
";

        assert_eq!(expected, super::lengthen_lines(original, 50));
    }

    /// リンクターゲットの保護テスト
    ///
    /// リンクターゲット定義が結合されないことを検証します。
    #[test]
    fn test_prettify_ignore_link_targets() {
        let original = "\
[a target]: https://example.com
[another target]: https://example.com
";

        // リンクターゲットは結合されないため、変更なし
        assert_eq!(original, super::lengthen_lines(original, 100));
    }

    /// 分割と結合の統合テスト
    ///
    /// comply と lengthen_lines を連続して適用した場合の
    /// 動作を検証します。
    #[test]
    fn test_sembr_then_prettify() {
        let original = "
hi there. do
not split
short sentences.
hi again.
";

        // 1回目の処理: 分割
        let expected = "
hi there.
do
not split
short sentences.
hi again.
";
        let processed = super::comply(original);
        assert_eq!(expected, processed);

        // 2回目の処理: 結合（制限50）
        let expected = "
hi there.
do not split
short sentences.
hi again.
";
        let processed = super::lengthen_lines(&processed, 50);
        assert_eq!(expected, processed);

        // 3回目の処理: さらに結合
        let expected = "
hi there.
do not split short sentences.
hi again.
";
        let processed = super::lengthen_lines(&processed, 50);
        assert_eq!(expected, processed);
    }

    /// 疑問符の処理テスト
    ///
    /// 疑問符（?）の特殊ケースを正しく処理できることを検証します。
    /// 特に "r?" (Gitのレビュー依頼) は分割しないようにします。
    #[test]
    fn test_sembr_question_mark() {
        let original = "
o? whatever
r? @reviewer
 r? @reviewer
";

        let expected = "
o?
whatever
r? @reviewer
 r? @reviewer
";

        assert_eq!(expected, super::comply(original));
    }
}
