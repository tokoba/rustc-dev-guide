# リント

このページでは、リント登録に関する機構と、
コンパイラでリントを実行する方法に関するいくつかのドキュメントを提供します。

[`LintStore`]は、すべてが回転する中心的なインフラストラクチャです。
`LintStore`は[`Session`]の一部として保持され、
`Session`が作成された直後にリントのリストが入力されます。

## リント対リントパス

コンパイラ内のリント機構には2つの部分があります：リントと
リントパス。残念ながら、私たちが持っているドキュメントの多くは、
両方を単に「リント」と呼んでいます。

まず、リント宣言自体があり、
ここに名前とデフォルトのリントレベルおよび他のメタデータが含まれます。
これらは通常、[`declare_lint!`]マクロによって定義されます。
これは、型[`&rustc_lint_defs::Lint`]の静的変数に要約されます
（ただし、これは将来変更される可能性があります。
マクロは、新しいフィールドを追加するのがやや扱いにくいためです。
すべてのマクロのように）。

<!-- date-check --> 2022年8月現在、
マクロを使用しない直接宣言に対してリントしています。

リント宣言は「状態」を持ちません - それらは単にグローバルな識別子と
リントの説明です。実行時に2回登録されないことをアサートします
（リント名によって）。

リントパスは、任意のリントの本質です。特に、
リントとリントパスの間に一対一の関係はありません；リントは、それを出力する
リントパスを持たない可能性があり、多くを持つ可能性があり、または1つだけを持つ可能性があります -- コンパイラは、
パスが特定のリントと何らかの形で関連しているかどうかを追跡しません。また、
リントは、他の作業の一部として頻繁に出力されます（例：型チェックなど）。

## 登録

### 高レベルの概要

[`rustc_interface::run_compiler`]では、
[`LintStore`]が作成され、
すべてのリントが登録されます。

リントには3つの「ソース」があります：

* 内部リント：rustcコードベースでのみ使用されるリント
* 組み込みリント：コンパイラに組み込まれており、外部ソースから提供されないリント
* `rustc_interface::Config`[`register_lints`]：構築中にコンパイラに渡されるリント

リントは、[`LintStore::register_lint`]関数を介して登録されます。これは、
任意のリントに対して一度だけ行うべきです。そうでないと、ICEが発生します。

登録が完了すると、`Arc`に配置することで、リントストアを「フリーズ」します。

リントパスは、カテゴリの1つ（展開前、早期、後期、後期モジュール）に
別々に登録されます。パスはクロージャとして登録されます --
つまり、`impl Fn() -> Box<dyn X>`。ここで、`dyn X`は早期または後期の
リントパストレイトオブジェクトです。リントパスを実行するとき、クロージャを実行し、
次にリントパスメソッドを呼び出します。リントパスメソッドは`&mut self`を取るため、
内部状態を追跡できます。

#### 内部リント

これらは、コンパイラまたは`clippy`のようなドライバーだけが使用するリントです。
[`rustc_lint::internal`]にあります。

このようなリントの例は、リントパスが`declare_lint_pass!`マクロを使用して実装されており、
手動で実装されていないことをチェックするものです。これは
`LINT_PASS_IMPL_WITHOUT_MACRO`リントで実現されています。

これらのリントの登録は、[`rustc_lint::new_lint_store`]内で呼び出される
[`rustc_lint::register_internals`]関数で行われます。

#### 組み込みリント

これらは主に2つの場所で説明されています。
`rustc_lint_defs::builtin`と`rustc_lint::builtin`。
多くの場合、最初のものはリント自体の定義を提供し、
後者はリントパスの定義（および実装）を提供しますが、
これは常に真実ではありません。

組み込みリントの登録は、
[`rustc_lint::register_builtins`]関数で行われます。
内部リントと同様に、
これは[`rustc_lint::new_lint_store`]内で行われます。

#### ドライバーリント

これらは、`rustc_interface::Config`[`register_lints`]フィールドを介して
ドライバーによって提供されるリントです。これはコールバックです。ドライバーは、
既に設定されている場合、追加するコールバック内で現在設定されている関数を呼び出すべきです。
ドライバーがこれにアクセスするための最良の方法は、`Callbacks::config`関数を
オーバーライドすることです。これにより、`Config`構造体への直接アクセスが得られます。

## コンパイラリントパスは1つのパスに結合されます

コンパイラ内では、パフォーマンス上の理由から、通常、数十の
リントパスを登録しません。代わりに、各種類の単一のリントパスがあります（例：
`BuiltinCombinedModuleLateLintPass`）。これは内部的にすべての
個々のリントパスを呼び出します；これは、（多くの場合空の）トレイトメソッドの
それぞれに対して動的ディスパッチではなく静的ディスパッチの利点を得るためです。

理想的には、これを行う必要はないでしょう。コードの理解の複雑さが増すためです。
ただし、現在の型消去されたリントストアアプローチでは、
パフォーマンス上の理由から有益です。

[`LintStore`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/struct.LintStore.html
[`LintStore::register_lint`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/struct.LintStore.html#method.register_lints
[`rustc_lint::register_builtins`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/fn.register_builtins.html
[`rustc_lint::register_internals`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/fn.register_internals.html
[`rustc_lint::new_lint_store`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/fn.new_lint_store.html
[`declare_lint!`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/macro.declare_lint.html
[`register_lints`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/interface/struct.Config.html#structfield.register_lints
[`&rustc_lint_defs::Lint`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint_defs/struct.Lint.html
[`Session`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/struct.Session.html
[`rustc_interface::run_compiler`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/index.html#reexport.run_compiler
[`rustc_lint::internal`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/internal/index.html
