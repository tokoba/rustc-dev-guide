# ブートストラップでツールを書く

ブートストラップで書くことができるツールには3つのタイプがあります：

- **`Mode::ToolBootstrap`**

  これは、インツリーコンパイラから何も必要とせず、stage0 `rustc` で実行できるツールに使用します。出力は「bootstrap-tools」ディレクトリに配置されます。このモードは、ターゲットライブラリを含む stage0 コンパイラで完全にビルドされた汎用ツール用で、ステージ 0 でのみ機能します。

- **`Mode::ToolStd`**

  これは、ローカルでビルドされた std に依存するツールに使用します。出力は「stageN-tools」ディレクトリに入ります。このモードはめったに使用されず、主に `libtest` を必要とする `compiletest` に使用されます。

- **`Mode::ToolRustcPrivate`**

  これは、`rustc_private` メカニズムを使用し、したがってローカルでビルドされた `rustc` とその rlib アーティファクトに依存するツールに使用します。これは他のモードよりも複雑です。なぜなら、ツールは `rustc` に使用されるのと同じコンパイラでビルドされ、「stageN-tools」ディレクトリに配置される必要があるからです。`Mode::ToolRustcPrivate` を選択すると、`ToolBuild` 実装が自動的にこれを処理します。何か特定のことにビルダーのコンパイラを使用する必要がある場合は、ツールの [`Step`] から返される `ToolBuildResult` から取得できます。

ツールのタイプに関係なく、ツールの [`Step`] 実装から `ToolBuildResult` を返す必要があり、その中で `ToolBuild` を使用します。

[`Step`]: https://doc.rust-lang.org/nightly/nightly-rustc/bootstrap/core/builder/trait.Step.html
