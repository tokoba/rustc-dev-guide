# コンパイラソースの高レベル概要

[コンパイラが何をするか][orgch]を見てきたので、次にrustcのソースコードが存在する[`rust-lang/rust`]リポジトリの構造を見てみましょう。

[`rust-lang/rust`]: https://github.com/rust-lang/rust

> この章を読む前に、コンパイラがどのように動作するかを紹介する["Overview of the compiler"][orgch]章を読むと役立つかもしれません。

[orgch]: ./overview.md

## ワークスペース構造

[`rust-lang/rust`]リポジトリは、コンパイラ、標準ライブラリ([`core`]、[`alloc`]、[`std`]、[`proc_macro`]、[`etc`])、および[`rustdoc`]を含む単一の大きなcargoワークスペースで構成されており、ビルドシステムと完全なRustディストリビューションを構築するための多数のツールとサブモジュールも含まれています。

リポジトリは3つの主要なディレクトリで構成されています:

- [`compiler/`]は`rustc`のソースコードを含んでいます。これは、一緒にコンパイラを構成する多くのクレートで構成されています。

- [`library/`]は標準ライブラリ([`core`]、[`alloc`]、[`std`]、[`proc_macro`]、[`test`])、およびRustランタイム([`backtrace`]、[`rtstartup`]、[`lang_start`])を含んでいます。

- [`tests/`]はコンパイラテストを含んでいます。

- [`src/`]は[`rustdoc`]、[`clippy`]、[`cargo`]、ビルドシステム、言語ドキュメントなどのソースコードを含んでいます。

