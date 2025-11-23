# コンパイラのビルドと実行方法

<div class="warning">

`profile = "library"` ユーザー、または `download-rustc = true | "if-unchanged"` を使用するユーザーへ：
`download-rustc` がアクティブな場合（つまり、コンパイラの変更がない場合）の `./x test library/std` フローは現在壊れています。
これは <https://github.com/rust-lang/rust/issues/142505> で追跡されています。このケースでは `./x test` フローのみが影響を受けます。`./x {check,build} library/std` は引き続き機能するはずです。

短期的には、`./x test library/std` のために `download-rustc` を無効にする必要がある場合があります。これは次のいずれかの方法で行うことができます：

1. `./x test library/std --set rust.download-rustc=false`
2. または `bootstrap.toml` で `rust.download-rustc=false` を設定します。

残念ながら、これには stage 1 コンパイラのビルドが必要です。ブートストラップチームはこれに取り組んでいますが、保守可能な修正の実装には時間がかかっています。

</div>


コンパイラは `x.py` というツールを使用してビルドされます。これを実行するには Python がインストールされている必要があります。

## クイックスタート

コンパイラを実行するための簡易的なクイックスタートについては、[クイックスタート](./quickstart.md) をご覧ください。


## ソースコードの取得

メインリポジトリは [`rust-lang/rust`][repo] です。これには、コンパイラ、標準ライブラリ（`core`、`alloc`、`test`、`proc_macro` などを含む）、および多数のツール（例：`rustdoc`、ブートストラッピングインフラストラクチャなど）が含まれています。

[repo]: https://github.com/rust-lang/rust

`rustc` で作業するための最初のステップは、リポジトリをクローンすることです：

```bash
git clone https://github.com/rust-lang/rust.git
cd rust
```

### リポジトリの部分クローン

リポジトリのサイズが大きいため、遅いインターネット接続でクローンすると時間がかかり、すべてのファイルとディレクトリの完全な履歴を保存するためのディスク容量が必要です。代わりに、git に _部分クローン_ を実行するように指示することができます。これにより、現在のファイルの内容のみを完全に取得し、履歴を遡るときなどにさらなるファイルの内容を自動的に取得します。すべての git コマンドは通常どおり動作し続けますが、未ロードの履歴のポイントを訪れるにはインターネット接続が必要になるという代償があります。

```bash
git clone --filter='blob:none' https://github.com/rust-lang/rust.git
cd rust
```

