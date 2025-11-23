# コンパイラチームについて

> NOTE:
> チームに関する詳細は[on Forge]に存在しており、以下のほとんどを時代遅れにしています。

rustcは[Rust compiler team][team]によってメンテナンスされています。このチームに属する人々は、リグレッションを追跡し、新機能を実装するために協力しています。
Rustコンパイラチームのメンバーは、rustcとその設計に重要な貢献をした人々です。

[on Forge]: https://forge.rust-lang.org/compiler
[team]: https://www.rust-lang.org/governance/teams/compiler

## ディスカッション

現在、コンパイラチームはZulipでチャットしています:

- チームチャットはZulipインスタンスの[`t-compiler`][zulip-t-compiler]ストリームで行われます
- 他にもいくつかの関連するZulipチャンネルがあります。
  例えば、[`t-compiler/help`][zulip-help]では、人々がrustc開発について助けを求めることができます。[`t-compiler/meetings`][zulip-meetings]では、チームが週次のトリアージとステアリングミーティングを開催します。

## レビュアー

コンパイラの特定の部分について誰が質問に答えられるかを知りたい場合、または誰が何に取り組んでいるかを知りたい場合は、[triagebot.toml's assign section][map]をチェックしてください。
これには、コンパイラのさまざまな部分のリストと、各部分のレビュアーのリストが含まれています。

[map]: https://github.com/rust-lang/rust/blob/HEAD/triagebot.toml

## Rustコンパイラミーティング

コンパイラチームは週次ミーティングを開催し、トリアージを行い、一般的に新しいバグやリグレッションを把握し、重要なことを議論しています。
これらは[Zulip][zulip-meetings]で開催されます。大まかには次のように進行します:

- **アナウンス、MCP/FCP、およびWGチェックイン:** 重要なことについて、チームの他のメンバーに気づいてほしいことをいくつか共有します。また、MCPとFCPのステータスを共有し、この機会を利用していくつかのWGから作業の最新情報を得ます。
- **ベータおよび安定版のノミネーションを確認:** これらは、それぞれベータおよび安定版へバックポートするもののノミネーションです。
  その後、コンパイラが以前に動作していたコードを壊した新しいケースを探します。リグレッションは修正すべき重要な問題であるため、P-criticalまたはP-highとタグ付けされる可能性が高いです。主な例外はバグ修正です(ただし、そこでも最初に[警告を出すこと][procedure]を目指すことがよくあります)。
- **P-criticalおよびP-highバグのレビュー:** P-criticalおよびP-highバグは、進捗を積極的に追跡するのに十分重要なものです。P-criticalおよびP-highバグには、理想的には常に担当者がいるべきです。
- **`S-waiting-on-t-compiler`および`I-compiler-nominated`の問題を確認:** これらは、チームからのフィードバックが求められている問題です。
- **パフォーマンストリアージレポートを確認:** パフォーマンスを悪化させたPRを確認し、パフォーマンスのリグレッションを元に戻す価値があるか、または将来のPRでリグレッションに対処できるかを決定しようとします。

ミーティングは現在、木曜日の午前10時(ボストン時間、通常UTC-4、ただし夏時間により複雑になることがあります)に開催されています。

[procedure]: ./bug-fix-procedure.md
[zulip-t-compiler]: https://rust-lang.zulipchat.com/#narrow/stream/131828-t-compiler
[zulip-help]: https://rust-lang.zulipchat.com/#narrow/stream/182449-t-compiler.2Fhelp
[zulip-meetings]: https://rust-lang.zulipchat.com/#narrow/stream/238009-t-compiler.2Fmeetings

## チームメンバーシップ

Rustチームのメンバーシップは、通常、誰かがコンパイラに長期間にわたって重要な貢献をしている場合に提供されます。メンバーシップは認識であると同時に義務でもあります:
コンパイラチームのメンバーは、一般的に、レビューやその他の作業を行うだけでなく、保守を支援することが期待されています。

コンパイラチームのメンバーになることに興味がある場合、最初にすべきことは、バグを修正したり、ワーキンググループに参加したりすることです。バグを見つける良い方法の1つは、
[open issues tagged with E-easy](https://github.com/rust-lang/rust/issues?q=is%3Aopen+is%3Aissue+label%3AE-easy)または
[E-mentor](https://github.com/rust-lang/rust/issues?q=is%3Aopen+is%3Aissue+label%3AE-mentor)を探すことです。

また、[closed due to inactivity](https://github.com/rust-lang/rust/pulls?q=is%3Apr+label%3AS-inactive)のPRの墓場を掘り下げることもできます。
その中のいくつかには、まだ有用な作業が含まれている可能性があります - 関連する問題があればそれを参照してください - そして、元の著者が時間がなかった仕上げのタッチだけが必要かもしれません。

### r+権限

rustcに個々のPRをいくつか作成すると、r+権限が提供されることがよくあります。これは、「bors」(rustcにどのPRをランドさせるかを管理するロボット)にPRをマージするよう指示する権利があることを意味します
([borsと話す方法についてのいくつかの説明はこちら][homu-guide])。

[homu-guide]: https://bors.rust-lang.org/

レビュアーのガイドラインは次のとおりです:

- 誰に割り当てられているかに関係なく、いつでもPRをレビューすることを歓迎します。ただし、次の場合を除き、PRをr+しないでください:
  - コードのその部分に自信がある。
  - 他の誰もそれを最初にレビューしたくないと確信している。
    - 例えば、コードの特に敏感な部分に触れているため、ランド前にレビューしたいという希望を表明する人がいることがあります。
- レビュー時には常に礼儀正しくしてください: あなたはRustプロジェクトの代表者であるため、[Code of Conduct]に関しては通常以上に努力することが期待されています。

[Code of Conduct]: https://www.rust-lang.org/policies/code-of-conduct

### レビュアーローテーション

r+権限を取得したら、[reviewer rotation]に追加されることもできます。
[triagebot]は、受信PRをレビュアーに[automatically assigns]するボットです。
追加された場合、PRをレビューするためにランダムに選択されます。レビューするのに不安を感じるPRが割り当てられた場合は、`r? @so-and-so`のようなコメントを残して他の誰かに割り当てることもできます - 誰にリクエストすればよいかわからない場合は、`r? @nikomatsakis for reassignment`と書くだけで、@nikomatsakisが誰かを選択します。

[reviewer rotation]: https://github.com/rust-lang/rust/blob/36285c5de8915ecc00d91ae0baa79a87ed5858d5/triagebot.toml#L528-L577
[triagebot]: https://github.com/rust-lang/triagebot/
[automatically assigns]: https://forge.rust-lang.org/triagebot/pr-assignment.html

レビュアーローテーションに参加することは、私たち全員のレビュー負担を軽減するため、非常に感謝されています!ただし、人々のPRにタイムリーなフィードバックを提供する時間がない場合は、リストに載らない方が良いかもしれません。
