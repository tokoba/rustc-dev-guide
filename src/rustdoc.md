# Rustdoc概要

`rustdoc`は、コンパイラと標準ライブラリと共にツリー内に存在します。
この章は、その仕組みについて説明します。
Rustdocの機能とその使用方法については、
[Rustdocブック](https://doc.rust-lang.org/nightly/rustdoc/)を参照してください。
rustdocの動作の詳細については、["Rustdoc internals"の章][Rustdoc internals]を参照してください。

[Rustdoc internals]: ./rustdoc-internals.md

`rustdoc`は`rustc`の内部機構（もちろん標準ライブラリも）を使用するため、
`rustdoc`をビルドする前に、コンパイラと`std`を一度ビルドする必要があります。

Rustdocは完全にクレート[`librustdoc`][rd]内に実装されています。
クレートの内部表現（HIR）と、アイテムの型についてのクエリを実行する機能を得るまで、
コンパイラを実行します。
[HIR]と[クエリ][queries]については、リンク先の章で説明されています。

[HIR]: ./hir.md
[queries]: ./query.md
[rd]: https://github.com/rust-lang/rust/tree/HEAD/src/librustdoc

`librustdoc`は、その後ドキュメントのセットをレンダリングするために2つの主要なステップを実行します：

* ASTをドキュメント作成により適した形式に「クリーン化」する（そして
  コンパイラの変更に対して多少耐性を持たせる）。
* このクリーン化されたASTを使用して、クレートのドキュメントを1ページずつレンダリングする。

当然、これ以外にも多くのことがあり、これらの説明は多くの詳細を省略していますが、
これが高レベルの概要です。

（補足：`librustdoc`はライブラリクレートです！
`rustdoc`バイナリは[`src/tools/rustdoc`][bin]のプロジェクトを使用して作成されます。
ただし、文字通り行っているのは、このクレートの`lib.rs`にある`main()`を呼び出すことだけです。）

[bin]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/rustdoc

## チートシート

* 開始する前に`./x setup tools`を実行してください。これにより、rustdocやその他のツールの開発に適した設定で`x`が設定され、
  rustcをビルドする代わりにコピーをダウンロードします。
* `./x check rustdoc`を使用してコンパイルエラーを素早くチェックします。
* `./x build library rustdoc`を使用して、他のプロジェクトで実行できる
  使用可能なrustdocを作成します。
  * `rustdoc --test`を使用できるようにするには、`library/test`を追加します。
  * `rustup toolchain link stage2 build/host/stage2`を実行して、
    rustup環境に`stage2`というカスタムツールチェーンを追加します。
    実行後、任意のディレクトリで`cargo +stage2 doc`を実行すると、
    ローカルでコンパイルしたrustdocでビルドされます。
* `./x doc library`を使用して、このrustdocで標準ライブラリのドキュメントを生成します。
  * 完成したドキュメントは`build/host/doc`（`core`、`alloc`、`std`の下）にあります。
  * これらのドキュメントをウェブサーバーにコピーする場合は、`build/host/doc`全体をコピーしてください。
    CSS、JS、フォント、ランディングページがそこにあります。
  * フロントエンドのデバッグには、[`bootstrap.toml`]で`rust.docs-minification`オプションを無効にします。
* `./x test tests/rustdoc*`を使用して、stage1のrustdocを使用してテストを実行します。
  * テストの詳細については、[Rustdoc internals]を参照してください。
* `./x.py test tidy --extra-checks=js`を使用して、rustdocのJavaScriptチェック（`eslint`、`es-check`、`tsc`）を実行します。

> **注意：** `./x.py test tidy`は、JS/TSソースが変更されたときに既にこれらのチェックを自動的に実行します；`--extra-checks=js`は明示的に強制します。

### JavaScript CIチェック

RustdocのJavaScriptとTypeScriptは、CI中に`eslint`、`es-check`、`tsc`によってチェックされます（compiletestではありません）。
これらは`tidy`ジョブの一部として実行されます。

```console
./x.py test tidy --extra-checks=js
```

`--extra-checks=js`フラグは、CIで実行されるフロントエンドのリント機能を有効にします。

[`bootstrap.toml`]: ./building/how-to-build-and-run.md

## コード構造

このセクションのすべてのパスは、rust-lang/rustリポジトリの`src/librustdoc/`からの相対パスです。

* HTMLを出力するコードのほとんどは、`html/format.rs`と`html/render/mod.rs`にあります。
  `impl std::fmt::Display`を返す関数の集まりです。
* 上記の関数によってレンダリングされるデータ型は、`clean/types.rs`で定義されています。
  `HIR`と`rustc_middle::ty` IRからこれらを作成する関数は、
  `clean/mod.rs`にあります。
* rustdocをテストハーネスとして使用する際に固有のビットは、
  `doctest.rs`にあります。
* Markdownレンダラーは`html/markdown.rs`に読み込まれ、
  指定されたMarkdownブロックからdoctestを抽出する関数も含まれています。
* フロントエンドのCSSとJavaScriptは`html/static/`に保存されています。
  * JavaScriptについては、型アノテーションは[TypeScript風のJSDoc]コメントと
    外部の`.d.ts`ファイルを使用して書かれています。
    このため、コード自体は平文の有効なJavaScriptのままです。
    `tsc`はリンターとしてのみ使用しています。

[TypeScript風のJSDoc]: https://www.typescriptlang.org/docs/handbook/jsdoc-supported-types.html

## テスト

`rustdoc`の統合テストは、複数のテストスイートに分かれています。
詳細については、[Rustdocテストスイート](tests/compiletest.md#rustdoc-test-suites)を参照してください。

## 制約

JavaScriptを無効にした状態とローカルファイルを閲覧する状態で、rustdocが合理的にうまく動作するように努めています。
[サポートされているブラウザのリスト]があります。

ローカルファイル（`file:///` URL）のサポートには、いくつかの驚くべき制約があります。
`localStorage`やService Workersのような、セキュアオリジンを必要とする特定のブラウザ機能は、
確実には動作しません。
このような機能は使用できますが、それらなしでもページが使用可能であることを確認する必要があります。

Rustdocは[関数本体の型チェックを行いません][platform-specific docs]。
これは、[typeckの組み込みクエリをオーバーライド][override queries]し、
[名前解決エラーを抑制][silencing name resolution errors]し、
[不透明型を解決しない][not resolving opaque types]ことで機能します。
これにはいくつかの注意点があります：特に、rustdocは型チェック本体を必要とするコンパイラのどの部分も実行*できません*；
たとえば、`.rlib`ファイルを生成したり、ほとんどのlintを実行したりすることはできません。
最終的にはこのモデルから移行したいと考えていますが、
[これを使用している人々][async-std]のための代替案が必要です；
[さまざまな][zulip stop accepting broken code]
[以前の][rustdoc meeting 2024-07-08] [Zulip][compiler meeting 2023-01-26] [ディスカッション][notriddle rfc]を参照してください。
このハックを削除すると壊れるコードの例については、
[`tests/rustdoc-ui/error-in-impl-trait`]を参照してください。

[platform-specific docs]: https://doc.rust-lang.org/rustdoc/advanced-features.html#interactions-between-platform-specific-docs
[override queries]: https://github.com/rust-lang/rust/blob/52bf0cf795dfecc8b929ebb1c1e2545c3f41d4c9/src/librustdoc/core.rs#L299-L323
[silencing name resolution errors]: https://github.com/rust-lang/rust/blob/52bf0cf795dfecc8b929ebb1c1e2545c3f41d4c9/compiler/rustc_resolve/src/late.rs#L4517
[not resolving opaque types]: https://github.com/rust-lang/rust/blob/52bf0cf795dfecc8b929ebb1c1e2545c3f41d4c9/compiler/rustc_hir_analysis/src/check/check.rs#L188-L194
[async-std]: https://github.com/rust-lang/rust/issues/75100
[rustdoc meeting 2024-07-08]: https://rust-lang.zulipchat.com/#narrow/channel/393423-t-rustdoc.2Fmeetings/topic/meeting.202024-07-08/near/449969836
[compiler meeting 2023-01-26]: https://rust-lang.zulipchat.com/#narrow/channel/238009-t-compiler.2Fmeetings/topic/.5Bweekly.5D.202023-01-26/near/323755789
[zulip stop accepting broken code]: https://rust-lang.zulipchat.com/#narrow/stream/266220-rustdoc/topic/stop.20accepting.20broken.20code
[notriddle rfc]: https://rust-lang.zulipchat.com/#narrow/channel/266220-t-rustdoc/topic/Pre-RFC.3A.20stop.20accepting.20broken.20code
[`tests/rustdoc-ui/error-in-impl-trait`]: https://github.com/rust-lang/rust/tree/163cb4ea3f0ae3bc7921cc259a08a7bf92e73ee6/tests/rustdoc-ui/error-in-impl-trait
[サポートされているブラウザのリスト]: https://rust-lang.github.io/rfcs/1985-tiered-browser-support.html#supported-browsers

## 複数回の実行、同じ出力ディレクトリ

Rustdocは、さまざまな入力に対して複数回実行できます。出力は同じディレクトリに設定されます。
これは、cargoが現在のクレートの依存関係のドキュメントを生成する方法です。
ユーザーが気になるすべてのドキュメントを含む大きなドキュメントバンドルを手動で作成することもできます。

HTMLは各クレートごとに独立して生成されますが、出力ディレクトリにクレートを追加する際に更新される
クレート間の情報がいくつかあります：

* `crates<SUFFIX>.js`は、出力ディレクトリ内のすべてのクレートのリストを保持します。
* `search-index<SUFFIX>.js`は、すべての検索可能なアイテムのリストを保持します。
* 各トレイトについて、`implementors/.../trait.TraitName.js`の下にファイルがあり、
   そのトレイトの実装者のリストが含まれています。
   実装者は、トレイトとは異なるクレートにある場合があり、新しいものが見つかるたびにJSファイルが更新されます。

## ユースケース

rustdocで作業する際に心に留めておくべき主要なユースケースがいくつかあります：

### 標準ライブラリドキュメント

これらは、Rustのリリースプロセスの一部として<https://doc.rust-lang.org/std>に公開されます。
安定版リリースは、<https://doc.rust-lang.org/1.57.0/std/>のような特定のバージョン付きURLにもアップロードされます。
ベータ版とナイトリー版のドキュメントは、
<https://doc.rust-lang.org/beta/std/>と<https://doc.rust-lang.org/nightly/std/>に公開されます。
ドキュメントは[promote-release
ツール](https://github.com/rust-lang/promote-release)でアップロードされ、S3からCloudFrontで配信されます。

標準ライブラリドキュメントには、alloc、core、proc_macro、std、testの5つのクレートが含まれています。

### docs.rs

クレートがcrates.ioに公開されると、docs.rsが自動的にドキュメントをビルドして公開します。
たとえば、<https://docs.rs/serde/latest/serde/>のようになります。
常に現在のナイトリー版のrustdocでビルドされるため、rustdocに加えた変更はすべて「即座に安定」し、
docs.rsで即座に公開されます。
古いドキュメントは時々のみ再ビルドされるため、docs.rsで古いリリースを閲覧すると、UIにいくつかのバリエーションが表示されます。
クレート作成者は再ビルドをリクエストでき、最新のrustdocで実行されます。

Docs.rsは、ストレージを節約し、上部にナビゲーションバーを表示するために、rustdocの出力に対していくつかの変換を実行します。
特に、main.jsやrustdoc.cssなどの特定の静的ファイルは、
同じバージョンのrustdocの複数の呼び出しで共有される場合があります。
crates.jsやsidebar-items.jsなどの他のファイルは、異なる呼び出しで異なります。
フォントなどのさらに他のファイルは決して変更されません。
これらのカテゴリは、
`src/librustdoc/html/render/write_shared.rs`の`SharedResource`列挙型を使用して区別されます。

docs.rsのドキュメントは常に一度に1つのクレートに対して生成されるため、
検索とサイドバー機能には現在のクレートの依存関係は含まれません。

### ローカルで生成されたドキュメント

クレート作成者は、ローカルにチェックアウトしたクレートで`cargo doc --open`を実行して、ドキュメントを表示できます。
これは、書いているドキュメントが有用で正しく表示されることを確認するのに役立ちます。
クレートの作成者ではないが使用したい人がクレートのドキュメントを表示するのにも役立ちます。
どちらの場合も、人々は`--document-private-items` Cargoフラグを使用して、
通常は表示されないプライベートメソッド、フィールドなどを表示できます。

デフォルトでは、`cargo doc`はクレートとそのすべての依存関係のドキュメントを生成します。
これにより、非常に大きなドキュメントバンドルと、大きく（そして遅い）検索コーパスが生成される可能性があります。
Cargoフラグ`--no-deps`は、その動作を抑制し、クレートのみのドキュメントを生成します。

### セルフホスト型プロジェクトドキュメント

一部のプロジェクトは独自のドキュメントをホストしています。
これは、ローカルでドキュメントを生成し、ウェブサーバーにコピーするだけで簡単に行えます。
RustdocのHTML出力は、フラグによって広範囲にカスタマイズできます。
ユーザーはテーマを追加し、デフォルトのテーマを設定し、任意のHTMLを挿入できます。
詳細については、`rustdoc --help`を参照してください。
