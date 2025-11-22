//! # rustc_driver 基本サンプルプログラム
//!
//! このサンプルは、rustc_driverを使用してRustコンパイラを実行し、
//! コールバック機構を通じてコンパイルプロセスを制御する方法を示しています。
//!
//! ## 主な用途
//! - カスタムコンパイラドライバの開発
//! - コンパイル過程の監視とカスタマイズ
//! - コード解析・変換ツールの実装
//! - コンパイラプラグインの開発
//!
//! ## 動作概要
//! 1. カスタムファイルローダーでメモリ内のコードを提供
//! 2. パース後にASTを文字列化して表示
//! 3. 解析後に型情報を取得して表示
//!
//! nightly-2025-03-28 でテスト済み

#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use std::io;
use std::path::Path;
use std::sync::Arc;

use rustc_ast_pretty::pprust::item_to_string;
use rustc_driver::{Compilation, run_compiler};
use rustc_interface::interface::{Compiler, Config};
use rustc_middle::ty::TyCtxt;

/// カスタムファイルローダー
///
/// ファイルシステムからファイルを読み込む代わりに、
/// メモリ内に保持されたコードを提供するローダーです。
///
/// # 利点
/// - 一時ファイルの作成が不要
/// - テストの高速化
/// - サンドボックス環境での実行が容易
///
/// # 実装詳細
/// "main.rs" という仮想ファイルのみをサポートし、
/// ハードコードされたサンプルプログラムを提供します。
struct MyFileLoader;

/// FileLoaderトレイトの実装
///
/// rustcのファイルアクセスをカスタマイズするインターフェースです。
/// コンパイラがソースコードやライブラリファイルにアクセスする際に使用されます。
impl rustc_span::source_map::FileLoader for MyFileLoader {
    /// ファイルの存在確認
    ///
    /// コンパイラがファイルをロードする前に、その存在を確認するために呼び出されます。
    /// このローダーは "main.rs" のみを仮想的に提供します。
    ///
    /// # 引数
    /// - `path`: 確認するファイルのパス
    ///
    /// # 戻り値
    /// - "main.rs" の場合: `true`（ファイルが存在する）
    /// - その他のパス: `false`（ファイルが存在しない）
    ///
    /// # 使用シーン
    /// - モジュール解決時の候補ファイル確認
    /// - use文で参照されるファイルの存在チェック
    /// - コンパイル対象ファイルの検証
    fn file_exists(&self, path: &Path) -> bool {
        // main.rs のみ存在するとみなす
        // これにより、不要なファイルアクセスを防止
        path == Path::new("main.rs")
    }

    /// テキストファイルの読み込み
    ///
    /// ソースコードファイルを読み込む際に呼び出されます。
    /// このローダーは "main.rs" に対して固定のサンプルコードを返します。
    ///
    /// # 引数
    /// - `path`: 読み込むファイルのパス
    ///
    /// # 戻り値
    /// - `Ok(String)`: "main.rs" の場合、サンプルプログラムの文字列
    /// - `Err(io::Error)`: その他のパスの場合、エラー
    ///
    /// # サンプルプログラムの内容
    /// ```rust,ignore
    /// static MESSAGE: &str = "Hello, World!";
    /// fn main() {
    ///     println!("{MESSAGE}");
    /// }
    /// ```
    ///
    /// このプログラムは：
    /// - グローバル静的変数 MESSAGE を定義
    /// - main 関数で MESSAGE を出力
    fn read_file(&self, path: &Path) -> io::Result<String> {
        if path == Path::new("main.rs") {
            // main.rs のサンプルコードを返す
            // r#"..."# は生文字列リテラル（エスケープ不要）
            Ok(r#"
static MESSAGE: &str = "Hello, World!";
fn main() {
    println!("{MESSAGE}");
}
"#
            .to_string())
        } else {
            // 未知のファイルパスの場合はエラー
            // io::Error::other() はカスタムエラーメッセージを作成
            Err(io::Error::other("oops"))
        }
    }

    /// バイナリファイルの読み込み
    ///
    /// バイナリファイル（.rlib, .so などのコンパイル済みライブラリ）を
    /// 読み込む際に呼び出されます。
    ///
    /// # 引数
    /// - `_path`: 読み込むファイルのパス（使用しないため接頭辞 _ で無視）
    ///
    /// # 戻り値
    /// 常に `Err` を返し、バイナリファイルの読み込みをサポートしないことを示します。
    ///
    /// # 注意
    /// 外部クレートを使用する場合、.rlib ファイルなどを提供する必要がありますが、
    /// このサンプルではスタンドアロンコードのみを扱うため実装していません。
    fn read_binary_file(&self, _path: &Path) -> io::Result<Arc<[u8]>> {
        // バイナリファイルの読み込みは未サポート
        // このローダーはソースコード解析専用
        Err(io::Error::other("oops"))
    }
}

