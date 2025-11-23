# はじめに

Rust への貢献に興味を持っていただき、ありがとうございます！
貢献の方法はたくさんあり、すべてに感謝しています。

これが初めての貢献の場合、[ウォークスルー][walkthrough]の章は、典型的な貢献がどのように進むかの良い例を示すことができます。

このドキュメントは包括的であることを_意図していません_。
最も有用なことのクイックガイドであることを意図しています。
詳細については、[コンパイラのビルドと実行方法](building/how-to-build-and-run.md)を参照してください。

[internals]: https://internals.rust-lang.org
[rust-zulip]: https://rust-lang.zulipchat.com
[coc]: https://www.rust-lang.org/policies/code-of-conduct
[walkthrough]: ./walkthrough.md

## 質問する

質問がある場合は、[Rust Zulip サーバー][rust-zulip]または[internals.rust-lang.org][internals]に投稿してください。
詳細については、公式ウェブサイトの[チームとワーキンググループのリスト][governance]と[コミュニティページ][community]を参照してください。

[governance]: https://www.rust-lang.org/governance
[community]: https://www.rust-lang.org/community

リマインダーとして、すべてのコントリビューターは[行動規範][coc]に従うことが期待されています。

コンパイラチーム（または `t-compiler`）は通常、Zulip の[#t-compiler チャネル][z-t-compiler]に常駐しています。コンパイラの仕組みに関する質問は [#t-compiler/help][z-help] で行うことができます。

[z-t-compiler]: https://rust-lang.zulipchat.com/#narrow/channel/131828-t-compiler
[z-help]: https://rust-lang.zulipchat.com/#narrow/channel/182449-t-compiler.2Fhelp

**質問してください！** 多くの人が「専門家の時間を無駄にしている」と感じていますが、`t-compiler` の誰もそのようには感じていません。コントリビューターは私たちにとって重要です。

また、快適に感じる場合は、パブリックなトピックを好むようにしてください。これにより、他の人が質問と回答を見ることができ、おそらくそれらをこのガイドに統合することもできます :)

**ヒント**：英語のネイティブスピーカーでなく、文章に不安がある場合は、翻訳ツールを使って支援を受けることができます。ただし、長く複雑な単語を生成する LLM ツールの使用は避けてください。日常のチームワークでは、**シンプルで明確な言葉**が簡単な理解のために最適です。小さなタイプミスや文法の間違いでさえ、あなたをより人間的に見せることができ、人々は人間とより良くつながります。

### 専門家

すべての `t-compiler` メンバーが `rustc` のすべての部分の専門家というわけではありません。
これはかなり大きなプロジェクトです。
コンパイラのさまざまな部分についての専門知識を持っている可能性のある人を見つけるには、[triagebot assign グループを参照してください][map]。`triagebot.toml` ファイルで `[assign*` で始まるセクションです。
ただし、誰に ping すればよいかわからない場合でも、遠慮なく質問してください。

