# 安定化のリクエスト

**注意**：このページは*言語*機能の安定化についてです。*ライブラリ*機能の安定化については、[ライブラリ機能の安定化]を参照してください。

[ライブラリ機能の安定化]: ./stability.md#stabilizing-a-library-feature

不安定な機能が十分にテストされ、未解決の懸念がなくなったら、誰でもその安定化を推進できます。ただし、その機能に取り組んだ人々を巻き込むのが賢明です。次の手順に従ってください：

## 必要に応じてRFCを書く

機能が[lang experiment]の一部であった場合、langチームは一般的に安定化の前にまずRFCを受け入れることを望みます。

[lang experiment]: https://lang-team.rust-lang.org/how_to/experiment.html

## ドキュメントのPR

<a id="updating-documentation"></a>

機能は[`src/doc/unstable-book`]にある[`Unstable Book`]でドキュメント化されている可能性があります。機能ゲートのページが存在する場合は削除してください。そのドキュメントの有用な部分を他の場所に統合してください。

更新が必要なドキュメントの場所には以下が含まれます：

- [The Reference]：これは詳細に完全に更新する必要があり、lang-docsチームのメンバーがPRをレビューし、承認してから安定化をマージできるようにする必要があります。
- [The Book]：これは必要に応じて更新されます。確信が持てない場合は、このリポジトリにissueを開くと、議論できます。
- 標準ライブラリのドキュメント：必要に応じて更新されます。言語機能は通常これを必要としませんが、`?`が言語に追加されたときのように慣用的な例の書き方を変える機能である場合、ライブラリドキュメントでこれらを更新することが重要です。標準ライブラリのキーワードドキュメントとABIドキュメントもレビューしてください。言語の変更によって更新が必要になることがあります。
- [Rust by Example]：必要に応じて更新されます。

この新機能に関するドキュメントを更新するために、上記のリポジトリのPRを準備してください。これらのリポジトリのメンテナーは、安定化プロセス全体が完了するまでこれらのPRを開いたままにします。その間、次のステップに進むことができます。

## 安定化レポートを書く

[このリポジトリにあるテンプレート][srt]を使用して安定化レポートを作成してください。

安定化レポートには以下がまとめられます：

- RFCが受け入れられてからの主な設計決定と逸脱。langチームによってFCPされたか、その他の方法で受け入れられた決定と、初めてlangチームに提示される決定の両方を含みます。
  - 多くの場合、最終的に安定化される言語機能は、元のRFCから大幅な設計の逸脱があります。それは問題ありませんが、これらの逸脱は慎重に強調され、説明される必要があります。
- RFCが受け入れられてから行われた作業を、言語機能を前進させた主要なコントリビューターを認めながらまとめます。

[*安定化テンプレート*][srt]には、この機能とlangのサブチーム（例：types、opsem、lang-docsなど）との間のつながりを浮き彫りにし、一般的に見落とされがちな項目を特定することを目的とした一連の質問が含まれています。

[srt]: ./stabilization_report_template.md

安定化レポートは通常、安定化PR（次のセクションを参照）のメインコメントとして投稿されます。

## 安定化PR

すべての機能は異なり、一部はこのガイドで議論する以上のステップが必要になる場合があります。

安定化がlangチームによって検討される前に、機能を説明するReferenceへの完全なPRが必要であり、安定化PRがマージされる前に、このPRはlang-docsチームによってレビューおよび承認されている必要があります。

### 機能ゲートリストの更新

