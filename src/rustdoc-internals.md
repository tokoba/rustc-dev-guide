# Rustdoc internals

このページは、[`rustdoc`]のパスとモードについて説明します。
`rustdoc`の概要については、["Rustdoc overview"の章](./rustdoc.md)を参照してください。

[`rustdoc`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/rustdoc

## crateからcleanへ

[`core.rs`]には、2つの中心的なアイテムがあります：[`rustdoc::core::DocContext`]
構造体と、[`rustdoc::core::run_global_ctxt`]関数です。
後者は、`rustdoc`が引き継げる地点まで`rustc`を呼び出してクレートをコンパイルする場所です。
前者は、クレートをクロールしてドキュメントを収集する際に使用される状態コンテナです。

クレートのクロールの主要なプロセスは、[`clean/mod.rs`]で、`clean_`で始まる名前を持ついくつかの関数を通じて行われます。
各関数は`hir`または`ty`のデータ構造を受け取り、
`rustdoc`で使用される`clean`構造体を出力します。
たとえば、[ライフタイムを変換するこの関数]：

```rust,ignore
fn clean_lifetime<'tcx>(lifetime: &hir::Lifetime, cx: &mut DocContext<'tcx>) -> Lifetime {
    if let Some(
        rbv::ResolvedArg::EarlyBound(did)
        | rbv::ResolvedArg::LateBound(_, _, did)
        | rbv::ResolvedArg::Free(_, did),
    ) = cx.tcx.named_bound_var(lifetime.hir_id)
        && let Some(lt) = cx.args.get(&did).and_then(|arg| arg.as_lt())
    {
        return lt.clone();
    }
    Lifetime(lifetime.ident.name)
}
```

また、`clean/mod.rs`は、後でドキュメントページをレンダリングするために使用される「クリーン化された」[抽象構文木（`AST`）][ast]の型を定義します。
それぞれには通常、`rustc`から何らかの[`AST`][ast]または[高レベル中間表現（`HIR`）][hir]型を受け取り、
適切な「クリーン化された」型に変換する`clean_*`関数が付随します。
モジュールや関連アイテムのような「大きな」アイテムには、
`clean`関数に追加の処理がある場合がありますが、ほとんどの部分で
これらの`impl`は単純な変換です。
このモジュールへの「エントリポイント」は、
[`run_global_ctxt`]によって呼び出される[`clean::utils::krate`][ck0]です。

[`clean::utils::krate`][ck1]の最初のステップは、
[`visit_ast::RustdocVisitor`]を呼び出して、モジュールツリーを中間の[`visit_ast::Module`]に処理することです。
これは実際に[`rustc_hir::Crate`]をクロールして、名前解決のさまざまな側面を正規化するステップです。たとえば：

* `#[doc(inline)]`と`#[doc(no_inline)]`の処理
* importのglobとサイクルを処理して、重複や無限のディレクトリツリーがないようにする
* プライベートアイテムのpublic `use`エクスポートをインライン化するか、モジュールページに「Reexport」行を表示する
* ベースアイテムが非表示の場合は、`#[doc(hidden)]`を持つアイテムをインライン化する
* 再エクスポートとして定義されているかどうかに関係なく、クレートルートに`#[macro_export]`されたマクロを表示する

このステップの後、`clean::krate`は[`clean_doc_module`]を呼び出し、
実際に`HIR`アイテムをクリーン化された[`AST`][ast]に変換します。
これは、クロスクレートのインライン化が実行されるステップでもあり、
`rustc_middle`のデータ構造をクリーン化された[`AST`][ast]に変換する必要があります。

`clean/mod.rs`で起こるもう1つの主要なことは、
ドキュメントコメントと`#[doc=""]`属性を、
手書きのドキュメントを取得する任意のものに存在する[`Attributes`]構造体の別のフィールドに収集することです。
これにより、このドキュメントをプロセスの後半で収集しやすくなります。

このプロセスの主要な出力は、対象クレート内の公開ドキュメント化可能なアイテムを説明する
[`Item`]のツリーを持つ[`clean::types::Crate`]です。

[`Attributes`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/clean/types/struct.Attributes.html
[`clean_doc_module`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/clean/fn.clean_doc_module.html
[`clean::types::Crate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/clean/types/struct.Crate.html
[`clean/mod.rs`]: https://github.com/rust-lang/rust/blob/HEAD/src/librustdoc/clean/mod.rs
[`core.rs`]: https://github.com/rust-lang/rust/blob/HEAD/src/librustdoc/core.rs
[`Item`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/clean/types/struct.Item.html
[`run_global_ctxt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/core/fn.run_global_ctxt.html
[`rustc_hir::Crate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Crate.html
[`rustdoc::core::DocContext`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/core/struct.DocContext.html
[`rustdoc::core::run_global_ctxt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/core/fn.run_global_ctxt.html
[`visit_ast::Module`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/visit_ast/struct.Module.html
[`visit_ast::RustdocVisitor`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/visit_ast/struct.RustdocVisitor.html
[ast]: ./ast-validation.md
[ck0]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/clean/utils/fn.krate.html#
[ck1]: https://doc.rust-lang.org/nightly/nightly-rustc/src/rustdoc/clean/utils.rs.html#31-77
[hir]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/index.html
[ライフタイムを変換するこの関数]: https://doc.rust-lang.org/nightly/nightly-rustc/src/rustdoc/clean/mod.rs.html#256-267

### ガソリンスタンド以外のパス（または：[Hot Potato](https://www.youtube.com/watch?v=WNFBIt5HxdY)）

次の主要なステップに進む前に、クリーン化された[`AST`][ast]に対していくつかの重要な「パス」が発生します。
これらのパスのいくつかは`lint`とレポートですが、一部はアイテムを変更または生成します。

これらはすべて[`librustdoc/passes`]ディレクトリに実装されており、パスごとに1つのファイルがあります。
デフォルトでは、これらのパスはすべてクレートで実行されますが、
プライベート/非表示のアイテムをドロップするパスは、
`--document-private-items`を`rustdoc`に渡すことでバイパスできます。
以前の一連の[`AST`][ast]変換とは異なり、
パスは_クリーン化された_クレートで実行されることに注意してください。

<!-- date-check --> 2023年3月時点のパスのリストは次のとおりです：

* `calculate-doc-coverage`は、`--show-coverage`フラグに使用される情報を計算します。

* `check-doc-test-visibility`は、`doctest`の可視性関連の`lint`を実行します。このパスは
  `strip-private`の前に実行されるため、`run-lints`とは別にする必要があります。

* `collect-intra-doc-links`は[intra-docリンク](https://doc.rust-lang.org/nightly/rustdoc/write-documentation/linking-to-items-by-name.html)を解決します。

* `collect-trait-impls`は、クレート内の各アイテムの`trait` `impl`を収集します。
  たとえば、`trait`を実装する`struct`を定義すると、このパスは
  その`struct`がその`trait`を実装していることを記録します。

* `propagate-doc-cfg`は、`#[doc(cfg(...))]`を子アイテムに伝播します。

* `run-lints`は、`passes/lint`で定義された`rustdoc`の`lint`の一部を実行します。
  これは実行される最後のパスです。

  * `bare_urls`は、リンク化されていないリンクを検出します。たとえば、Markdownで
    `Go to https://example.com/.`のようなものです。リンクを角括弧で囲むことを提案します：
    `Go to <https://example.com/>.`でリンク化します。
    これは、<!-- date-check: may 2022 --> `rustdoc::bare_urls` `lint`の背後にあるコードです。

  * `check_code_block_syntax`は、Rustコードブロック内の構文を検証します
    (<code>```rust</code>)

  * `html_tags`は、ドキュメントコメント内の無効な`HTML`（閉じられていない`<span>`など）を検出します。

* `strip-hidden`と`strip-private`は、すべての`doc(hidden)`とプライベートアイテムを出力から削除します。
  `strip-private`は`strip-priv-imports`を含みます。
  基本的に、目標は公開ドキュメントに関連しないアイテムを削除することです。
  このパスは、`--document-hidden-items`が渡されたときにスキップされます。

* `strip-priv-imports`は、クレートからすべてのプライベートインポートステートメント（`use`、`extern
  crate`）を削除します。
  これは、`rustdoc`が*公開*インポートを処理するために必要です。
  アイテムのドキュメントをモジュールにインライン化するか、
  インポートを含む「Reexports」セクションを作成します。
  このパスは、これらすべてのインポートが実際にドキュメントに関連していることを保証します。
  技術的には`--document-private-items`が渡されたときにのみ実行されますが、
  `strip-private`も同じことを達成します。

* `strip-private`は、外部から見えないすべてのプライベートアイテムをクレートから削除します。
  このパスは、`--document-private-items`が渡されたときにスキップされます。

`librustdoc/passes`には[`stripper`]モジュールもありますが、
これは`strip-*`パスのユーティリティ関数のコレクションであり、パス自体ではありません。

[`librustdoc/passes`]: https://github.com/rust-lang/rust/tree/HEAD/src/librustdoc/passes
[`stripper`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/passes/stripper/index.html

## cleanからHTMLへ

ここから、`rustdoc`の「第2フェーズ」が始まります。
このフェーズは主に[`librustdoc/formats`]と[`librustdoc/html`]フォルダに存在し、
すべて[`formats::renderer::run_format`]から始まります。
このコードは、`impl FormatRenderer`する型を設定する責任があり、
`HTML`の場合は[`Context`]です。

この構造体には、`run_format`によって呼び出されてドキュメントのレンダリングを駆動するメソッドが含まれています：

* `init`は、`static.files`、検索インデックス、`src/`を生成します
* `item`は、アイテムの`HTML`ファイル自体を生成します
* `after_krate`は、`all.html`のような他のグローバルリソースを生成します

`item`では、[Askama]テンプレートと手動の`write!()`呼び出しの組み合わせを介して、
[`html/layout.rs`]から始まる「ページレンダリング」が発生します。
テンプレートに変換されていない部分は、
一連の`std::fmt::Display`実装と、
`&mut std::fmt::Formatter`を渡す関数内で発生します。

アイテムとドキュメントから実際に`HTML`を生成する部分は、
[`html/render/print_item.rs`]で定義されている[`print_item`]から始まり、
レンダリングされる`Item`の種類に基づいて、いくつかの`item_*`関数のいずれかに切り替わります。

どの種類のレンダリングコードを探しているかによって、
おそらく[`html/render/mod.rs`]で「`struct`ページにどのセクションを印刷すべきか」のような主要なアイテムについて見つけるか、
[`html/format.rs`]で「他のアイテムの一部としてwhere句をどのように印刷すべきか」のような小さなコンポーネントピースについて見つけるでしょう。

`rustdoc`が手書きのドキュメントと一緒に印刷すべきアイテムに遭遇すると、
Markdownパーサーとインターフェースする[`html/markdown.rs`]を呼び出します。
これは、Markdownの文字列をラップし、`HTML`テキストを出力するために`fmt::Display`を実装する一連の型として公開されます。
Markdownパーサーを実行する前に、脚注とテーブルなどの特定の機能を有効にし、
Rustコードブロックに構文ハイライトを追加する（`html/highlight.rs`経由）ように特別な注意を払います。
また、[`find_codes`]という関数もあり、
これは`find_testable_codes`によって呼び出され、
Rustコードブロックを特にスキャンして、
テストランナーコードがクレート内のすべての`doctest`を見つけられるようにします。

[`find_codes`]: https://doc.rust-lang.org/nightly/nightly-rustc/src/rustdoc/html/markdown.rs.html#749-818
[`formats::renderer::run_format`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/formats/renderer/fn.run_format.html
[`html/format.rs`]: https://github.com/rust-lang/rust/blob/HEAD/src/librustdoc/html/format.rs
[`html/layout.rs`]: https://github.com/rust-lang/rust/blob/HEAD/src/librustdoc/html/layout.rs
[`html/markdown.rs`]: https://github.com/rust-lang/rust/blob/HEAD/src/librustdoc/html/markdown.rs
[`html/render/mod.rs`]: https://github.com/rust-lang/rust/blob/HEAD/src/librustdoc/html/render/mod.rs
[`html/render/print_item.rs`]: https://github.com/rust-lang/rust/blob/HEAD/src/librustdoc/html/render/print_item.rs
[`librustdoc/formats`]: https://github.com/rust-lang/rust/tree/HEAD/src/librustdoc/formats
[`librustdoc/html`]: https://github.com/rust-lang/rust/tree/HEAD/src/librustdoc/html
[`print_item`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/html/render/print_item/fn.print_item.html
[Askama]: https://docs.rs/askama/latest/askama/

### スープからナッツまで（または：["最初の`Cell`から私たちまで続く途切れない糸"][video]）

[video]: https://www.youtube.com/watch?v=hOLAGYmUQV0

`HTML`生成中であっても、`rustdoc`はコンパイラに型情報を直接要求できることに注意することが重要です。
これは[以前はそうではありませんでした]が、
`rustdoc`のアーキテクチャの多くはそうしないように設計されていましたが、
現在は`TyCtxt`が`formats::renderer::run_format`に渡され、
`HTML`と（<!-- date-check --> 2025年11月時点で不安定な）JSON形式の両方の生成を実行するために使用されます。

この変更により、`TyCtxt`クエリから簡単に導出できる「clean」[`AST`][ast]からデータを削除する他の変更が可能になり、
通常、「clean」からフィールドを削除するPRを受け入れます（ソフト非推奨になっています）が、
これは`rustdoc`が実行される他の2つの制約によって複雑になります：

* 実際に型チェックをパスしないクレートに対してドキュメントを生成できます。
  これは、相互に排他的なプラットフォーム構成をカバーするドキュメントを生成するために使用されます。
  たとえば、`libstd`には、サポートされているすべてのオペレーティングシステムをカバーする単一のドキュメントパッケージがあります。
  これは、`rustdoc`が`HIR`からドキュメントを生成できる必要があることを意味します。
* ドキュメントはクレート間でインライン化できます。クレートメタデータには`HIR`が含まれていないため、
  インライン化されたドキュメントを`rustc_middle`データから生成できる必要があります。

「clean」[`AST`][ast]は、両方の入力形式の共通出力形式として機能します。
cleanには`HIR`に直接対応しないデータもあります。たとえば、
`collect-trait-impls`パスによって生成されたautoトレイトとblanket `impl`の合成`impl`などです。

一部の追加データは、`html::render::context::{Context, SharedContext}`に格納されます。
これら2つの型は、
マルチスレッドドキュメント生成を備えた最終的な将来のために`rustdoc`のデータを分離する方法として機能し、
また物事を整理するためにも機能します：

* [`Context`]は、現在のページを生成するために使用されるデータを格納します。たとえば、
  そのパス、使用された`HTML` IDのリスト（重複する`id=""`を回避するため）、
  `SharedContext`へのポインタなどです。
* [`SharedContext`]は、ページごとに変化しないデータを格納します。たとえば、
  `tcx`ポインタ、すべての型のリストなどです。

[`Context`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/html/render/context/struct.Context.html
[以前はそうではありませんでした]: https://github.com/rust-lang/rust/pull/80090
[`SharedContext`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/html/render/context/struct.SharedContext.html

## その他のトリック

これらすべては、RustクレートからHTMLドキュメントを生成するプロセスを説明していますが、
`rustdoc`が実行される他のいくつかの主要なモードがあります。
スタンドアロンのMarkdownファイルで実行することもできますし、
RustコードまたはスタンドアロンのMarkdownファイルで`doctest`を実行することもできます。
前者の場合、
`html/markdown.rs`に直接ショートカットします。
オプションで、出力`HTML`に目次を挿入するモードを含みます。

後者の場合、`rustdoc`は、
`test.rs`で関連するドキュメントを取得するために同様の部分コンパイルを実行しますが、
完全なクリーンとレンダリングのプロセスを経る代わりに、
手書きのドキュメントだけを取得するために、はるかに単純なクレートウォークを実行します。
前述の`html/markdown.rs`の"`find_testable_code`"と組み合わせて、
テストランナーに渡す前に実行するテストのコレクションを構築します。
`test.rs`の注目すべき場所の1つは、`make_test`関数です。
ここで、手書きの`doctest`が実行可能なものに変換されます。

`make_test`についての追加の読み物は
[こちら](https://quietmisdreavus.net/code/2018/02/23/how-the-doctests-get-made/)にあります。

## ローカルでのテスト

生成された`HTML`ドキュメントの一部の機能は、
ページ間で使用されるローカルストレージを必要とする場合があり、
これは`HTTP`サーバーなしではうまく機能しません。
これらの機能をローカルでテストするには、次のようにローカル`HTTP`サーバーを実行できます：

```console
$ ./x doc library
# ドキュメントは`build/[YOUR ARCH]/doc`に生成されました。
$ python3 -m http.server -d build/[YOUR ARCH]/doc
```

これで、インターネット上でホストされているかのようにドキュメントを閲覧できます。
たとえば、`std`のURLは`rust/std/`になります。

## 参照

* [`rustdoc` APIドキュメント]
* [`rustdoc`の概要](./rustdoc.md)
* [rustdocユーザーガイド]

[`rustdoc` APIドキュメント]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/
[rustdocユーザーガイド]: https://doc.rust-lang.org/nightly/rustdoc/
