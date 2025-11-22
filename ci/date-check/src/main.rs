//! # ドキュメント日付チェックツール
//!
//! このツールは、Markdownドキュメント内の日付参照コメントを検索し、
//! 古くなった日付（6ヶ月以上経過）を検出してレポートを生成します。
//!
//! ## 用途
//! - ドキュメントの鮮度管理
//! - 定期的なドキュメントレビューの自動化
//! - GitHub Issueの自動生成
//!
//! ## 日付コメントの形式
//! ```markdown
//! <!-- date-check: January 2024 -->
//! または
//! <!-- date-check --> January 2024
//! ```
//!
//! ## 使用方法
//! ```bash
//! date-check /path/to/docs
//! ```

use std::collections::BTreeMap;
use std::convert::TryInto as _;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fmt, fs, process};

use chrono::{Datelike as _, Month, TimeZone as _, Utc};
use glob::glob;
use regex::Regex;

/// 日付を表す構造体
///
/// 年と月のみを保持し、日付チェックに必要な情報を提供します。
/// 日と時刻は不要なため、年月のみで十分です。
///
/// # フィールド
/// - `year`: 年（例: 2024）
/// - `month`: 月（1-12）
///
/// # 使用例
/// ```rust,ignore
/// let date = Date { year: 2024, month: 1 };
/// println!("{}", date); // "2024-01"
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Date {
    /// 西暦年（例: 2024）
    year: u32,
    /// 月（1=1月, 2=2月, ..., 12=12月）
    month: u32,
}

impl Date {
    /// 指定した日付からの経過月数を計算
    ///
    /// このメソッドは、2つの日付の差を月単位で計算します。
    /// ドキュメントの古さを判定するために使用されます。
    ///
    /// # 引数
    /// - `other`: 比較対象の日付（過去の日付を想定）
    ///
    /// # 戻り値
    /// - `Some(u32)`: self が other より未来の場合、経過月数
    /// - `None`: self が other より過去の場合（無効なケース）
    ///
    /// # 計算方法
    /// 1. chrono ライブラリで各日付を DateTime に変換
    /// 2. 2つの日付間の日数差を計算
    /// 3. 日数を30で割って概算の月数を算出
    ///
    /// # 注意
    /// 日数÷30の簡易計算のため、実際の月数とは多少の誤差があります。
    /// しかし、6ヶ月以上の差を検出する目的には十分な精度です。
    ///
    /// # 使用例
    /// ```rust,ignore
    /// let old_date = Date { year: 2024, month: 1 };
    /// let new_date = Date { year: 2024, month: 7 };
    /// assert_eq!(new_date.months_since(old_date), Some(6));
    /// ```
    fn months_since(self, other: Date) -> Option<u32> {
        // self の日付を chrono の DateTime に変換
        // 各月の1日0時0分0秒として扱う
        let self_chrono =
            Utc.with_ymd_and_hms(self.year.try_into().unwrap(), self.month, 1, 0, 0, 0).unwrap();

        // other の日付を chrono の DateTime に変換
        let other_chrono =
            Utc.with_ymd_and_hms(other.year.try_into().unwrap(), other.month, 1, 0, 0, 0).unwrap();

        // 2つの日付間の期間を計算
        // signed_duration_since は負の値も返せる（self が過去の場合）
        let duration_since = self_chrono.signed_duration_since(other_chrono);

        // 日数差を取得し、30で割って月数に変換
        // 30日/月は簡易的な計算だが、長期間の比較には十分
        let months_since = duration_since.num_days() / 30;

        // 負の値（self が過去）の場合は None を返す
        // 正の値の場合は u32 に変換して Some で返す
        if months_since < 0 { None } else { Some(months_since.try_into().unwrap()) }
    }
}

