//! # Rustコンパイラを使ってASTと対話するサンプルプログラム
//!
//! このサンプルは、rustc_driverを使用してRustコンパイラを実行し、
//! 抽象構文木（AST）や高レベル中間表現（HIR）を解析する方法を示しています。
//!
//! ## 主な用途
//! - コード解析ツールの開発
//! - カスタムリンターやフォーマッタの実装
//! - メタプログラミングツールの作成
//! - コード生成ツールの開発
//!
//! ## 動作概要
//! 1. カスタムファイルローダーでメモリ内のコードを提供
//! 2. コンパイルの各フェーズでコールバックを実行
//! 3. AST解析フェーズで各アイテムを文字列化して表示
//! 4. 型解析フェーズで特定の式の型情報を取得
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
/// このローダーは、実際のファイルシステムからファイルを読み込む代わりに、
/// メモリ内に保持されたコードを提供します。これにより、一時ファイルを作成せずに
/// 動的に生成したコードをコンパイルできます。
///
/// # 用途
/// - テストコードの動的生成
/// - IDEでの編集中のコード解析（未保存の内容を解析）
/// - コード生成ツールでの即座のコンパイル
///
/// # 実装の詳細
/// main.rs という仮想ファイルのみを提供し、それ以外のファイルアクセスは
/// エラーを返します。
struct MyFileLoader;

/// FileLoaderトレイトの実装
///
/// rustcのソースマップシステムが使用するインターフェースを実装します。
/// これにより、コンパイラがファイルにアクセスする際の動作をカスタマイズできます。
impl rustc_span::source_map::FileLoader for MyFileLoader {
    /// ファイルの存在確認
    ///
    /// コンパイラがファイルをロードする前に、ファイルの存在を確認するために
    /// 呼び出されます。このローダーは "main.rs" のみを提供します。
    ///
    /// # 引数
    /// - `path`: 確認対象のファイルパス
    ///
    /// # 戻り値
    /// パスが "main.rs" の場合は `true`、それ以外は `false`
    ///
    /// # 使用例
    /// ```rust,ignore
    /// if file_loader.file_exists(Path::new("main.rs")) {
    ///     let content = file_loader.read_file(Path::new("main.rs"))?;
    /// }
    /// ```
    fn file_exists(&self, path: &Path) -> bool {
        // main.rs という名前のファイルのみ存在するとみなす
        // これにより、他のファイルへのアクセスを防ぎ、制御されたテスト環境を維持
        path == Path::new("main.rs")
    }

    /// ファイルの読み込み（テキスト形式）
    ///
    /// コンパイラがソースファイルを読み込む際に呼び出されます。
    /// このローダーは、"main.rs" に対してハードコードされた
    /// サンプルプログラムを返します。
    ///
    /// # 引数
    /// - `path`: 読み込むファイルのパス
    ///
    /// # 戻り値
    /// - `Ok(String)`: "main.rs" の場合、サンプルコードを含む文字列
    /// - `Err(io::Error)`: それ以外のパスの場合、エラー
    ///
    /// # サンプルコードの内容
    /// Hello World を出力するシンプルなプログラムです。
    /// - `message` 変数に文字列リテラルを代入
    /// - `println!` マクロでメッセージを出力
    ///
    /// # エラー処理
    /// 未知のファイルパスに対しては、カスタムエラーメッセージ "oops" を返します。
    fn read_file(&self, path: &Path) -> io::Result<String> {
        if path == Path::new("main.rs") {
            // main.rs の内容をハードコードで返す
            // この方法により、実際のファイルシステムへのアクセスが不要になる
            Ok(r#"
fn main() {
    let message = "Hello, World!";
    println!("{message}");
}
"#
            .to_string())
        } else {
            // main.rs 以外のファイルへのアクセスはエラーとして処理
            // これにより、依存関係の読み込みなど予期しない動作を防ぐ
            Err(io::Error::other("oops"))
        }
    }

