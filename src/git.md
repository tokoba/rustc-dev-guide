# Git の使用

Rust プロジェクトは、ソースコードを管理するために [Git] を使用しています。貢献するには、変更がコンパイラに組み込まれるように、その機能にある程度精通している必要があります。

[Git]: https://git-scm.com

このページの目標は、新しいコントリビューターが直面するより一般的な質問や問題のいくつかをカバーすることです。ここではいくつかの Git の基本について説明しますが、これがまだ少し速すぎると感じる場合は、[Atlassian のこのチュートリアル][atlassian-git]の Beginner と Getting started セクションなど、Git の入門記事をまず読むことをお勧めします。GitHub も初心者向けの[ドキュメント][documentation]と[ガイド][guides]を提供しています。あるいは、より詳しい [Git の本][book from Git]を参照することもできます。

このガイドは不完全です。このページが役に立たない Git の問題に遭遇した場合は、[issue を開いて][open an issue]、修正方法を文書化できるようにしてください。

[open an issue]: https://github.com/rust-lang/rustc-dev-guide/issues/new
[book from Git]: https://git-scm.com/book/en/v2/
[atlassian-git]: https://www.atlassian.com/git/tutorials/what-is-version-control
[documentation]: https://docs.github.com/en/get-started/quickstart/set-up-git
[guides]: https://guides.github.com/introduction/git-handbook/

## 前提条件

Git をインストールし、[rust-lang/rust] をフォークし、フォークされたリポジトリを PC にクローンしたと仮定します。Git とのやり取りにはコマンドラインインターフェイスを使用します。一般的に同じことができる GUI や IDE 統合も多数あります。

[rust-lang/rust]: https://github.com/rust-lang/rust

フォークをクローンした場合、ローカルリポジトリで `origin` を使用して参照できます。公式の rust-lang/rust リポジトリのリモートも設定すると便利です。

```console
git remote add upstream https://github.com/rust-lang/rust.git
```

HTTPS を使用している場合、または

```console
git remote add upstream git@github.com:rust-lang/rust.git
```

SSH を使用している場合です。

**注意**：このページは `rust-lang/rust` のワークフローに特化していますが、Rust プロジェクトの他のリポジトリに貢献する際にも役立つでしょう。


## 標準プロセス

以下は、ほとんどの小さな変更と PR に使用する可能性が高い通常の手順です：

 1. 変更が `main` の上で行われていることを確認します：`git checkout main`。
 2. Rust リポジトリから最新の変更を取得します：`git pull upstream main --ff-only`（詳細については [No-Merge Policy][no-merge-policy] を参照してください）。
 3. 変更用の新しいブランチを作成します：`git checkout -b issue-12345-fix`。
 4. リポジトリに変更を加えてテストします。
 5. `git add src/changed/file.rs src/another/change.rs` で変更をステージし、`git commit` でコミットします。もちろん、途中でコミットを作成することも良い考えです。`git add .` は避けてください。サブモジュールの更新など、コミットすべきでない変更を誤ってコミットしやすくなるためです。`git status` を使用して、ステージし忘れたファイルがあるかどうかを確認できます。
 6. 変更をフォークにプッシュします：`git push --set-upstream origin issue-12345-fix`（コミットを追加した後は `git push` を使用でき、リベースまたはプルアンドリベース後は `git push --force-with-lease` を使用できます）。
 7. フォークから `rust-lang/rust` の `main` ブランチへの [PR を開きます][ghpullrequest]。

[ghpullrequest]: https://guides.github.com/activities/forking/#making-a-pull-request