/// カスタムコンパイラコールバック
///
/// rustc_driverのコンパイルプロセスにフックを挿入するための構造体です。
/// コンパイルの各段階で任意の処理を実行できます。
///
/// # 用途
/// - コンパイルの進行状況の監視
/// - 各フェーズで生成されるデータの取得
/// - カスタム解析や警告の追加
/// - 中間表現の変換や最適化
struct MyCallbacks;

/// Callbacksトレイトの実装
///
/// rustc_driverが提供するコールバックインターフェースです。
/// コンパイルの各段階で適切なメソッドが呼び出されます。
impl rustc_driver::Callbacks for MyCallbacks {
    /// コンフィグ設定時のコールバック
    ///
    /// コンパイラの設定が初期化される際に最初に呼び出されます。
    /// ここでコンパイラの動作をカスタマイズできます。
    ///
    /// # 引数
    /// - `config`: コンパイラ設定オブジェクトへの可変参照
    ///
    /// # 処理内容
    /// カスタムファイルローダーを設定し、ファイルアクセスを制御します。
    /// これにより、メモリ内のコードをコンパイルできるようになります。
    ///
    /// # カスタマイズ可能な項目
    /// - ファイルローダー（file_loader）
    /// - コンパイラオプション（opts）
    /// - 出力設定（output_dir, output_file）
    /// - リント設定（lint_caps）
    /// など
    fn config(&mut self, config: &mut Config) {
        // カスタムファイルローダーを設定
        // Box::new() でヒープに配置し、所有権を Config に移譲
        config.file_loader = Some(Box::new(MyFileLoader));
    }

    /// クレートルートのパース後のコールバック
    ///
    /// ソースコードがパースされ、抽象構文木（AST）が構築された直後に呼び出されます。
    /// この時点では、型情報や名前解決の結果はまだ利用できません。
    ///
    /// # 引数
    /// - `_compiler`: コンパイラインスタンス（今回は使用しない）
    /// - `krate`: パース済みのクレートAST（可変参照）
    ///
    /// # 戻り値
    /// - `Compilation::Continue`: 次のフェーズに進む
    /// - `Compilation::Stop`: コンパイルを中断する
    ///
    /// # 処理内容
    /// クレート内の全てのトップレベルアイテム（関数、構造体、static など）を
    /// イテレートし、ソースコード文字列に変換して表示します。
    ///
    /// # ASTの用途
    /// - シンタックスハイライト
    /// - コードフォーマッタ
    /// - マクロ展開前の構文チェック
    /// - ドキュメント生成
    ///
    /// # 出力例
    /// ```text
    /// static MESSAGE: &str = "Hello, World!";
    /// fn main() {
    ///     println!("{MESSAGE}");
    /// }
    /// ```
    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        // クレートの全アイテムをイテレート
        // items には以下が含まれる：
        // - 関数定義 (fn)
        // - 構造体定義 (struct)
        // - 列挙型定義 (enum)
        // - trait 定義
        // - impl ブロック
        // - static/const 定義
        // - モジュール定義 (mod)
        // - use 宣言
        for item in &krate.items {
            // AST ノードを人間が読める形式のソースコード文字列に変換
            // item_to_string() は rustc_ast_pretty::pprust モジュールの関数
            // プリティプリント（整形印刷）により、元のソースコードに近い形式で出力
            println!("{}", item_to_string(&item));
        }

