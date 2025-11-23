# @rustbotをマスターする

`@rustbot`（`triagebot`とも呼ばれます）は、通常であれば`rust-lang`組織のGitHubメンバーシップが必要な特定のタスクを、すべてのコントリビューターが実行できるようにするユーティリティロボットです。`rustc`へのコントリビューターにとって最も興味深い機能は、issueのクレームとラベルの再設定です。

## Issueのクレーム

`@rustbot`は、誰でもissueを自分に割り当てることができるコマンドを提供しています。作業したいissueを見つけたら、そのissueのコメントとして次のメッセージを送信できます：

    @rustbot claim

これにより、まだ割り当て先がない場合、`@rustbot`にそのissueをあなたに割り当てるように指示します。GitHubの制限により、間接的に割り当てられる場合があることに注意してください。つまり、`@rustbot`が代わりにプレースホルダーとして自身を割り当て、トップコメントを編集してissueがあなたに割り当てられたことを反映します。

issueの割り当てを解除したい場合は、`@rustbot`に別のコマンドがあります：

    @rustbot release-assignment

## Issueのラベル再設定

issueやPRのラベルを変更することも、通常は組織のメンバーのみが行えます。しかし、`@rustbot`を使えば、いくつかの制限付きではありますが、自分でissueにラベルを付け直すことができます。これは主に2つの場合に役立ちます：

**Issueトリアージの支援**：この記事を書いている時点で、Rustのissueトラッカーには5,000を超えるオープンなissueがあります。そのため、ラベルは可能な限り整理するための最も強力なツールです。issueトラッカーで何時間も過ごしてトリアージする必要はありませんが、issueを開く場合、自分でラベルを付けることに抵抗がなければ、自由にラベルを付けてください。

**PRのステータスの更新**：私たちは「ステータスラベル」を使ってPRのステータスを反映させています。たとえば、PRにマージの競合がある場合、自動的に`S-waiting-on-author`が割り当てられ、レビュアーはあなたがPRをリベースするまでレビューしない可能性があります。ブランチをリベースしたら、自分でラベルを変更して`S-waiting-on-author`ラベルを削除し、`S-waiting-on-review`を追加する必要があります。この場合、`@rustbot`コマンドは次のようになります：

    @rustbot label -S-waiting-on-author +S-waiting-on-review

このコマンドの構文はかなり緩いので、このコマンド呼び出しには他のバリエーションもあります。ラベルを更新するためのショートカットもあります。たとえば、`@rustbot ready`は上記のコマンドと同じことを行います。詳細については、[ラベリングに関するドキュメントページ][labeling]と[ショートカット][shortcuts]を参照してください。

[labeling]: https://forge.rust-lang.org/triagebot/labeling.html
[shortcuts]: https://forge.rust-lang.org/triagebot/shortcuts.html

## その他のコマンド

`@rustbot`の機能について興味がある場合は、その[ドキュメント]を確認してください。これはボットのリファレンスとして意図されており、ボットがアップグレードされるたびに最新の状態に保たれるべきです。

`@rustbot`はリリースチームによって保守されています。既存のコマンドに関するフィードバックや新しいコマンドの提案がある場合は、[Zulip][zulip]で気軽に連絡するか、[triagebot リポジトリ][repo]にissueを提出してください。

[documentation]: https://forge.rust-lang.org/triagebot/index.html
[zulip]: https://rust-lang.zulipchat.com/#narrow/stream/224082-t-release.2Ftriagebot
[repo]: https://github.com/rust-lang/triagebot/