/// Display トレイトの実装
///
/// 日付を "YYYY-MM" 形式で表示します。
/// この形式は ISO 8601 の年月表記に準拠しています。
impl fmt::Display for Date {
    /// 日付を文字列に変換
    ///
    /// # フォーマット
    /// - 年: 4桁（ゼロ埋め）
    /// - 月: 2桁（ゼロ埋め）
    /// - 区切り: ハイフン
    ///
    /// # 引数
    /// - `f`: フォーマッタ
    ///
    /// # 戻り値
    /// フォーマット結果
    ///
    /// # 出力例
    /// - `Date { year: 2024, month: 1 }` → "2024-01"
    /// - `Date { year: 2024, month: 12 }` → "2024-12"
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // {:04} は4桁のゼロ埋め
        // {:02} は2桁のゼロ埋め
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

/// 日付コメントを検出する正規表現を生成
///
/// Markdownファイル内の日付参照コメントにマッチする正規表現を構築します。
/// 2つの形式をサポートします：
/// 1. `<!-- date-check: January 2024 -->`
/// 2. `<!-- date-check --> January 2024`
///
/// # 戻り値
/// コンパイル済みの正規表現オブジェクト
///
/// # 正規表現の構造
/// - `(?x)`: 無意味な空白を無視するモード（可読性向上）
/// - 2つのパターンを `|` で結合（OR条件）
/// - キャプチャグループで月名（m1/m2）と年（y1/y2）を抽出
///
/// # サポートする月名
/// - 英語の月名（January, February, ...）
/// - 略称も可（Jan, Feb, ...）
/// - 大文字小文字を区別しない
///
/// # 使用例
/// ```rust,ignore
/// let regex = make_date_regex();
/// assert!(regex.is_match("<!-- date-check: January 2024 -->"));
/// ```
fn make_date_regex() -> Regex {
    Regex::new(
        r"(?x) # insignificant whitespace mode
        # パターン1: <!-- date-check: January 2024 -->
        (<!--\s*               # 開始タグ（<!-- の後に任意の空白）
          date-check:\s*      # "date-check:" の後に任意の空白
          (?P<m1>[[:alpha:]]+)\s+  # 月名（英字）をキャプチャ、後に空白
          (?P<y1>\d{4})\s*-->      # 4桁の年をキャプチャ、後に任意の空白と -->
        )
        |                     # OR
        # パターン2: <!-- date-check --> January 2024
        (<!--\s*              # 開始タグ
          date-check\s*-->\s+ # "date-check" の後に --> と空白
          (?P<m2>[[:alpha:]]+)\s+  # 月名をキャプチャ
          (?P<y2>\d{4})\b          # 4桁の年をキャプチャ、\b は単語境界
        )
    ",
    )
    .unwrap()
}

/// ファイルから日付情報を収集
///
/// ファイルの内容から全ての日付コメントを抽出し、
/// 行番号と日付のペアのベクタを返します。
///
/// # 引数
/// - `date_regex`: 日付検出用の正規表現
/// - `text`: ファイルの内容（文字列）
///
/// # 戻り値
/// `Vec<(usize, Date)>`: (行番号, 日付) のペアのリスト
///
/// # 処理の流れ
/// 1. 正規表現で全てのマッチを検索
/// 2. 各マッチから月名と年を抽出
/// 3. 月名を数値に変換（January → 1）
/// 4. マッチ位置から行番号を計算
/// 5. 結果をベクタに格納
///
/// # 行番号の計算
/// - マッチした位置までの改行文字をカウント
/// - 1始まりの行番号を使用
///
/// # 使用例
/// ```rust,ignore
/// let regex = make_date_regex();
/// let text = "<!-- date-check: January 2024 -->\nSome text";
/// let dates = collect_dates_from_file(&regex, text);
/// assert_eq!(dates, vec![(1, Date { year: 2024, month: 1 })]);
/// ```
fn collect_dates_from_file(date_regex: &Regex, text: &str) -> Vec<(usize, Date)> {
    // 現在の行番号を追跡（1始まり）
    let mut line = 1;

    // 最後のキャプチャの終了位置（行番号計算用）
    let mut end_of_last_cap = 0;

    // 正規表現で全てのマッチを検索
    date_regex
        .captures_iter(text)
        // 各キャプチャから月と年を抽出
        .filter_map(|cap| {
            // パターン1（m1, y1）またはパターン2（m2, y2）のどちらかが存在
            if let (Some(month), Some(year), None, None) | (None, None, Some(month), Some(year)) =
                (cap.name("m1"), cap.name("y1"), cap.name("m2"), cap.name("y2"))
            {
                // 年を文字列から数値に変換
                let year = year.as_str().parse().expect("year");

                // 月名を Month 列挙型に変換し、数値（1-12）を取得
                // FromStr トレイトにより "January" → Month::January
                // number_from_month() により Month::January → 1
                let month = Month::from_str(month.as_str()).expect("month").number_from_month();

                // バイト範囲と日付のペアを返す
                Some((cap.get(0).expect("all").range(), Date { year, month }))
            } else {
                // どちらのパターンにもマッチしない場合は None
                None
            }
        })
        // バイト範囲から行番号を計算
        .map(|(byte_range, date)| {
            // 前回のキャプチャから今回のキャプチャまでの改行数をカウント
            // この範囲内の '\n' の数が経過した行数
            line += text[end_of_last_cap..byte_range.end].chars().filter(|c| *c == '\n').count();

            // 次回のために終了位置を更新
            end_of_last_cap = byte_range.end;

            // (行番号, 日付) のペアを返す
            (line, date)
        })
        .collect()
}

/// 複数のファイルから日付情報を収集
///
/// 指定されたパスのイテレータから全てのファイルを読み込み、
/// 日付コメントを含むファイルとその日付情報をマップとして返します。
///
/// # 引数
/// - `paths`: ファイルパスのイテレータ
///
/// # 戻り値
/// `BTreeMap<PathBuf, Vec<(usize, Date)>>`: パスをキー、日付リストを値とするマップ
///
/// # BTreeMapを使う理由
/// - キーがソートされる（パス名の辞書順）
/// - 出力結果が常に一定の順序になる
/// - レポートの可読性が向上
///
/// # 処理の流れ
/// 1. 日付検出用の正規表現を作成
/// 2. 各ファイルを読み込み
/// 3. ファイルから日付情報を抽出
/// 4. 日付が見つかったファイルのみマップに追加
///
/// # エラー処理
/// ファイル読み込みに失敗した場合、unwrap() でパニックします。
/// これはCI環境での使用を想定しており、エラーは即座に検出すべきため。
///
/// # 使用例
/// ```rust,ignore
/// let paths = glob("docs/**/*.md").unwrap().map(Result::unwrap);
/// let dates = collect_dates(paths);
/// ```
fn collect_dates(paths: impl Iterator<Item = PathBuf>) -> BTreeMap<PathBuf, Vec<(usize, Date)>> {
    // 日付検出用の正規表現を一度だけ生成
    let date_regex = make_date_regex();

    // 結果を格納するマップ（BTreeMapは自動的にキーでソート）
    let mut data = BTreeMap::new();

    // 各パスを処理
    for path in paths {
        // ファイルを読み込み（失敗時はパニック）
        let text = fs::read_to_string(&path).unwrap();

        // ファイルから日付を収集
        let dates = collect_dates_from_file(&date_regex, &text);

        // 日付が見つかった場合のみマップに追加
        // 空のベクタは追加しない（メモリとレポートの無駄を削減）
        if !dates.is_empty() {
            data.insert(path, dates);
        }
    }

    data
}

/// 古い日付をフィルタリング
///
/// 収集した日付情報から、指定した月数以上経過した日付のみを抽出します。
/// ドキュメントの更新が必要な箇所を特定するために使用されます。
///
/// # 引数
/// - `current_month`: 現在の年月
/// - `min_months_since`: 最小経過月数（これ以上経過していたら古いとみなす）
/// - `dates_by_file`: ファイルごとの日付情報
///
/// # 戻り値
/// フィルタリング後の日付情報（古い日付のみ）
///
/// # フィルタリング条件
/// `current_month.months_since(date) >= min_months_since`
///
/// # 処理の流れ
/// 1. 各ファイルの日付リストをイテレート
/// 2. 各日付について経過月数を計算
/// 3. min_months_since 以上の日付のみ残す
/// 4. 結果が空のファイルは除外
///
/// # 使用例
/// ```rust,ignore
/// let current = Date { year: 2024, month: 7 };
/// let filtered = filter_dates(current, 6, dates_by_file.into_iter());
/// // 2024年1月以前の日付のみが残る
/// ```
fn filter_dates(
    current_month: Date,
    min_months_since: u32,
    dates_by_file: impl Iterator<Item = (PathBuf, Vec<(usize, Date)>)>,
) -> impl Iterator<Item = (PathBuf, Vec<(usize, Date)>)> {
    dates_by_file
        // 各ファイルを処理
        .map(move |(path, dates)| {
            (
                path,
                // 日付リストをフィルタリング
                dates
                    .into_iter()
                    .filter(|(_, date)| {
                        // 経過月数を計算
                        current_month
                            .months_since(*date)
                            // expect: 未来の日付が見つかった場合はエラー
                            // （通常は起こらないはずだが、データ整合性チェック）
                            .expect("found date that is after current month")
                            // min_months_since 以上経過している日付のみ残す
                            >= min_months_since
                    })
                    .collect::<Vec<_>>(),
            )
        })
        // 空のベクタ（全ての日付が除外された）を持つファイルを除外
        .filter(|(_, dates)| !dates.is_empty())
}

/// メイン関数
///
/// コマンドライン引数を解析し、ドキュメントディレクトリ内の
/// 古い日付コメントを検出してレポートを生成します。
///
/// # コマンドライン引数
/// - 第1引数: ドキュメントルートディレクトリのパス
///
/// # 処理の流れ
/// 1. コマンドライン引数を取得
/// 2. Markdownファイルを検索
/// 3. 日付コメントを収集
/// 4. 6ヶ月以上経過した日付をフィルタリング
/// 5. レポートを生成して標準出力に表示
///
/// # 出力形式
/// GitHub Issueに貼り付けられる形式のMarkdownを出力します：
/// ```markdown
/// Date Reference Triage for 2024-07
/// ## Procedure
/// （手順の説明）
/// ## Dates
/// - path/to/file.md
///   - [ ] line 123: 2024-01
/// ```
///
/// # 終了コード
/// - 0: 古い日付なし（全て最新）
/// - 1: 古い日付が見つかった
///
/// # エラー処理
/// - 引数不足: エラーメッセージを表示して終了コード1で終了
/// - ファイル読み込みエラー: パニック
fn main() {
    // コマンドライン引数を取得
    let mut args = env::args();

    // 引数の数をチェック（プログラム名 + ルートディレクトリ = 2個必要）
    if args.len() == 1 {
        // 引数が不足している場合、エラーメッセージを表示
        eprintln!("error: expected root of Markdown directory as CLI argument");
        // 終了コード1で終了（エラーを示す）
        process::exit(1);
    }

    // 第2引数（インデックス1）を取得してルートディレクトリとする
    // nth(1) は最初（0）をスキップして次（1）を取得
    let root_dir = args.nth(1).unwrap();

    // ルートディレクトリのPathを作成
    let root_dir_path = Path::new(&root_dir);

    // glob パターンを構築（例: "docs/**/*.md"）
    // ** は再帰的なディレクトリマッチング
    // *.md は全てのMarkdownファイル
    let glob_pat = format!("{}/**/*.md", root_dir);

    // 現在の日付を取得
    let today_chrono = Utc::now().date_naive();

    // 現在の年月を Date 構造体に変換
    // year_ce() は (is_ce, year) のタプルを返し、.1 で年を取得
    let current_month = Date { year: today_chrono.year_ce().1, month: today_chrono.month() };

    // Markdownファイルから日付情報を収集
    // glob() はファイルパスのイテレータを返す
    // map(Result::unwrap) でエラーをパニックに変換
    let dates_by_file = collect_dates(glob(&glob_pat).unwrap().map(Result::unwrap));

    // 6ヶ月以上経過した日付のみをフィルタリング
    // into_iter() で所有権を移動
    // collect() でイテレータをBTreeMapに変換
    let dates_by_file: BTreeMap<_, _> =
        filter_dates(current_month, 6, dates_by_file.into_iter()).collect();

    // 結果の出力
    if dates_by_file.is_empty() {
        // 古い日付がない場合（全て最新）
        println!("empty");
    } else {
        // 古い日付が見つかった場合、GitHub Issue用のレポートを生成

        // レポートのタイトル
        println!("Date Reference Triage for {}", current_month);

        // 手順セクション
        println!("## Procedure");
        println!();

        // 手順の説明文
        // 各日付をチェックして更新する方法を説明
        println!(
            "Each of these dates should be checked to see if the docs they annotate are \
             up-to-date. Each date should be updated (in the Markdown file where it appears) to \
             use the current month ({current_month}), or removed if the docs it annotates are not \
             expected to fall out of date quickly.",
            current_month = today_chrono.format("%B %Y"),
        );
        println!();

        // チェックボックスの使い方
        println!(
            "Please check off each date once a PR to update it (and, if applicable, its \
             surrounding docs) has been merged. Please also mention that you are working on a \
             particular set of dates so duplicate work is avoided."
        );
        println!();

        // Issue のクローズ手順
        println!("Finally, once all the dates have been updated, please close this issue.");
        println!();

        // 日付リストセクション
        println!("## Dates");
        println!();

        // 各ファイルと日付を出力
        for (path, dates) in dates_by_file {
            // ファイルパス（ルートディレクトリからの相対パス）
            // strip_prefix() で共通プレフィックスを除去
            // unwrap_or() で失敗時は元のパスを使用
            println!("- {}", path.strip_prefix(&root_dir_path).unwrap_or(&path).display(),);

            // ファイル内の各日付
            for (line, date) in dates {
                // チェックボックス形式で出力
                // - [ ] はGitHubのタスクリスト記法（未チェック状態）
                println!("  - [ ] line {}: {}", line, date);
            }
        }
        println!();
    }
}

// テストモジュール
#[cfg(test)]
mod tests {
    use super::*;

