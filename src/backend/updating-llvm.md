# LLVMの更新

<!-- date-check: Aug 2024 -->
Rustは複数のLLVMバージョンに対するビルドをサポートしています：

* 現在のLLVM開発ブランチのtip-of-treeは通常、数日以内にサポートされます。そのような修正のためのPRは `llvm-main` でタグ付けされています。
* 最新リリースのメジャーバージョンは常にサポートされています。
* 先行する1つまたは2つのメジャーバージョンは通常サポートされています。

デフォルトでは、Rustは[rust-lang/llvm-project repository]で独自のフォークを使用します。このフォークは、上流プロジェクトの `release/$N.x` ブランチに基づいています。ここで `$N` は最新リリースのメジャーバージョン、または現在リリース候補フェーズにあるメジャーバージョンです。フォークは、`main` 開発ブランチに基づくことはありません。

私たちのLLVMフォークは以下のみを受け入れます：

* 既に上流にランディングした変更のバックポート。
* CI環境に影響するビルド問題の回避策。

SGX有効化のためのgrandfathered-inされた1つのパッチを除き、最初に上流化されていない機能的パッチは受け入れません。

LLVMの更新には、異なる手順を持つ3つのタイプがあります：

* 現在のメジャーLLVMバージョンがサポートされている間のバックポート。
* 現在のメジャーLLVMバージョンがサポートされなくなった後のバックポート（または変更が上流のバックポートの対象外である場合）。
* 新しいメジャーLLVMバージョンへの更新。

## バックポート（上流でサポートされている）

現在のメジャーLLVMバージョンが上流でサポートされている間は、修正を最初に上流でバックポートし、その後リリースブランチをRustフォークにマージバックする必要があります。

1. バグ修正が上流LLVMにあることを確認します。
2. まだ行われていない場合は、上流リリースブランチへのバックポートをリクエストします。LLVMのコミットアクセス権がある場合は、[バックポートプロセス]に従ってください。そうでない場合は、バックポートをリクエストするissueを開きます。バックポートが承認されてマージされたら続行します。
3. rustcが現在使用しているブランチを特定します。`src/llvm-project` サブモジュールは常に[rust-lang/llvm-project repository]のブランチに固定されています。
4. rust-lang/llvm-projectリポジトリをフォークします。
5. 適切なブランチ（通常 `rustc/a.b-yyyy-mm-dd` という名前）をチェックアウトします。
6. `git remote add upstream https://github.com/llvm/llvm-project.git` を使用して上流リポジトリのリモートを追加し、`git fetch upstream` を使用してフェッチします。
7. `upstream/release/$N.x` ブランチをマージします。
8. このブランチをフォークにプッシュします。
9. rust-lang/llvm-projectに以前と同じブランチへのプルリクエストを送信します。PR説明で修正しているRustおよび/またはLLVM issueを必ず参照してください。
10. PRがマージされるのを待ちます。
11. バグ修正を含む `src/llvm-project` サブモジュールを更新するPRをrust-lang/rustに送信します。これは通常、ローカルで `git submodule update --remote src/llvm-project` を実行することで行えます。
12. PRがマージされるのを待ちます。

