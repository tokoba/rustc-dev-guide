//! # Rustコンパイラから診断情報を取得するサンプルプログラム
//!
//! このサンプルは、rustc_interfaceを使用してRustコンパイラを実行し、
//! 型エラーなどの診断情報をプログラム内で取得する方法を示しています。
//!
//! ## 主な用途
//! - IDEやエディタプラグインでのコンパイルエラー表示
//! - 自動コード検証ツールの実装
//! - カスタムリンターの開発
//!
//! ## 動作概要
//! 1. カスタムエミッタ（DebugEmitter）を作成し、診断情報をバッファに格納
//! 2. 意図的に型エラーを含むコードをコンパイル
//! 3. コンパイラが検出したエラーをバッファから取得して表示
//!
//! nightly-2025-03-28 でテスト済み

#![feature(rustc_private)]

extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_span;

use std::sync::{Arc, Mutex};

use rustc_errors::emitter::Emitter;
use rustc_errors::registry::{self, Registry};
use rustc_errors::translation::Translate;
use rustc_errors::{DiagInner, FluentBundle};
use rustc_session::config;
use rustc_span::source_map::SourceMap;

/// カスタム診断エミッタ
///
/// このエミッタは、コンパイラが生成した診断情報（エラー、警告など）を
/// 標準エラー出力に表示する代わりに、メモリ上のバッファに蓄積します。
///
/// # フィールド
/// - `source_map`: ソースコードの位置情報を管理するマップ。
///   エラー位置の特定やコードスニペットの表示に使用します。
/// - `diagnostics`: 診断情報を格納する共有バッファ。
///   スレッドセーフな操作のため Arc<Mutex<>> でラップしています。
///
/// # 使用例
/// ```rust,ignore
/// let buffer: Arc<Mutex<Vec<DiagInner>>> = Arc::default();
/// let diagnostics = buffer.clone();
/// let emitter = DebugEmitter {
///     source_map: parse_sess.clone_source_map(),
///     diagnostics,
/// };
/// ```
struct DebugEmitter {
    source_map: Arc<SourceMap>,
    diagnostics: Arc<Mutex<Vec<DiagInner>>>,
}

/// Translateトレイトの実装
///
/// このトレイトは、診断メッセージの国際化（i18n）をサポートするためのものです。
/// DebugEmitterでは翻訳機能を使用しないため、シンプルな実装となっています。
impl Translate for DebugEmitter {
    /// Fluentバンドルの取得
    ///
    /// Fluentは Mozilla が開発した国際化フレームワークです。
    /// このエミッタでは翻訳を行わないため、常に None を返します。
    ///
    /// # 戻り値
    /// 常に `None` を返します（翻訳機能を使用しないため）。
    fn fluent_bundle(&self) -> Option<&FluentBundle> {
        None
    }

    /// フォールバック用のFluentバンドルの取得
    ///
    /// このメソッドは、通常のバンドルが利用できない場合の
    /// フォールバック処理として呼び出されます。
    /// DebugEmitterでは翻訳を想定していないため、
    /// このメソッドが呼ばれた場合はパニックします。
    ///
    /// # パニック
    /// このメソッドが呼ばれた場合、常にパニックします。
    fn fallback_fluent_bundle(&self) -> &FluentBundle {
        panic!("this emitter should not translate message")
    }
}

/// Emitterトレイトの実装
///
/// Emitterトレイトは、rustcのコンパイラが生成する診断情報を
/// どのように処理するかを定義するインターフェースです。
/// 通常はターミナルに色付きで出力しますが、ここではメモリに保存します。
impl Emitter for DebugEmitter {
    /// 診断情報の出力処理
    ///
    /// コンパイラがエラーや警告を検出するたびに、このメソッドが呼び出されます。
    /// 標準のエミッタは診断情報を標準エラー出力に表示しますが、
    /// このカスタムエミッタは診断情報をベクタに追加して保存します。
    ///
    /// # 引数
    /// - `diag`: 診断情報本体（エラーメッセージ、重大度レベル、位置情報など）
    /// - `_`: レジストリ（診断コードの詳細情報を含む）。今回は使用しないため無視。
    ///
    /// # 処理の流れ
    /// 1. diagnosticsフィールドのMutexをロック（排他制御）
    /// 2. unwrap()でロックが成功したことを確認
    /// 3. 診断情報をベクタにpush
    ///
    /// # スレッドセーフ性
    /// Mutexを使用しているため、複数のスレッドから同時に呼び出されても安全です。
    fn emit_diagnostic(&mut self, diag: DiagInner, _: &Registry) {
        // Mutexをロックして排他的アクセスを確保
        // コンパイラは並列処理を行うため、複数スレッドからの同時アクセスを防ぐ必要がある
        self.diagnostics.lock().unwrap().push(diag);
    }

    /// ソースマップの取得
    ///
    /// ソースマップは、診断情報に含まれるバイトオフセットを
    /// 実際のファイル名、行番号、列番号に変換するために使用されます。
    ///
    /// # 戻り値
    /// SourceMapへの参照を返します。これにより、エラー位置の
    /// 人間が読める形式への変換が可能になります。
    ///
    /// # 使用例
    /// エラーメッセージを表示する際、「バイト位置123」ではなく
    /// 「main.rs:5:10」のような形式で位置を示すために使用されます。
    fn source_map(&self) -> Option<&SourceMap> {
        Some(&self.source_map)
    }
}