    /// months_since メソッドのテスト
    ///
    /// 2つの日付間の月数計算が正しいことを検証します。
    ///
    /// # テストケース
    /// - 2020年3月 から 2021年1月 = 約10ヶ月
    #[test]
    fn test_months_since() {
        let date1 = Date { year: 2020, month: 3 };
        let date2 = Date { year: 2021, month: 1 };
        // 10ヶ月の差があることを確認
        assert_eq!(date2.months_since(date1), Some(10));
    }

    /// 日付正規表現のテスト（マッチするケース）
    ///
    /// 正規表現が様々な形式の日付コメントを正しく認識することを検証します。
    ///
    /// # テストケース
    /// - 小文字の月名（jan, january）
    /// - 大文字始まりの月名（Jan, January）
    /// - 2つの異なるコメント形式
    /// - 末尾の空白や句読点
    #[test]
    fn test_date_regex() {
        let regex = &make_date_regex();

        // 形式1のテスト
        assert!(regex.is_match("<!-- date-check: jan 2021 -->"));
        assert!(regex.is_match("<!-- date-check: january 2021 -->"));
        assert!(regex.is_match("<!-- date-check: Jan 2021 -->"));
        assert!(regex.is_match("<!-- date-check: January 2021 -->"));

        // 形式2のテスト
        assert!(regex.is_match("<!-- date-check --> jan 2021"));
        assert!(regex.is_match("<!-- date-check --> january 2021"));
        assert!(regex.is_match("<!-- date-check --> Jan 2021"));
        assert!(regex.is_match("<!-- date-check --> January 2021"));

        // 末尾に空白や句読点がある場合
        assert!(regex.is_match("<!-- date-check --> jan 2021 "));
        assert!(regex.is_match("<!-- date-check --> jan 2021."));
    }

