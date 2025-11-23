# GCC コード生成バックエンド

コンパイラテストスイートのサブセットを GCC コード生成バックエンドで CI 上で実行し、このバックエンドとコンパイラの統合を壊す可能性のある変更を見つけるのに役立てています。

GCC コード生成バックエンドに関する一般的なバグや問題に遭遇した場合は、遠慮なく [`rustc_codegen_gcc` リポジトリ](https://github.com/rust-lang/rustc_codegen_gcc) に issue を開いてください。

現在、バックエンドは `x86_64-unknown-linux-gnu` ターゲットのみをサポートしていることに注意してください。

## GCC バックエンド CI エラーに遭遇した場合

CI の `x86_64-gnu-gcc` ジョブで GCC コード生成バックエンドで実行されたテストに関連するエラーに遭遇した場合、次のコマンドを使用して GCC バックエンドを使用して UI テストをローカルで実行できます。これは CI で起こることを再現します：

```bash
./x test tests/ui \
  --set 'rust.codegen-backends = ["llvm", "gcc"]' \
  --set 'rust.debug-assertions = false' \
  --test-codegen-backend gcc
```

別のテストスイートが CI で失敗した場合は、`tests/ui` の部分を変更する必要があります。

CI ジョブ全体をローカルで再現するには、`cargo run --manifest-path src/ci/citool/Cargo.toml run-local x86_64-gnu-gcc` を実行できます。
詳細については、[Docker を使用したテスト](../docker.md) を参照してください。

### GCC ジョブが失敗した場合にどうすべきか？

GCC ジョブテストが失敗し、その失敗が GCC バックエンドによって引き起こされた可能性があるように見える場合は、`@rust-lang/wg-gcc-backend` を使用して [cg-gcc ワーキンググループ](https://github.com/orgs/rust-lang/teams/wg-gcc-backend) にピングできます。

GCC バックエンドで実行時に失敗するコンパイラテストの修正が自明でない場合は、`//@ ignore-backends: gcc` [compiletest ディレクティブ](../directives.md) を使用して `cg_gcc` で実行されるときにそのテストを無視できます。

## どのコード生成バックエンドがビルドされるかを選択

`rust.codegen-backends = [...]` bootstrap オプションは、どのコード生成バックエンドがビルドされ、
生成される `rustc` の sysroot に含まれるかに影響します。
GCC コード生成バックエンドを使用するには、`bootstrap.toml` でこの配列に `"gcc"` を含める必要があります：

```toml
rust.codegen-backends = ["llvm", "gcc"]
```

`bootstrap.toml` ファイルを変更したくない場合は、代わりに `--set 'rust.codegen-backends=["llvm", "gcc"]'` を指定して `x` コマンドを実行できます。
例：

```bash
./x build --set 'rust.codegen-backends=["llvm", "gcc"]'
```

`codegen-backends` 配列の最初のバックエンドが、ビルドされた `rustc` の*デフォルトバックエンド*として使用されるバックエンドを決定します。
これはまた、stage 1 標準ライブラリ（または stage 2 以降でビルドされるもの）のコンパイルに使用されるバックエンドも決定します。
デフォルトで GCC バックエンドを使用する `rustc` を生成するには、
この配列の最初の要素として `"gcc"` を配置できます：

```bash
./x build --set 'rust.codegen-backends=["gcc"]' library
```

## テストで使用されるコード生成バックエンドを選択

GCC コード生成バックエンドを使用してテスト Rust プログラムをビルドするコンパイラテストを実行するには、`--test-codegen-backend` フラグを使用できます：

```bash
./x test tests/ui --test-codegen-backend gcc
```

これが機能するためには、テスト対象のコンパイラの sysroot ディレクトリに GCC コード生成バックエンドが[利用可能](#choosing-which-codegen-backends-are-built)である必要があることに注意してください。

## CI から GCC をダウンロード

`gcc.download-ci-gcc` bootstrap オプションは、GCC（GCC コード生成バックエンドの依存関係）を
CI からダウンロードするか、ローカルでビルドするかを制御します。
デフォルト値は `true` で、GCC ソースにローカルな変更がなく、指定されたホストターゲットが CI で利用可能な場合は、GCC を CI からダウンロードします。

## バックエンド自体のテストを実行

コンパイラのテストスイートを GCC コード生成バックエンドを使用して実行するだけでなく、バックエンド自体のテストスイートを実行することもできます。

次のコマンドを使用してそれを行います：

```text
./x test rustc_codegen_gcc
```

これが機能するためには、バックエンドを[有効](#choosing-which-codegen-backends-are-built)にする必要があります。
