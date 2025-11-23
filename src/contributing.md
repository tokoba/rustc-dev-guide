# 貢献手順

## バグレポート

バグは不幸なことですが、ソフトウェアにおいては現実です。
知らないことは修正できないので、自由に報告してください。
何かがバグかどうか確信が持てない場合でも、とにかく問題を開いてください。

**バグを公開して報告することがRustユーザーにセキュリティリスクをもたらすと考える場合は、[instructions for reporting security vulnerabilities][vuln]に従ってください**。

[vuln]: https://www.rust-lang.org/policies/security

ナイトリーチャンネルを使用している場合は、バグをファイルする前に、最新のツールチェーンでバグが存在するかどうかを確認してください。
すでに修正されている可能性があります。

バグを報告する前に、時間があれば[search existing issues]してください。他の誰かがすでにあなたのエラーを報告している可能性があるためです。
これは常にうまくいくとは限らず、何を検索すればよいかを知るのが難しい場合もあるため、これは追加のクレジットと考えてください。
重複したレポートを誤ってファイルしても気にしません。

同様に、バグに遭遇した他の人があなたの問題を見つけるのを助けるために、説明的なタイトルで問題をファイルすることを検討してください。タイトルには、それに固有の情報が含まれている必要があります。
これには、使用される言語またはコンパイラ機能、バグをトリガーする条件、またはエラーメッセージの一部(ある場合)が含まれます。
例としては、**impl Traitの戻り位置のライフタイム推論で「不可能なケースに到達しました」**が挙げられます。

問題を開くことは、[this link][create an issue]に従って、適切に提供されたテンプレートのフィールドに記入するだけで簡単です。

## バグ修正または「通常の」コード変更

ほとんどのPRでは、特別な手順は必要ありません。
[open a PR]するだけで、レビュー、承認、マージされます。
これには、ほとんどのバグ修正、リファクタリング、およびその他のユーザーに見えない変更が含まれます。
次のいくつかのセクションでは、このルールの例外について説明します。

また、WIP PRまたはGitHub [Draft PRs]を開くことは完全に受け入れられることに注意してください。
一部の人々は、途中でフィードバックを得たり、コラボレーターとコードを共有したりできるように、これを行うことを好みます。
他の人は、CIを利用してPRをビルドおよびテストできるように、これを行います(例:遅いマシンで開発している場合)。

[open a PR]: #pull-requests
[Draft PRs]: https://github.blog/2019-02-14-introducing-draft-pull-requests/

## 新機能

Rustには強力な後方互換性の保証があります。
したがって、新機能を安定版Rustに直接実装することはできません。
代わりに、stable、beta、nightlyの3つのリリースチャネルがあります。

- **Stable**: これは一般的な使用のための最新の安定リリースです。
- **Beta**: これは次のリリースです(6週間以内に安定版になります)。
- **Nightly**: `main`ブランチに従います。
  これは、オプトインの機能ゲートを介して不安定な機能が使用されることを意図した唯一のチャネルです。

詳細については、[this chapter on implementing new features](./implementing_new_features.md)を参照してください。

### 破壊的変更

破壊的変更には、dev-guideに[dedicated section][Breaking Changes]があります。

### 主要な変更

コンパイラチームには、破壊を引き起こすかどうかに関係なく、大規模な変更に対する特別なプロセスがあります。
このプロセスは、主要変更提案(MCP)と呼ばれます。
MCPは、(完全なRFCまたはチームとの設計ミーティングとは対照的に)コンパイラへの大規模な変更に関するフィードバックを得るための比較的軽量なメカニズムです。

MCPが必要になる可能性のあるものの例には、主要なリファクタリング、重要な型への変更、コンパイラが何かを行う方法への重要な変更、または小規模なユーザー向けの変更が含まれます。

**疑問がある場合は、[on Zulip]で尋ねてください。
多くの作業を費やしたPRが最終的にマージされないのは残念です!** MCPの詳細については、[See this document][mcpinfo]を参照してください。