    /// バイナリファイルの読み込み
    ///
    /// バイナリファイル（コンパイル済みライブラリなど）の読み込みに使用されます。
    /// このローダーはバイナリファイルをサポートしないため、常にエラーを返します。
    ///
    /// # 引数
    /// - `_path`: 読み込むファイルのパス（使用しないため無視）
    ///
    /// # 戻り値
    /// 常に `Err` を返し、バイナリファイルの読み込みをサポートしないことを示します。
    ///
    /// # 注意
    /// 外部クレートを使用する場合、このメソッドで .rlib ファイルなどを
    /// 返す必要がありますが、このサンプルではスタンドアロンコードのみを扱います。
    fn read_binary_file(&self, _path: &Path) -> io::Result<Arc<[u8]>> {
        // バイナリファイルの読み込みはサポートしない
        // スタンドアロンのソースコード解析のみを想定
        Err(io::Error::other("oops"))
    }
}

/// カスタムコンパイラコールバック
///
/// rustc_driverは、コンパイルの各フェーズでコールバックを呼び出すことができます。
/// このコールバックにより、コンパイルの途中で任意の処理を実行できます。
///
/// # 用途
/// - コンパイルプロセスの監視
/// - 各フェーズでの中間データの取得
/// - カスタム解析や変換の実行
struct MyCallbacks;

/// Callbacksトレイトの実装
///
/// rustc_driverが提供するコールバックインターフェースを実装します。
/// これにより、コンパイルの各段階で処理を挿入できます。
impl rustc_driver::Callbacks for MyCallbacks {
    /// コンフィグ設定時のコールバック
    ///
    /// コンパイラの設定が初期化される際に呼び出されます。
    /// ここでカスタムファイルローダーを設定します。
    ///
    /// # 引数
    /// - `config`: コンパイラの設定オブジェクト（可変参照）
    ///
    /// # 処理内容
    /// カスタムファイルローダー（MyFileLoader）を設定することで、
    /// ファイルシステムへのアクセスを制御します。
    fn config(&mut self, config: &mut Config) {
        // カスタムファイルローダーを設定
        // これにより、メモリ内のコードをファイルシステムから読み込んだかのように扱える
        config.file_loader = Some(Box::new(MyFileLoader));
    }

    /// クレートルートのパース後のコールバック
    ///
    /// ソースコードがパースされ、抽象構文木（AST）が構築された直後に呼び出されます。
    /// この段階では、まだマクロ展開や名前解決は行われていません。
    ///
    /// # 引数
    /// - `_compiler`: コンパイラインスタンスへの参照（今回は未使用）
    /// - `krate`: パースされたクレートのAST（可変参照）
    ///
    /// # 戻り値
    /// - `Compilation::Continue`: コンパイルを続行
    /// - `Compilation::Stop`: コンパイルを中断
    ///
    /// # 処理内容
    /// クレート内の全てのトップレベルアイテム（関数、構造体など）を
    /// ソースコード文字列に変換して表示します。
    ///
    /// # 出力例
    /// ```text
    /// fn main() {
    ///     let message = "Hello, World!";
    ///     println!("{message}");
    /// }
    /// ```
    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        // クレート内の全アイテムをイテレート
        // krate.items には関数、構造体、trait、impl などが含まれる
        for item in &krate.items {
            // item_to_string は AST ノードをソースコード文字列に変換する
            // これにより、ASTの構造を人間が読める形式で確認できる
            println!("{}", item_to_string(&item));
        }

