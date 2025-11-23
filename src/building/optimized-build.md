# コンパイラの最適化ビルド

できるだけ最適化された `rustc` のビルドをコンパイルするために使用できる複数の追加のビルド設定オプションと技術があります（たとえば、Linux ディストリビューション用に `rustc` をビルドする場合）。さまざまな Rust ターゲットに対するこれらの設定オプションの状態は [こちら] で追跡されています。このページでは、`rustc` を自分でビルドする際にこれらのアプローチをどのように使用できるかを説明します。

[こちら]: https://github.com/rust-lang/rust/issues/103595

## リンク時最適化

リンク時最適化は、プログラムのパフォーマンスを向上させることができる強力なコンパイラ技術です。`rustc` をビルドする際に（Thin-）LTO を有効にするには、`bootstrap.toml` で `rust.lto` 設定オプションを `"thin"` に設定します：

```toml
[rust]
lto = "thin"
```

> `rustc` の LTO は現在、`x86_64-unknown-linux-gnu` ターゲットに対してのみサポートされ、テストされていることに注意してください。他のターゲットは動作する *可能性がありますが*、保証はありません。特に、LTO 最適化された `rustc` は現在、Windows で [誤コンパイル] を生成します。

[誤コンパイル]: https://github.com/rust-lang/rust/issues/109114

Linux で LTO を有効にすると、最大 10% の高速化が [生成されました]。

[生成されました]: https://github.com/rust-lang/rust/pull/101403#issuecomment-1288190019

## メモリアロケータ

`rustc` に異なるメモリアロケータを使用すると、大幅なパフォーマンス向上をもたらす可能性があります。`jemalloc` アロケータを有効にしたい場合は、`bootstrap.toml` で `rust.jemalloc` オプションを `true` に設定できます：

```toml
[rust]
jemalloc = true
```

> このオプションは現在、Linux および macOS ターゲットに対してのみサポートされていることに注意してください。

## コードジェネレーションユニット

`rustc` クレートごとのコードジェネレーションユニットの数を減らすと、コンパイラのビルドが高速になる可能性があります。`bootstrap.toml` で `rustc` と `libstd` のコードジェネレーションユニットの数を次のオプションで変更できます：

```toml
[rust]
codegen-units = 1
codegen-units-std = 1
```

## 命令セット

デフォルトでは、`rustc` は汎用的な（保守的な）命令セットアーキテクチャ（選択されたターゲットに応じて）用にコンパイルされ、できるだけ多くの CPU をサポートします。特定の命令セットアーキテクチャ用に `rustc` をコンパイルしたい場合は、`RUSTFLAGS` で `target_cpu` コンパイラオプションを設定できます：

```bash
RUSTFLAGS="-C target_cpu=x86-64-v3" ./x build ...
```

LLVM も特定の命令セット用にコンパイルしたい場合は、`bootstrap.toml` で `llvm` フラグを設定できます：

```toml
[llvm]
cxxflags = "-march=x86-64-v3"
cflags = "-march=x86-64-v3"
```

## プロファイルガイド最適化

プロファイルガイド最適化（より一般的には、フィードバック指向最適化）を適用すると、最大 15% の `rustc` パフォーマンスの大幅な向上を生み出すことができます（[1]、[2]）。ただし、これらの技術は単純に設定オプションで有効にされるのではなく、`rustc` を複数回コンパイルし、選択されたベンチマークでプロファイリングする複雑なビルドワークフローが必要です。

エンドユーザーに配布される `rustc` を [PGO]（プロファイルガイド最適化）と [BOLT]（ポストリンクバイナリオプティマイザ）で最適化するために使用される `opt-dist` というツールがあります。`src/tools/opt-dist` にあるツールを調べて、それに基づいてカスタム PGO ビルドワークフローを構築するか、直接使用してみることができます。ツールは現在、Rust の継続的統合ワークフローで使用する方法にかなりハードコードされており、異なる環境で動作させるにはカスタム変更が必要になる可能性があることに注意してください。

[1]: https://blog.rust-lang.org/inside-rust/2020/11/11/exploring-pgo-for-the-rust-compiler.html#final-numbers-and-a-benchmarking-plot-twist
[2]: https://github.com/rust-lang/rust/pull/96978

[PGO]: https://doc.rust-lang.org/rustc/profile-guided-optimization.html

[BOLT]: https://github.com/llvm/llvm-project/blob/main/bolt/README.md

ツールを使用するには、いくつかの外部依存関係を提供する必要があります：

- Python3 インタープリタ（`x.py` を実行するため）。
- コンパイルされた LLVM ツールチェーン、`llvm-profdata` バイナリを含む。オプションで、BOLT を使用したい場合は、`llvm-bolt` と `merge-fdata` バイナリがツールチェーンで利用可能である必要があります。

これらの依存関係は、[`Environment`] 構造体の実装によって `opt-dist` に提供されます。これは、PGO/BOLT パイプラインが行われるディレクトリと、Python や LLVM などの外部依存関係を指定します。

以下は、`opt-dist` をローカル（CI 外）で使用する方法の例です：

1. `bootstrap.toml` ファイルでメトリクスを有効にします。`opt-dist` はそれが有効になっていることを期待しているためです：

    ```toml
   [build]
   metrics = true
   ```

2. 次のコマンドでツールをビルドします：

    ```bash
    ./x build tools/opt-dist
    ```

3. `local` モードでツールを実行し、必要なパラメータを提供します：

    ```bash
    ./build/host/stage1-tools-bin/opt-dist local \
      --target-triple <target> \ # ターゲットを選択、例：「x86_64-unknown-linux-gnu」
      --checkout-dir <path>    \ # rust チェックアウトへのパス、例：「.」
      --llvm-dir <path>        \ # ビルドされた LLVM ツールチェーンへのパス、例：「/foo/bar/llvm/install」
      -- python3 x.py dist       # 実際のビルドコマンドを渡す
    ```

    変更できるさらなるパラメータを確認するには、`--help` を実行できます。

[`Environment`]: https://github.com/rust-lang/rust/blob/ee451f8faccf3050c76cdcd82543c917b40c7962/src/tools/opt-dist/src/environment.rs#L5

> 注意：`opt-dist` をローカルで実行する代わりに実際の CI パイプラインを実行したい場合は、`cargo run --manifest-path src/ci/citool/Cargo.toml run-local dist-x86_64-linux` を実行できます。