[mcpinfo]: https://forge.rust-lang.org/compiler/proposals-and-stabilization.html#how-do-i-submit-an-mcp
[on Zulip]: https://rust-lang.zulipchat.com/#narrow/stream/131828-t-compiler

### パフォーマンス

コンパイラのパフォーマンスは重要です。
過去数年間に[gradually improving it][perfdash]するために多くの努力が払われてきました。

[perfdash]: https://perf.rust-lang.org/dashboard.html

変更がパフォーマンスのリグレッション(または改善)を引き起こす可能性があると思われる場合は、「perf run」をリクエストできます(レビュアーも承認前にリクエストする場合があります)。
これは、変更を加えたコンパイラでベンチマークのコレクションをコンパイルする別のボットです。
数値は[here][perf]で報告され、最新の`main`に対する変更の比較を確認できます。

> 一般的なRustコードのパフォーマンスの紹介については、rustc開発でも役立つ[The Rust Performance Book]を参照してください。

[perf]: https://perf.rust-lang.org
[The Rust Performance Book]: https://nnethercote.github.io/perf-book/

## プルリクエスト

プルリクエスト(または略してPR)は、Rustを変更するために使用する主要なメカニズムです。
GitHub自体には、プルリクエスト機能の使用に関する優れた[great documentation][about-pull-requests]があります。
私たちは["fork and pull" model][development-models]を使用しています。
これは、貢献者が変更を個人のフォークにプッシュし、それらの変更をソースリポジトリに取り込むためのプルリクエストを作成するものです。
Rustへの貢献時にGitを使用する方法については、[a chapter](git.md)があります。

