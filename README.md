[![CI](https://github.com/rust-lang/rustc-dev-guide/actions/workflows/ci.yml/badge.svg)](https://github.com/rust-lang/rustc-dev-guide/actions/workflows/ci.yml)


これはrustcの動作を説明するガイドを構築するための共同作業です。このガイドの目的は、新しい貢献者がrustcに慣れるのを助けること、また経験豊富な人々が以前に取り組んだことのないコンパイラの新しい部分を理解するのを助けることです。

[You can read the latest version of the guide here.](https://rustc-dev-guide.rust-lang.org/)

コンパイラ自体の[rustdocs][rustdocs]も役立つでしょう。
これらはガイドとして意図されていないことに注意してください。上から下まで読むのではなく、探しているドキュメントを検索することをお勧めします。

[rustdocs]: https://doc.rust-lang.org/nightly/nightly-rustc

標準ライブラリの開発に関するドキュメントについては、[`std-dev-guide`](https://std-dev-guide.rust-lang.org/)を参照してください。

### ガイドへの貢献

このガイドは今日でも有用ですが、まだやるべきことがたくさんあります。

ガイドの改善を手伝っていただける場合は、大歓迎です！[issue tracker](https://github.com/rust-lang/rustc-dev-guide/issues)にたくさんのissueがあります。作業の重複を避けるため、取り組みたいissueにコメントを投稿してください。何か欠けていると思う場合は、それについてissueを開いてください！

**一般的に、コンパイラの動作がわからなくても、それは問題ではありません！** その場合、コードを**知っている**人、または一緒にペアを組んで解明したい人と話す時間を少し設けます。そして、学んだことを書き留める作業を進めることができます。

一般的に、コンパイラのコードの特定の部分について書くときは、関連する部分へのリンクを[rustc rustdocs][rustdocs]に含めることをお勧めします。

### ビルド手順

ローカルの静的HTMLサイトをビルドするには、[`mdbook`](https://github.com/rust-lang/mdBook)を次のコマンドでインストールします：

```
cargo install mdbook mdbook-linkcheck2 mdbook-mermaid
```

リポジトリのルートで次のコマンドを実行します：

```
mdbook build --open
```

ビルドファイルは`book/html`ディレクトリにあります。

### リンク検証

ドキュメントに含まれるURLを検証するために`mdbook-linkcheck2`を使用しています。リンクチェックはローカルではデフォルトで実行**されません**が、CIでは実行されます。ローカルで有効にするには、次の例のように環境変数`ENABLE_LINKCHECK=1`を設定します。

```
ENABLE_LINKCHECK=1 mdbook serve
```

### 目次

各ページには`pagetoc.js`によって自動生成される目次があります。
スタイリング用の`pagetoc.css`が関連付けられています。

## rustcとのjoshサブツリーの同期

このリポジトリは[josh](https://josh-project.github.io/josh/intro.html)サブツリーとして`rust-lang/rust`にリンクされています。[rustc-josh-sync](https://github.com/rust-lang/josh-sync)ツールを使用して同期を実行できます。

同期の実行方法に関するガイドは[here](./src/external-repos.md#synchronizing-a-josh-subtree)にあります。
