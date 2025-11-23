<!-- date-check: Jul 2025 -->

# 新しい言語機能の実装

コンパイラに新しい重要な機能を実装したい場合は、
すべてがスムーズに進むように、このプロセスを経る必要があります。

**注意：このセクションは*言語*機能に関するものであり、*ライブラリ*機能ではありません。
ライブラリ機能は[別のプロセス]を使用します。**

言語への変更を提案する際の手順については、[Rust言語設計チームの手順][lang-propose]も参照してください。

[a different process]: ./stability.md
[lang-propose]: https://lang-team.rust-lang.org/how_to/propose.html

## @rfcbot FCPプロセス

変更が小さく、議論の余地がなく、破壊的ではなく、
安定版言語にユーザーが観察可能な方法で影響を与えない、または新しい不安定な機能を追加しない場合は、
PRを書いて、そのコードの部分を知っている誰かからr+を得るだけで済みます。
しかし、そうでない場合は、もっと多くのことをする必要があります。
コンパイラ内部の作業であっても、
チームの残りのメンバーからの合意なしに議論の余地のある変更をプッシュするのは悪い考えです
（「分散システム」の意味では、知らないものを壊さないようにするため、
そして社会的な意味では、PRの争いを避けるため）。

チームの合意が必要な変更については、
最終コメント期間（FCP）を提案するプロセスを使用します。
関連するチームのメンバーでない場合（したがって@rfcbot権限がない場合）は、
メンバーの誰かに開始を依頼してください。
彼ら自身が懸念を持っていない限り、彼らは開始すべきです。

FCPプロセスは合意が必要な場合にのみ必要です。
変更に合意を必要とするプロセスがなく、
誰も問題を抱えないと思う場合は、r+のみに依存しても構いません。
例えば、
予約されたコンパイラ内部`rustc_`名前空間の不安定なコマンドラインフラグや
属性を追加または変更することは、
コンパイラ開発や標準ライブラリの使用のためであり、
nightlyエコシステムで広く使用されることを期待しない限り、
FCPなしで構いません。
一部のチームは、このようなシナリオで使用するより軽量なプロセスを持っています。
例えば、
コンパイラチームは、完全な合意を必要とせずにサポートとフィードバックを得る
軽量な方法として、Major Change Proposal（[MCP]）を提出することを推奨しています。

[MCP]: https://forge.rust-lang.org/compiler/proposals-and-stabilization.html#how-do-i-submit-an-mcp

FCPを提案するために、実装を完全にr+の準備ができている必要はありませんが、
一般的には、少なくとも概念実証があると良いでしょう。
そうすれば、人々は何について話しているのかを見ることができます。

FCPが提案されると、チームのすべてのメンバーがFCPに署名する必要があります。
全員が署名した後、
10日間の「最終コメント期間」（名前の由来）があり、誰でもコメントでき、
懸念が提起されなければ、PR/issueはFCP承認を得ます。

## 機能を書くためのロジスティクス

機能を機能する方法で実装するために、
いくつかの「ロジスティック」な障害を乗り越える必要があるかもしれません。

### 警告サイクル