/// メイン関数
///
/// このプログラムは、意図的に型エラーを含むRustコードをコンパイルし、
/// コンパイラが検出したエラーを取得して表示します。
///
/// # 処理の流れ
/// 1. 診断情報を格納するバッファを作成
/// 2. rustc_interfaceのConfigを設定（カスタムエミッタを含む）
/// 3. 型エラーを含むコードをコンパイル実行
/// 4. 収集された診断情報を表示
///
/// # 実行例
/// ```text
/// DiagInner {
///     level: Error,
///     messages: [("mismatched types", ...)],
///     code: Some(E0308),
///     span: main.rs:3:19: 3:20,
///     ...
/// }
/// ```
fn main() {
    // 診断情報を格納するための共有バッファを作成
    // Arc（Atomic Reference Count）により、複数の所有者でバッファを共有可能
    // Mutexにより、複数スレッドからの安全なアクセスを保証
    let buffer: Arc<Mutex<Vec<DiagInner>>> = Arc::default();

    // バッファのクローンを作成し、エミッタに渡す用に保持
    // Arcはポインタのクローンなので、実際のデータは複製されない
    let diagnostics = buffer.clone();

    // rustcコンパイラの設定を構築
    let config = rustc_interface::Config {
        // コマンドライン引数相当のオプション（デフォルト設定を使用）
        opts: config::Options::default(),

        // コンパイル対象のソースコード
        // このプログラムには意図的に型エラーが含まれている：
        // `let x: &str = 1;` は &str 型の変数に整数リテラル 1 を代入しようとしている
        input: config::Input::Str {
            name: rustc_span::FileName::Custom("main.rs".into()),
            input: "
fn main() {
    let x: &str = 1;
}
"
            .into(),
        },

        // cfg!マクロで使用される設定（空）
        crate_cfg: Vec::new(),

        // cfg!チェック用の設定（空）
        crate_check_cfg: Vec::new(),

        // 出力ディレクトリ（指定なし）
        output_dir: None,

        // 出力ファイル（指定なし）
        output_file: None,

        // カスタムファイルローダー（指定なし、デフォルトのファイルシステムを使用）
        file_loader: None,

        // ロケールリソース（エラーメッセージの言語設定など）
        locale_resources: rustc_driver::DEFAULT_LOCALE_RESOURCES.to_owned(),

        // リントレベルの上限設定（空のハッシュマップ）
        lint_caps: rustc_hash::FxHashMap::default(),

        // パースセッション作成時のコールバック
        // ここでカスタムエミッタを設定し、診断情報をバッファに格納するようにする
        psess_created: Some(Box::new(|parse_sess| {
            // 診断コンテキスト（dcx）に対してカスタムエミッタを設定
            // これにより、全ての診断情報がDebugEmitterを通じて処理される
            parse_sess.dcx().set_emitter(Box::new(DebugEmitter {
                // ソースマップをクローン（エラー位置の特定に必要）
                source_map: parse_sess.clone_source_map(),
                // 診断情報格納用のバッファ
                diagnostics,
            }));
        })),

        // リント登録用のコールバック（指定なし）
        register_lints: None,

        // クエリオーバーライド用のコールバック（指定なし）
        override_queries: None,

        // 診断コードのレジストリ（エラーコードの説明情報など）
        registry: registry::Registry::new(rustc_errors::codes::DIAGNOSTICS),

        // コード生成バックエンドのカスタマイズ（指定なし）
        make_codegen_backend: None,

        // 展開された引数リスト（空）
        expanded_args: Vec::new(),

        // ICE（Internal Compiler Error）レポート用のファイル（指定なし）
        ice_file: None,

        // トラッキングされない状態のハッシュ値（指定なし）
        hash_untracked_state: None,

        // 内部機能使用フラグ
        using_internal_features: &rustc_driver::USING_INTERNAL_FEATURES,
    };

    // コンパイラを実行
    // このクロージャ内でコンパイルの各フェーズを制御できる
    rustc_interface::run_compiler(config, |compiler| {
        // ソースコードをパースして抽象構文木（AST）を取得
        let krate = rustc_interface::passes::parse(&compiler.sess);

        // グローバルコンテキストを作成し、型チェックを実行
        rustc_interface::create_and_enter_global_ctxt(&compiler, krate, |tcx| {
            // 全ての関数本体に対して型チェックを並列実行
            // par_hir_body_owners は並列イテレータを提供し、
            // 複数のCPUコアを活用して高速に処理する
            tcx.par_hir_body_owners(|item_def_id| {
                // 各アイテムの型チェックを実行
                // ensure_ok() は型エラーがあっても処理を続行する
                // （エラー情報はエミッタを通じてバッファに蓄積される）
                tcx.ensure_ok().typeck(item_def_id);
            });
        });

        // 重要: コンパイラがエラーを検出している場合、
        // 通常はこのクロージャの終了時にabort()が呼ばれプログラムが強制終了する
        // これを回避するため、エラーカウントをリセットして正常終了させる
        compiler.sess.dcx().reset_err_count();
    });

    // バッファに蓄積された診断情報を読み出して表示
    // for_each を使用して各診断情報を処理
    buffer.lock().unwrap().iter().for_each(|diagnostic| {
        // {:#?} はDebugトレイトの「きれいな」フォーマット
        // 診断情報を構造化された形式で表示
        println!("{diagnostic:#?}");
    });
}
