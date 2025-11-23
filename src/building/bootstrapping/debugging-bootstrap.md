# ブートストラップのデバッグ

ブートストラップをデバッグ（およびプロファイリング）する主な方法は2つあります。1つ目は println によるロギング、2つ目は `tracing` 機能です。

## `println` によるロギング

ブートストラップには広範な非構造化ロギングがあります。そのほとんどは `--verbose` フラグの後ろにゲートされています（さらに詳細な情報を得るには `-vv` を渡してください）。

実行された Cargo コマンドの詳細な出力や他の種類の詳細なログを確認したい場合は、ブートストラップを呼び出す際に `-v` または `-vv` を渡してください。ログは非構造化されており、圧倒される可能性があることに注意してください。

```
$ ./x dist rustc --dry-run -vv
learning about cargo
running: RUSTC_BOOTSTRAP="1" "/home/jyn/src/rust2/build/x86_64-unknown-linux-gnu/stage0/bin/cargo" "metadata" "--format-version" "1" "--no-deps" "--manifest-path" "/home/jyn/src/rust2/Cargo.toml" (failure_mode=Exit) (created at src/bootstrap/src/core/metadata.rs:81:25, executed at src/bootstrap/src/core/metadata.rs:92:50)
running: RUSTC_BOOTSTRAP="1" "/home/jyn/src/rust2/build/x86_64-unknown-linux-gnu/stage0/bin/cargo" "metadata" "--format-version" "1" "--no-deps" "--manifest-path" "/home/jyn/src/rust2/library/Cargo.toml" (failure_mode=Exit) (created at src/bootstrap/src/core/metadata.rs:81:25, executed at src/bootstrap/src/core/metadata.rs:92:50)
...
```

## ブートストラップにおける `tracing`

ブートストラップには条件付きの `tracing` 機能があり、以下の機能を提供します：

- [`tracing`][tracing] のイベントとスパンを使用した構造化ロギングを有効にします。
- 実行されたステップとコマンドの階層と期間を視覚化するために使用できる [Chrome trace file] を生成します。
  - 生成された `chrome-trace.json` ファイルは、Chrome の `chrome://tracing` タブで開くか、例えば [Perfetto] を使用して開くことができます。
- 実行されたステップ間の依存関係を視覚化する [GraphViz] グラフを生成します。
  - 生成された `step-graph-*.dot` ファイルは、例えば [xdot] を使用してステップグラフを視覚化するか、例えば `dot -Tsvg` を使用して GraphViz ファイルを SVG ファイルに変換できます。
- コマンド実行サマリーを生成します。これは、どのコマンドが実行され、それらの実行のうちいくつがキャッシュされたか、どのコマンドが最も遅かったかを示します。
  - 生成された `command-stats.txt` ファイルは、シンプルで人間が読みやすい形式です。

構造化ログは標準エラー出力（`stderr`）に書き込まれ、他の出力は `<build-dir>/bootstrap-trace/<pid>` ディレクトリのファイルに保存されます。便宜上、ブートストラップは最新の生成されたトレース出力ディレクトリへのシンボリックリンクを `<build-dir>/bootstrap-trace/latest` に作成します。

> `--dry-run` でブートストラップを実行すると、トレース出力ディレクトリが変更される可能性があることに注意してください。ブートストラップは、実行の最後に常にトレース出力ファイルが保存されたパスを出力します。

[tracing]: https://docs.rs/tracing/0.1.41/tracing/index.html
[Chrome trace file]: https://www.chromium.org/developers/how-tos/trace-event-profiling-tool/
[Perfetto]: https://ui.perfetto.dev/
[GraphViz]: https://graphviz.org/doc/info/lang.html
[xdot]: https://github.com/jrfonseca/xdot.py

### `tracing` 出力の有効化

条件付き `tracing` 機能を有効にするには、`BOOTSTRAP_TRACING` 環境変数を使用してブートストラップを実行します。


```bash
BOOTSTRAP_TRACING=trace ./x build library --stage 1
```

出力例[^unstable]：

```
$ BOOTSTRAP_TRACING=trace ./x build library --stage 1 --dry-run
Building bootstrap
    Finished `dev` profile [unoptimized] target(s) in 0.05s
15:56:52.477  INFO > tool::LibcxxVersionTool {target: x86_64-unknown-linux-gnu} (builder/mod.rs:1715)
15:56:52.575  INFO > compile::Assemble {target_compiler: Compiler { stage: 0, host: x86_64-unknown-linux-gnu, forced_compiler: false }} (builder/mod.rs:1715)
15:56:52.575  INFO > tool::Compiletest {compiler: Compiler { stage: 0, host: x86_64-unknown-linux-gnu, forced_compiler: false }, target: x86_64-unknown-linux-gnu} (builder/mod.rs:1715)
15:56:52.576  INFO  > tool::ToolBuild {build_compiler: Compiler { stage: 0, host: x86_64-unknown-linux-gnu, forced_compiler: false }, target: x86_64-unknown-linux-gnu, tool: "compiletest", path: "src/tools/compiletest", mode: ToolBootstrap, source_type: InTree, extra_features: [], allow_features: "internal_output_capture", cargo_args: [], artifact_kind: Binary} (builder/mod.rs:1715)
15:56:52.576  INFO   > builder::Libdir {compiler: Compiler { stage: 0, host: x86_64-unknown-linux-gnu, forced_compiler: false }, target: x86_64-unknown-linux-gnu} (builder/mod.rs:1715)
15:56:52.576  INFO    > compile::Sysroot {compiler: Compiler { stage: 0, host: x86_64-unknown-linux-gnu, forced_compiler: false }, force_recompile: false} (builder/mod.rs:1715)
15:56:52.578  INFO > compile::Assemble {target_compiler: Compiler { stage: 0, host: x86_64-unknown-linux-gnu, forced_compiler: false }} (builder/mod.rs:1715)
15:56:52.578  INFO > tool::Compiletest {compiler: Compiler { stage: 0, host: x86_64-unknown-linux-gnu, forced_compiler: false }, target: x86_64-unknown-linux-gnu} (builder/mod.rs:1715)
15:56:52.578  INFO  > tool::ToolBuild {build_compiler: Compiler { stage: 0, host: x86_64-unknown-linux-gnu, forced_compiler: false }, target: x86_64-unknown-linux-gnu, tool: "compiletest", path: "src/tools/compiletest", mode: ToolBootstrap, source_type: InTree, extra_features: [], allow_features: "internal_output_capture", cargo_args: [], artifact_kind: Binary} (builder/mod.rs:1715)
15:56:52.578  INFO   > builder::Libdir {compiler: Compiler { stage: 0, host: x86_64-unknown-linux-gnu, forced_compiler: false }, target: x86_64-unknown-linux-gnu} (builder/mod.rs:1715)
15:56:52.578  INFO    > compile::Sysroot {compiler: Compiler { stage: 0, host: x86_64-unknown-linux-gnu, forced_compiler: false }, force_recompile: false} (builder/mod.rs:1715)
    Finished `release` profile [optimized] target(s) in 0.11s
Tracing/profiling output has been written to <src-root>/build/bootstrap-trace/latest
Build completed successfully in 0:00:00
```

