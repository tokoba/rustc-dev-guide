# エラーとリント

`rustc`は優れたエラーメッセージを提供するために多くの努力が払われています。
この章は、コンパイラからコンパイルエラーとリントを出力する方法についてです。

## 診断構造

診断エラーの主要な部分は以下の通りです：

```
error[E0000]: main error message
  --> file.rs:LL:CC
   |
 LL | <code>
   | -^^^^- secondary label
   |  |
   |  primary label
   |
   = note: note without a `Span`, created with `.note`
note: sub-diagnostic message for `.span_note`
  --> file.rs:LL:CC
   |
LL | more code
   |      ^^^^
```

- レベル（`error`、`warning`など）。メッセージの重要度を示します。
  （[診断レベル](#diagnostic-levels)を参照）
- コード（例えば、「型の不一致」の場合、`E0308`）。これにより、
  ユーザーはエラーコードインデックス内の問題の詳細説明を通じて、
  現在のエラーについてより多くの情報を得ることができます。すべての診断に
  コードがあるわけではありません。例えば、リントによって作成された診断にはありません。
- メッセージ。これは問題の主要な説明です。一般的で、
  単独で成り立つことができるようにすべきです。そうすれば、単独で表示されても意味をなします。
- 診断ウィンドウ。これにはいくつかのものが含まれます：
  - プライマリスパンの開始位置のパス、行番号、列番号。
  - ユーザーの影響を受けたコードとその周辺。
  - ユーザーのコードに下線を引くプライマリおよびセカンダリスパン。これらのスパンには
    オプションで1つ以上のラベルを含めることができます。
    - プライマリスパンには、問題を説明するのに十分なテキストが必要です。
      それが唯一表示されているものである場合（例えば、
      IDEで）でも、まだ意味をなす必要があります。「空間認識」である（コードを
      指している）ため、エラーメッセージよりも一般的により簡潔にできます。
    - 複数のスパンラベルが重複する場合に乱雑な出力が予見される場合、
      適切に出力を調整することは良い考えです。例えば、
      `if/else arms have incompatible types`エラーは、アームがすべて同じ行にある場合、
      アームの1つが空の場合、およびこれらのケースのいずれも当てはまらない場合に、
      異なるスパンを使用します。
- サブ診断。すべてのエラーには、エラーのメイン部分に似た
  複数のサブ診断を持つことができます。これらは、説明の順序がコードの順序と
  一致しない場合に使用されます。説明の順序が「順序フリー」である場合、
  メイン診断でセカンダリラベルを活用することが好まれます。通常、より冗長ではありません。

テキストは事実に基づいており、複数の文が_必要_でない限り、大文字化と句点を避けるべきです：

```txt
error: the fobrulator needs to be krontrificated
```

コードまたは識別子がメッセージまたはラベルに表示される必要がある場合、
バッククォートで囲む必要があります：

```txt
error: the identifier `foo.bar` is invalid
```

### エラーコードと説明

ほとんどのエラーには関連するエラーコードがあります。エラーコードは、
エラーをトリガーする方法の例と、エラーに関する詳細な情報を含む長形式の
説明にリンクされています。これらは`--explain`フラグで表示するか、
[error index]経由で表示できます。

一般的なルールとして、説明がエラー自体よりも多くの情報を提供する場合、
エラーにコード（関連する説明付き）を付けます。多くの場合、
すべての情報を出力されたエラー自体に入れる方が良いです。しかし、
時にはそれがエラーを冗長にするか、すべてのケースに有用な情報を含めるには
トリガーが多すぎる場合があり、その場合は説明を追加することが良い考えです。[^estebank]
いつものように、よくわからない場合は、レビュアーに聞いてください！

関連するエラーコードを持つ新しいエラーを追加することにした場合は、
プロセスに関するガイドと重要な詳細について、[このセクション][error-codes]をお読みください。

[^estebank]: この経験則は**@estebank**によって[ここ][estebank-comment]で提案されました。

[error index]: https://doc.rust-lang.org/error-index.html
[estebank-comment]: https://github.com/rust-lang/rustc-dev-guide/pull/967#issuecomment-733218283
[error-codes]: ./diagnostics/error-codes.md

### リント対固定診断

一部のメッセージは[リント](#lints)を介して出力され、ユーザーはレベルを制御できます。
ほとんどの診断はハードコードされており、ユーザーはレベルを制御できません。

通常、診断が「固定」であるかリントであるかは明らかですが、
いくつかのグレーゾーンがあります。

いくつかの例を挙げます：

- 借用チェッカーエラー：これらは固定エラーです。ユーザーはこれらの診断のレベルを
  調整して借用チェッカーを黙らせることはできません。
- デッドコード：これはリントです。ユーザーはおそらく自分のクレートにデッドコードを
  望まないでしょうが、これをハードエラーにすると、リファクタリングと開発が
  非常に苦痛になります。
- [future-incompatible lints]：
  これらは消音可能なリントです。
  これらを固定エラーにすると、壊れすぎることが決定されたため、
  代わりに警告が出力され、
  最終的には固定（ハード）エラーになります。

ハードコードされた警告（`span_warn`のようなメソッドを使用する）は、
通常のコードでは避けるべきであり、代わりにリントを使用することが好まれます。
CLIフラグを使用した警告など、一部のケースでは、ハードコードされた警告の使用が必要です。

エラーレベルのリントをいつ固定エラーの代わりに使用するかのガイドラインについては、
以下の`deny` [lint level](#diagnostic-levels)を参照してください。

[future-incompatible lints]: #future-incompatible-lints

## 診断出力スタイルガイド

- 平易な簡単な英語で書いてください。メッセージが、小さい可能性のある画面（しばらく
  クリーニングされていない）で表示されたときに、パーティーの後にベッドから出てきた
  普通のプログラマーによって理解できない場合、それは複雑すぎます。
- `Error`、`Warning`、`Note`、および`Help`メッセージは小文字で始まり、
  句読点で終わりません。
- エラーメッセージは簡潔であるべきです。ユーザーはこれらのエラーメッセージを何度も見ることになり、
  より詳細な説明は`--explain`フラグで表示できます。とはいえ、理解しにくいほど
  簡潔にしないでください。
- 「illegal」という言葉は違法です。代わりに「invalid」またはより具体的な単語を使用してください。
- エラーは発生するコードのスパンを文書化する必要があります（[`rustc_errors::DiagCtxt`][DiagCtxt]の
  `span_*`メソッドまたは診断構造体の`#[primary_span]`を使用して簡単にこれを行います）。
  また、スパンが大きすぎない場合は、エラーに貢献した他のスパンを`note`します。
- スパンを使用してメッセージを出力する場合、問題を示すために必要な最小の量まで
  スパンを減らすようにしてください
- 同じエラーに対して複数のエラーメッセージを出力しないようにしてください。これには
  重複の検出が必要になる場合があります。
- コンパイラが特定のエラーメッセージに対して情報が少なすぎる場合、
  コンパイラチームと相談して、より多くの情報を追加できるライブラリコードの新しい属性を追加してください。
  例えば、[`#[rustc_on_unimplemented]`](#rustc_on_unimplemented)を参照してください。利用可能な場合は、
  これらのアノテーションを使用してください！
- Rustの学習曲線はかなり急であり、
  コンパイラメッセージは重要な学習ツールであることを覚えておいてください。
- コンパイラについて話すときは、`Rust`や`rustc`ではなく、`the compiler`と呼んでください。
- アイテムのリストを書くときは、[オックスフォードカンマ](https://en.wikipedia.org/wiki/Serial_comma)を使用してください。

### リント命名

[RFC 0344]から、リント名は次のガイドラインに従って一貫している必要があります：

基本的なルールは：リント名は「allow *lint-name*」または「allow *lint-name* items」として読んだときに
意味をなす必要があります。例えば、「allow `deprecated` items」と「allow `dead_code`」は意味をなしますが、
「allow `unsafe_block`」は文法的に正しくありません（複数形であるべきです）。

- リント名は、チェックされる悪いことを述べる必要があります。例えば、`deprecated`、
  そうすれば`#[allow(deprecated)]`（items）が正しく読めます。したがって、`ctypes`は
  適切な名前ではありません；`improper_ctypes`が適切です。

- 任意のアイテムに適用されるリント（安定性リントのような）は、チェックするものだけを
  言及する必要があります：`deprecated_items`ではなく`deprecated`を使用します。
  これによりリント名が短く保たれます。（再び、「allow *lint-name* items」と考えてください。）

- リントが特定の文法クラスに適用される場合、そのクラスを言及し、
  複数形を使用します：`unused_variable`ではなく`unused_variables`を使用します。
  これにより`#[allow(unused_variables)]`が正しく読めます。

- 不要、未使用、または無用なコードの側面をキャッチするリントは、
  `unused`という用語を使用する必要があります。例えば、`unused_imports`、`unused_typecasts`。

- 関数名に使用するのと同じ方法でスネークケースを使用します。

[RFC 0344]: https://github.com/rust-lang/rfcs/blob/master/text/0344-conventions-galore.md#lints

### 診断レベル

異なる診断レベルのガイドライン：

- `error`：コンパイラがプログラムをコンパイルできない問題を検出したときに出力されます。
  プログラムが無効であるか、プログラマーが特定の`warning`をエラーにすることを決定したためです。

- `warning`：コンパイラがプログラムについて奇妙なことを検出したときに出力されます。
  警告疲れを避け、実際にコードに問題がない場合の誤検知を避けるために、
  警告を追加する際には注意が必要です。警告を発行するのが適切な場合の例：

  - ユーザーが*行動を取る必要がある*状況。例えば、非推奨のアイテムを交換する、
    または`Result`を使用するが、それ以外はコンパイルを妨げない。
  - コードのセマンティクスに影響を与えずに削除できる不要な構文。
    例えば、未使用のコード、または不要な`unsafe`。
  - 非常に間違っている可能性が高い、危険な、または混乱するコードだが、
    言語は技術的に許可しており、エラーにする準備や自信がまだない。
    例えば、`unused_comparisons`（範囲外の比較）または
    `bindings_with_variant_name`（ユーザーはおそらくパターンにバインディングを
    作成するつもりではなかった）。
  - [Future-incompatible lints](#future-incompatible)、何かが偶然または
    誤って過去に受け入れられたが、拒否するとエコシステムで過度の破壊を引き起こす場合。
  - スタイルの選択。例えば、キャメルケースまたはスネークケース、または2018エディションの`dyn`トレイト
    警告。これらには追加される高いバーがあり、
    例外的な状況でのみ使用されるべきです。他のスタイルの選択は、
    デフォルトで許可されるリント、またはClippyやrustfmtのような他のツールの一部であるべきです。

- `help`：`error`または`warning`に続いて出力され、ユーザーに問題を解決する方法に関する
  追加情報を提供します。これらのメッセージには、多くの場合、提案文字列と
  [`rustc_errors::Applicability`]信頼レベルが含まれ、ツールによる自動ソース修正を
  ガイドします。詳細については、[Suggestions](#suggestions)セクションを参照してください。

  エラーまたは警告部分は、問題を修正する方法を提案*すべきではありません*、
  「help」サブ診断のみが提案すべきです。

- `note`：警告またはエラーの原因となったコードの追加の状況と部分をより多くの
  コンテキストと識別するために出力されます。例えば、借用チェッカーは、
  以前の競合する借用をメモします。

  `help`対`note`：`help`は、ユーザーが問題を修正するために行うことができる変更を
  示すために使用されるべきです。`note`は、他のコンテキスト、情報と事実、読むべき
  オンラインリソースなど、それ以外のすべてに使用されるべきです。

*lint levels*と混同しないでください。そのガイドラインは：

- `forbid`：リントはデフォルトで`forbid`にすべきではありません。
- `deny`：`error`診断レベルに相当します。いくつかの例：

  - 警告レベルから卒業した将来互換性のないまたはエディションベースのリント。
  - 非常に高い信頼度で正しくないものだが、
    それを通過させるためのエスケープハッチが欲しいもの。

- `warn`：`warning`診断レベルに相当します。ガイドラインについては上記の`warning`を参照してください。
- `allow`：デフォルトで`allow`にすべきリントの種類の例：

  - リントの誤検知率が高すぎる。
  - リントが意見的すぎる。
  - リントが実験的である。
  - リントは、通常は強制されないものを強制するために使用されます。
    例えば、`unsafe_code`リントは、アンセーフコードの使用を防ぐために使用できます。

リントレベルに関する詳細情報は、[rustc book][rustc-lint-levels]および[reference][reference-diagnostics]にあります。

[`rustc_errors::Applicability`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/enum.Applicability.html
[reference-diagnostics]: https://doc.rust-lang.org/nightly/reference/attributes/diagnostics.html#lint-check-attributes
[rustc-lint-levels]: https://doc.rust-lang.org/nightly/rustc/lints/levels.html

## 役立つヒントとオプション

### エラーのソースを見つける

特定のエラーが出力される場所を見つけるには、主に3つの方法があります：

- エラーメッセージ/ラベルまたはエラーコードのサブパートを`grep`します。これは
  通常うまく機能し、簡単ですが、エラーを出力するコードが、比較的深い
  コールスタックの背後でエラーが構築されるコードから削除される場合があります。
  それでも、方向性を掴むには良い方法です。
- ナイトリー専用フラグ`-Z treat-err-as-bug=1`で`rustc`を呼び出すと、
  最初に出力されるエラーを内部コンパイラエラーとして扱い、エラーが出力された時点で
  スタックトレースを取得できます。他のエラーでトリガーしたい場合は、`1`を
  他の値に変更してください。

  このアプローチには制限があります：
  - 一部の呼び出しは、コンパイルされた`rustc`にインライン化されるため、スタックトレースから省略されます。
  - エラーの_構築_は、_出力_される場所から遠く離れています。
    `grep`アプローチで直面した問題に似ています。
    場合によっては、順番に出力するために複数のエラーをバッファリングします。
- `-Z track-diagnostics`で`rustc`を呼び出すと、エラーと一緒にエラー作成場所が
  出力されます。

通常の開発慣行が適用されます：`debug!()`ステートメントの賢明な使用と、
物事がどの順序で起こっているかを把握するためにブレークポイントをトリガーするための
デバッガーの使用。

## `Span`

[`Span`][span]は、コンパイルされているコード内の場所を表すために`rustc`で使用される
主要なデータ構造です。`Span`はHIRとMIRのほとんどの構造に添付されており、
より情報の多いエラー報告を可能にします。

[span]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/struct.Span.html

`Span`は[`SourceMap`][sourcemap]で検索して、[`span_to_snippet`][sptosnip]や
`SourceMap`の他の同様のメソッドでエラーを表示するのに便利な「スニペット」を取得できます。

[sourcemap]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/source_map/struct.SourceMap.html
[sptosnip]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/source_map/struct.SourceMap.html#method.span_to_snippet

## エラーメッセージ

[`rustc_errors`][errors]クレートは、エラーを報告するために使用されるほとんどの
ユーティリティを定義します。

[errors]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/index.html

診断は、`Diagnostic`トレイトを実装する型として実装できます。これは、診断の出力ロジックと
メインコードパスの分離を強制するため、新しい診断に推奨されます。それほど複雑でない
診断の場合、`Diagnostic`トレイトは派生できます -- [Diagnostic structs][diagnostic-structs]を参照してください。
トレイト実装内では、以下で説明するAPIを通常通り使用できます。

[diagnostic-structs]: ./diagnostics/diagnostic-structs.md

[`DiagCtxt`][DiagCtxt]には、エラーを作成および出力するメソッドがあります。これらのメソッドは
通常、`span_err`または`struct_span_err`または`span_warn`などの名前を持ちます...
たくさんあります；それらは異なるタイプの「エラー」を出力します。警告、エラー、致命的エラー、提案などです。

[DiagCtxt]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.DiagCtxt.html

一般的に、このようなメソッドには2つのクラスがあります：エラーを直接出力するものと、
出力するものをより細かく制御できるもの。例えば、
[`span_err`][spanerr]は、与えられた`Span`で与えられたエラーメッセージを出力しますが、
[`struct_span_err`][strspanerr]は代わりに[`Diag`][diag]を返します。

これらのメソッドのほとんどは文字列を受け入れますが、新しい診断には翻訳可能な診断の
型付き識別子を使用することをお勧めします（[Translation][translation]を参照）。

[translation]: ./diagnostics/translation.md

`Diag`を使用すると、[`emit`][emit]メソッドを呼び出す前に、関連するノートと提案を
エラーに追加できます。（`Diag`を出力または[cancel][cancel]しないと、ICEが発生します。）
何ができるかについての詳細は[docs][diag]を参照してください。

[spanerr]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.DiagCtxt.html#method.span_err
[strspanerr]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.DiagCtxt.html#method.struct_span_err
[diag]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.Diag.html
[emit]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.Diag.html#method.emit
[cancel]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.Diag.html#method.cancel

```rust,ignore
// `Diag`を取得します。これはまだエラーを出力_しません_。
let mut err = sess.dcx.struct_span_err(sp, fluent::example::example_error);

// 場合によっては、`sp`がマクロによって生成されたかどうかを確認して、
// マクロ生成コードについての奇妙なエラーを出力しないようにする必要があります。

if let Ok(snippet) = sess.source_map().span_to_snippet(sp) {
    // スニペットを使用して提案された修正を生成します
    err.span_suggestion(suggestion_sp, fluent::example::try_qux_suggestion, format!("qux {}", snippet));
} else {
    // スニペットを生成できなかった場合、具体的な「suggestion」の代わりに
    // 「help」メッセージを出力します。実際には、これに到達することは
    // ほとんどありません。
    err.span_help(suggestion_sp, fluent::example::qux_suggestion);
}

// エラーを出力します
err.emit();
```

```fluent
example-example-error = oh no! this is an error!
  .try-qux-suggestion = try using a qux here
  .qux-suggestion = you could use a qux here instead
```

## 提案

ユーザーに彼らのコードが正確に_なぜ_間違っているかを伝えることに加えて、多くの場合、
それを修正する方法を伝えることもできます。この目的のために、
[`Diag`][diag]は構造化された提案APIを提供しており、ターミナルでコード提案を
見やすくフォーマットするか、（`--error-format json`フラグが渡されたときに）
[`rustfix`][rustfix]のようなツールで消費するためにJSONとしてフォーマットします。

[rustfix]: https://github.com/rust-lang/rustfix

すべての提案が機械的に適用されるべきではなく、提案されたコードに対する信頼度があります。
高い（`Applicability::MachineApplicable`）から低い（`Applicability::MaybeIncorrect`）まで。
レベルを選択する際は保守的にしてください。提案をするには、`Diag`の
[`span_suggestion`][span_suggestion]メソッドを使用します。最後の引数は、
提案が機械的に適用可能かどうかをツールにヒントを提供します。

提案は、現在のコンテンツを置き換える対応するコードを持つ1つ以上のスパンを指します。

それらに付随するメッセージは、次のコンテキストで理解可能である必要があります：

- 独立したサブ診断として表示される（これはデフォルトの出力です）
- 影響を受けるスパンを指すラベルとして表示される（冗長性のヒューリスティックが
  満たされる場合、これは自動的に行われます）
- コンテンツのない`help`サブ診断として表示される（提案がテキストから明らかな場合に
  使用されますが、ツールがそれらを適用できるようにしたい場合）
- 表示されない（_非常に_明らかな場合に使用されますが、ツールがそれらを適用できる
  ようにしたい場合）

[span_suggestion]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.Diag.html#method.span_suggestion

例えば、`qux`の提案を機械適用可能にするには、次のようにします：

```rust,ignore
let mut err = sess.dcx.struct_span_err(sp, fluent::example::message);

if let Ok(snippet) = sess.source_map().span_to_snippet(sp) {
    err.span_suggestion(
        suggestion_sp,
        fluent::example::try_qux_suggestion,
        format!("qux {}", snippet),
        Applicability::MachineApplicable,
    );
} else {
    err.span_help(suggestion_sp, fluent::example::qux_suggestion);
}

err.emit();
```

これは次のようなエラーを出力するかもしれません

```console
$ rustc mycode.rs
error[E0999]: oh no! this is an error!
 --> mycode.rs:3:5
  |
3 |     sad()
  |     ^ help: try using a qux here: `qux sad()`

error: aborting due to previous error

For more information about this error, try `rustc --explain E0999`.
```

提案が複数行にまたがる場合や、複数の提案がある場合など、いくつかのケースでは、
提案は独自に表示されます：

```console
error[E0999]: oh no! this is an error!
 --> mycode.rs:3:5
  |
3 |     sad()
  |     ^
help: try using a qux here:
  |
3 |     qux sad()
  |     ^^^

error: aborting due to previous error

For more information about this error, try `rustc --explain E0999`.
```

[`Applicability`][appl]の可能な値は：

- `MachineApplicable`：機械的に適用できます。
- `HasPlaceholders`：提案にプレースホルダーテキストがあるため、機械的に適用できません。
  例えば：```try adding a type: `let x: <type>` ```。
- `MaybeIncorrect`：提案が良いものであるかどうかわからないため、機械的に適用できません。
- `Unspecified`：上記のケースのどれに該当するかわからないため、機械的に適用できません。

[appl]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/enum.Applicability.html

### 提案スタイルガイド

- 提案は質問であってはいけません。特に、「did you mean」のような言葉は避けるべきです。
  場合によっては、特定の提案がなされる理由が不明確です。このような場合、
  提案が何であるかについて正直である方が良いです。

  「did you mean: `Foo`」対「there is a struct with a similar name: `Foo`」を比較してください。

- メッセージには「the following」、「as shown」などのフレーズを含めないでください。
  何について話しているかを伝えるためにスパンを使用してください。
- メッセージには「to do xyz, use」または「to do xyz, use abc」のような
  さらなる指示を含めることができます。
- メッセージには関数、変数、または型の名前を含めることができますが、
  式全体は避けてください。

## リント

コンパイラのリントインフラストラクチャは、[`rustc_middle::lint`][rlint]モジュールで定義されています。

[rlint]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/lint/index.html

### リントはいつ実行されますか？

異なるリントは、リントが仕事をするために必要な情報に基づいて異なる時間に実行されます。
一部のリントは*パス*にグループ化され、パス内のリントが単一のビジターを介して
一緒に処理されます。パスの一部は次のとおりです：

- 展開前パス：[マクロ展開]前の[ASTノード]で動作します。これは
  一般的に避けるべきです。
  - 例：[`keyword_idents`]は、将来のエディションでキーワードになる識別子を
    チェックしますが、マクロで使用される識別子に敏感です。

- 早期リントパス：[マクロ展開]と名前解決の後、[AST lowering]の直前の[ASTノード]で動作します。
  これらのリントは純粋に構文的なリント用です。
  - 例：[`unused_parens`]リントは、`if`条件のような、必要ではない状況での
    括弧付き式をチェックします。

- 後期リントパス：[分析]の終わり近く（借用チェックなどの後）の[HIRノード]で動作します。
  これらのリントは完全な型情報を利用できます。
  ほとんどのリントは後期です。
  - 例：[`invalid_value`]リント（明らかに無効な未初期化値をチェックする）は、
    型情報が必要なため、後期リントです。型が未初期化のまま残すことができるかどうかを
    判断するために型情報が必要です。

- MIRパス：[MIRノード]で動作します。これは他のパスとは少し異なります；
  MIRノードで動作するリントには、実行するための独自のメソッドがあります。
  - 例：[`arithmetic_overflow`]リントは、オーバーフローする可能性のある定数値を
    検出したときに出力されます。

ほとんどのリントはパスシステムを介してうまく機能し、かなり
簡単なインターフェイスと簡単な統合方法があります（主に特定の`check`関数を実装するだけです）。
ただし、一部のリントは、コンパイラのどこかの特定のコードパスに
存在する方が書きやすいです。例えば、
[`unused_mut`]リントは、借用チェッカーの情報と状態が必要なため、借用チェッカーに実装されています。

これらのインラインリントの一部は、リントシステムが準備できる前に発火します。これらの
リントは*バッファリング*され、コンパイラの後の段階でリントシステムが準備できるまで
保持されます。[Linting early in the compiler](#linting-early-in-the-compiler)を参照してください。

[ASTノード]: the-parser.md
[AST lowering]: ./hir/lowering.md
[HIRノード]: hir.md
[MIRノード]: mir/index.md
[マクロ展開]: macro-expansion.md
[分析]: part-4-intro.md
[`keyword_idents`]: https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#keyword-idents
[`unused_parens`]: https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-parens
[`invalid_value`]: https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#invalid-value
[`arithmetic_overflow`]: https://doc.rust-lang.org/rustc/lints/listing/deny-by-default.html#arithmetic-overflow
[`unused_mut`]: https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-mut

### リント定義用語

リントは[`LintStore`][LintStore]を介して管理され、さまざまな方法で登録されます。
次の用語は、一般的に登録方法に基づくリントの異なるクラスを指します。

- *組み込み*リントは、コンパイラソース内で定義されます。
- *ドライバー登録*リントは、外部ドライバーによってコンパイラドライバーが作成されるときに
  登録されます。これは、例えばClippyが使用するメカニズムです。
- *ツール*リントは、`clippy::`または`rustdoc::`のようなパスプレフィックスを持つリントです。
- *内部*リントは、rustcソースツリー自体でのみ実行され、コンパイラソース内で
  通常の組み込みリントのように定義される`rustc::`スコープのツールリントです。

リント登録に関する詳細情報は、[LintStore]の章にあります。

[LintStore]: diagnostics/lintstore.md

### リントの宣言

組み込みコンパイラリントは[`rustc_lint`][builtin]クレートで定義されています。
他のクレートで実装する必要があるリントは[`rustc_lint_defs`]で定義されています。
可能であれば、`rustc_lint`にリントを配置することを好むべきです。
1つの利点は、依存関係のルートに近いため、作業が
はるかに速くなることです。

[builtin]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/index.html
[`rustc_lint_defs`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint_defs/index.html

すべてのリントは、`LintPass` `trait`を実装する`struct`を介して実装されます（また、
より具体的なリントパストレイト、`EarlyLintPass`または`LateLintPass`のいずれかを実装することもできます。
これは、リントを実行するのに最適なタイミングによって異なります）。
トレイト実装により、リンターがASTをウォークする際に特定の構文構造を
チェックできます。その後、非常に似た方法でリントを出力することを
選択できます。

また、[`declare_lint!`]マクロを介して特定のリントのメタデータを宣言します。
このマクロには、名前、デフォルトレベル、簡単な説明、およびその他の
詳細が含まれます。

リントとリントパスをコンパイラに登録する必要があることに注意してください。

例えば、次のリントは`while true { ... }`の使用をチェックし、
代わりに`loop { ... }`を使用することを提案します。

```rust,ignore
// `WHILE_TRUE`というリントを宣言します
declare_lint! {
    WHILE_TRUE,

    // warn-by-default
    Warn,

    // この文字列はリントの説明です
    "suggest using `loop { }` instead of `while true { }`"
}

// これは構造体とリントパスを宣言し、関連するリントのリストを提供します。
// コンパイラは現在、関連するリントを直接使用しません（例えば、
// パスを実行しないか、パスが適切なリントのセットを出力することを
// チェックする）。ただし、リントパスのget_lintsメソッドを介して
// リントを登録する可能性があるため、ここで正確であることは良いことです（この
// マクロが生成する）。
declare_lint_pass!(WhileTrue => [WHILE_TRUE]);

// `WhileTrue`リントのヘルパー関数。
// 任意の量の括弧をトラバースし、最初の非括弧式を返します。
fn pierce_parens(mut expr: &ast::Expr) -> &ast::Expr {
    while let ast::ExprKind::Paren(sub) = &expr.kind {
        expr = sub;
    }
    expr
}

// `EarlyLintPass`には多くのメソッドがあります。このリントに必要なのは
// `check_expr`の定義をオーバーライドするだけですが、独自のリントのために
// 他のメソッドをオーバーライドすることもできます。メソッドの完全な
// リストについては、rustcドキュメントを参照してください。
impl EarlyLintPass for WhileTrue {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, e: &ast::Expr) {
        if let ast::ExprKind::While(cond, ..) = &e.kind
            && let ast::ExprKind::Lit(ref lit) = pierce_parens(cond).kind
            && let ast::LitKind::Bool(true) = lit.kind
            && !lit.span.from_expansion()
        {
            let condition_span = cx.sess.source_map().guess_head_span(e.span);
            cx.struct_span_lint(WHILE_TRUE, condition_span, |lint| {
                lint.build(fluent::example::use_loop)
                    .span_suggestion_short(
                        condition_span,
                        fluent::example::suggestion,
                        "loop".to_owned(),
                        Applicability::MachineApplicable,
                    )
                    .emit();
            })
        }
    }
}
```

```fluent
example-use-loop = denote infinite loops with `loop {"{"} ... {"}"}`
  .suggestion = use `loop`
```

[`declare_lint!`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint_defs/macro.declare_lint.html

### エディションゲートリント

場合によっては、新しいエディションでリントの動作を変更したいことがあります。これを行うには、
`declare_lint!`の呼び出しに遷移を追加するだけです：

```rust,ignore
declare_lint! {
    pub ANONYMOUS_PARAMETERS,
    Allow,
    "detects anonymous parameters",
    Edition::Edition2018 => Warn,
}
```

これにより、`ANONYMOUS_PARAMETERS`リントは2015エディションではデフォルトで許可されますが、
2018エディションではデフォルトで警告されます。

詳細については、[Edition-specific lints](./guides/editions.md#edition-specific-lints)を参照してください。

### 機能ゲートリント

機能に属するリントは、機能がクレートで有効になっている場合にのみ使用可能であるべきです。
これをサポートするために、リント宣言には次のような機能ゲートを含めることができます：

```rust,ignore
declare_lint! {
    pub SOME_LINT_NAME,
    Warn,
    "a new and useful, but feature gated lint",
    @feature_gate = sym::feature_name;
}
```

### 将来互換性のないリント

コンパイラ内での`future-incompatible`という用語の使用は、rustcがコンパイラのユーザーに
公開するものよりもわずかに広い意味を持っています。

rustc内部では、将来互換性のないリントは、ユーザーが書いたコードが将来コンパイルされない
可能性があることをユーザーに通知するためのものです。一般的に、将来互換性のないコードは
2つの理由で存在します：
- ユーザーはコンパイラが誤って受け入れた不健全なコードを書いています。健全性の
穴を修正すること（ユーザーのコードを壊すこと）はRustの後方互換性保証内ですが、
リントは、これがrustcの今後のバージョンで発生することをユーザーに警告するためにあります。
*コードが使用するエディションに関係なく*。これは、rustcがユーザーに「将来互換性がない」として
排他的に公開する意味です。
- ユーザーは、今後の*エディション*でコンパイルされなくなるか、意味が変わるコードを書いています。
これらはしばしば「エディションリント」と呼ばれ、通常、ユーザーがクレートのエディションを
更新すると壊れるコードに対してリントするために使用されるさまざまな「エディション互換性」リント
グループ（例：`rust_2021_compatibility`）で見ることができます。
詳細については、[migration lints](guides/editions.md#migration-lints)を参照してください。

将来互換性のないリントは、`@future_incompatible`追加「フィールド」で宣言する必要があります：

```rust,ignore
declare_lint! {
    pub ANONYMOUS_PARAMETERS,
    Allow,
    "detects anonymous parameters",
    @future_incompatible = FutureIncompatibleInfo {
        reference: "issue #41686 <https://github.com/rust-lang/rust/issues/41686>",
        reason: FutureIncompatibilityReason::EditionError(Edition::Edition2018),
    };
}
```

将来互換性のない変更が発生する理由を説明する`reason`フィールドに注目してください。
これにより、ユーザーが受け取る診断メッセージが変更され、リントが追加される
リントグループが決定されます。上記の例では、リントは「エディションリント」
（その「理由」が`EditionError`であるため）であり、匿名パラメーターの使用が
Rust 2018以降でコンパイルされなくなることをユーザーに通知します。

[LintStore::register_lints][fi-lint-groupings]内で、`future_incompatible`
フィールドを持つリントは、エディションベースのリントグループ（その`reason`が
エディションに関連している場合）または`future_incompatibility`リントグループに配置されます。

[fi-lint-groupings]: https://github.com/rust-lang/rust/blob/51fd129ac12d5bfeca7d216c47b0e337bf13e0c2/compiler/rustc_lint/src/context.rs#L212-L237

`declare_lint!`マクロでサポートされていないオプションの組み合わせが必要な場合は、
いつでも`declare_lint!`マクロを変更してこれをサポートできます。

### リントの名前変更または削除

リントが不適切に名前付けされているか、もはや必要ないと判断された場合、
リントは名前変更または削除のために登録する必要があり、ユーザーが古いリント名を使用しようとすると警告が
トリガーされます。名前変更/削除を宣言するには、[`rustc_lint::register_builtins`]関数の
コードに[`store.register_renamed`]または[`store.register_removed`]を使用して行を追加します。

```rust,ignore
store.register_renamed("single_use_lifetime", "single_use_lifetimes");
```

[`store.register_renamed`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/struct.LintStore.html#method.register_renamed
[`store.register_removed`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/struct.LintStore.html#method.register_removed
[`rustc_lint::register_builtins`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/fn.register_builtins.html

### リントグループ

リントはグループで有効にできます。これらのグループは、
[`rustc_lint::lib`][builtin]の[`register_builtins`][rbuiltins]関数で宣言されます。
`add_lint_group!`マクロは、新しいグループを宣言するために使用されます。

[rbuiltins]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/fn.register_builtins.html

例えば、

```rust,ignore
add_lint_group!(sess,
    "nonstandard_style",
    NON_CAMEL_CASE_TYPES,
    NON_SNAKE_CASE,
    NON_UPPER_CASE_GLOBALS);
```

これは、リストされたリントを有効にする`nonstandard_style`グループを定義します。
ユーザーは、ソースコードで`!#[warn(nonstandard_style)]`属性を使用するか、
コマンドラインで`-W nonstandard-style`を渡すことで、これらのリントを有効にできます。

一部のリントグループは`LintStore::register_lints`で自動的に作成されます。例えば、
理由が`FutureIncompatibilityReason::FutureReleaseError`（`@future_incompatible`が
`declare_lint!`で使用されるときのデフォルト）である`FutureIncompatibleInfo`で宣言された
リントは、`future_incompatible`リントグループに追加されます。エディションには、
指定されたエディションで壊れる将来互換性のないコードを通知するリントに対して、
自動的に生成される独自のリントグループ（例：`rust_2021_compatibility`）もあります。

### コンパイラの早い段階でのリント

時折、リントシステムが初期化される前に実行されるリントを定義する必要があります
（例えば、解析中またはマクロ展開中）。これは、警告を出力すべきか、エラーを出力すべきか、
または何も出力すべきでないかを知るために、リントレベルを計算する必要があるため、
問題があります。

この問題を解決するために、リントシステムが処理されるまでリントをバッファリングします。
[`Session`][sessbl]と[`ParseSess`][parsebl]の両方に、後でリントをバッファリングできる
`buffer_lint`メソッドがあります。リントシステムは、後でバッファリングされた
リントを自動的に処理します。

[sessbl]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/struct.Session.html#method.buffer_lint
[parsebl]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/parse/struct.ParseSess.html#method.buffer_lint

したがって、コンパイルの早い段階で実行されるリントを定義するには、通常のようにリントを定義しますが、
`buffer_lint`でリントを呼び出します。

#### コンパイラのさらに早い段階でのリント

パーサー（`rustc_ast`）は、他の`rustc*`クレートに依存関係を持てないという点で興味深いです。
特に、すべてのコンパイラリントインフラストラクチャが定義されている`rustc_middle::lint`または
`rustc_lint`に依存できません。それは面倒です！

これを解決するために、`rustc_ast`は独自のバッファリングされたリントタイプを定義し、
`ParseSess::buffer_lint`がそれを使用します。マクロ展開後、これらのバッファリングされた
リントは、コンパイラの残りの部分で使用される`Session::buffered_lints`にダンプされます。

## JSON診断出力

コンパイラは、診断をJSONオブジェクトとして出力する`--error-format json`フラグを受け入れます
（`cargo fix`などのツールの利益のために）。次のようになります：

```console
$ rustc json_error_demo.rs --error-format json
{"message":"cannot add `&str` to `{integer}`","code":{"code":"E0277","explanation":"\nYou tried to use a type which doesn't implement some trait in a place which\nexpected that trait. Erroneous code example:\n\n```compile_fail,E0277\n// here we declare the Foo trait with a bar method\ntrait Foo {\n    fn bar(&self);\n}\n\n// we now declare a function which takes an object implementing the Foo trait\nfn some_func<T: Foo>(foo: T) {\n    foo.bar();\n}\n\nfn main() {\n    // we now call the method with the i32 type, which doesn't implement\n    // the Foo trait\n    some_func(5i32); // error: the trait bound `i32 : Foo` is not satisfied\n}\n```\n\nIn order to fix this error, verify that the type you're using does implement\nthe trait. Example:\n\n```\ntrait Foo {\n    fn bar(&self);\n}\n\nfn some_func<T: Foo>(foo: T) {\n    foo.bar(); // we can now use this method since i32 implements the\n               // Foo trait\n}\n\n// we implement the trait on the i32 type\nimpl Foo for i32 {\n    fn bar(&self) {}\n}\n\nfn main() {\n    some_func(5i32); // ok!\n}\n```\n\nOr in a generic context, an erroneous code example would look like:\n\n```compile_fail,E0277\nfn some_func<T>(foo: T) {\n    println!(\"{:?}\", foo); // error: the trait `core::fmt::Debug` is not\n                           //        implemented for the type `T`\n}\n\nfn main() {\n    // We now call the method with the i32 type,\n    // which *does* implement the Debug trait.\n    some_func(5i32);\n}\n```\n\nNote that the error here is in the definition of the generic function: Although\nwe only call it with a parameter that does implement `Debug`, the compiler\nstill rejects the function: It must work with all possible input types. In\norder to make this example compile, we need to restrict the generic type we're\naccepting:\n\n```\nuse std::fmt;\n\n// Restrict the input type to types that implement Debug.\nfn some_func<T: fmt::Debug>(foo: T) {\n    println!(\"{:?}\", foo);\n}\n\nfn main() {\n    // Calling the method is still fine, as i32 implements Debug.\n    some_func(5i32);\n\n    // This would fail to compile now:\n    // struct WithoutDebug;\n    // some_func(WithoutDebug);\n}\n```\n\nRust only looks at the signature of the called function, as such it must\nalready specify all requirements that will be used for every type parameter.\n"},"level":"error","spans":[{"file_name":"json_error_demo.rs","byte_start":50,"byte_end":51,"line_start":4,"line_end":4,"column_start":7,"column_end":8,"is_primary":true,"text":[{"text":"    a + b","highlight_start":7,"highlight_end":8}],"label":"no implementation for `{integer} + &str`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the trait `std::ops::Add<&str>` is not implemented for `{integer}`","code":null,"level":"help","spans":[],"children":[],"rendered":null}],"rendered":"error[E0277]: cannot add `&str` to `{integer}`\n --> json_error_demo.rs:4:7\n  |\n4 |     a + b\n  |       ^ no implementation for `{integer} + &str`\n  |\n  = help: the trait `std::ops::Add<&str>` is not implemented for `{integer}`\n\n"}
{"message":"aborting due to previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to previous error\n\n"}
{"message":"For more information about this error, try `rustc --explain E0277`.","code":null,"level":"","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0277`.\n"}
```

出力は一連の行であり、それぞれがJSONオブジェクトですが、
一連の行全体は、残念ながら有効なJSONではないことに注意してください。これにより、
そのようなものを必要とするツールやトリック（[`python3 -m json.tool`へのパイプ](https://docs.python.org/3/library/json.html#module-json.tool)など）が妨げられます。
（これは、各行/オブジェクトが
フラッシュされるときに送信できるように、LSPパフォーマンスの目的で意図的だったと推測されますか？）

また、「人間」出力を文字列として含む「rendered」フィールドに注目してください；
これは、UIテストが構造化されたJSONと「人間」出力（*色なし*）の両方を
すべてを2回コンパイルせずに確認できるように導入されました。

「人間」可読およびjson形式のエミッターは、`rustc_errors`の下にあり、
両方とも`rustc_ast`クレートから[rustc_errors crate](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/index.html)に移動されました。

JSONエミッターは、JSONシリアライゼーション用に[独自の`Diagnostic`構造体](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/json/struct.Diagnostic.html)
（およびサブ構造体）を定義します。これを
[`errors::Diag`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.Diag.html)と混同しないでください！

## `#[rustc_on_unimplemented]`

この属性により、トレイト定義は、実装が期待されたが見つからなかった場合に
エラーメッセージを変更できます。属性内の文字列リテラルは形式文字列であり、
名前付きパラメーターでフォーマットできます。どのパラメーターが許可されるかについては、
以下のFormattingセクションを参照してください。

```rust,ignore
#[rustc_on_unimplemented(message = "an iterator over \
    elements of type `{A}` cannot be built from a \
    collection of type `{Self}`")]
trait MyIterator<A> {
    fn next(&mut self) -> A;
}

fn iterate_chars<I: MyIterator<char>>(i: I) {
    // ...
}

fn main() {
    iterate_chars(&[1, 2, 3][..]);
}
```

ユーザーがこれをコンパイルすると、次のように表示されます；

```txt
error[E0277]: an iterator over elements of type `char` cannot be built from a collection of type `&[{integer}]`
  --> src/main.rs:13:19
   |
13 |     iterate_chars(&[1, 2, 3][..]);
   |     ------------- ^^^^^^^^^^^^^^ the trait `MyIterator<char>` is not implemented for `&[{integer}]`
   |     |
   |     required by a bound introduced by this call
   |
note: required by a bound in `iterate_chars`
```

次の内容を変更できます：

- メインエラーメッセージ（`message`）
- ラベル（`label`）
- ノート（`note`）

例えば、次の属性

```rust,ignore
#[rustc_on_unimplemented(message = "message", label = "label", note = "note")]
trait MyIterator<A> {
    fn next(&mut self) -> A;
}
```

次の出力を生成します：

```text
error[E0277]: message
  --> <file>:10:19
   |
10 |     iterate_chars(&[1, 2, 3][..]);
   |     ------------- ^^^^^^^^^^^^^^ label
   |     |
   |     required by a bound introduced by this call
   |
   = help: the trait `MyIterator<char>` is not implemented for `&[{integer}]`
   = note: note
note: required by a bound in `iterate_chars`
```

これまでに説明した機能は、
[`#[diagnostic::on_unimplemented]`](https://doc.rust-lang.org/nightly/reference/attributes/diagnostics.html#the-diagnosticon_unimplemented-attribute)でも利用できます。
可能であれば、代わりにそれを使用すべきです。

### フィルタリング

より的を絞ったエラーメッセージを可能にするために、
これらのフィールドの適用を`on`でフィルタリングできます。

次のブールフラグでフィルタリングできます：

- `crate_local`：トレイト境界が満たされないことを引き起こすコードが
   ユーザーのクレートの一部であるかどうか。これは、依存関係の変更を
   必要とするコード変更を提案することを避けるために使用されます。
- `direct`：これが派生した義務ではなく、ユーザーが指定した義務であるかどうか。
- `from_desugaring`：`?`または`try`ブロックなど、何らかの種類の脱糖にいるかどうか。
   このフラグも一致できます。以下を参照してください。

次の名前と値に一致できます。`name = "value"`を使用します：

- `cause`：`ObligationCauseCode`列挙型の1つのバリアントに一致します。
   `"MainFunctionType"`のみがサポートされています。
- `from_desugaring`：`DesugaringKind`列挙型の特定のバリアントに一致します。
   脱糖は、そのバリアント名で識別されます。例えば、
   `?`脱糖の場合は`"QuestionMark"`、`try`ブロックの場合は`"TryBlock"`。
- `Self`とトレイトの任意のジェネリック引数。例えば、`Self = "alloc::string::String"`
   または`Rhs="i32"`。

コンパイラは、一致する複数の値を提供できます。例えば：

- self_ty、型引数が解決された状態と解決されていない状態でプリティプリントされます。
- 整数型がわかっている場合は`"{integral}"`。
- `"[]"`、`"[{ty}]"`、`"[{ty}; _]"`、`"[{ty}; $N]"`（該当する場合）。
- 上記のスライスと配列への参照。
- selfが関数の場合は`"fn"`、`"unsafe fn"`、または`"#[target_feature] fn"`。
- 型が数値だがまだ推論していない場合は`"{integer}"`と`"{float}"`。
- 上記の組み合わせ。例えば、`"[{integral}; _]"`。

例えば、`Iterator`トレイトは次の方法でフィルタリングできます：

```rust,ignore
#[rustc_on_unimplemented(
    on(Self = "&str", note = "call `.chars()` or `.as_bytes()` on `{Self}`"),
    message = "`{Self}` is not an iterator",
    label = "`{Self}` is not an iterator",
    note = "maybe try calling `.iter()` or a similar method"
)]
pub trait Iterator {}
```

これにより、次の出力が生成されます：

```text
error[E0277]: `Foo` is not an iterator
 --> src/main.rs:4:16
  |
4 |     for foo in Foo {}
  |                ^^^ `Foo` is not an iterator
  |
  = note: maybe try calling `.iter()` or a similar method
  = help: the trait `std::iter::Iterator` is not implemented for `Foo`
  = note: required by `std::iter::IntoIterator::into_iter`

error[E0277]: `&str` is not an iterator
 --> src/main.rs:5:16
  |
5 |     for foo in "" {}
  |                ^^ `&str` is not an iterator
  |
  = note: call `.chars()` or `.bytes() on `&str`
  = help: the trait `std::iter::Iterator` is not implemented for `&str`
  = note: required by `std::iter::IntoIterator::into_iter`
```

`on`フィルターは、`cfg`属性と同様に、`all`、`any`、`not`述語を受け入れます：

```rust,ignore
#[rustc_on_unimplemented(on(
    all(Self = "&str", T = "alloc::string::String"),
    note = "you can coerce a `{T}` into a `{Self}` by writing `&*variable`"
))]
pub trait From<T>: Sized {
    /* ... */
}
```

### フォーマット

文字列リテラルは、中括弧で囲まれたパラメーターを受け入れる形式文字列ですが、
位置パラメーターとリストパラメーターおよび形式指定子は受け入れられません。
次のパラメーター名が有効です：

- `Self`とトレイトのすべてのジェネリックパラメーター。
- `This`：属性が付いているトレイトの名前（ジェネリックなし）。
- `Trait`：「糖化された」トレイトの名前。`TraitRefPrintSugared`を参照してください。
- `ItemContext`：私たちがいる`hir::Node`の種類。`"an async block"`、
   `"a function"`、`"an async function"`などのようなもの。

次のようなもの：

```rust,ignore
#![feature(rustc_attrs)]

#[rustc_on_unimplemented(message = "Self = `{Self}`, \
    T = `{T}`, this = `{This}`, trait = `{Trait}`, \
    context = `{ItemContext}`")]
pub trait From<T>: Sized {
    fn from(x: T) -> Self;
}

fn main() {
    let x: i8 = From::from(42_i32);
}
```

メッセージを次のようにフォーマットします

```text
"Self = `i8`, T = `i32`, this = `From`, trait = `From<i32>`, context = `a function`"
```
