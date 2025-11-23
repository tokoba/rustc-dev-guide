# `rustc_driver`と`rustc_interface`

## `rustc_driver`

[`rustc_driver`]は本質的に`rustc`の`main`関数です。
これは、[`rustc_interface`]クレートで定義されたインターフェースを使用して、
コンパイラのさまざまなフェーズを正しい順序で実行するための接着剤として機能します。可能な限り、[`rustc_interface`]ではなく[`rustc_driver`]を使用することが推奨されます。

[`rustc_driver`]のメインエントリーポイントは[`rustc_driver::run_compiler`][rd_rc]です。
このビルダーは、rustcと同じコマンドライン引数、[`Callbacks`]の実装、およびいくつかの他のオプションのオプションを受け入れます。
[`Callbacks`]は、カスタムコンパイラ設定を可能にする`trait`であり、
コンパイルのさまざまなフェーズの後にカスタムコードを実行することもできます。

## `rustc_interface`

[`rustc_interface`]クレートは、コンパイルプロセスを手動で駆動するための低レベルAPIを外部ユーザーに提供し、
サードパーティがクレートを分析したり、[`rustc_driver`]が十分に柔軟でない場合（例：`rustdoc`がコードをコンパイルして出力を提供する場合）のコンパイラのアドホックなエミュレーションのために、`rustc`の内部をライブラリとして効果的に使用できるようにします。

[`rustc_interface`]のメインエントリーポイント（[`rustc_interface::run_compiler`][i_rc]）は、コンパイラの設定変数と、
まだ解決されていない[`Compiler`]を受け取る`closure`を受け取ります。
[`run_compiler`][i_rc]は、設定から`Compiler`を作成し、それを`closure`に渡します。
`closure`内では、`Compiler`を使用してさまざまな関数を呼び出し、クレートをコンパイルして結果を取得できます。
[`rustc_interface`]の使用方法の最小限の例は[ここ][example]で確認できます。

[`rustc_interface`]を使用するさまざまな関数の使用例は、`rustc_driver`の実装、
特に[`rustc_driver_impl::run_compiler`][rdi_rc]
（[`rustc_interface::run_compiler`][i_rc]と混同しないでください）を見ることで確認できます。

> **警告：** その性質上、内部コンパイラAPIは常に
> 不安定です。とはいえ、不必要に壊さないように努めています。

[`Compiler`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/interface/struct.Compiler.html
[`rustc_driver`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver/
[`rustc_interface`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/index.html
[`Callbacks`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver/trait.Callbacks.html
[example]: https://github.com/rust-lang/rustc-dev-guide/blob/main/examples/rustc-interface-example.rs
[i_rc]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/interface/fn.run_compiler.html
[rd_rc]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver/fn.run_compiler.html
[rdi_rc]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver_impl/fn.run_compiler.html
