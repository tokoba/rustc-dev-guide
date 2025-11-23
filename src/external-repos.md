# 外部リポジトリの使用

`rust-lang/rust` Git リポジトリは、`rust-lang` 組織内のいくつかの他のリポジトリに依存しています。依存関係を使用する主な方法は3つあります：
1. crates.io 経由の Cargo 依存関係として（例：`rustc-rayon`）
2. Git（例：`clippy`）または [josh]（例：`miri`）のサブツリーとして
3. Git サブモジュールとして（例：`cargo`）

一般的なルールとして：
- エコシステムの他の人にとって有用な可能性があるライブラリには crates.io を使用する
- コンパイラの内部に依存し、破壊的変更がある場合に更新する必要があるツールにはサブツリーを使用する
- コンパイラから独立したツールにはサブモジュールを使用する

## 外部依存関係（サブツリー）

以下の外部プロジェクトは、何らかの形式の `subtree` を使用して管理されています：

* [clippy](https://github.com/rust-lang/rust-clippy)
* [miri](https://github.com/rust-lang/miri)
* [portable-simd](https://github.com/rust-lang/portable-simd)
* [rustfmt](https://github.com/rust-lang/rustfmt)
* [rust-analyzer](https://github.com/rust-lang/rust-analyzer)
* [rustc_codegen_cranelift](https://github.com/rust-lang/rustc_codegen_cranelift)
* [rustc-dev-guide](https://github.com/rust-lang/rustc-dev-guide)
* [compiler-builtins](https://github.com/rust-lang/compiler-builtins)
* [stdarch](https://github.com/rust-lang/stdarch)

`submodule` 依存関係（以下を参照）とは対照的に、`subtree` 依存関係は単なる通常のファイルとディレクトリであり、ツリー内で更新できます。ただし、可能であれば、これらのツールに固有の機能強化、バグ修正などは、それぞれのアップストリームリポジトリ内のツールに対して直接提出する必要があります。例外として、新しいツール機能やテストを実装するために rustc の変更が必要な場合は、1つの集合的な rustc PR で行う必要があります。

`subtree` 依存関係は現在、2つの異なるアプローチで管理されています：

* `git subtree` を使用
    * `clippy`（[同期ガイド](https://doc.rust-lang.org/nightly/clippy/development/infrastructure/sync.html#performing-the-sync-from-rust-langrust-to-clippy)）
    * `portable-simd`（[同期スクリプト](https://github.com/rust-lang/portable-simd/blob/master/subtree-sync.sh)）
    * `rustfmt`
    * `rustc_codegen_cranelift`（[同期スクリプト](https://github.com/rust-lang/rustc_codegen_cranelift/blob/113af154d459e41b3dc2c5d7d878e3d3a8f33c69/scripts/rustup.sh#L7)）
* [josh](#synchronizing-a-josh-subtree) ツールを使用
    * `miri`
    * `rust-analyzer`
    * `rustc-dev-guide`
    * `compiler-builtins`
    * `stdarch`

### Josh サブツリー

[josh] ツールは、Git サブツリーの代替であり、Git 履歴を異なる方法で管理し、より大きなリポジトリに対してよりスケールします。josh で作業するには特定のツールが必要です。以下で説明する同期を支援するために、ヘルパーツール [`rustc-josh-sync`][josh-sync] を提供しています。

### Josh サブツリーの同期

Josh サブツリーの更新を実行するために、[`rustc-josh-sync`][josh-sync] という専用ツールを使用します。以下のコマンドは、すべての Josh サブツリーに使用できますが、`miri` では pull 中にいくつかの[追加手順](https://github.com/rust-lang/miri/blob/master/CONTRIBUTING.md#advanced-topic-syncing-with-the-rustc-repo)を実行する必要があることに注意してください。

次のコマンドを使用してツールをインストールできます：
```
cargo install --locked --git https://github.com/rust-lang/josh-sync
```

プル（rust-lang/rust からサブツリーへの変更を同期）とプッシュ（サブツリーから rust-lang/rust への変更を同期）の両方は、サブツリーリポジトリから実行されます（したがって、まずターミナルでそのリポジトリのチェックアウトディレクトリに切り替えます）。

#### プルの実行
1) サブツリーへの PR を作成するために使用される新しいブランチをチェックアウトします
2) pull コマンドを実行します
    ```
    rustc-josh-sync pull
    ```
3) ブランチをフォークにプッシュし、サブツリーリポジトリへの PR を作成します
    - `gh` CLI がインストールされている場合、`rustc-josh-sync` が PR を作成できます。

#### プッシュの実行

> 注意：
> 続行する前に、[on josh-sync README] の Git に関するガイダンスをご覧ください。

1) push コマンドを実行して、`<gh-username>` アカウントの `rustc` フォークに `<branch-name>` という名前のブランチを作成します
    ```
    rustc-josh-sync push <branch-name> <gh-username>
    ```
2) `<branch-name>` から `rust-lang/rust` への PR を作成します

### 新しい Josh サブツリー依存関係の作成

`git subtree` または `git submodule` からリポジトリ依存関係を josh に移行したい場合は、[このガイド](https://hackmd.io/7pOuxnkdQDaL1Y1FQr65xg)をチェックしてください。

### git サブツリーの同期

サブツリーベースの依存関係に加えられた変更は、定期的にこのリポジトリとアップストリームツールリポジトリの間で同期する必要があります。

サブツリーの同期は通常、それぞれのツールメンテナーによって処理されます。他のユーザーも同期 PR を提出することができますが、そのためにはローカル Git インストールを変更し、非常に正確な指示に従う必要があります。これらの指示は、いくつかの有用なヒントやコツとともに、Clippy の Contributing ガイドの [syncing subtree changes][clippy-sync-docs] セクションに文書化されています。指示はサブツリーベースのツールに対して使用できますが、正しい対応するサブツリーディレクトリとリモートリポジトリを使用してください。

同期プロセスは2つの方向に進みます：`subtree push` と `subtree pull` です。

`subtree push` は、このリポジトリのコピーに発生したすべての変更を取得し、ローカルの変更に一致するリモートリポジトリにコミットを作成します。サブツリーに触れたすべてのローカルコミットがリモートリポジトリにコミットを引き起こしますが、ファイルを指定されたディレクトリからツールリポジトリのルートに移動するように変更されます。

`subtree pull` は、最後の `subtree pull` 以降のツールリポジトリからのすべての変更を取得し、これらのコミットをツールの変更を Rust リポジトリの指定されたディレクトリに移動するマージコミットとともに rustc リポジトリに追加します。

常に最初に push を行い、それをツールのデフォルトブランチにマージすることをお勧めします。次に、pull を行うと、競合なくマージが機能します。pull 中に競合を解決することは確かに可能ですが、PR がすぐにマージされず、新しい競合がある場合、競合解決をやり直す必要がある場合があります。`git subtree pull` の結果をリベースしようとしないでください。マージコミットのリベースは一般的に悪い考えです。

常に `-P` プレフィックスをサブツリーディレクトリと対応するリモートリポジトリに指定する必要があります。間違ったディレクトリまたはリポジトリを指定すると、間違ったディレクトリを間違ったリモートリポジトリにプッシュしようとする非常に楽しいマージが発生します。幸いなことに、rustc のプルされたコミットまたはリモートのプッシュされたブランチのいずれかを破棄して再試行することで、結果なしにこれを中止できます。数千のコミットが同期されようとしているため、これが起こっていることは通常かなり明白です。

[clippy-sync-docs]: https://doc.rust-lang.org/nightly/clippy/development/infrastructure/sync.html

### 新しいサブツリー依存関係の作成

既存のリポジトリから新しいサブツリー依存関係を作成したい場合は、（このリポジトリのルートディレクトリから！）次のように呼び出します

```
git subtree add -P src/tools/clippy https://github.com/rust-lang/rust-clippy.git master
```

これにより、いかなる状況でもリベースしてはならない新しいコミットが作成されます！リベースする必要がある場合は、コミットを削除して操作をやり直してください。

これで完了です。`src/tools/clippy` ディレクトリは、Clippy が rustc モノレポの一部であるかのように動作するため、あなた（またはサブツリーを同期する他の人）以外は、実際には `git subtree` を使用する必要がありません。

## 外部依存関係（サブモジュール）

Rust のビルドでは、[git サブモジュール][git submodules]を使用して追跡される外部 Git リポジトリも使用されます。完全なリストは [`.gitmodules`] ファイルにあります。これらのプロジェクトの一部は必須（標準ライブラリ用の `stdarch` など）であり、一部はオプション（`src/doc/book` など）です。

サブモジュールの使用法については、[Git の使用章](git.md#git-submodules)で詳しく説明しています。

一部のサブモジュールは、ビルドされないか、テストが合格しないという「壊れた」状態にあることが許可されています。例えば、[The Rust Reference] のようなドキュメントブックなどです。これらのプロジェクトのメンテナーは、プロジェクトが壊れた状態にあるときに通知され、できるだけ早く修正する必要があります。現在のステータスは [toolstate ウェブサイト][toolstate website]で追跡されています。詳細については、Forge の [Toolstate 章][Toolstate chapter]をご覧ください。実際には、ドキュメントが壊れた toolstate を持つことは非常にまれです。

beta および stable チャネルでは破壊は許可されておらず、PR がマージされる前に対処する必要があります。また、beta カットまでの週に `main` で壊れることも許可されていません。

[git submodules]: https://git-scm.com/book/en/v2/Git-Tools-Submodules
[`.gitmodules`]: https://github.com/rust-lang/rust/blob/HEAD/.gitmodules
[The Rust Reference]: https://github.com/rust-lang/reference/
[toolstate website]: https://rust-lang-nursery.github.io/rust-toolstate/
[Toolstate chapter]: https://forge.rust-lang.org/infra/toolstate.html
[josh]: https://josh-project.github.io/josh/intro.html
[josh-sync]: https://github.com/rust-lang/josh-sync
[on josh-sync README]: https://github.com/rust-lang/josh-sync#git-peculiarities