コンパイラの特定の部分の専門家を見つける別の方法は、最近のコミットを見ることです。例えば、1.68.2 リリース以降に名前解決に取り組んだ人を見つけるには、`git shortlog -n 1.68.2.. compiler/rustc_resolve/` を実行できます。「Rollup merge」で始まるコミットや `@bors` によるコミットは無視してください（これらのコミットの詳細については、[CI 貢献手順](./contributing.md#ci)を参照してください）。

[map]: https://github.com/rust-lang/rust/blob/HEAD/triagebot.toml

### エチケット

質問にできるだけ多くの有用な情報を含めるように心がけることをお願いしますが、Rust への貢献に不慣れな場合、これは難しいことを認識しています。

コンテキストを提供せずに誰かに ping するだけでは、少し迷惑で、ノイズを作るだけなので、`t-compiler` の人々が1日に多くの ping を受け取ることに注意してください。

## 何に取り組むべきか？

Rust プロジェクトは非常に大きく、プロジェクトのどの部分に支援が必要か、または初心者に適した出発点はどこかを知ることが難しい場合があります。以下にいくつかの推奨される出発点を示します。

### 簡単またはメンター付きの issue

どこから始めるか探している場合は、次の[issue 検索][help-wanted-search]をチェックしてください。これらのラベルの説明については、[トリアージ][Triage]を参照してください。
興味のある分野に検索をフィルタリングすることもできます。例えば：

- `repo:rust-lang/rust-clippy` は clippy の issue のみを表示します
- `label:T-compiler` はコンパイラに関連する issue のみを表示します
- `label:A-diagnostics` は診断の issue のみを表示します

すべての重要な作業や初心者向けの作業に issue ラベルが付いているわけではありません。ラベルが付いていない作業を見つける方法については、以下を参照してください。

[help-wanted-search]: https://github.com/issues?q=is%3Aopen+is%3Aissue+org%3Arust-lang+no%3Aassignee+label%3AE-easy%2C%22good+first+issue%22%2Cgood-first-issue%2CE-medium%2CEasy%2CE-help-wanted%2CE-mentor+-label%3AS-blocked+-linked%3Apr+
[Triage]: ./contributing.md#issue-triage

### 繰り返しの作業

一部の作業は、1人で行うには大きすぎます。このような場合、コントリビューター間で作業を調整するために「追跡 issue」を持つことが一般的です。大きな時間的コミットメントなしで作業を簡単に引き受けることができる追跡 issue の例を以下に示します：

- *ここに繰り返しの作業アイテムを追加してください。*

繰り返しの作業を見つけた場合は、ここに追加してください！

### Clippy issue

[Clippy] プロジェクトは、貢献プロセスを初心者にできるだけ優しくするために長い時間を費やしてきました。プロセスやコンパイラの内部に慣れるために、まずこれに取り組むことを検討してください。

始め方の手順については、[Clippy 貢献ガイド][clippy-contributing]を参照してください。

[Clippy]: https://doc.rust-lang.org/clippy/
[clippy-contributing]: https://github.com/rust-lang/rust-clippy/blob/master/CONTRIBUTING.md

### 診断 issue

多くの診断 issue は自己完結型であり、コンパイラの詳細なバックグラウンド知識を必要としません。診断 issue のリストは[こちら][diagnostic-issues]で確認できます。

[diagnostic-issues]: https://github.com/rust-lang/rust/issues?q=is%3Aissue+is%3Aopen+label%3AA-diagnostics+no%3Aassignee

### 放棄されたプルリクエストの引き継ぎ

場合によっては、コントリビューターがプルリクエストを送信しますが、後で作業する時間が十分にないことがわかったり、単に興味がなくなったりします。このような PR は最終的に閉じられ、`S-inactive` ラベルが付けられます。これらの PR のいくつかを調べて、作業を引き継ぐことができます。このような PR のリストは[こちら][abandoned-prs]で確認できます。

PR がその間に別の方法で実装されている場合、`S-inactive` ラベルを削除する必要があります。
そうでなく、変更にまだ関心があるようであれば、プルリクエストを最新の `main` ブランチの上にリベースして新しいプルリクエストを送信し、機能に関する作業を続けることができます。

[abandoned-prs]: https://github.com/rust-lang/rust/pulls?q=is%3Apr+label%3AS-inactive+is%3Aclosed

### テストを書く

解決されているが回帰テストがない issue は、`E-needs-test` ラベルでマークされています。単体テストを書くことは、低リスクで優先度の低いタスクであり、新しいコントリビューターがテストインフラストラクチャと貢献ワークフローに慣れる絶好の機会を提供します。

### std（標準ライブラリ）への貢献

[std-dev-guide](https://std-dev-guide.rust-lang.org/)を参照してください。

### 他の Rust プロジェクトへのコード貢献

`rust-lang/rust` リポジトリ以外にも、`cargo`、`miri`、`rustup` など、貢献できる他のプロジェクトがたくさんあります。

これらのリポジトリには、独自の貢献ガイドラインと手順がある場合があります。多くはワーキンググループによって所有されています。詳細については、これらのリポジトリの README のドキュメントを参照してください。

### その他の貢献方法

大きな `rust-lang/rust` コードベースに直接飛び込むのが快適でない場合、他にも貢献する方法がたくさんあります。

次のタスクは、多くのバックグラウンド知識がなくても実行可能ですが、非常に役立ちます：

- [ドキュメントを書く][wd]：少し大胆に感じる場合は、コードの一部を読んで、それに対して doc コメントを書いてみてください。これにより、コンパイラの一部を学びながら、有用な成果物を作成できます！
- [issue のトリアージ][triage]：issue の分類、複製、最小化は、Rust メンテナーにとって非常に役立ちます。
- [ワーキンググループ][wg]：Rust 関連のさまざまなトピックに関する多くのワーキンググループがあります。
- [users.rust-lang.org][users]または [Stack Overflow][so]で質問に答える。
- [RFC プロセス](https://github.com/rust-lang/rfcs)に参加する。
- [要求されたコミュニティライブラリ][community-library]を見つけて、それをビルドし、[Crates.io](http://crates.io)に公開する。言うは易く行うは難しですが、非常に、非常に価値があります！

[users]: https://users.rust-lang.org/
[so]: http://stackoverflow.com/questions/tagged/rust
[community-library]: https://github.com/rust-lang/rfcs/labels/A-community-library
[wd]: ./contributing.md#writing-documentation
[wg]: https://rust-lang.github.io/compiler-team/working-groups/

## クローンとビルド

[「コンパイラのビルドと実行方法」](./building/how-to-build-and-run.md)を参照してください。

## コントリビューター手順

このセクションは[「貢献手順」](./contributing.md)の章に移動しました。

## その他のリソース

このセクションは[「このガイドについて」][more-links]の章に移動しました。

[more-links]: ./about-this-guide.md#other-places-to-find-information