        // コンパイルを続行して次のフェーズに進む
        Compilation::Continue
    }

    /// 解析完了後のコールバック
    ///
    /// 型チェック、借用チェックなど全ての解析が完了した後に呼び出されます。
    /// この段階では、型情報や型推論の結果にアクセスできます。
    ///
    /// # 引数
    /// - `_compiler`: コンパイラインスタンスへの参照（今回は未使用）
    /// - `tcx`: 型コンテキスト。全ての型情報と中間表現にアクセス可能
    ///
    /// # 戻り値
    /// - `Compilation::Continue`: 次のフェーズ（コード生成など）へ進む
    /// - `Compilation::Stop`: ここでコンパイルを終了（今回はこちら）
    ///
    /// # 処理内容
    /// 1. トップレベルアイテムを走査してmain関数を探す
    /// 2. main関数内の最初のlet文を見つける
    /// 3. 初期化式の型情報を取得して表示
    ///
    /// # 出力例
    /// ```text
    /// Expr {
    ///     kind: Lit("Hello, World!"),
    ///     ...
    /// }: &str
    /// ```
    fn after_analysis(&mut self, _compiler: &Compiler, tcx: TyCtxt<'_>) -> Compilation {
        // クレート内のトップレベルアイテムをイテレート
        // hir_free_items() は、impl ブロック内ではないフリースタンディングなアイテムを返す
        for id in tcx.hir_free_items() {
            let item = &tcx.hir_item(id);

            // パターンマッチングでmain関数内の特定ノードを探索
            // ItemKind::Fn は関数定義を表す
            if let rustc_hir::ItemKind::Fn { body, .. } = item.kind {
                // 関数本体の式を取得
                // HIR（高レベル中間表現）では、関数本体は単一の式として表現される
                let expr = &tcx.hir_body(body).value;

                // 関数本体がブロック式であることを確認
                // ExprKind::Block はブロック { ... } を表す
                if let rustc_hir::ExprKind::Block(block, _) = expr.kind {
                    // ブロックの最初の文を取得
                    // stmts[0] は `let message = "Hello, World!";` に対応
                    if let rustc_hir::StmtKind::Let(let_stmt) = block.stmts[0].kind {
                        // let文の初期化式を取得
                        if let Some(expr) = let_stmt.init {
                            // 式のHIR ID（階層的識別子）を取得
                            // これは `"Hello, World!"` という文字列リテラルを指す
                            let hir_id = expr.hir_id;

                            // main関数の定義IDを取得
                            // これは型チェックのコンテキストを特定するために必要
                            let def_id = item.hir_id().owner.def_id;

                            // 型チェック結果を取得し、式の型を抽出
                            // typeck() は関数全体の型チェック結果を返す
                            // node_type() で特定のノードの型を取得
                            let ty = tcx.typeck(def_id).node_type(hir_id);

                            // 式の構造と型を表示
                            // {:#?} は構造体を複数行で見やすくフォーマット
                            // {:?} は型を簡潔に表示（この場合は &str）
                            println!("{expr:#?}: {ty:?}");
                        }
                    }
                }
            }
        }

        // コンパイルをここで停止（コード生成は不要）
        // 解析のみを目的とする場合、この設定により高速化できる
        Compilation::Stop
    }
}

/// メイン関数
///
/// rustc_driverを起動し、カスタムコールバックを使用して
/// コンパイルプロセスを制御します。
///
/// # 処理の流れ
/// 1. コマンドライン引数を構築（rustcと同じ形式）
/// 2. カスタムコールバック（MyCallbacks）を指定してコンパイラを実行
/// 3. コールバック内でASTとHIRを解析
///
/// # 引数形式
/// rustc_driverは、実際の rustc コマンドと同じ引数形式を期待します：
/// - 第1引数: 実行ファイル名（慣例的に "rustc"、内容は無視される）
/// - 第2引数以降: コンパイル対象のファイル名やオプション
///
/// # 実行結果
/// 標準出力に以下が表示されます：
/// 1. パース後のAST（関数定義の文字列表現）
/// 2. 解析後の型情報（特定の式の詳細な構造と型）
fn main() {
    // run_compiler を呼び出してコンパイラを実行
    run_compiler(
        &[
            // 第1引数: プログラム名（rustcの場合は "rustc"）
            // rustc_driver はこの引数を無視するが、互換性のために必要
            "ignored".to_string(),

            // 第2引数: コンパイル対象のファイル名
            // MyFileLoader がこの名前に対してコードを提供する
            "main.rs".to_string(),
        ],
        // カスタムコールバックのインスタンスを渡す
        // &mut により、コールバック内で状態を変更できる
        &mut MyCallbacks,
    );
}