> **潜在的に大規模で複雑な、横断的および/または非常にドメイン固有の変更に関するアドバイス**
>
> ローテーション中のコンパイラレビュアーは、通常、よく知っているコンパイラの領域がありますが、
> よく知らない領域もあります。PRに大規模で複雑な、横断的および/または高度にドメイン固有の変更が含まれている場合、
> そのようなPRのすべての変更をレビューすることに自信を持つ適切なレビュアーを見つけることが非常に困難になります。これは、
> 変更がコンパイラ固有のものだけでなく、標準ライブラリチームなど他のチームのレビュアーの管轄下にある変更も含まれている場合にも当てはまります。[There's a bot][triagebot]が関連するチームに通知し、変更されたファイルに基づいて特定のアラートを設定している人にpingします。
>
> そのような変更を行う前に、**コンパイラチームと事前に提案された変更について議論する**ことを強くお勧めします。また、コンパイラチームと協力して、**大規模で潜在的にレビュー不可能なPRを、より個別にレビュー可能な一連の小さなPRに分解できるか**を確認してください。
>
> [#t-compiler thread on Zulip][t-compiler]を作成して、提案された変更について議論できます。
>
> コンパイラチームと事前にコミュニケーションを取ることは、いくつかの点で役立ちます:
>
> 1. PRがタイムリーにレビューされる可能性が高まります。
>     - 実際のPRを開く*前に*適切なレビュアーを特定するのを手伝ったり、変更手順をナビゲートするためのアドバイザーや連絡係を見つけたり、try-jobs、perf runs、crater runsを適切に実行するのを手伝ったりできます。
> 2. コンパイラチームが変更を追跡するのに役立ちます。
> 3. コンパイラチームは、変更の方向がコンパイラチームが好む方向と一致しているかどうかを早期かつ頻繁にバイブチェックできます。
> 4. コンパイラチームが受け入れたくない大規模な変更に多大な時間と労力を投資した状況、または変更がコンパイラチームが同意しない方向にあることを非常に遅く知った状況を避けるのに役立ちます。

[about-pull-requests]: https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/about-pull-requests
[development-models]: https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/getting-started/about-collaborative-development-models#fork-and-pull-model
[t-compiler]: https://rust-lang.zulipchat.com/#narrow/stream/131828-t-compiler
[triagebot]: https://github.com/rust-lang/rust/blob/HEAD/triagebot.toml

### ブランチを最新に保つ

rust-lang/rustのCIは、ブランチがベースにしているコミットではなく、現在の`main`に直接パッチを適用します。
これにより、ブランチが古くなっている場合、明示的なマージコンフリクトがなくても予期しない失敗が発生する可能性があります。

必要な場合にのみブランチを更新してください: マージコンフリクトがある場合、アップストリームCIが壊れていて緑のPRをブロックしている場合、またはメンテナがリクエストした場合。
レビュー中にすでに緑のPRを更新することは避けてください(必要でない限り)。
レビュー中は、フィードバックに対処するためにインクリメンタルコミットを行ってください。
スカッシュまたはリベースは、最後に、またはレビュアーがリクエストした場合にのみ行うことをお勧めします。

更新する場合は、`git push --force-with-lease`を使用し、何が変更されたかを説明する簡単なコメントを残してください。
一部のリポジトリは、リベースではなく`upstream/main`からマージすることを好みます。
プロジェクトの規約に従ってください。
詳細な手順については、[keeping things up to date](git.md#keeping-things-up-to-date)を参照してください。

リベース後は、CIが実行される前に問題をキャッチするために、[run the relevant tests locally](tests/intro.md)を実行することをお勧めします。

### r?

すべてのプルリクエストは他の人によってレビューされます。
[@rustbot]というボットがあり、変更したファイルに基づいてランダムな人を自動的に割り当ててリクエストをレビューします。

特定の人にプルリクエストのレビューをリクエストする場合は、
プルリクエストの説明またはコメントに`r?`を追加できます。
例えば、@awesome-reviewerにレビューをリクエストする場合は、
プルリクエストの説明の最後に次のように追加します:

    r? @awesome-reviewer

[@rustbot]は、ランダムな人の代わりに、そのレビュアーにPRを割り当てます。
これは完全にオプションです。

`r? rust-lang/groupname`と書くことで、特定のチームからランダムなレビュアーを割り当てることもできます。
例えば、診断の変更を行っている場合、
次のように追加することで、診断チームからレビュアーを取得できます:

    r? rust-lang/diagnostics

可能な`groupname`の完全なリストについては、
[triagebot.toml config file]の`adhoc_groups`セクション、
または[rust-lang teams database]のチームのリストを確認してください。

### レビューを待つ

> NOTE
>
> プルリクエストのレビュアーは、多くの場合、能力の限界で作業しており、
> 彼らの多くはボランティアベースで貢献しています。
> レビューの遅延を最小限に抑えるために、
> プルリクエストの著者と割り当てられたレビュアーは、レビューラベル
> (`S-waiting-on-review`および`S-waiting-on-author`)を更新し続けることを確認し、
> 適切な場合にこれらのコマンドを呼び出す必要があります:
>
> - `@rustbot author`:
>   レビューが終了し、
>   PR著者はコメントを確認して適切に行動する必要があります。
>
> - `@rustbot review`:
>   著者はレビューの準備ができており、
>   このPRはレビュアーのキューに再度キューされます。

レビュアーは人間であり、その大部分は空き時間に`rustc`に取り組んでいることに注意してください。
これは、応答してPRをレビューするのに時間がかかる可能性があることを意味します。
また、レビュアーは自分に割り当てられたPRの一部を見逃す可能性があることも意味します。

PRを前進させるために、Triage WGは定期的に、レビューを待っていて、少なくとも2週間議論されていないすべてのPRを通過します。
2週間以内にレビューを受けられない場合は、Zulip([#t-release/triage])でTriage WGに気軽に尋ねてください。
彼らは、いつpingするか、誰が休暇中かなどについての知識を持っています。

レビュアーは、GitHub codeレビューインターフェースを使用していくつかの変更をリクエストする場合があります。
また、一部のPRに対して特別な手順をリクエストする場合もあります。
そのような手順の例については、[Crater]および[Breaking Changes]章を参照してください。

[r?]: https://github.com/rust-lang/rust/pull/78133#issuecomment-712692371
[#t-release/triage]: https://rust-lang.zulipchat.com/#narrow/stream/242269-t-release.2Ftriage
[Crater]: tests/crater.md

### CI

人間によってレビューされることに加えて、プルリクエストは自動的にテストされます。
継続的インテグレーション(CI)のおかげです。
基本的に、プルリクエストを開いて更新するたびに、
CIはコンパイラをビルドし、[compiler test suite]に対してテストし、
プルリクエストがRustのスタイルガイドラインに準拠しているかどうかを確認するなどの他のテストも実行します。

継続的インテグレーションテストを実行することで、PR著者は最初のレビューサイクルを経ずに早期にミスをキャッチでき、レビュアーが特定のプルリクエストのステータスを把握するのにも役立ちます。

Rustには十分なCI容量があり、変更をプッシュするたびに計算リソースを無駄にすることを心配する必要はありません。
また、生産性を向上させることができる場合は、CIを使用して変更をテストすることも完全に問題ありません(さらに推奨されます!)。
特に、ローカルで完全な`./x test`スイートを実行することはお勧めしません。
実行に非常に時間がかかるためです。

### r+

誰かがあなたのプルリクエストをレビューした後、彼らは`r+`でプルリクエストに注釈を残します。
次のようになります:

    @bors r+

これは、愛らしい統合ボット[@bors]に、プルリクエストが承認されたことを伝えます。
その後、PRは[merge queue]に入り、[@bors]は
サポートしている*すべての*プラットフォームで*すべての*テストを実行します。
すべてがうまくいくと、[@bors]はあなたのコードを`main`にマージし、プルリクエストを閉じます。

変更の規模に応じて、わずかに異なる形式の`r+`が表示される場合があります:

    @bors r+ rollup

追加の`rollup`は、[@bors]にこの変更を常に「ロールアップ」する必要があることを伝えます。
ロールアップされた変更は、プロセスをスピードアップするために、他のPRと一緒にテストおよびマージされます。
通常、互いに競合しないと予想される小さな変更のみが「常にロールアップ」としてマークされます。

辛抱強く待ってください。
これには時間がかかることがあり、キューが長い場合があります。
また、PRは手動でマージされることは決してないことに注意してください。

[@rustbot]: https://github.com/rustbot
[@bors]: https://github.com/bors

### PRを開く

プルリクエスト(PR)をファイルする準備ができましたか?
素晴らしい!
注意すべき点がいくつかあります。

すべてのプルリクエストは、特定のブランチをターゲットにする必要があると確信していない限り、`main`ブランチに対してファイルする必要があります。

PRを送信する前に、いくつかのスタイルチェックを実行してください:

    ./x test tidy --bless

このチェックをすべてのプルリクエスト(およびプルリクエストのすべての新しいコミット)の前に行うことをお勧めします。
このチェックを忘れないようにするために、すべてのプッシュ前に[git hooks]を追加できます。
CIもtidyを実行し、tidyが失敗した場合は失敗します。

Rustは_マージコミットなしポリシー_に従っています。
つまり、マージコンフリクトに遭遇した場合、
マージする代わりに常にリベースすることが期待されます。
例えば、
`main`ブランチから機能ブランチに最新の変更を取り込む場合は、常にリベースを使用してください。
PRにマージコミットが含まれている場合、`has-merge-commits`としてマークされます。
マージコミットを削除したら(例:インタラクティブリベースを通じて)、
ラベルを再度削除する必要があります:

    @rustbot label -has-merge-commits

詳細については、[this chapter][labeling]を参照してください。

マージコンフリクトに遭遇した場合、またはレビュアーが変更を実行するよう求めた場合、PRは`S-waiting-on-author`としてマークされます。
それらを解決したら、`@rustbot`を使用して`S-waiting-on-review`としてマークする必要があります:

    @rustbot ready

GitHubでは、[closing issues using keywords][closing-keywords]が可能です。
この機能は、問題トラッカーを整理するために使用する必要があります。
ただし、一般的には、
PRの説明に「closes #123」テキストを配置することが好まれます(問題コミットではなく)。
特にリベース中に、コミット内の問題番号を引用すると、問題の「スパム」が発生する可能性があります。

ただし、PRが安定版からベータ版または安定版から安定版へのリグレッションを修正し、ベータ版および/または安定版のバックポートが受け入れられた場合(つまり、`beta-accepted`および/または`stable-accepted`とマークされている場合)は、そのようなキーワードを使用しないでください。修正が`main`にランドすると、対応する問題が自動的に閉じられることを望んでいないためです。
問題をどこかで言及しながら、PR説明を更新してください。
例えば、`Fixes (after beta backport) #NNN.`と書くことができます。

さらなるアクションについては、タイトルが`[beta]`または`[stable]`で始まり、問題のPRをバックポートするPRを鋭く監視してください。
それがマージされたら、関連する問題を閉じることができます。
クロージングコメントは、関連するすべてのPRに言及する必要があります。
問題を閉じる権限がない場合は、
レビュアーに閉じてもらうように元のPRにコメントを残してください。

[labeling]: ./rustbot.md#issue-relabeling
[closing-keywords]: https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue

### PRを元に戻す

PRがミスコンパイル、重大なパフォーマンスのリグレッション、またはその他の重大な問題を引き起こした場合、リグレッションテストケースでそのPRを元に戻したい場合があります。
また、[revert policy]をForgeドキュメント(主にレビュアーを対象としていますが、PR著者にとっても有用な情報が含まれています)で確認することもできます。

PRに巨大な変更が含まれている場合、元に戻すのが難しくなり、後続の更新でインクリメンタルな修正をレビューするのがより困難になる可能性があります。
または、そのPRの特定のコードが後続のPRによって大きく依存されている場合、元に戻すことが困難になる可能性があります。

そのような場合、問題のあるコードを特定し、一部の入力に対して無効にすることができます。例として[#128271][#128271]があります。

MIR最適化の場合、[#132356][#132356]に示されているように、`-Zunsound-mir-opt`オプションを使用してmir-optをゲートすることもできます。

[revert policy]: https://forge.rust-lang.org/compiler/reviews.html?highlight=revert#reverts
[#128271]: https://github.com/rust-lang/rust/pull/128271
[#132356]: https://github.com/rust-lang/rust/pull/132356

## 外部依存関係

このセクションは、["Using External Repositories"](./external-repos.md)に移動しました。

## ドキュメントの記述

ドキュメントの改善は非常に歓迎されます。
`doc.rust-lang.org`のソースは
ツリーの[`src/doc`]にあり、標準APIドキュメントはソースコード自体から生成されます(例:[`library/std/src/lib.rs`][std-root])。ドキュメントのプルリクエストは、他のプルリクエストと同じように機能します。

[`src/doc`]: https://github.com/rust-lang/rust/tree/HEAD/src/doc
[std-root]: https://github.com/rust-lang/rust/blob/HEAD/library/std/src/lib.rs#L1

ドキュメント関連の問題を見つけるには、[A-docs label]を使用してください。

ドキュメントのスタイルガイドラインは[RFC 1574]にあります。

標準ライブラリのドキュメントをビルドするには、`x doc --stage 1 library --open`を使用してください。
本(例:不安定な本)のドキュメントをビルドするには、`x doc src/doc/unstable-book.`を使用してください。
結果は`build/host/doc`に表示され、デフォルトのブラウザで自動的に開かれます。
詳細については、[Building Documentation](./building/compiler-documenting.md#building-documentation)を参照してください。

また、`rustdoc`を直接使用して小さな修正を確認することもできます。
例えば、`rustdoc src/doc/reference.md`は、referenceを`doc/reference.html`にレンダリングします。
CSSが乱れる可能性がありますが、HTMLが正しいことを確認できます。

**内部ドキュメント**に対するタイポグラフィ/スペルチェックの修正は受け付けていません。
通常、チャーンやレビュー時間に見合わないためです。
内部ドキュメントの例は、コードコメントとrustc apiドキュメントです。
ただし、同じPRで他の改善を伴う場合は、それらを修正してもかまいません。

### rustc-dev-guideへの貢献

[rustc-dev-guide]への貢献はいつでも歓迎されており、[the rust-lang/rustc-dev-guide repo][rdgrepo]で直接行うことができます。
そのリポジトリの問題トラッカーも、やるべきことを見つける素晴らしい方法です。
初心者と上級コンパイラ開発者の両方のための問題があります!

覚えておくべきことがいくつかあります:

- 過度に長い行を避け、セマンティックラインブレーク(各文の後で行を改行する)を使用するようにしてください。
  行の長さに厳密な制限はありません。
  文または文の一部を適切な最後まで同じ行に流してください。

  ci/sembrのツールを使用してこれを支援できます。
  ヘルプ出力は次のコマンドで表示できます:

  ```console
  cargo run --manifest-path ci/sembr/Cargo.toml -- --help
  ```

- ガイドにテキストを貢献する際は、読者が情報をどれだけ信頼できるかを知るために、いくつかの時間枠および/または理由で情報をコンテキスト化してください。
  妥当な量のコンテキストを提供することを目指してください。これには以下が含まれますが、これらに限定されません:

  - テキストが時代遅れになる可能性がある理由(「変更」以外)。
    変更はプロジェクト全体で一定です。

  - コメントが追加された日付。例えば、_「現在、...」_や_「今のところ、...」_と書く代わりに、
    日付を追加することを検討してください。次のいずれかの形式で:
    - Jan 2021
    - January 2021
    - jan 2021
    - january 2021

    CIアクション(`.github/workflows/date-check.yml`にあります)があり、
    6ヶ月以上経過したものを示す月次レポートを生成します
    ([example](https://github.com/rust-lang/rustc-dev-guide/issues/2052))。

    アクションが日付を取得するには、日付を指定する前に特別な注釈を追加してください:

    ```md
    <!-- date-check --> Nov 2025
    ```

    例:

    ```md
    As of <!-- date-check --> Nov 2025, the foo did the bar.
    ```

    日付を表示されるレンダリング出力の一部にしたくない場合は、
    代わりに次を使用してください:

    ```md
    <!-- date-check: Nov 2025 -->
    ```

  - 変更プロセスのさらなる説明を提供する可能性がある、または情報が古くなっていないことを確認する方法を提供する可能性がある、関連するWG、トラッキング問題、`rustc` rustdocページ、または同様のものへのリンク。

- テキストがかなり長くなる場合(数ページスクロール以上)または複雑になる場合(4つ以上のサブセクション)、最初に目次があると役立つ場合があります。
  上部に`<!-- toc -->`マーカーを含めることで、自動生成できます。

#### ⚠️ 注意: rustc-dev-guide変更をどこに貢献するか

rustc-dev-guide変更をどこに貢献するか、そしてそうすることの利点に関する詳細情報については、
[the rustc-dev-guide working group documentation]を参照してください。

## 問題のトリアージ

<https://forge.rust-lang.org/release/issue-triaging.html>を参照してください。

[stable-]: https://github.com/rust-lang/rust/labels?q=stable
[beta-]: https://github.com/rust-lang/rust/labels?q=beta
[I-\*-nominated]: https://github.com/rust-lang/rust/labels?q=nominated
[I-prioritize]: https://github.com/rust-lang/rust/labels/I-prioritize
[tracking issues]: https://github.com/rust-lang/rust/labels/C-tracking-issue
[beta-backport]: https://forge.rust-lang.org/release/backporting.html#beta-backporting-in-rust-langrust
[stable-backport]: https://forge.rust-lang.org/release/backporting.html#stable-backporting-in-rust-langrust
[metabug]: https://github.com/rust-lang/rust/labels/metabug
[regression-]: https://github.com/rust-lang/rust/labels?q=regression
[relnotes]: https://github.com/rust-lang/rust/labels/relnotes
[S-tracking-]: https://github.com/rust-lang/rust/labels?q=s-tracking
[the rustc-dev-guide working group documentation]: https://forge.rust-lang.org/wg-rustc-dev-guide/index.html#where-to-contribute-rustc-dev-guide-changes

### rfcbotラベル

[rfcbot]は、非同期決定の調整プロセスを追跡するために独自のラベルを使用します。
例えば、変更の承認または拒否などです。
これは[RFCs]、問題、およびプルリクエストに使用されます。

| ラベル | 色 | 説明 |
|--------|-------|-------------|
| [proposed-final-comment-period] | <span class="label-color" style="background-color:#ededed;">&#x2003;</span>&nbsp;Gray | 最終コメント期間に入るために、すべてのチームメンバーのサインオフを待っています。 |
| [disposition-merge] | <span class="label-color" style="background-color:#008800;">&#x2003;</span>&nbsp;Green | 変更をマージする意図を示します。 |
| [disposition-close] | <span class="label-color" style="background-color:#dd0000;">&#x2003;</span>&nbsp;Red | 変更を受け入れずに閉じる意図を示します。 |
| [disposition-postpone] | <span class="label-color" style="background-color:#ededed;">&#x2003;</span>&nbsp;Gray | 今回は変更を受け入れず、後日延期する意図を示します。 |
| [final-comment-period] | <span class="label-color" style="background-color:#1e76d9;">&#x2003;</span>&nbsp;Blue | マージまたは閉じる前に最終コメントを求めています。 |
| [finished-final-comment-period] | <span class="label-color" style="background-color:#f9e189;">&#x2003;</span>&nbsp;Light Yellow | 最終コメント期間が終了し、問題はマージまたは閉じられます。 |
| [postponed] | <span class="label-color" style="background-color:#fbca04;">&#x2003;</span>&nbsp;Yellow | 問題は延期されました。 |
| [closed] | <span class="label-color" style="background-color:#dd0000;">&#x2003;</span>&nbsp;Red | 問題は拒否されました。 |
| [to-announce] | <span class="label-color" style="background-color:#ededed;">&#x2003;</span>&nbsp;Gray | 最終コメント期間を終えた問題で、公に発表する必要があります。注意: rust-lang/rustリポジトリは、このラベルを異なる方法で使用しており、トリアージミーティングで問題を発表します。 |

[disposition-merge]: https://github.com/rust-lang/rust/labels/disposition-merge
[disposition-close]: https://github.com/rust-lang/rust/labels/disposition-close
[disposition-postpone]: https://github.com/rust-lang/rust/labels/disposition-postpone
[proposed-final-comment-period]: https://github.com/rust-lang/rust/labels/proposed-final-comment-period
[final-comment-period]: https://github.com/rust-lang/rust/labels/final-comment-period
[finished-final-comment-period]: https://github.com/rust-lang/rust/labels/finished-final-comment-period
[postponed]: https://github.com/rust-lang/rfcs/labels/postponed
[closed]: https://github.com/rust-lang/rfcs/labels/closed
[to-announce]: https://github.com/rust-lang/rfcs/labels/to-announce
[rfcbot]: https://github.com/anp/rfcbot-rs/
[RFCs]: https://github.com/rust-lang/rfcs

## 役立つリンクと情報

このセクションは["About this guide"]章に移動しました。

["About this guide"]: about-this-guide.md#other-places-to-find-information
[search existing issues]: https://github.com/rust-lang/rust/issues?q=is%3Aissue
[Breaking Changes]: bug-fix-procedure.md
[triagebot.toml config file]: https://github.com/rust-lang/rust/blob/HEAD/triagebot.toml
[rust-lang teams database]: https://github.com/rust-lang/team/tree/HEAD/teams
[compiler test suite]: tests/intro.md
[merge queue]: https://bors.rust-lang.org/queue/rust
[git hooks]: https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks
[A-docs label]: https://github.com/rust-lang/rust/issues?q=is%3Aopen%20is%3Aissue%20label%3AA-docs
[RFC 1574]: https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md#appendix-a-full-conventions-text
[rustc-dev-guide]: https://rustc-dev-guide.rust-lang.org/
[rdgrepo]: https://github.com/rust-lang/rustc-dev-guide
[create an issue]: https://github.com/rust-lang/rust/issues/new/choose