[`compiler/rustc_feature/src/unstable.rs`]に不安定な機能ゲートの中心的なリストがあります。`declare_features!`マクロを探してください。安定化を目指している機能のエントリがあるはずです。次のようなものです（[rust-lang/rust#32409]から取得）：

```rust,ignore
// pub(restricted) visibilities (RFC 1422)
(unstable, pub_restricted, "CURRENT_RUSTC_VERSION", Some(32409)),
```

上記の行は[`compiler/rustc_feature/src/accepted.rs`]に移動する必要があります。`declare_features!`呼び出しのエントリはソートされているので、正しい場所を見つけてください。完了すると、次のようになります：

```rust,ignore
// pub(restricted) visibilities (RFC 1422)
(accepted, pub_restricted, "CURRENT_RUSTC_VERSION", Some(32409)),
// これを変更したことに注意
```

（過去の変更のバージョン番号がファイルにあることに気づくと思いますが、安定化が行われることを期待するrustcバージョンを入れるのではなく、`CURRENT_RUSTC_VERSION`を使用する必要があります。）

### 機能ゲートの既存の使用を削除する

次に、コードベースで機能文字列（この場合は`pub_restricted`）を検索して、どこに現れるかを見つけます。`std`および任意のrustcクレート（`library/`および`compiler/`の下のテストフォルダーを含むが、トップレベルの`tests/`は除く）からの`#![feature(XXX)]`の使用を`#![cfg_attr(bootstrap, feature(XXX))]`に変更します。これには、現在のベータ（これが必要なのは機能がまだ現在のベータで不安定であるため）を使用してビルドされるstage0の機能ゲートのみが含まれます。

また、任意のテスト（例：`tests/`の下）からそれらの文字列を削除します。機能ゲートを具体的にターゲットとするテスト（つまり、機能を使用するために機能ゲートが必要であることをテストするが、他には何もない）がある場合は、単にテストを削除します。

### 機能を使用するために機能ゲートを必要としない

最も重要なのは、機能ゲートが存在しない場合にエラーをフラグするコードを削除することです（機能は安定と見なされるため）。機能が新しい構文を使用するために検出できる場合、そのコードの一般的な場所は`compiler/rustc_ast_passes/src/feature_gate.rs`です。たとえば、次のようなコードが表示される場合があります：

```rust,ignore
gate_all!(pub_restricted, "`pub(restricted)` syntax is experimental");
```

`gate_all!`マクロは、`pub_restricted`機能が有効になっていない場合にエラーを報告します。これは不要になりました。`pub(restricted)`が安定しているためです。

よりサブトルな機能の場合、次のようなコードが見つかる場合があります：

```rust,ignore
if self.tcx.features().async_fn_in_dyn_trait() { /* XXX */ }
```

この`pub_restricted`フィールド（機能にちなんで名付けられている）は、通常、機能フラグが存在しない場合はfalse、存在する場合はtrueになります。したがって、コードをフィールドがtrueであると仮定するように変換します。この場合、それは`if`を削除して`/* XXX */`だけを残すことを意味します。

```rust,ignore
if self.tcx.sess.features.borrow().pub_restricted { /* XXX */ }
次のようになります
/* XXX */

if self.tcx.sess.features.borrow().pub_restricted && something { /* XXX */ }
 次のようになります
if something { /* XXX */ }
```

[rust-lang/rust#32409]: https://github.com/rust-lang/rust/issues/32409
[`compiler/rustc_feature/src/accepted.rs`]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_feature/src/accepted.rs
[`compiler/rustc_feature/src/unstable.rs`]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_feature/src/unstable.rs
[The Reference]: https://github.com/rust-lang/reference
[The Book]: https://github.com/rust-lang/book
[Rust by Example]: https://github.com/rust-lang/rust-by-example
[`Unstable Book`]: https://doc.rust-lang.org/unstable-book/index.html
[`src/doc/unstable-book`]: https://github.com/rust-lang/rust/tree/HEAD/src/doc/unstable-book

## チーム指名

安定化PRを開くときは、langチームとそのアドバイザー（`@rust-lang/lang @rust-lang/lang-advisors`）および機能に関連するその他のチームにCCしてください。たとえば：

- `@rust-lang/types`：型システムの相互作用について。
- `@rust-lang/opsem`：unsafeコードとの相互作用について。
- `@rust-lang/compiler`：実装の堅牢性について。
- `@rust-lang/libs-api`：標準ライブラリAPIまたはその保証の変更について。
- `@rust-lang/lang-docs`：Referenceでこれをどのようにドキュメント化すべきかについての質問について。

安定化レポートとともに安定化PRが開かれた後、即座のコメントのために少し待ちます。そのようなコメントが「落ち着いて」、PRがlangチームによる検討の準備ができたと感じたら、次回のlang会議で検討するためのアジェンダに載せるために[PRをノミネート](https://lang-team.rust-lang.org/how_to/nominate.html)します。

`rust-lang`組織のメンバーでない場合は、割り当てられたレビュアーにあなたの代わりに関連チームにCCするように依頼できます。

## PRでFCPを提案する

langチームおよび他の関連チームが安定化をレビューし、質問に答えた後、チームの1人のメンバーがコメントすることで安定化の承認を提案する場合があります：

```text
@rfcbot fcp merge
```

十分な数のチームメンバーがレビューすると、PRは「最終コメント期間」（FCP）に移行します。新しい懸念が提起されなければ、この期間は完了し、通常の方法で実装レビュー後にPRをマージできます。

## 安定化のレビューとマージ

安定化では、`r+`を与える前に、PRが次のことを確認してください：

- チームが安定化のために提案したものと、Referenceで文書化されているものと一致している。
- チームが懸念を解決または回避するために途中で要求することを決定した変更が含まれている。
- 安定化レポートおよび関連するRFCや以前のlang FCPで説明されているものと正確に一致している。
- 指定され、安定化のために受け入れられ、Referenceで文書化されているもの以外の動作を安定版で公開しない。
- これらのことを確実に示す十分なテストがある。
- テストは、もちろん機能が動作することを包括的に示す必要がある。また、一般的なミスが犯されたときに表示される診断や、機能が誤って使用されたときの診断を示すことも考えてください。

各テスト内で、テストの目的とそれが実証しようとしている一連の不変条件を説明するコメントをテストの上部に含めてください。これはレビューにとって大きな助けになります。

テストカバレッジに既知のまたは意図的なギャップがある場合は、それを説明してください。

テストフォルダーと個々のテストをコンテキスト化してリンクしてください。

特に、PRをレビューするときは、langチームが考慮し、指定しなかったユーザーに見える詳細に注意してください。それを見つけた場合は、それを説明してlangチーム用にPRをノミネートしてください。