場合によっては、機能やバグ修正が一部のエッジケースで既存のプログラムを壊す可能性があります。
その場合、
craterランを実行して影響を評価し、
[エディションゲートlint](diagnostics.md#edition-gated-lints)に使用されるものと同様の
将来の互換性lintを追加することをお勧めします。

### 安定性

私たちは[Rustの安定性を重視しています]。
安定版で動作し実行されるコードは（ほとんど）壊れるべきではありません。
そのため、
チームの合意とコードレビューだけで機能を世界にリリースしたくありません。
nightlyでその機能を使用する実際の経験を得たいと思いますし、
その経験に基づいて機能を変更したいかもしれません。

それを可能にするために、
ユーザーがその新しい機能に誤って依存しないようにする必要があります。
そうしないと、
特に実験に時間がかかるか遅れ、機能がtrainで安定版に到達する場合、
事実上安定版になってしまい、
人々のコードを壊さずに変更を加えることができなくなります。

私たちがそれを行う方法は、すべての新機能が機能ゲートされていることを確認することです。
機能ゲート（`#[feature(foo)]`）を有効にしないと使用できず、
安定版/ベータコンパイラでは有効にできません。
技術的な詳細については、[コード内の安定性]セクションを参照してください。

最終的には、機能を使用する十分な経験を得て、必要な変更を加え、
満足した後、[ここ]で説明されている安定化プロセスを使用して世界に公開します。
それまでは、機能は確定していません。
機能のすべての部分を変更できますし、機能を完全に書き直したり削除したりすることもできます。
機能は、長期間不安定で変更されないことで既得権を得ることはありません。

###  追跡Issue

不安定な機能のステータス、
nightlyでの使用中に得た経験、
および安定化を妨げる懸念を追跡するために、
すべての機能ゲートには追跡issueが必要です。
機能に関連するissueやPRを作成するときは、この追跡issueを参照し、
機能の進捗に関する更新があるときは、追跡issueに投稿してください。

承認されたRFCまたは承認されたlang実験の一部である機能については、
そのための追跡issueを使用してください。

他の機能については、その機能の追跡issueを作成してください。
issueタイトルは「Tracking issue for YOUR FEATURE」にする必要があります。
["Tracking Issue" issueテンプレート][template]を使用してください。

[template]: https://github.com/rust-lang/rust/issues/new?template=tracking_issue.md

### Lang実験

コンパイラに組み込むには、
言語にユーザーが見える効果を持つ機能（不安定なものでも）は、
承認されたRFCまたは承認された[lang experiment]の一部である必要があります。

新しいlang実験を提案するには、
動機と意図された解決策を説明する`rust-lang/rust`にissueを開きます。
承認されると、このissueが実験の追跡issueになるので、
これらの他の詳細を含めながら追跡issue[template]を使用してください。
lang チームにissueをノミネートし、`@rust-lang/lang`と`@rust-lang/lang-advisors`をCCしてください。
実験が承認されると、追跡issueは`B-experimental`としてマークされます。

lang実験に関連する機能フラグは、
機能のRFCが承認されるまで`incomplete`としてマークする必要があります。

[lang experiment]: https://lang-team.rust-lang.org/how_to/experiment.html

##  コード内の安定性

新しい不安定な機能を実装するには、以下の手順を実行する必要があります。

1. [追跡issue]を開くか特定します。
   承認されたRFCまたは承認されたlang実験の一部である機能については、
   そのための追跡issueを使用してください。

   追跡issueに`C-tracking-issue`と関連する`F-feature_name`ラベルを付けます
   （必要に応じてそのラベルを追加します）。

1. 機能ゲートの名前を選びます（RFCの場合は、RFCの名前を使用します）。

1. `rustc_span/src/symbol.rs`の`Symbols {...}`ブロックに機能名を追加します。

   このブロックはアルファベット順である必要があることに注意してください。

1. `rustc_feature/src/unstable.rs`の不安定な`declare_features`ブロックに
   機能ゲート宣言を追加します。

   ```rust ignore
   /// description of feature
   (unstable, $feature_name, "CURRENT_RUSTC_VERSION", Some($tracking_issue_number))
   ```

   まだ追跡issueを開いていない場合
   （例えば、機能が承認される可能性が高いかどうかについて初期フィードバックが欲しい場合）、
   一時的に`None`を使用できます - ただし、PRをマージする前に必ず更新してください！

   例えば：

   ```rust ignore
   /// Allows defining identifiers beyond ASCII.
   (unstable, non_ascii_idents, "CURRENT_RUSTC_VERSION", Some(55467), None),
   ```

   機能は不完全としてマークでき、
   タイプを`incomplete`に設定することで
   デフォルトで警告する[`incomplete_features` lint]をトリガーします。

   [`incomplete_features` lint]: https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#incomplete-features

   ```rust ignore
   /// Allows deref patterns.
   (incomplete, deref_patterns, "CURRENT_RUSTC_VERSION", Some(87121), None),
   ```

   lang実験に関連する機能フラグは、
   機能のRFCが承認されるまで`incomplete`としてマークする必要があります。

   [セマンティックマージコンフリクト]を避けるために、
   `1.70`または別の明示的なバージョン番号の代わりに`CURRENT_RUSTC_VERSION`を使用してください。

   [semantic merge conflicts]: https://bors.tech/essay/2017/02/02/pitch/

1. 機能ゲートが設定されていない限り、新機能の使用を防ぎます。
   コンパイラのほとんどの場所で、
   `tcx.features().$feature_name()`という式を使用して確認できます。

    機能ゲートが設定されていない場合は、
    機能以前の動作を維持するか、エラーを発生させる必要があります。
    どちらが理にかなっているかによります。
    エラーは一般的に[`rustc_session::parse::feature_err`]を使用する必要があります。
    エラーを追加する例については、[#81015]を参照してください。

   新しい構文を導入する機能の場合、展開前のゲーティングを代わりに使用する必要があります。
   解析中に新しい構文が解析されるとき、
   シンボルは現在のクレートの[`GatedSpans`]に
   `self.sess.gated_span.gate(sym::my_feature, span)`を介して挿入する必要があります。

   ゲートされたスパンに挿入された後、
   スパンは実際に機能を拒否する[`rustc_ast_passes::feature_gate::check_crate`]関数で
   チェックする必要があります。
   どのようにゲートされるかは正確な機能のタイプによって異なりますが、
   おそらく`gate_all!()`マクロを使用します。

1. 機能ゲートなしで機能を使用できないことを確認するテストを追加します。
   `tests/ui/feature-gates/feature-gate-$feature_name.rs`を作成してください。
   `./x test tests/ui/feature-gates/ --bless`を実行して、
   対応する`.stderr`ファイルを生成できます。

1. 不安定なブックにセクションを追加します。
   `src/doc/unstable-book/src/language-features/$feature_name.md`に追加してください。

1. 新機能のために多くのテストを書いてください。できれば`tests/ui/$feature_name/`に。
   テストのないPRは受け入れられません！

1. PRをレビューしてもらい、マージしてください。
   これで、Rustに機能を正常に実装しました！

[`GatedSpans`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/parse/struct.GatedSpans.html
[#81015]: https://github.com/rust-lang/rust/pull/81015
[`rustc_session::parse::feature_err`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/parse/fn.feature_err.html
[`rustc_ast_passes::feature_gate::check_crate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast_passes/feature_gate/fn.check_crate.html
[value the stability of Rust]: https://github.com/rust-lang/rfcs/blob/master/text/1122-language-semver.md
[stability in code]: #stability-in-code
[here]: ./stabilization_guide.md
[tracking issue]: #tracking-issues
[add-feature-gate]: ./feature-gates.md#adding-a-feature-gate

## テストの呼びかけ

実装が完了すると、
機能はnightlyユーザーが使用できるようになりますが、まだ安定版Rustの一部ではありません。
これは、[メインのRustブログ][rust-blog]にブログ投稿を書き、
「テストの呼びかけ」を発行する良い機会です。

以前のそのようなブログ投稿には、次のものがあります。

1. [The push for GATs stabilization](https://blog.rust-lang.org/2021/08/03/GATs-stabilization-push/)
2. [Changes to `impl Trait` in Rust 2024](https://blog.rust-lang.org/2024/09/05/impl-trait-capture-rules.html)
3. [Async Closures MVP: Call for Testing!](https://blog.rust-lang.org/inside-rust/2024/08/09/async-closures-call-for-testing/)

または、[*This Week in Rust*][twir]には、これのための[セクション][twir-cft]があります。
これが使用された例の1つは次のとおりです。

- [Call for testing on boolean literals as cfg predicates](https://github.com/rust-lang/rust/issues/131204#issuecomment-2569314526)

どのオプションを選択するかは、言語変更の重要性によって異なるかもしれませんが、
[*This Week in Rust*][twir]セクションは、
メインのRustブログの専用投稿よりも目立たない可能性があることに注意してください。

## 磨き上げ

ユーザーに磨き上げられた体験を提供することは、rustcに機能を実装するだけではありません。
私たちが出荷するすべてのツールとリソースについて考える必要があります。
この作業には次のものが含まれます。

- [Rust Reference][reference]で言語機能を文書化する。
- 新しい構文をフォーマットするために[`rustfmt`]を拡張する（該当する場合）。
- [`rust-analyzer`]を拡張する（該当する場合）。
  この作業の範囲は、言語機能の性質によって異なる場合があります。
  一部の機能は*完全な*サポートでブロックされる必要がないためです。
   - 言語機能が、[`rust-analyzer`]でサポートが実装される前に
     単に存在するだけでユーザー体験を低下させる場合、
     langチームはブロッキングの懸念を提起する可能性があります。
   - そのような例には、[`rust-analyzer`]が解析できない新しい構文や、
     理解できない型推論の変更が含まれる可能性があり、
     それらが偽の診断につながる場合です。

## 安定化

機能のライフサイクルの最終ステップは[安定化][stab]です。
これは、機能がすべてのRustユーザーが使用できるようになる時です。
この時点で、
後方互換性のない変更は一般的に許可されなくなります
（詳細については、langチームの[定義されたsemverポリシー](https://rust-lang.github.io/rfcs/1122-language-semver.html)を参照してください）。
安定化の詳細については、[安定化ガイド][stab]を参照してください。


[stab]: ./stabilization_guide.md
[rust-blog]: https://github.com/rust-lang/blog.rust-lang.org/
[twir]: https://github.com/rust-lang/this-week-in-rust
[twir-cft]: https://this-week-in-rust.org/blog/2025/01/22/this-week-in-rust-583/#calls-for-testing
[`rustfmt`]: https://github.com/rust-lang/rustfmt
[`rust-analyzer`]: https://github.com/rust-lang/rust-analyzer
[reference]: https://github.com/rust-lang/reference