PR例：
[#59089](https://github.com/rust-lang/rust/pull/59089)

## バックポート（上流でサポートされていない）

上流のLLVMリリースは、GAリリース後2〜3か月間のみサポートされます。上流のバックポートが受け入れられなくなったら、変更を直接フォークにチェリーピックする必要があります。

1. バグ修正が上流LLVMにあることを確認します。
2. rustcが現在使用しているブランチを特定します。`src/llvm-project` サブモジュールは常に[rust-lang/llvm-project repository]のブランチに固定されています。
3. rust-lang/llvm-projectリポジトリをフォークします。
4. 適切なブランチ（通常 `rustc/a.b-yyyy-mm-dd` という名前）をチェックアウトします。
5. `git remote add upstream https://github.com/llvm/llvm-project.git` を使用して上流リポジトリのリモートを追加し、`git fetch upstream` を使用してフェッチします。
6. `git cherry-pick -x` を使用して関連するコミットをチェリーピックします。
7. このブランチをフォークにプッシュします。
8. rust-lang/llvm-projectに以前と同じブランチへのプルリクエストを送信します。PR説明で修正しているRustおよび/またはLLVM issueを必ず参照してください。
9. PRがマージされるのを待ちます。
10. バグ修正を含む `src/llvm-project` サブモジュールを更新するPRをrust-lang/rustに送信します。これは通常、ローカルで `git submodule update --remote src/llvm-project` を実行することで行えます。
11. PRがマージされるのを待ちます。

PR例：
[#59089](https://github.com/rust-lang/rust/pull/59089)

## 新しいLLVMリリースの更新

<!-- date-check: Jul 2023 -->

バグ修正とは異なり、LLVMの新しいリリースへの更新は通常、はるかに多くの作業が必要です。ここでは、コミットを後方にチェリーピックすることが合理的にできないため、完全な更新を行う必要があります。ここでやるべきことがたくさんあるので、それぞれを詳しく見ていきましょう。

1. LLVMは、最新のリリースバージョンがブランチされたことを発表します。これは[llvm/llvm-project repository]のブランチとして表示され、通常 `release/$N.x` という名前です。ここで `$N` はリリースされるLLVMのバージョンです。

1. この `release/$N.x` ブランチから[rust-lang/llvm-project repository]に新しいブランチを作成し、`rustc/a.b-yyyy-mm-dd` という名前を付けます。ここで `a.b` はブランチ時のLLVMのツリー内の現在のバージョン番号で、残りの部分は現在の日付です。

1. llvm-projectリポジトリにRust固有のパッチを適用します。すべての機能とバグ修正は上流にありますが、上流化する意味がない奇妙なビルド関連のパッチがしばしばあります。これらのパッチは通常、rustcが現在使用しているrust-lang/llvm-projectブランチの最新のパッチです。

1. `rust` リポジトリで新しいLLVMをビルドします。これを行うには、`src/llvm-project` リポジトリを自分のブランチと作成したリビジョンに更新します。通常、`.gitmodules` をLLVMサブモジュールの新しいブランチ名で更新することもお勧めです。サブモジュールの更新が元に戻らないように、`src/llvm-project` への変更をコミットしたことを確認してください。実行すべきコマンドをいくつか示します：

   * `./x build src/llvm-project` - LLVMがまだビルドされることをテスト
   * `./x build` - rustcの残りをビルド

   更新されたLLVMバインディングでコンパイルするために、[`llvm-wrapper/*.cpp`][`llvm-wrapper`]を更新する必要がある可能性があります。古いLLVMバージョンでもバインディングがコンパイルされるように、`#ifdef` などを使用する必要があることに注意してください。

   `profile = "compiler"` や `./x setup` で設定される他のデフォルトは、ソースからビルドする代わりにCIからLLVMをダウンロードします。変更が使用されていることを確認するために、これを一時的に無効にする必要があります。これは、`bootstrap.toml` に次の設定を含めることで行います：

   ```toml
   [llvm]
   download-ci-llvm = false
   ```

1. 他のプラットフォームでの回帰をテストします。LLVMには、非Tier-1アーキテクチャ用に少なくとも1つのバグがあることが多いため、これをborsに送信する前にもう少しテストを行うことをお勧めします！リソースが不足している場合は、現状のままPRをborsに送信できますが、とにかくテストされます。

   理想的には、いくつかのプラットフォームでLLVMをビルドしてテストします：

   * Linux
   * macOS
   * Windows

   その後、CIも実行するいくつかのDockerコンテナを実行します：

   * `./src/ci/docker/run.sh wasm32`
   * `./src/ci/docker/run.sh arm-android`
   * `./src/ci/docker/run.sh dist-various-1`
   * `./src/ci/docker/run.sh dist-various-2`
   * `./src/ci/docker/run.sh armhf-gnu`

1. `rust-lang/rust` へのPRを準備します。`rust-lang/llvm-project` のメンテナと協力して、そのリポジトリのブランチにコミットを入れ、その後 `rust-lang/rust` にPRを送信できます。少なくとも `src/llvm-project` を変更し、おそらく [`llvm-wrapper`] も変更します。

   <!-- date-check: mar 2025 -->
   > 以前のLLVM更新の例：
   > * [LLVM 17](https://github.com/rust-lang/rust/pull/115959)
   > * [LLVM 18](https://github.com/rust-lang/rust/pull/120055)
   > * [LLVM 19](https://github.com/rust-lang/rust/pull/127513)
   > * [LLVM 20](https://github.com/rust-lang/rust/pull/135763)

   [`llvm-wrapper`] の互換性を、実際に `src/llvm-project` を更新する前のPRとしてランディングするのが最も簡単な場合があります。この方法では、LLVMの問題に取り組んでいる間、新しいLLVMを試すことに興味のある他の人が、C++バインディングの更新作業から恩恵を受けることができます。

1. 今後数か月にわたり、LLVMは継続的に `release/a.b` ブランチにコミットをプッシュします。これらのバグ修正も必要になることがよくあります。そのマージプロセスは、`git merge` 自体を使用してLLVMの `release/a.b` ブランチをステップ2で作成したブランチとマージすることです。これは通常、LLVMのリリースブランチが焼き上がっている間に必要に応じて複数回行われます。

1. その後、LLVMはバージョン `a.b` のリリースを発表します。

1. LLVMの公式リリース後、rust-lang/llvm-projectリポジトリに新しいブランチを再度作成するプロセスに従います。今回は新しい日付で作成します。Rustをそのバージョンを使用するように更新するPRがマージされるのはその時だけです。

   `rust-lang/llvm-project` のコミット履歴は、`git rebase` が行われるため、はるかにきれいに見えるはずです。ここでは、いくつかのRust固有のコミットだけがLLVMのストックリリースブランチの上にスタックされます。

### 注意点と落とし穴

理想的には、上記の指示はかなりスムーズですが、それらを進める際に心に留めておくべきいくつかの注意点があります：

* LLVMのバグは見つけるのが難しいです。遠慮なくヘルプを求めてください！二分探索は間違いなくここでの友達です（LLVMのビルドには永遠にかかりますが、それでも二分探索はあなたの友達です）。[Dev Desktops]を利用できることに注意してください。これは、貢献者に強力なハードウェアへのリモートアクセスを提供するイニシアチブです。
* 一般的な質問がある場合は、[wg-llvm]が役立ちます。
* GitHubでのブランチの作成は特権的な操作なので、おそらく書き込みアクセス権を持つ誰かにブランチを作成してもらう必要があります。

[rust-lang/llvm-project repository]: https://github.com/rust-lang/llvm-project
[llvm/llvm-project repository]: https://github.com/llvm/llvm-project
[`llvm-wrapper`]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_llvm/llvm-wrapper
[wg-llvm]: https://rust-lang.zulipchat.com/#narrow/stream/187780-t-compiler.2Fwg-llvm
[Dev Desktops]: https://forge.rust-lang.org/infra/docs/dev-desktop.html
