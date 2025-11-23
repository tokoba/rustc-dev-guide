# 配布アーティファクトのビルド

配布用にコンパイラをビルドしてパッケージ化したい場合があります。その場合は、このコマンドを実行してください：

```bash
./x dist
```

# ソースからのインストール

Rust（および設定で構成されたツール）をソースからビルドしてインストールしたい場合があります。その場合は、このコマンドを実行してください：

```bash
./x install
```

   注意：コンパイラへの変更をテストしている場合は、コンパイラをビルド（`./x build` を使用）してから、[ここ][create-rustup-toolchain] で説明されているようにツールチェーンを作成したい場合があります。

   たとえば、作成したツールチェーンが「foo」と呼ばれる場合、`rustc +foo ...`（... は残りの引数を表します）で呼び出します。

Rust（および設定ファイル内のツール）をグローバルにインストールする代わりに、`DESTDIR` 環境変数を設定してインストールパスを変更できます。インストールパスをより動的に設定したい場合は、設定ファイルの [install options] を使用することをお勧めします。

[create-rustup-toolchain]: ./how-to-build-and-run.md#creating-a-rustup-toolchain
[install options]: https://github.com/rust-lang/rust/blob/f7c8928f035370be33463bb7f1cd1aeca2c5f898/config.example.toml#L422-L442