        // コンパイルを次のフェーズに進める
        // Continue を返すことで、名前解決や型チェックなどが実行される
        Compilation::Continue
    }

    /// 解析完了後のコールバック
    ///
    /// 全ての解析（型チェック、借用チェックなど）が完了した後に呼び出されます。
    /// この時点で、完全な型情報と中間表現にアクセスできます。
    ///
    /// # 引数
    /// - `_compiler`: コンパイラインスタンス（今回は使用しない）
    /// - `tcx`: 型コンテキスト（Type Context）
    ///
    /// # 戻り値
    /// - `Compilation::Continue`: コード生成フェーズに進む
    /// - `Compilation::Stop`: ここでコンパイルを終了（今回はこちら）
    ///
    /// # TyCtxt（型コンテキスト）とは
    /// rustcの中心的なデータ構造で、以下の情報にアクセスできます：
    /// - 全ての型情報
    /// - HIR（高レベル中間表現）
    /// - 定義情報（DefId による識別）
    /// - 型推論の結果
    /// - トレイト実装情報
    /// - ライフタイム情報
    ///
    /// # 処理内容
    /// 1. クレート内の全フリーアイテムを走査
    /// 2. static 変数と関数を抽出
    /// 3. 各アイテムの型情報を取得して表示
    ///
    /// # 出力例
    /// ```text
    /// Ident { name: "MESSAGE", ... }: &str
    /// Ident { name: "main", ... }: fn()
    /// ```
    fn after_analysis(&mut self, _compiler: &Compiler, tcx: TyCtxt<'_>) -> Compilation {
        // プログラムの解析と型の検査
        // クレート内のフリーアイテムをイテレート
        // フリーアイテム = トップレベルのアイテム（impl ブロック内でないもの）
        for id in tcx.hir_free_items() {
            // アイテムのHIR表現を取得
            // HIR（High-level Intermediate Representation）は：
            // - ASTよりも構造化されている
            // - 脱糖（desugaring）済み（for ループが loop に変換されているなど）
            // - 型情報と結びついている
            let item = &tcx.hir_item(id);

            // アイテムの種類でマッチング
            match item.kind {
                // static 変数または関数の場合のみ処理
                // | はORパターン（どちらかにマッチ）
                // .. は残りのフィールドを無視
                rustc_hir::ItemKind::Static(ident, ..) | rustc_hir::ItemKind::Fn { ident, .. } => {
                    // アイテムの型を取得
                    // type_of() は rustc のクエリシステムを通じて型情報を取得
                    // クエリシステムは結果をキャッシュするため、効率的
                    //
                    // 返される型：
                    // - Static の場合: 変数の型（例: &str, i32 など）
                    // - Fn の場合: 関数シグネチャ（例: fn(), fn(i32) -> String など）
                    let ty = tcx.type_of(item.hir_id().owner.def_id);

                    // アイテムの識別子と型を表示
                    // {ident:?} は識別子のデバッグ表示
                    // {ty:?} は型のデバッグ表示
                    // \t はタブ文字（アラインメント用）
                    println!("{ident:?}:\t{ty:?}")
                }

                // その他のアイテム（struct, enum, trait, impl など）は無視
                // () は何もしないことを表す
                _ => (),
            }
        }

        // コンパイルをここで停止
        // Compilation::Stop を返すことで、コード生成フェーズをスキップ
        // 解析のみが目的の場合、これにより：
        // - コンパイル時間の短縮
        // - 不要なバイナリ生成の回避
        // - メモリ使用量の削減
        Compilation::Stop
    }
}

/// メイン関数
///
/// rustc_driverを起動し、カスタムコールバックを使用して
/// コンパイルプロセスを実行します。
///
/// # 処理の流れ
/// 1. コマンドライン引数を構築
/// 2. run_compiler() を呼び出してコンパイルを開始
/// 3. MyCallbacks のメソッドが各フェーズで呼び出される
/// 4. 結果が標準出力に表示される
///
/// # コマンドライン引数の形式
/// rustc_driver は実際の rustc コマンドと同じ引数形式を期待します：
/// ```text
/// rustc [OPTIONS] INPUT
/// ```
///
/// このサンプルでは：
/// - 第1引数: "ignored" （プログラム名、内容は無視される）
/// - 第2引数: "main.rs" （コンパイル対象のファイル名）
///
/// # 実行結果
/// 標準出力に以下が表示されます：
/// 1. パース後: ソースコードの整形された文字列
/// 2. 解析後: 各アイテムの識別子と型情報
///
/// # rustc_driver vs rustc_interface
/// - rustc_driver: コマンドライン互換のインターフェース
/// - rustc_interface: より構造化されたプログラマティックAPI
///
/// rustc_driver は以下の場合に適しています：
/// - rustc のドロップイン置き換え
/// - コマンドライン引数の解析が必要
/// - 既存のrustcツールチェインとの統合
fn main() {
    // rustc_driver のメインエントリーポイントを呼び出す
    run_compiler(
        // コマンドライン引数の配列
        // &[...] は文字列スライスの配列
        &[
            // 第1引数: プログラム名
            // 慣例的に実行ファイル名（"rustc"など）を指定
            // rustc_driver はこの値を使用しないが、互換性のために必要
            "ignored".to_string(),

            // 第2引数: コンパイル対象のファイル名
            // MyFileLoader がこの名前に対応するコードを提供する
            // 実際のファイルシステムにこのファイルが存在する必要はない
            "main.rs".to_string(),
        ],

        // コールバックオブジェクトへの可変参照
        // &mut により、コールバック内で状態を変更可能
        // コンパイルの各フェーズで MyCallbacks のメソッドが呼び出される
        &mut MyCallbacks,
    );
}
