# ドキュメントのビルド

この章では、標準ライブラリ（std）やコンパイラ（rustc）などのツールチェーンコンポーネントのドキュメントをビルドする方法について説明します。

- すべてをドキュメント化する

  これはベータツールチェーンの `rustdoc` を使用するため、rustdoc が活発に開発されているため、stage 1 rustdoc とは（わずかに）異なる出力を生成します：

  ```bash
  ./x doc
  ```

  CI と同じようにドキュメントが見えることを確認したい場合：

  ```bash
  ./x doc --stage 1
  ```

  これにより、（現在の）rustdoc がビルドされ、それがコンポーネントをドキュメント化するために使用されます。

- 個別のテストを実行したり、特定のコンポーネントをビルドするのと同様に、必要なドキュメントだけをビルドできます：

  ```bash
  ./x doc src/doc/book
  ./x doc src/doc/nomicon
  ./x doc compiler library
  ```

  書籍の完全なリストについては、[nightly docs index page](https://doc.rust-lang.org/nightly/) をご覧ください。

- 内部 rustc アイテムをドキュメント化する

  コンパイラのドキュメントはデフォルトではビルドされません。`x doc` でデフォルトで作成するには、`bootstrap.toml` を変更します：

  ```toml
  [build]
  compiler-docs = true
  ```

  有効にすると、内部コンパイラアイテムのドキュメントもビルドされることに注意してください。

  注意：コンパイラのドキュメントは [this link] にあります。

[this link]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/
