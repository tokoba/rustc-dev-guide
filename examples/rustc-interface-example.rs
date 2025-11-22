//! # rustc_interface 基本サンプルプログラム
//!
//! このサンプルは、rustc_interfaceを使用してRustコンパイラを
//! プログラム内から実行する基本的な方法を示しています。
//!
//! ## 主な用途
//! - コンパイラをライブラリとして組み込む
//! - コード解析ツールやリンターの基盤
//! - メタプログラミングツールの開発
//! - 教育目的でのコンパイラ動作の理解
//!
//! ## 動作概要
//! 1. メモリ内のコードをコンパイラに渡す
//! 2. 抽象構文木（AST）をパースして表示
//! 3. 型情報を解析してアイテムの型を表示
//!
//! nightly-2025-03-28 でテスト済み

#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_span;

use rustc_errors::registry;
use rustc_hash::FxHashMap;
use rustc_session::config;

/// メイン関数
///
/// rustc_interfaceを使用してRustコードをコンパイルし、
/// その過程で抽象構文木と型情報を取得します。
///
/// # 処理の流れ
/// 1. Config構造体を作成してコンパイラの設定を行う
/// 2. メモリ内のソースコードをInput::Strで渡す
/// 3. run_compiler でコンパイルを実行
/// 4. パースフェーズでASTを取得・表示
/// 5. 型解析フェーズで型情報を取得・表示
///
/// # サンプルコード
/// このプログラムがコンパイルするコードは：
/// ```rust,ignore
/// static HELLO: &str = "Hello, world!";
/// fn main() {
///     println!("{HELLO}");
/// }
/// ```
///
/// # 出力例
/// ```text
/// Crate { ... }  // ASTの構造
/// Ident { name: "HELLO", ... }: &str
/// Ident { name: "main", ... }: fn()
/// ```
fn main() {
    // コンパイラの設定を構築
    // rustc_interface::Config はコンパイルに必要な全ての設定を保持
    let config = rustc_interface::Config {
        // コマンドラインオプション
        // デフォルト設定を使用（最適化レベル、デバッグ情報など）
        opts: config::Options::default(),

        // cfg! マクロで使用される条件コンパイル設定
        // 例: cfg!(target_os = "linux") などで使用される
        // 空のベクタは追加のcfg設定がないことを意味する
        crate_cfg: Vec::new(), // FxHashSet<(String, Option<String>)>

        // check_cfg で使用される設定検証用の情報
        // cargo.toml の [lints.rust] check-cfg に相当
        // 不正なcfg設定を検出するために使用される
        crate_check_cfg: Vec::new(), // CheckCfg

        // コンパイル対象の入力
        // Input::Str を使うことで、ファイルを作成せずにメモリ内のコードをコンパイル可能
        input: config::Input::Str {
            // ファイル名（仮想的な名前、エラーメッセージなどで使用される）
            name: rustc_span::FileName::Custom("main.rs".into()),

            // コンパイルするソースコード
            // ここでは静的変数とmain関数を含むシンプルなプログラム
            // - HELLO: グローバルな文字列スライス
            // - main: エントリーポイント関数
            input: r#"
static HELLO: &str = "Hello, world!";
fn main() {
    println!("{HELLO}");
}
"#
            .into(),
        },

        // コンパイル成果物の出力先ディレクトリ
        // None の場合、カレントディレクトリを使用
        // このサンプルでは実際にファイルを出力しないため None
        output_dir: None, // Option<PathBuf>

        // 出力ファイルのパス
        // 実行ファイルや .rlib ファイルの出力先を指定
        // None の場合、デフォルトの命名規則が使用される
        output_file: None, // Option<PathBuf>

        // カスタムファイルローダー
        // ファイルシステムからのファイル読み込みをカスタマイズ可能
        // None の場合、標準のファイルシステムローダーが使用される
        file_loader: None, // Option<Box<dyn FileLoader + Send + Sync>>

        // ロケールリソース（エラーメッセージの言語設定など）
        // rustc_driver::DEFAULT_LOCALE_RESOURCES は標準の英語メッセージを提供
        locale_resources: rustc_driver::DEFAULT_LOCALE_RESOURCES.to_owned(),

        // リントレベルの上限設定
        // 特定のリントが特定のレベル以上に設定されないように制限
        // 例: allow(dead_code) を deny にできないようにするなど
        // 空のマップは制限なしを意味する
        lint_caps: FxHashMap::default(), // FxHashMap<lint::LintId, lint::Level>

        // パースセッション作成時のコールバック
        // ParseSess が作成された直後に呼び出される
        // カスタムエミッタの設定などに使用
        // このサンプルでは使用しない
        psess_created: None, //Option<Box<dyn FnOnce(&mut ParseSess) + Send>>

        // リント登録用のコールバック
        // プラグインやカスタムリントを登録する際に使用
        // コンパイラのリントストアが非共有状態の時に呼び出される
        //
        // 注意: 既存のコールバックがある場合、新しいコールバック内で
        // それも呼び出す必要がある
        register_lints: None, // Option<Box<dyn Fn(&Session, &mut LintStore) + Send + Sync>>

        // クエリシステムのオーバーライド用コールバック
        // rustcの内部クエリシステム（遅延評価・メモ化機構）を
        // カスタマイズする際に使用
        //
        // 第2パラメータ: ローカルプロバイダ（現在のクレート用）
        // 第3パラメータ: 外部プロバイダ（依存クレート用）
        override_queries: None, // Option<fn(&Session, &mut ty::query::Providers<'_>, &mut ty::query::Providers<'_>)>

        // 診断コードのレジストリ
        // エラーコード（E0001など）の説明テキストを提供
        // rustc_errors::codes::DIAGNOSTICS は標準のエラーコード情報を含む
        registry: registry::Registry::new(rustc_errors::codes::DIAGNOSTICS),

        // コード生成バックエンドのカスタマイズ
        // デフォルトのLLVMバックエンドの代わりに、カスタムバックエンドを使用可能
        // None の場合、デフォルトのバックエンドが使用される
        make_codegen_backend: None,

        // 展開されたコマンドライン引数
        // @argfile 展開後の引数リスト
        // 通常は空で問題ない
        expanded_args: Vec::new(),

        // ICE（Internal Compiler Error）レポート用のファイルパス
        // コンパイラがクラッシュした場合、このファイルに詳細情報を出力
        // None の場合、デフォルトのパスが使用される
        ice_file: None,

        // トラッキングされない状態のハッシュ値
        // インクリメンタルコンパイルで使用される
        // None の場合、デフォルトの動作
        hash_untracked_state: None,

        // 内部機能使用フラグ
        // rustc の内部APIを使用していることを示すフラグ
        // 安定性保証の対象外であることを明示
        using_internal_features: &rustc_driver::USING_INTERNAL_FEATURES,
    };

    // コンパイラを実行
    // クロージャ内でコンパイルの各フェーズにアクセス可能
    rustc_interface::run_compiler(config, |compiler| {
        // フェーズ1: ソースコードをパースして抽象構文木（AST）を取得
        // parse() はレキシング（字句解析）とパース（構文解析）を実行
        // - レキシング: ソースコードをトークン列に分割
        // - パース: トークン列からAST構造を構築
        let krate = rustc_interface::passes::parse(&compiler.sess);

        // ASTをDebugフォーマットで表示
        // {krate:?} は AST の構造を出力（非常に詳細）
        // AST には以下の情報が含まれる：
        // - モジュール構造
        // - 関数、構造体、trait の定義
        // - 式、文、パターンの階層構造
        println!("{krate:?}");

        // フェーズ2: 型解析と型推論を実行
        // create_and_enter_global_ctxt は以下を実行：
        // 1. 名前解決（識別子を定義に結びつける）
        // 2. 型チェック（型の整合性を検証）
        // 3. 借用チェック（所有権・ライフタイムの検証）
        // 4. その他の静的解析
        rustc_interface::create_and_enter_global_ctxt(&compiler, krate, |tcx| {
            // tcx (Type Context) は型情報への中心的なアクセスポイント
            // 全ての型情報、HIR、定義情報にアクセス可能

            // クレート内の全てのフリーアイテムをイテレート
            // フリーアイテム: impl ブロック内ではないトップレベルのアイテム
            // - 関数定義
            // - 構造体、enum、trait 定義
            // - static、const 定義
            // - モジュール定義
            for id in tcx.hir_free_items() {
                // アイテムのHIR（高レベル中間表現）を取得
                // HIR は AST よりも型情報が付加され、構造化されている
                let item = tcx.hir_item(id);

                // アイテムの種類でマッチング
                match item.kind {
                    // static 変数または関数の場合
                    rustc_hir::ItemKind::Static(ident, ..)
                    | rustc_hir::ItemKind::Fn { ident, .. } => {
                        // アイテムの型を取得
                        // type_of() はクエリシステムを通じて型情報を取得
                        // - Static の場合: 変数の型（例: &str）
                        // - Fn の場合: 関数の型（例: fn()）
                        let ty = tcx.type_of(item.hir_id().owner.def_id);

                        // アイテム名と型を表示
                        // {:?} はDebugトレイトを使用した簡潔な表示
                        // 例: Ident { name: "HELLO", ... }: &str
                        println!("{ident:?}:\t{ty:?}")
                    }

                    // その他のアイテム（struct, enum, trait など）は無視
                    _ => (),
                }
            }
        });
    });
}
