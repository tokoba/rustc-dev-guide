# `rustdoc-gui`テストスイート

> **FIXME**: このセクションはスタブです。充実させるのを手伝ってください！

このページは、`rustdoc`の「GUI」（つまり、ブラウザでレンダリングされるHTML/JS/CSS）をテストするために使用される`rustdoc-gui`という名前のテストスイートについて説明します。
他のrustdoc固有のテストスイートについては、[Rustdocテストスイート]を参照してください。

これらは、[puppeteer]を使用してヘッドレスブラウザでテストを実行し、レンダリングとインタラクティビティをチェックする[`browser-UI-test`]と呼ばれるNodeJSベースのツールを使用します。この形式のテストを書く方法については、[`tests/rustdoc-gui/README.md`][rustdoc-gui-readme]と[`.goml`形式の説明][goml-script]を参照してください。

[Rustdocテストスイート]: ../tests/compiletest.md#rustdoc-test-suites
[`browser-UI-test`]: https://github.com/GuillaumeGomez/browser-UI-test/
[puppeteer]: https://pptr.dev/
[rustdoc-gui-readme]: https://github.com/rust-lang/rust/blob/HEAD/tests/rustdoc-gui/README.md
[goml-script]: https://github.com/GuillaumeGomez/browser-UI-test/blob/main/goml-script.md