[`alloc`]: https://github.com/rust-lang/rust/tree/HEAD/library/alloc
[`backtrace`]: https://github.com/rust-lang/backtrace-rs/
[`cargo`]: https://github.com/rust-lang/cargo
[`clippy`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/clippy
[`compiler/`]: https://github.com/rust-lang/rust/tree/HEAD/compiler
[`core`]: https://github.com/rust-lang/rust/tree/HEAD/library/core
[`etc`]: https://github.com/rust-lang/rust/tree/HEAD/src/etc
[`lang_start`]: https://github.com/rust-lang/rust/blob/HEAD/library/std/src/rt.rs
[`library/`]: https://github.com/rust-lang/rust/tree/HEAD/library
[`proc_macro`]: https://github.com/rust-lang/rust/tree/HEAD/library/proc_macro
[`rtstartup`]: https://github.com/rust-lang/rust/tree/HEAD/library/rtstartup
[`rust-lang/rust`]: https://github.com/rust-lang/rust
[`rustdoc`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/rustdoc
[`src/`]: https://github.com/rust-lang/rust/tree/HEAD/src
[`std`]: https://github.com/rust-lang/rust/tree/HEAD/library/std
[`test`]: https://github.com/rust-lang/rust/tree/HEAD/library/test
[`tests/`]: https://github.com/rust-lang/rust/tree/HEAD/tests

## コンパイラ

コンパイラは、さまざまな[`compiler/`]クレートで実装されています。
[`compiler/`]クレートはすべて`rustc_*`で始まる名前を持っています。これらは、極小から巨大まで、約50の相互依存するクレートのコレクションです。実際のバイナリ(つまり`main`関数)である`rustc`クレートもありますが、これは[`rustc_driver`]クレートを呼び出す以外は何もしません。[`rustc_driver`]クレートは、他のクレートでのコンパイルのさまざまな部分を駆動します。

これらのクレートの依存関係順序は複雑ですが、大まかには次のようになります:

1. `rustc`(バイナリ)は[`rustc_driver::main`][main]を呼び出します。
1. [`rustc_driver`]は他の多くのクレートに依存していますが、主なものは[`rustc_interface`]です。
1. [`rustc_interface`]は他のほとんどのコンパイラクレートに依存しています。これは、コンパイル全体を駆動するためのかなり汎用的なインターフェースです。
1. 他のほとんどの`rustc_*`クレートは[`rustc_middle`]に依存しており、これはコンパイラ内の多くの中心的なデータ構造を定義しています。
1. [`rustc_middle`]と他のほとんどのクレートは、コンパイラの初期部分を表すいくつかのクレート(例:パーサー)、基本的なデータ構造(例:[`Span`])、またはエラー報告に依存しています:
   [`rustc_data_structures`]、[`rustc_span`]、[`rustc_errors`]など。

[`rustc_data_structures`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_data_structures/index.html
[`rustc_driver`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver/index.html
[`rustc_errors`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/index.html
[`rustc_interface`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/index.html
[`rustc_middle`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/index.html
[`rustc_span`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/index.html
[`Span`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/struct.Span.html
[main]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver/fn.main.html

正確な依存関係は、他のRustパッケージと同様に`cargo tree`を実行することで確認できます:

```console
cargo tree --package rustc_driver
```

最後に一つ: [`src/llvm-project`]はLLVMのフォークのサブモジュールです。
ブートストラップ中に、LLVMがビルドされ、[`compiler/rustc_llvm`]クレートには(C++で書かれた)LLVMのRustラッパーが含まれており、コンパイラがそれとインターフェースできるようになっています。

この本のほとんどはコンパイラに関するものなので、ここではこれらのクレートについてこれ以上説明しません。

[`compiler/rustc_llvm`]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_llvm
[`src/llvm-project`]: https://github.com/rust-lang/rust/tree/HEAD/src/
[`Cargo.toml`]: https://github.com/rust-lang/rust/blob/HEAD/Cargo.toml

### 全体像

コンパイラの依存関係構造は、2つの主な要因によって影響を受けています:

1. 組織化。コンパイラは_巨大な_コードベースです。1つの不可能に大きなクレートになってしまうでしょう。部分的には、依存関係構造はコンパイラのコード構造を反映しています。
2. コンパイル時間。コンパイラを複数のクレートに分割することで、cargoを使用したインクリメンタル/並列コンパイルをより有効に活用できます。特に、クレート間の依存関係をできるだけ少なくして、1つを変更した場合に再ビルドするクレートの数を減らすよう努めています。

依存関係ツリーの最下部には、コンパイラ全体で使用されるいくつかのクレート(例:[`rustc_span`])があります。コンパイルプロセスの非常に初期の部分(例:[パーシングと抽象構文木(`AST`)][parser])は、これらのみに依存しています。

[`AST`][parser]が構築され、他の初期分析が完了した後、コンパイラの[クエリシステム][query]がセットアップされます。クエリシステムは、関数ポインタを使用して巧妙にセットアップされます。これにより、クレート間の依存関係を壊し、より多くの並列コンパイルが可能になります。クエリシステムは[`rustc_middle`]で定義されているため、その後のコンパイラのほぼすべての部分がこのクレートに依存しています。これは非常に大きなクレートであり、コンパイル時間が長くなります。いくつかの試みがなされましたが、成功の度合いは様々です。別の副作用として、関連する機能が異なるクレートに散在することがあります。例えば、リント機能は初期の部分、[`rustc_lint`]、[`rustc_middle`]、およびその他の場所に見られます。

理想的には、インクリメンタルおよび並列コンパイルがコンパイル時間を妥当に保つことで、より少ない、より凝集性の高いクレートにすることができるでしょう。しかし、インクリメンタルおよび並列コンパイルはまだそこまで優れていないため、今のところ別々のクレートに分割することが私たちの解決策です。

依存関係ツリーの最上部には[`rustc_driver`]と[`rustc_interface`]があり、これはクエリシステムの不安定なラッパーで、コンパイルのさまざまな段階を駆動するのに役立ちます。コンパイラの他のコンシューマー(例:[`rustdoc`]または将来的には`rust-analyzer`)は、このインターフェースを異なる方法で使用する可能性があります。
[`rustc_driver`]クレートは、最初にコマンドライン引数を解析し、次に[`rustc_interface`]を使用してコンパイルを完了まで駆動します。

[parser]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/index.html
[`rustc_lint`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/index.html
[query]: ./query.md

## rustdoc

[`rustdoc`]のほとんどは[`librustdoc`]にあります。ただし、[`rustdoc`]バイナリ自体は[`src/tools/rustdoc`]であり、[`rustdoc::main`]を呼び出す以外は何もしません。

ドキュメント用の`JavaScript`と`CSS`も[`src/tools/rustdoc-js`]と[`src/tools/rustdoc-themes`]にあります。`--output-format=json`の型定義は[`src/rustdoc-json-types`]の別のクレートにあります。

[`rustdoc`]の詳細については、[this chapter][rustdoc-chapter]を参照してください。

[`librustdoc`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/index.html
[`rustdoc::main`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc/fn.main.html
[`src/tools/rustdoc-js`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/rustdoc-js
[`src/tools/rustdoc-themes`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/rustdoc-themes
[`src/tools/rustdoc`]:  https://github.com/rust-lang/rust/tree/HEAD/src/tools/rustdoc
[`src/rustdoc-json-types`]: https://github.com/rust-lang/rust/tree/HEAD/src/rustdoc-json-types
[rustdoc-chapter]: ./rustdoc.md

## テスト

上記のすべてのテストスイートは[`tests/`]にあります。テストスイートの詳細については、[in this chapter][testsch]を参照してください。

テストハーネスは[`src/tools/compiletest/`][`compiletest/`]にあります。

[`tests/`]: https://github.com/rust-lang/rust/tree/HEAD/tests
[testsch]: ./tests/intro.md

## ビルドシステム

リポジトリには、コンパイラ、標準ライブラリ、[`rustdoc`]などをビルドするためのツール、テスト、完全なRustディストリビューションのビルドなどのためのツールがいくつかあります。

主要なツールの1つは[`src/bootstrap/`]です。ブートストラップの詳細については、[in this chapter][bootstch]を参照してください。このプロセスでは、[`tidy/`]や[`compiletest/`]などの[`src/tools/`]の他のツールも使用される場合があります。

[`compiletest/`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/compiletest
[`src/bootstrap/`]: https://github.com/rust-lang/rust/tree/HEAD/src/bootstrap
[`src/tools/`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools
[`tidy/`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/tidy
[bootstch]: ./building/bootstrapping/intro.md

## 標準ライブラリ

このコードは、不安定な([`nightly`])機能を使用できることを除いて、他のほとんどのRustクレートとかなり似ています。
標準ライブラリは、[`libstd or the "standard facade"`]と呼ばれることがあります。

[`libstd or the "standard facade"`]: https://rust-lang.github.io/rfcs/0040-libstd-facade.html
[`nightly`]: https://doc.rust-lang.org/nightly/nightly-rustc/

## その他

`rust-lang/rust`リポジトリには、完全なRustディストリビューションのビルドに関連する他の多くのものがあります。ほとんどの場合、これらについて心配する必要はありません。

これらには次のものが含まれます:
- [`src/ci`]: CI設定。多くのプラットフォームで多くのテストを実行するため、これは実際にはかなり広範囲にわたります。
- [`src/doc`]: サブモジュールを含む様々なドキュメント。
- [`src/etc`]: その他のユーティリティ。
- その他...

[`src/ci`]: https://github.com/rust-lang/rust/tree/HEAD/src/ci
[`src/doc`]: https://github.com/rust-lang/rust/tree/HEAD/src/doc
[`src/etc`]: https://github.com/rust-lang/rust/tree/HEAD/src/etc