リベースが必要で競合が発生している場合は、[リベース](#rebasing)を参照してください。長期実行中の機能/issue で作業している間に upstream を追跡したい場合は、[最新の状態に保つ][no-merge-policy]を参照してください。

レビュアーが変更を要求した場合、変更の手順は同じですが、いくつかのステップがスキップされます：

 1. コードの最新バージョンに変更を加えていることを確認します：`git checkout issue-12345-fix`。
 2. 以前と同じように、追加の変更を行い、ステージし、コミットします。
 3. これらの変更をフォークにプッシュします：`git push`。

 [no-merge-policy]: #keeping-things-up-to-date

## Git の問題のトラブルシューティング

古くなっている場合、`rust-lang/rust` をゼロからクローンする必要はありません！修復不可能なほど混乱させたと思っても、リポジトリ全体を再度ダウンロードする必要のない Git の状態を修正する方法があります。以下は、遭遇する可能性のある一般的な問題です：

### 誤ってマージコミットを作成してしまいました。

Git には、ブランチを最新の変更で更新する2つの方法があります：マージとリベースです。Rust は[リベースを使用します][no-merge-policy]。マージコミットを作成してしまった場合、修正するのはそれほど難しくありません：`git rebase -i upstream/main`。

リベースの詳細については、[リベース](#rebasing)を参照してください。

### GitHub でフォークを削除してしまいました！

これは Git の観点からは問題ではありません。`git remote -v` を実行すると、次のように表示されます：

```console
$ git remote -v
origin  git@github.com:jyn514/rust.git (fetch)
origin  git@github.com:jyn514/rust.git (push)
upstream        https://github.com/rust-lang/rust (fetch)
upstream        https://github.com/rust-lang/rust (fetch)
```

フォークの名前を変更した場合は、次のように URL を変更できます：

```console
git remote set-url origin <URL>
```

ここで `<URL>` は新しいフォークです。

### 誤ってサブモジュールを変更してしまいました

通常、人々は rustbot が `cargo` が変更されたというコメントを GitHub に投稿したときにこれに気付きます：

![rustbot submodule comment](./img/rustbot-submodules.png)

Web UI で競合に気付くこともあります：

![conflict in src/tools/cargo](./img/submodule-conflicts.png)

最も一般的な原因は、変更後にリベースし、最初に `x` を実行してサブモジュールを更新せずに `git add .` を実行したことです。あるいは、`x fmt` の代わりに `cargo fmt` を実行してサブモジュール内のファイルを変更し、その後変更をコミットした可能性があります。

修正するには、次のことを行います（cargo 以外のサブモジュールを変更した場合は、`src/tools/cargo` をそのサブモジュールへのパスに置き換えてください）：

1. どのコミットに誤った変更があるかを確認します：`git log --stat -n1 src/tools/cargo`
2. そのコミットへの変更を元に戻します：`git checkout <my-commit>~ src/tools/cargo`。`~` を文字通り入力しますが、`<my-commit>` はステップ 1 の出力に置き換えます。
3. Git に変更をコミットするよう伝えます：`git commit --fixup <my-commit>`
4. 変更したすべてのサブモジュールについて、ステップ 1〜3 を繰り返します。
    - 複数の異なるコミットでサブモジュールを変更した場合は、変更した各コミットに対してステップ 1〜3 を繰り返す必要があります。`git log` コマンドが自分が作成していないコミットを表示したときに停止する必要があります。
5. 既存のコミットに変更をスカッシュします：`git rebase --autosquash -i upstream/main`
6. [変更をプッシュします](#standard-process)。

### リベースしようとすると「error: cannot rebase」と表示されます

リベースするときによく見られる2つのエラー：
```console
error: cannot rebase: Your index contains uncommitted changes.
error: Please commit or stash them.
```
```console
error: cannot rebase: You have unstaged changes.
error: Please commit or stash them.
```

（2つの違いについては、<https://git-scm.com/book/en/v2/Getting-Started-What-is-Git%3F#_the_three_states> を参照してください。）

これは、最後にコミットを作成してから変更を加えたことを意味します。リベースできるようにするには、変更をコミットするか、リベースを終了したときにまだコミットされないようにする「スタッシュ」と呼ばれる一時的なコミットを作成します。Git がこの「スタッシュ」を自動的に作成するように設定することをお勧めします。これにより、ほぼすべての場合に「cannot rebase」エラーを防ぐことができます：

```console
git config --global rebase.autostash true
```

スタッシュの詳細については、<https://git-scm.com/book/en/v2/Git-Tools-Stashing-and-Cleaning> を参照してください。

### 'Untracked Files: src/stdarch' と表示されます。

これは `library/` ディレクトリへの移動から残されたものです。残念ながら、`git rebase` はサブモジュールの名前変更に従わないため、ディレクトリを自分で削除する必要があります：

```console
rm -r src/stdarch
```

### `<<< HEAD` と表示されます。

おそらくリベースまたはマージ競合の最中です。競合を修正する方法については、[競合](#rebasing-and-conflicts)を参照してください。変更を気にせず、リポジトリのクリーンなコピーを取得したいだけの場合は、`git reset` を使用できます：

```console
# 警告: これはローカルの変更をすべて破棄します！代わりに競合を解決することを検討してください。
git reset --hard main
```

### failed to push some refs

`git push` は正しく機能せず、次のように表示されます：

```console
 ! [rejected]        issue-xxxxx -> issue-xxxxx (non-fast-forward)
error: failed to push some refs to 'https://github.com/username/rust.git'
hint: Updates were rejected because the tip of your current branch is behind
hint: its remote counterpart. Integrate the remote changes (e.g.
hint: 'git pull ...') before pushing again.
hint: See the 'Note about fast-forwards' in 'git push --help' for details.
```

これが提供するアドバイスは正しくありません！Rust の [「no-merge」ポリシー](#no-merge-policy)のため、`git pull` によって作成されるマージコミットは最終的な PR では許可されず、リベースの目的を無効にします！代わりに `git push --force-with-lease` を使用してください。

### Git が自分が書いていないコミットをリベースしようとします。

リベースリストに多数のコミット、マージコミット、または自分が書いていない他の人のコミットが表示される場合は、間違ったブランチにリベースしようとしている可能性があります。例えば、`rust-lang/rust` リモート `upstream` があるのに、`git rebase upstream/main` の代わりに `git rebase origin/main` を実行した可能性があります。修正するには、リベースを中止して代わりに正しいブランチを使用します：

```console
git rebase --abort
git rebase --interactive upstream/main
```

<details><summary>間違ったブランチにリベースする例を表示するにはここをクリックしてください</summary>

![Interactive rebase over the wrong branch](img/other-peoples-commits.png)

</details>

### サブモジュールに関するクイックノート

`git pull` でローカルリポジトリを更新すると、編集したことのないファイルが変更されたと Git が言うことがあります。例えば、`git status` を実行すると次のように表示されます（`new commits` の言及に注意してください）：

```console
On branch main
Your branch is up to date with 'origin/main'.

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
	modified:   src/llvm-project (new commits)
	modified:   src/tools/cargo (new commits)

no changes added to commit (use "git add" and/or "git commit -a")
```

これらの変更はファイルへの変更ではありません：サブモジュールへの変更です（詳細は[後で](#git-submodules)）。これらを取り除くには：

```console
git submodule update
```

一部のサブモジュールは実際には必要ありません。例えば、`download-ci-llvm` を使用している場合、`src/llvm-project` をチェックアウトする必要はありません。履歴を継続的にフェッチする必要を避けるために、`git submodule deinit -f src/llvm-project` を使用できます。これにより、再度変更されたものとして表示されることも回避されます。

## リベースと競合

ローカルでコードを編集するとき、フィーチャーブランチを作成したときに存在していた rust-lang/rust のバージョンに変更を加えています。そのため、PR を送信するときに、その後 rust-lang/rust に加えられた変更の一部が、加えた変更と競合している可能性があります。これが発生した場合、変更をマージする前に競合を解決する必要があります。そのためには、rust-lang/rust の上に作業をリベースする必要があります。

### リベース

フィーチャーブランチを rust-lang/rust の `main` ブランチの最新バージョンの上にリベースするには、ブランチをチェックアウトし、次のコマンドを実行します：

```console
git pull --rebase https://github.com/rust-lang/rust.git main
```

> 次のエラーが表示された場合：
> ```console
> error: cannot pull with rebase: Your index contains uncommitted changes.
> error: please commit or stash them.
> ```
> これは、作業ツリーにコミットされていない作業があることを意味します。その場合、リベースする前に `git stash` を実行し、リベースしてすべての競合を修正した後に `git stash pop` を実行します。

main でブランチをリベースすると、ブランチのすべての変更が `main` の最新バージョンに再適用されます。言い換えれば、Git は古いバージョンの `main` に加えた変更が、代わりに新しいバージョンの `main` に加えられたふりをしようとします。このプロセス中に、少なくとも1つの「リベース競合」に遭遇することが予想されます。これは、Git が変更を再適用しようとする試みが、他の変更と競合したため失敗したときに発生します。次のような行が出力に表示されるため、これが発生したことがわかります：

```console
CONFLICT (content): Merge conflict in file.rs
```

これらのファイルを開くと、次のような形式のセクションが表示されます：

```console
<<<<<<< HEAD
Original code
=======
Your code
>>>>>>> 8fbf656... Commit fixes 12345
```

これは、Git がリベース方法を理解できなかったファイル内の行を表します。`<<<<<<< HEAD` と `=======` の間のセクションには `main` のコードがあり、もう一方の側にはあなたのバージョンのコードがあります。競合にどのように対処するかを決定する必要があります。変更を保持するか、`main` の変更を保持するか、2つを組み合わせることができます。

一般的に、競合の解決は2つのステップで構成されます：まず、特定の競合を修正します。ファイルを編集して必要な変更を加え、そのプロセスで `<<<<<<<`、`=======`、`>>>>>>>` 行を削除します。次に、周囲のコードを確認します。競合があった場合、論理的なエラーも存在する可能性があります！ここで `x check` を実行して、明らかなエラーがないことを確認することをお勧めします。

すべての競合の修正が完了したら、`git add` を介して競合があったファイルをステージする必要があります。その後、`git rebase --continue` を実行して、競合を解決したことを Git に知らせ、リベースを終了する必要があります。

リベースが成功したら、`git push --force-with-lease` でフォークの関連ブランチを更新します。

### 最新の状態に保つ

[上記のセクション](#rebasing)は、リベース作業とマージ競合の処理に関する特定のガイドです。ローカルリポジトリを upstream の変更で最新の状態に保つ方法に関する一般的なアドバイスを以下に示します：

ローカルの `main` ブランチにいる間に定期的に `git pull upstream main` を使用すると、最新の状態が保たれます。フィーチャーブランチも最新の状態に保つ必要があります。プルした後、フィーチャーブランチをチェックアウトしてリベースできます：

```console
git checkout main
git pull upstream main --ff-only # マージコミットがないことを確認する
git rebase main feature_branch
git push --force-with-lease # （origin をローカルと同じに設定）
```

[No-Merge Policy](#no-merge-policy) に従ってマージを回避するために、`git config pull.ff only`（これはローカルリポジトリにのみ設定を適用します）を使用して、`--ff-only` または `--rebase` を毎回渡す必要なく、`git pull` 時に Git がマージコミットを作成しないようにすることをお勧めします。

main から `git push --force-with-lease` を使用して、フィーチャーブランチが GitHub 側の状態と同期していることを再確認することもできます。

## 高度なリベース

### コミットをスカッシュする

コミットを互いに「スカッシュ」すると、それらが単一のコミットにマージされます。これの良い点と悪い点は、履歴を簡素化することです。一方で、変更が行われたステップを追跡できなくなりますが、履歴は扱いやすくなります。

競合がなく、履歴をクリーンアップするためにスカッシュするだけの場合は、`git rebase --interactive --keep-base main` を使用します。これにより、PR のフォークポイントが同じままになり、リベース間で何が起こったかの差分を確認しやすくなります。

スカッシュは、競合解決の一部としても役立つ場合があります。ブランチに同じコードの連続した複数の書き換えが含まれている場合、またはリベース競合が非常に深刻な場合、`git rebase --interactive main` を使用してプロセスをより細かく制御できます。これにより、コミットをスキップしたり、スキップしないコミットを編集したり、適用される順序を変更したり、互いに「スカッシュ」したりすることができます。

あるいは、次のようにコミット履歴を犠牲にすることもできます：

```console
# すべての変更を1つのコミットにスカッシュして、競合に1回だけ対処すればよいようにします
git rebase --interactive --keep-base main  # そしてすべての変更をスカッシュします
git rebase main
# すべてのマージ競合を修正します
git rebase --continue
```

最後のいくつかのコミットだけをまとめたい場合もあります。おそらく、それらが「fixup」を表すだけで実際の変更ではないためです。例えば、`git rebase --interactive HEAD~2` を使用すると、2つのコミットのみを編集できます。

### `git range-diff`

リベースを完了し、変更をプッシュアップする前に、古いブランチと新しいブランチの間の変更を確認することをお勧めします。これは `git range-diff main @{upstream} HEAD` で行うことができます。

`range-diff` の最初の引数（この場合は `main`）は、古いブランチと新しいブランチを比較する基準となるリビジョンです。2番目の引数は、ブランチの古いバージョンです。この場合、`@upstream` は GitHub にプッシュしたバージョンを意味し、これはプルリクエストで人々が見るものと同じです。最後に、`range-diff` の3番目の引数は、ブランチの*新しい*バージョンです。この場合、これは `HEAD` で、ローカルリポジトリで現在チェックアウトされているコミットです。

同等の省略形 `git range-diff main @{u} HEAD` も使用できることに注意してください。

通常の Git diff とは異なり、range-diff 出力では、別の `-` または `+` の隣に `-` または `+` が表示されます。左側のマーカーは、古いブランチと新しいブランチの間の変更を示し、右側のマーカーは、コミットした変更を示します。したがって、range-diff は古い diff と新しい diff の間の違いを示すため、「diff の diff」と考えることができます。

以下は `git range-diff` 出力の例です（[Git のドキュメント][range-diff-example-docs]から引用）：

```console
-:  ------- > 1:  0ddba11 Prepare for the inevitable!
1:  c0debee = 2:  cab005e Add a helpful message at the start
2:  f00dbal ! 3:  decafe1 Describe a bug
    @@ -1,3 +1,3 @@
     Author: A U Thor <author@example.com>

    -TODO: Describe a bug
    +Describe a bug
    @@ -324,5 +324,6
      This is expected.

    -+What is unexpected is that it will also crash.
    ++Unexpectedly, it also crashes. This is a bug, and the jury is
    ++still out there how to fix it best. See ticket #314 for details.

      Contact
3:  bedead < -:  ------- TO-UNDO
```

（ターミナルの `git range-diff` 出力は、色があるため、この例よりもおそらく読みやすいでしょう。）

`git range-diff` のもう1つの機能は、`git diff` とは異なり、コミットメッセージも diff することです。この機能は、複数のコミットメッセージを修正する際に、正しい部分を変更したことを確認できるため便利です。

`git range-diff` は非常に便利なコマンドですが、出力形式に慣れるまでに時間がかかることがあります。Git のコマンドに関するドキュメント、特に [「Examples」セクション][range-diff-example-docs]も役立つ場合があります。

[range-diff-example-docs]: https://git-scm.com/docs/git-range-diff#_examples

## No-Merge ポリシー

rust-lang/rust リポジトリは、「リベースワークフロー」として知られているものを使用しています。これは、PR のマージコミットが受け入れられないことを意味します。その結果、ローカルで `git merge` を実行している場合は、代わりにリベースする必要がある可能性があります。もちろん、これは常に正しいわけではありません。マージがファストフォワードになる場合（`git pull` が通常実行するマージのように）、マージコミットは作成されず、心配する必要はありません。一度 `git config merge.ff only`（これはローカルリポジトリに設定を適用します）を実行すると、実行するすべてのマージがこのタイプであることが保証され、間違いを犯すことができません。

この決定にはいくつかの理由があり、他のすべてと同様に、これはトレードオフです。主な利点は、一般的に線形のコミット履歴です。これにより、二分探索が大幅に簡素化され、履歴とコミットログがはるかに理解しやすくなります。

## レビューのためのヒント

**注意**：このセクションは PR を*レビューする*ためのものであり、作成するためのものではありません。

### 空白を非表示にする

GitHub には、空白の変更を無効にするボタンがあり、便利な場合があります。`git diff -w origin/main` を使用してローカルで変更を表示することもできます。

![hide whitespace](./img/github-whitespace-changes.png)

### PR をフェッチする

PR をローカルでチェックアウトするには、`git fetch upstream pull/NNNNN/head && git checkout FETCH_HEAD` を使用できます。

github の cli ツールも使用できます。Github は、ローカルでチェックアウトするためのコマンドをコピー＆ペーストできる PR のボタンを表示します。詳細については、<https://cli.github.com/> を参照してください。

![`gh` suggestion](./img/github-cli.png)

### 大きなコードセクションの移動

ファイル*内*での大きな移動に対する Git と Github のデフォルトの diff ビューはかなり貧弱です。各行が削除され、各行が追加されたものとして表示され、各行を自分で比較する必要があります。Git には、移動された行を別の色で表示するオプションがあります：

```console
git log -p --color-moved=dimmed-zebra --color-moved-ws=allow-indentation-change
```

詳細については、[`--color-moved` のドキュメント](https://git-scm.com/docs/git-diff#Documentation/git-diff.txt---color-movedltmodegt)を参照してください。

### range-diff

[PR 作成者向けの関連セクション](#git-range-diff)を参照してください。これは、フォースプッシュされたコードを比較して、予期しない変更がないことを確認するのに役立ちます。

### 特定のファイルへの変更を無視する

リポジトリ内の多くの大きなファイルは自動生成されます。それらのファイルへの変更を無視する diff を表示するには、次の構文を使用できます（例：Cargo.lock）：

```console
git log -p ':!Cargo.lock'
```

任意のパターンがサポートされています（例：`:!compiler/*`）。パターンは、`:` が先頭に付加されたパターンを示すために、`.gitignore` と同じ構文を使用します。

## Git サブモジュール

**注意**：サブモジュールは知っておくと良いことですが、`rustc` に貢献するための絶対的な前提条件では*ありません*。Git を初めて使用する場合は、このセクションを読む前に Git の主要な概念に慣れたほうがよいかもしれません。

`rust-lang/rust` リポジトリは、`rust` リポジトリ内から他の Rust プロジェクトを使用する方法として [Git サブモジュール][Git submodules]を使用しています。例には、Rust の `llvm-project` のフォーク、`cargo`、および `stdarch` や `backtrace` などのライブラリが含まれます。

これらのプロジェクトは、別個の Git（および GitHub）リポジトリで開発および保守され、独自の Git 履歴/コミット、issue トラッカー、PR を持っています。サブモジュールにより、`rust` リポジトリ内に埋め込まれたサブリポジトリのようなものを作成し、`rust` リポジトリ内のディレクトリのように使用できます。

例として `llvm-project` を取り上げます。`llvm-project` は [`rust-lang/llvm-project`] リポジトリで保守されていますが、コード生成と最適化のためにコンパイラによって `rust-lang/rust` で使用されます。これを `src/llvm-project` フォルダーのサブモジュールとして `rust` に取り込みます。

サブモジュールの内容は Git によって無視されます：サブモジュールはある意味でリポジトリの残りの部分から分離されています。ただし、`cd src/llvm-project` を試してから `git status` を実行すると：

```console
HEAD detached at 9567f08afc943
nothing to commit, working tree clean
```

Git に関する限り、もはや `rust` リポジトリにいるのではなく、`llvm-project` リポジトリにいます。「detached HEAD」状態、つまりブランチではなく特定のコミットにいることに気付くでしょう。

これは、他の依存関係と同様に、どのバージョンを使用するかを制御できるようにしたいためです。サブモジュールを使用すると、まさにそれができます：すべてのサブモジュールは特定のコミットに「固定」されており、手動で変更しない限り変更されません。`llvm-project` ディレクトリで `git checkout <commit>` を使用して `rust` ディレクトリに戻ると、`git add src/llvm-project` を実行するなどして、他の変更と同様にこの変更をステージできます。（変更をコミットにステージ*しない*場合、`x` を実行すると、サブモジュールを自動的に「更新」するときに前のコミットに戻ることで変更が元に戻されるリスクがあることに注意してください。）

このバージョン選択は通常、プロジェクトのメンテナーによって行われ、[このように][llvm-update]見えます。

Git サブモジュールは慣れるまでに時間がかかるので、まだ完全に明確でなくても心配しないでください。サブモジュールを直接使用する必要があることはめったになく、繰り返しますが、Rust に貢献するためにサブモジュールについてすべてを知っている必要はありません。サブモジュールが存在し、Git が適切かつ公正に便利に処理できる、ある種の埋め込まれたサブリポジトリ依存関係に対応していることを知っておいてください。

### サブモジュールのハードリセット

`git status` を実行すると、次のような状況に遭遇することがあります：

```console
Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
  (commit or discard the untracked or modified content in submodules)
        modified:   src/llvm-project (new commits, modified content)
```

`git submodule update` を実行しようとすると、次のようなエラーでひどく壊れます：

```console
error: RPC failed; curl 92 HTTP/2 stream 7 was not closed cleanly: CANCEL (err 8)
error: 2782 bytes of body are still expected
fetch-pack: unexpected disconnect while reading sideband packet
fatal: early EOF
fatal: fetch-pack: invalid index-pack output
fatal: Fetched in submodule path 'src/llvm-project', but it did not contain 5a5152f653959d14d68613a3a8a033fb65eec021. Direct fetching of that commit failed.
```

`(new commits, modified content)` が表示される場合は、次を実行できます：

```console
git submodule foreach git reset --hard
```

その後、`git submodule update` を再度試してください。

### Git サブモジュールの deinit

それでもうまくいかない場合は、すべての Git サブモジュールを deinit してみることができます...

```console
git submodule deinit -f --all
```

残念ながら、ローカルの Git サブモジュール構成が何らかの理由で完全に混乱することがあります。

### `fatal: not a git repository: <submodule>/../../.git/modules/<submodule>` の克服

何らかの理由で、次のような状況に遭遇することがあります：

```console
fatal: not a git repository: src/gcc/../../.git/modules/src/gcc
```

この状況では、指定されたサブモジュールパス、つまりこの例では `<submodule_path> = src/gcc` に対して、次のことを行う必要があります：

1. `rm -rf <submodule_path>/.git`
2. `rm -rf .git/modules/<submodule_path>/config`
3. 何らかの理由で `.gitconfig` ロックが孤立している場合は、`rm -rf .gitconfig.lock`

その後、`./x fmt` のようなことを行って、bootstrap にサブモジュールのチェックアウトを管理させます。

## `git blame` 中にコミットを無視する

機能を変更しない大規模な再フォーマット変更を含むコミットがいくつかあります。これらは、[`.git-blame-ignore-revs`](https://github.com/rust-lang/rust/blob/HEAD/.git-blame-ignore-revs)を介して `git blame` によって無視するように指示できます：

1. 無視するコミットのリストとして `.git-blame-ignore-revs` を使用するように `git blame` を設定します：`git config blame.ignorerevsfile .git-blame-ignore-revs`
2. `git blame` で無視したい適切なコミットを追加します。

`.git-blame-ignore-revs` に追加するコミットには、コミットが無視される*理由*を簡単に理解できるようにコメントを含めてください。

[Git submodules]: https://git-scm.com/book/en/v2/Git-Tools-Submodules
[`rust-lang/llvm-project`]: https://github.com/rust-lang/llvm-project
[llvm-update]: https://github.com/rust-lang/rust/pull/99464/files