> **注意**: [このリンク](https://github.blog/open-source/git/get-up-to-speed-with-partial-clone-and-shallow-clone/) は、このタイプのチェックアウトについて詳しく説明しており、シャロークローンなどの他のモードと比較しています。

### リポジトリのシャロークローン

部分クローンの古い代替方法は、代わりにリポジトリをシャロークローンすることです。これを行うには、`git clone` コマンドで `--depth N` オプションを使用できます。これは、`git` にリポジトリをクローンするが、最後の `N` コミットに切り詰めるように指示します。

`--depth 1` を渡すと、`git` にリポジトリをクローンするが、`main` ブランチにある最新のコミットに履歴を切り詰めるように指示します。これは通常、ソースコードを閲覧したり、コンパイラをビルドしたりするには問題ありません。

```bash
git clone --depth 1 https://github.com/rust-lang/rust.git
cd rust
```

> **注意**: シャロークローンは、実行できる `git` コマンドを制限します。コンパイラに取り組み、貢献する予定がある場合は、一般的に [上記のように](#get-the-source-code) リポジトリを完全にクローンするか、代わりに [部分クローン](#partial-clone-the-repository) を実行することをお勧めします。
>
> たとえば、`git bisect` と `git blame` はコミット履歴へのアクセスを必要とするため、リポジトリが `--depth 1` でクローンされた場合は機能しません。

## `x.py` とは何ですか？

`x.py` は `rust` リポジトリのビルドツールです。ドキュメントをビルドし、テストを実行し、コンパイラと標準ライブラリをコンパイルできます。

この章では、生産的になるための基本に焦点を当てていますが、`x.py` についてもっと学びたい場合は、[この章を読んでください][bootstrap]。

[bootstrap]: ./bootstrapping/intro.md

また、`x.py` ではなく `x` を使用することをお勧めします：

> `./x` は、すべてのシステムで最も動作する可能性が高いです（Unix では Python バージョン検出を行うシェルスクリプトを実行し、Windows ではおそらく PowerShell スクリプトを実行します -- `./x.py` よりも壊れる可能性が低く、これは多くの場合ファイルをエディタで開くだけです）。[^1]

（`x.py` の周りには、`x.ps1` のようなプラットフォーム関連のスクリプトがあります）

これは絶対的なものではないことに注意してください。たとえば、Win10 の VSCode で Nushell を使用している場合、`x` または `./x` と入力しても、プログラムを呼び出すのではなく、エディタで `x.py` を開きます。:)

このガイドの残りの部分では、`x.py` ではなく `x` を直接使用します。次のコマンド：

```bash
./x check
```

は、次のように置き換えることができます：

```bash
./x.py check
```

### `x.py` の実行

`x.py` コマンドは、ほとんどの Unix システムで次の形式で直接実行できます：

```sh
./x <subcommand> [flags]
```

これは、ドキュメントと例が `x.py` を実行していると想定している方法です。いくつかの代替方法があります：

```sh
# 必要な `python3` コマンドがない場合は Unix シェルで
./x <subcommand> [flags]

# Windows Powershell で（PowerShell がスクリプトを実行するように設定されている場合）
./x <subcommand> [flags]
./x.ps1 <subcommand> [flags]

# Windows コマンドプロンプトで（.py ファイルが Python を実行するように設定されている場合）
x.py <subcommand> [flags]

# Python を自分で実行することもできます、例：
python x.py <subcommand> [flags]
```

Windows では、PowerShell コマンドで次のようなエラーが発生する場合があります：
```
PS C:\Users\vboxuser\rust> ./x
./x : File C:\Users\vboxuser\rust\x.ps1 cannot be loaded because running scripts is disabled on this system. For more
information, see about_Execution_Policies at https://go.microsoft.com/fwlink/?LinkID=135170.
At line:1 char:1
+ ./x
+ ~~~
    + CategoryInfo          : SecurityError: (:) [], PSSecurityException
    + FullyQualifiedErrorId : UnauthorizedAccess
```

PowerShell がローカルスクリプトを実行できるようにすることで、このエラーを回避できます：
```
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

#### `x.py` をもう少し便利に実行する

`src/tools/x` に `x` という `x.py` をラップするバイナリがあります。それが行うことは `x.py` を実行することだけですが、システム全体にインストールでき、チェックアウトの任意のサブディレクトリから実行できます。また、使用する適切なバージョンの `python` も検索します。

`cargo install --path src/tools/x` でインストールできます。

これが、[`x.py` とは何ですか](#what-is-xpy) セクションで宣言されているものと似ているグローバルにインストールされたバイナリユーティリティであることを明確にするために、これはシェルを呼び出してプラットフォーム関連のスクリプトを実行するのではなく、`x.py` を実行する独立したプロセスとして機能します。

## `bootstrap.toml` の作成

開始するには、`./x setup` を実行し、`compiler` デフォルトを選択します。これにより、いくつかの初期化が行われ、妥当なデフォルトで `bootstrap.toml` が作成されます。別のデフォルトを使用する場合（rustdoc など、コンパイラ以外の Rust の領域に貢献したい場合）、そのデフォルトに関する情報（`src/bootstrap/defaults` にあります）を読むようにしてください。他のデフォルトではビルドプロセスが異なる場合があります。

または、`bootstrap.toml` を手動で書くこともできます。利用可能なすべての設定と説明については、`bootstrap.example.toml` をご覧ください。変更する一般的な設定については、`src/bootstrap/defaults` をご覧ください。

すでに `rustc` をビルドしていて、LLVM に関連する設定を変更した場合、後続の設定変更を有効にするために `./x clean --all` を実行する必要がある場合があります。`./x clean` は LLVM の再ビルドを引き起こさないことに注意してください。

## 一般的な `x` コマンド

以下は、`rustc`、`std`、`rustdoc`、およびその他のツールで作業する際に最も一般的に使用される `x` コマンドの基本的な呼び出しです。

| コマンド | 使用するタイミング |
| ----------- | ------------------------------------------------------------------------------------------------------------ |
| `./x check` | ほとんどのものがコンパイルされるかどうかを素早くチェック；[rust-analyzer はこれを自動的に実行できます][rust-analyzer] |
| `./x build` | `rustc`、`std`、および `rustdoc` をビルド |
| `./x test` | すべてのテストを実行 |
| `./x fmt` | すべてのコードをフォーマット |

書かれているように、これらのコマンドは妥当な出発点です。ただし、深刻な開発作業には、それぞれに追加のオプションと引数があることを知っておく価値があります。特に、`./x build` と `./x test` は、コードのサブセットをコンパイルまたはテストする多くの方法を提供し、多くの時間を節約できます。

また、`x` は `compiler`、`library`、および `src/tools` ディレクトリのすべての種類のパスサフィックスをサポートしていることに注意してください。したがって、`x test src/tools/tidy` の代わりに `x test tidy` を単純に実行できます。または、`x build library/std` の代わりに `x build std` を実行できます。

[rust-analyzer]: suggested.html#configuring-rust-analyzer-for-rustc

テストと rustdoc の詳細については、[testing](../tests/running.md) と [rustdoc](../rustdoc.md) の章をご覧ください。

### コンパイラのビルド

ビルドには比較的大量のストレージスペースが必要です。コンパイラをビルドするには、10 または 15 ギガバイト以上の空き容量が必要な場合があります。

`bootstrap.toml` を作成したら、`x` を実行する準備が整いました。ここには多くのオプションがありますが、ローカルコンパイラをビルドするための最良の「go to」コマンドから始めましょう：

```console
./x build library
```

このコマンドは次のことを行います：
- stage0 コンパイラと stage0 `std` を使用して `rustc` をビルドします。
- ちょうどビルドされた stage1 コンパイラで `library`（標準ライブラリ）をビルドします。
- stage1 コンパイラと stage1 標準ライブラリを含む、動作する stage1 sysroot を組み立てます。

この最終製品（stage1 コンパイラ + そのコンパイラを使用してビルドされたライブラリ）は、他の Rust プログラムをビルドするために必要なものです（`#![no_std]` または `#![no_core]` を使用しない限り）。

stage1 `std` のビルドがボトルネックになる可能性がありますが、恐れることはありません。（ハッキーな）回避策があります... std の再ビルドを回避する方法については、[セクション][keep-stage] をご覧ください。

[keep-stage]: ./suggested.md#faster-rebuilds-with---keep-stage-std

時には、完全なビルドが必要ない場合があります。メソッドの名前を変更したり、関数のシグネチャを変更したりするような「型ベースのリファクタリング」を行う場合、はるかに高速なビルドのために代わりに `./x check` を使用できます。

このコマンド全体は、完全な `rustc` ビルドのサブセットを提供するだけであることに注意してください。**完全な** `rustc` ビルド（`./x build --stage 2 rustc` で取得するもの）には、さらにいくつかのステップがあります：

- stage1 コンパイラで `rustc` をビルドします。
  - ここで得られるコンパイラは「stage2」コンパイラと呼ばれ、前のコマンドの stage1 std を使用します。
- stage2 コンパイラで `librustdoc` および他の多くのものをビルドします。

これはほぼ必要ありません。

### 特定のコンポーネントのビルド

標準ライブラリで作業している場合、おそらく他のすべてのデフォルトコンポーネントをビルドする必要はありません。代わりに、次のように名前を指定することで、特定のコンポーネントをビルドできます：

```bash
./x build --stage 1 library
```

`x setup` で `library` プロファイルを選択した場合、`--stage 1` を省略できます（これがデフォルトです）。

## rustup ツールチェーンの作成

`rustc` を正常にビルドすると、`build` ディレクトリに多数のファイルが作成されます。実際に結果の `rustc` を実行するには、rustup ツールチェーンを作成することをお勧めします。最初のものは stage1 コンパイラ（上でビルドしたもの）を実行します。2 番目のものは stage2 コンパイラを実行します（ビルドしていませんが、ある時点でビルドする必要がある可能性があります；たとえば、テストスイート全体を実行したい場合）。

```bash
rustup toolchain link stage1 build/host/stage1
rustup toolchain link stage2 build/host/stage2
```

これで、ビルドした `rustc` を実行できます。`-vV` で実行すると、ローカル環境からのビルドを示す `-dev` で終わるバージョン番号が表示されるはずです：

```bash
$ rustc +stage1 -vV
rustc 1.48.0-dev
binary: rustc
commit-hash: unknown
commit-date: unknown
host: x86_64-unknown-linux-gnu
release: 1.48.0-dev
LLVM version: 11.0
```

rustup ツールチェーンは、`build` ディレクトリ内のコンパイルされた指定されたツールチェーンを指しているため、rustup ツールチェーンは、そのツールチェーン/ステージのために `x build` または `x test` が実行されるたびに更新されます。

**注意**: ビルドしたツールチェーンには `cargo` が含まれていません。この場合、`rustup` はインストールされた `nightly`、`beta`、または `stable` ツールチェーンからの `cargo` の使用にフォールバックします（この順序で）。不安定な `cargo` フラグを使用する必要がある場合は、まだインストールしていない場合は `rustup install nightly` を実行してください。詳細については、[rustup documentation on custom toolchains](https://rust-lang.github.io/rustup/concepts/toolchains.html#custom-toolchains) をご覧ください。

**注意**: rust-analyzer と IntelliJ Rust プラグインは、proc マクロで動作するために `rust-analyzer-proc-macro-srv` というコンポーネントを使用します。プロジェクトにカスタムツールチェーンを使用する予定がある場合（例：`rustup override set stage1` 経由で）、このコンポーネントをビルドすることをお勧めします：

```bash
./x build proc-macro-srv-cli
```

## クロスコンパイル用のターゲットのビルド

他のターゲット用にクロスコンパイルできるコンパイラを生成するには、任意の数の `target` フラグを `x build` に渡します。たとえば、ホストプラットフォームが `x86_64-unknown-linux-gnu` で、クロスコンパイルターゲットが `wasm32-wasip1` の場合、次のようにビルドできます：

```bash
./x build --target x86_64-unknown-linux-gnu,wasm32-wasip1
```

結果のコンパイラが proc マクロまたはビルドスクリプトを含むクレートをビルドできるようにする場合は、ホストプラットフォーム（この場合、`x86_64-unknown-linux-gnu`）のターゲットサポートを明示的にビルドする必要があることに注意してください。

`x build` にフラグを渡さずに常に他のターゲット用にビルドしたい場合は、`bootstrap.toml` の `[build]` セクションで次のように設定できます：

```toml
[build]
target = ["x86_64-unknown-linux-gnu", "wasm32-wasip1"]
```

一部のターゲット用にビルドするには、外部依存関係をインストールする必要があることに注意してください（例：musl ターゲット用にビルドするには、musl のローカルコピーが必要です）。ターゲット固有の設定（例：musl のローカルコピーへのパス）は、`bootstrap.toml` で提供する必要があります。ターゲット固有の設定キーについては、`bootstrap.example.toml` をご覧ください。

ターゲットをビルドするために必要な完全な設定の例については、[the rustc book](https://doc.rust-lang.org/rustc/platform-support.html) をご覧ください。左側の「Platform Support」見出しの下で任意のターゲットを選択し、そのターゲット用のコンパイラをビルドすることに関連するセクションをご覧ください。rustc ブックに対応するページがないターゲットの場合、Rust インフラストラクチャ自体がクロスコンパイルを設定するために使用する [Dockerfiles を検査する](../tests/docker.md) ことが役立つ場合があります。

前のセクションから rustup ツールチェーンを作成する手順に従った場合、コンパイラをビルドした後、次のようにクロスコンパイルに使用できます：

```bash
cargo +stage1 build --target wasm32-wasip1
```

## その他の `x` コマンド

他にも便利な `x` コマンドがいくつかあります。その一部については、他のセクションで詳しく説明します：

- ものをビルドする：
  - `./x build` – stage 1 コンパイラを使用してすべてをビルドします。`std` までではありません
  - `./x build --stage 2` – stage 2 コンパイラですべてをビルドします。`rustdoc` を含みます
- テストの実行（詳細については、[テストの実行に関するセクション](../tests/running.html) をご覧ください）：
  - `./x test library/std` – `std` からユニットテストと統合テストを実行します
  - `./x test tests/ui` – `ui` テストスイートを実行します
  - `./x test tests/ui/const-generics` - `ui` テストスイートの `const-generics/` サブディレクトリのすべてのテストを実行します
  - `./x test tests/ui/const-generics/const-types.rs` - `ui` テストスイートから単一のテスト `const-types.rs` を実行します

### ビルドディレクトリのクリーンアップ

時々、新しく始める必要がありますが、これは通常のケースではありません。これを実行する必要がある場合、ブートストラップがおそらく正しく動作していないため、何が間違っているかについてバグを報告すべきです。すべてをクリーンアップする必要がある場合は、1 つのコマンドを実行するだけです！

```bash
./x clean
```

`rm -rf build` も機能しますが、LLVM を再ビルドする必要があり、高速なコンピュータでも長い時間がかかる場合があります。

## ディスクスペースに関する注意

コンパイラをビルドする（特に stage 1 を超える場合）には、かなりの量の空きディスクスペースが必要になる場合があります。おそらく約 100GB です。rust-analyzer 用に別のビルドディレクトリがある場合（例：`build-rust-analyzer`）、これは増大します。これは、各ユーザーに [設定されたディスククォータ](https://github.com/rust-lang/simpleinfra/blob/8a59e4faeb75a09b072671c74a7cb70160ebef50/ansible/roles/dev-desktop/defaults/main.yml#L7) がある dev-desktop で簡単にヒットしますが、これはローカル開発にも適用されます。時々、次のことを行う必要がある場合があります：

- `build/` ディレクトリを削除します。
- `build-rust-analyzer/` ディレクトリを削除します（rust-analyzer 用の別のビルドディレクトリがある場合）。
- `cargo-bisect-rustc` を使用する場合、不要なツールチェーンをアンインストールします。`rustup toolchain list` でどのツールチェーンがインストールされているかを確認できます。

[^1]: issue[#1707](https://github.com/rust-lang/rustc-dev-guide/issues/1707)
