# Fuchsia 統合テスト

[Fuchsia](https://fuchsia.dev) は、約200万行の Rust コードを持つオープンソースのオペレーティングシステムです。[^loc] 過去に多くの[退行][regressions]を捉えており、その後 CI に含まれました。

## Fuchsia ジョブが壊れた場合にどうすべきか？

[fuchsia][fuchsia-ping] ピンググループに連絡して助けを求めてください。

```text
@rustbot ping fuchsia
```

## CI での Fuchsia のビルド

Fuchsia は、プルリクエストがマージされる前に実行される bors テストのスイートの一部としてビルドされます。

プルリクエストが Fuchsia ビルダーを壊す可能性があり、bors キューに送信する前にテストしたい場合は、単に bors に Fuchsia 統合をビルドする try ジョブを実行するように依頼してください：`@bors try jobs=x86_64-fuchsia`。

## Fuchsia をローカルでビルド

Fuchsia は Rust 以外の言語を使用しているため、ビルドシステムとして Cargo を使用しません。また、ツールチェーンのビルドを[特定の方法][build-toolchain]で構成する必要があります。

Fuchsia をビルドする推奨される方法は、Fuchsia のチェックアウトとビルドを実行する Docker スクリプトを使用することです。以前に Docker テストを実行したことがある場合は、Rust チェックアウトから次のコマンドを実行するだけで、ローカルの Rust ツールチェーンを使用して Fuchsia をダウンロードしてビルドできます。

```
src/ci/docker/run.sh x86_64-fuchsia
```

Docker でジョブを実行およびデバッグする方法の詳細については、[Docker を使用したテスト](../docker.md) の章を参照してください。

Fuchsia のチェックアウトは*大きい*ことに注意してください。執筆時点では、チェックアウトとビルドで 46G のスペースが必要です。想像できるように、完了するには時間がかかります。

### Fuchsia チェックアウトの変更

ローカルで Fuchsia をビルドしたい主な理由は、退行を調査する必要があるためです。Docker ビルドを実行した後、Rust チェックアウトの `obj/fuchsia` ディレクトリ内に Fuchsia チェックアウトが見つかります。[build-fuchsia.sh] スクリプトの `KEEP_CHECKOUT` 行を `KEEP_CHECKOUT=1` に変更すると、チェックアウトを必要に応じて変更し、上記のビルドコマンドを再実行できます。これにより、以前のすべてのビルド結果が再利用されます。

Fuchsia チェックアウトをカスタマイズするための詳細なオプションは、[build-fuchsia.sh] スクリプトで見つけることができます。

### Fuchsia ビルドのカスタマイズ

Rust CI で Fuchsia をビルドするために使用されるオプションの詳細については、[build-fuchsia.sh] から呼び出される [build_fuchsia_from_rust_ci.sh] スクリプトで見つけることができます。

Fuchsia ビルドシステムは、[Ninja] ファイルを生成してビルドの実行を Ninja に引き渡すメタビルドシステムである [GN] を使用します。

Fuchsia 開発者は `fx` を使用してビルドを実行し、その他の開発タスクを実行します。
このツールは Fuchsia チェックアウトの `.jiri_root/bin` にあります。一部のワークフローでは、これを `$PATH` に追加する必要があるかもしれません。

関連するいくつかの `fx` サブコマンドがあります：

- `fx set` はビルド引数を受け取り、`out/default/args.gn` に書き込み、
  GN を実行します。
- `fx build` は Ninja を使用して Fuchsia プロジェクトをビルドします。ビルド引数への変更を自動的に検出して GN を再実行します。デフォルトではすべてをビルドしますが、
  特定のターゲットをビルドするためのターゲットパスも受け入れます（下記参照）。
- `fx clippy` は特定の Rust ターゲット（またはすべて）で Clippy を実行します。Rust CI ビルドでは、ほとんどの Rust ターゲットでコード生成を実行しないようにこれを使用します。内部では、`fx build` と同じように Ninja を呼び出します。clippy の結果は、出力される前にビルド出力ディレクトリ内の json ファイルに保存されます。

#### ターゲットパス

GN は次のようなパスを使用してビルドターゲットを識別します：

```
//src/starnix/kernel:starnix_core
```

最初の `//` はチェックアウトのルートを意味し、残りのスラッシュはディレクトリ名です。`:` の後の文字列は、そのディレクトリの `BUILD.gn` ファイルで定義されたターゲットの_ターゲット名_です。

ターゲット名がディレクトリ名と同じ場合は省略できます。つまり、`//src/starnix/kernel` は `//src/starnix/kernel:kernel` と同じです。

これらのターゲットパスは `BUILD.gn` ファイル内で依存関係を参照するために使用され、`fx build` でも使用できます。

#### コンパイラフラグの変更

ターゲットに追加される GN `config` 内にカスタムコンパイラフラグを配置できます。簡単な例として：

```
config("everybody_loops") {
    rustflags = [ "-Zeverybody-loops" ]
}

rustc_binary("example") {
    crate_root = "src/bin.rs"
    # ...既存のキーがここにあります...
    configs += [ ":everybody_loops" ]
}
```

これにより、`example` ターゲットをビルドするときに rustc にフラグ `-Zeverybody-loops` が追加されます。また、そのターゲットに依存するすべてのターゲットに config を追加するために [`public_configs`] を使用することもできます。

ビルド内のすべての Rust ターゲットにフラグを追加したい場合は、[`//build/config:compiler`] config またはそのファイルで参照される OS 固有の config に rustflags を追加できます。`cflags` と `ldflags` は Rust ターゲットでは無視されることに注意してください。

#### ninja と rustc コマンドを直接実行

1つ下のレイヤーに行くと、`fx build` は `ninja` を呼び出し、それが最終的に `rustc` を呼び出します。すべてのビルドアクションは out ディレクトリ内で実行されます。これは通常、Fuchsia チェックアウト内の `out/default` です。

ninja にそれが呼び出す実際のコマンドを出力させるには、ターゲットのソースファイルの1つに構文エラーを追加するなどして、そのコマンドを強制的に失敗させます。
コマンドを取得したら、出力ディレクトリ内から実行できます。

ツールチェーン自体を変更した後、`out/default/args.gn` のビルド設定 `rustc_version_string` を変更して、`fx build` または `ninja` がすべての Rust ターゲットを再ビルドするようにする必要があります。これはテキストエディタで行うことができ、文字列の内容は重要ではありません。次のビルドから次のビルドに変更されればよいだけです。
[build_fuchsia_from_rust_ci.sh] は、ツールチェーンディレクトリをハッシュすることでこれを自動的に行います。

Fuchsia ウェブサイトには、[ビルドシステム] のより詳細なドキュメントがあります。

#### その他のヒントとコツ

`build_fuchsia_from_rust_ci.sh` を使用するときは、初回実行後に `fx set` コマンドをコメントアウトして、毎回 GN が再実行されないようにできます。これを行う場合は、version_string 行もコメントアウトして数秒節約できます。

`export NINJA_PERSISTENT_MODE=1` を設定して、初回ビルド後に ninja の起動時間を短縮します。

## Fuchsia ターゲットのサポート

Fuchsia ターゲットのサポートについて詳しく知りたい場合は、[rustc book][platform-support] の Fuchsia の章を参照してください。

[regressions]: https://gist.github.com/tmandry/7103eba4bd6a6fb0c439b5a90ae355fa
[build-toolchain]: https://fuchsia.dev/fuchsia-src/development/build/rust_toolchain
[build-fuchsia.sh]: https://github.com/rust-lang/rust/blob/221e2741c39515a5de6da42d8c76ee1e132c2c74/src/ci/docker/host-x86_64/x86_64-fuchsia/build-fuchsia.sh
[build_fuchsia_from_rust_ci.sh]: https://cs.opensource.google/fuchsia/fuchsia/+/main:scripts/rust/build_fuchsia_from_rust_ci.sh?q=build_fuchsia_from_rust_ci&ss=fuchsia
[platform-support]: https://doc.rust-lang.org/nightly/rustc/platform-support/fuchsia.html
[GN]: https://gn.googlesource.com/gn/+/main#gn
[Ninja]: https://ninja-build.org/
[`public_configs`]: https://gn.googlesource.com/gn/+/main/docs/reference.md#var_public_configs
[`//build/config:compiler`]: https://cs.opensource.google/fuchsia/fuchsia/+/main:build/config/BUILD.gn;l=121;drc=c26c473bef93b33117ae417893118907a026fec7
[ビルドシステム]: https://fuchsia.dev/fuchsia-src/development/build/build_system
[fuchsia-ping]: ../../notification-groups/fuchsia.md

[^loc]: 2024年6月時点で、Fuchsia には約200万行のファーストパーティ Rust コードと、ほぼ同量のサードパーティコードがあり、tokei によってカウントされています（コメントと空白を除く）。