[^unstable]: この出力は常にさらなる変更の対象となります。

#### トレース出力の制御

環境変数 `BOOTSTRAP_TRACING` は [`tracing_subscriber` フィルター][tracing-env-filter] を受け入れます。`BOOTSTRAP_TRACING=trace` を設定すると、すべてのログを有効にしますが、これは圧倒的かもしれません。そのため、フィルターを使用してログデータの量を減らすことができます。

どの種類のトレースログが必要かを制御するには、直交する2つの方法があります：

1. ログ **レベル** を指定できます（例：`debug` または `trace`）。
   - レベルを選択すると、同等以上の優先度レベルを持つすべてのイベント/スパンが表示されます。
2. ログ **ターゲット** も制御できます（例：`bootstrap` または `bootstrap::core::config` または `CONFIG_HANDLING` や `STEP` のようなカスタムターゲット）。
    - カスタムターゲットは、関心のあるスパンの種類を制限するために使用されます。`BOOTSTRAP_TRACING=trace` 出力はかなり冗長になる可能性があるためです。現在、以下のカスタムターゲットを使用できます：
        - `CONFIG_HANDLING`：設定処理に関連するスパンを表示します。
        - `STEP`：実行されたすべてのステップを表示します。実行されたコマンドは `info` イベントレベルを持ちます。
        - `COMMAND`：実行されたすべてのコマンドを表示します。実行されたコマンドは `trace` イベントレベルを持ちます。
        - `IO`：実行された I/O 操作を表示します。実行されたコマンドは `trace` イベントレベルを持ちます。
            - 多くの I/O は現在トレースされていないことに注意してください。

もちろん、それらを組み合わせることもできます（カスタムターゲットログは通常、追加で `TRACE` ログレベルの後ろにゲートされています）：

```bash
BOOTSTRAP_TRACING=CONFIG_HANDLING=trace,STEP=info,COMMAND=trace ./x build library --stage 1
```

[tracing-env-filter]: https://docs.rs/tracing-subscriber/0.3.19/tracing_subscriber/filter/struct.EnvFilter.html

`BOOTSTRAP_TRACING` を使用して指定するレベルは、Chrome トレースファイルに記録されるスパンにも影響することに注意してください。

##### FIXME(#96176): `compiler()` と `compiler_for()` の特定のトレース

追加のターゲット `COMPILER` と `COMPILER_FOR` は、`builder.compiler()` と `builder.compiler_for()` が何をするかをトレースするために使用されます。[#96176][cleanup-compiler-for] が解決された場合、これらは削除されるべきです。

[cleanup-compiler-for]: https://github.com/rust-lang/rust/issues/96176

### ブートストラップでの `tracing` の使用

`tracing::*` マクロと `tracing::instrument` proc-macro 属性の両方を `tracing` 機能の後ろにゲートする必要があります。例：

```rs
#[cfg(feature = "tracing")]
use tracing::instrument;

struct Foo;

impl Step for Foo {
    type Output = ();

    #[cfg_attr(feature = "tracing", instrument(level = "trace", name = "Foo::should_run", skip_all))]
    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        trace!(?run, "entered Foo::should_run");

        todo!()
    }

    fn run(self, builder: &Builder<'_>) -> Self::Output {
        trace!(?run, "entered Foo::run");

        todo!()
    }
}
```

`#[instrument]` については、以下を推奨します：

- 細かい粒度のために `trace` レベルの後ろにゲートし、コア関数については `debug` レベルにする可能性があります。
- `name = ".."` 経由で明示的にインストルメンテーション名を選択し、異なるステップの `run` などを区別します。
- トレースインフラが有効な場合にのみ、追加のものをビルドするなど、トレースによって異なる動作を引き起こさないように注意してください。

### rust-analyzer 統合？

残念ながら、ブートストラップは `rust-analyzer.linkedProjects` であるため、<https://github.com/rust-lang/rust-analyzer/issues/8521> で説明されているサポートの欠如により、r-a にブートストラップ自体を `tracing` 機能を有効にしてチェック/ビルドするように依頼することはできません。