    /// 日付正規表現のテスト（マッチしないケース）
    ///
    /// 不正な形式の日付コメントを正しく拒否することを検証します。
    ///
    /// # テストケース
    /// - 3桁の年（221）
    /// - 5桁の年（20221）
    /// - 数値の月名（01）
    #[test]
    fn test_date_regex_fail() {
        let regexes = &make_date_regex();

        // 不正な年（3桁）
        assert!(!regexes.is_match("<!-- date-check: jan 221 -->"));
        // 不正な年（5桁）
        assert!(!regexes.is_match("<!-- date-check: jan 20221 -->"));
        // 数値の月名（サポート外）
        assert!(!regexes.is_match("<!-- date-check: 01 2021 -->"));

        // 形式2でも同様
        assert!(!regexes.is_match("<!-- date-check --> jan 221"));
        assert!(!regexes.is_match("<!-- date-check --> jan 20222"));
        assert!(!regexes.is_match("<!-- date-check --> 01 2021"));
    }

    /// ファイルからの日付収集のテスト
    ///
    /// 複雑なテキストから日付を正しく抽出し、
    /// 正確な行番号を計算できることを検証します。
    ///
    /// # テストケース
    /// - 様々な形式の日付コメント
    /// - 複数行にわたるコメント
    /// - 行の途中にあるコメント
    /// - 正確な行番号の計算
    #[test]
    fn test_collect_dates_from_file() {
        // テスト用のMarkdownテキスト
        // 様々な形式と位置の日付コメントを含む
        let text = r"
Test1
<!-- date-check: jan 2021 -->
Test2
Foo<!-- date-check: february 2021
-->
Test3
Test4
Foo<!-- date-check: Mar 2021 -->Bar
<!-- date-check:April 2021
-->
Test5
Test6
Test7
<!-- date-check:

may 2021 -->
Test8
Test1
<!-- date-check -->  jan 2021
Test2
Foo<!-- date-check
--> february 2021
Test3
Test4
Foo<!-- date-check -->  mar 2021 Bar
<!-- date-check
--> apr 2021
Test5
Test6
Test7
<!-- date-check

 --> may 2021
Test8
 <!--
   date-check
 --> june 2021.
        ";

        // 期待される結果
        // (行番号, 日付) のペアのベクタ
        assert_eq!(
            collect_dates_from_file(&make_date_regex(), text),
            vec![
                (3, Date { year: 2021, month: 1 }),   // jan
                (6, Date { year: 2021, month: 2 }),   // february（複数行）
                (9, Date { year: 2021, month: 3 }),   // Mar（行の途中）
                (11, Date { year: 2021, month: 4 }),  // April（複数行）
                (17, Date { year: 2021, month: 5 }),  // may（空行あり）
                (20, Date { year: 2021, month: 1 }),  // jan（形式2）
                (23, Date { year: 2021, month: 2 }),  // february（形式2、複数行）
                (26, Date { year: 2021, month: 3 }),  // mar（形式2、行の途中）
                (28, Date { year: 2021, month: 4 }),  // apr（形式2、複数行）
                (34, Date { year: 2021, month: 5 }),  // may（形式2、空行あり）
                (38, Date { year: 2021, month: 6 }),  // june（複数行、句読点あり）
            ],
        );
    }
}
