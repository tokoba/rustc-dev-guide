# 翻訳

<div class="warning">
rustcの現在の診断翻訳インフラストラクチャ（
<!-- date-check --> 2024年10月現在）は、残念ながらコンパイラ貢献者に
いくつかの摩擦を引き起こしており、現在のインフラストラクチャは、コンパイラ貢献者と
翻訳チームの両方のニーズによりよく対応する再設計を主に待っています。現在アクティブな
再設計提案はありません（
<!-- date-check --> 2024年10月現在）！

ステータスアップデートについては、追跡issue <https://github.com/rust-lang/rust/issues/132181>
を参照してください。

内部リント`untranslatable_diagnostic`と`diagnostic_outside_of_impl`をダウングレードしました。
これらの内部リントは以前、新しいコードが現在の翻訳インフラストラクチャを使用することを
要求していました。ただし、翻訳インフラは、まだ提案されていない再設計を待っているため、
再作業が必要です。そのため、現在の翻訳インフラの使用を義務付けていません。
*したい*場合、またはコードがよりクリーンになる場合はインフラを使用してください。
ただし、より多くの柔軟性が必要な場合は、翻訳インフラをバイパスしてください。
</div>

rustcの診断インフラストラクチャは、[Fluent]を使用して翻訳可能な診断を
サポートしています。

## 翻訳可能な診断の書き方

翻訳可能な診断を書く方法は2つあります：

1. シンプルな診断の場合、診断（またはサブ診断）派生を使用します。
   （「シンプル」診断とは、サブ診断を出力するかどうかを決定するのに多くのロジックを
   必要としないため、診断構造体として表現できるものです。）[診断およびサブ診断構造体
   ドキュメント](./diagnostic-structs.md)を参照してください。
2. 型付き識別子を`Diag` APIと一緒に使用します（`Diagnostic`、`Subdiagnostic`、
   または`LintDiagnostic`実装内で）。

翻訳可能な診断を追加または変更する場合、
翻訳について心配する必要はありません。
元の英語メッセージを更新するだけで十分です。
現在、
翻訳可能な診断を定義する各クレートには、独自のFluentリソースがあります。
これは、`messages.ftl`という名前のファイルで、
クレートのルートにあります
（例：`compiler/rustc_expand/messages.ftl`）。

## Fluent

Fluentは「非対称ローカライゼーション」のアイデアを中心に構築されており、
翻訳の表現力をソース言語（rustcの場合は英語）の文法から切り離すことを目指しています。
翻訳前、rustcの診断は、ユーザーに表示されるメッセージを構築するために
補間に大きく依存していました。補間された文字列は、自然に聞こえる翻訳を書くために、
英語の文字列よりも多い、少ない、または単に異なる補間が必要になる可能性があるため、
翻訳が困難です。これらすべてが、サポートするためにコンパイラのソースコードの
変更を必要とします。

診断メッセージはFluentリソースで定義されます。与えられたロケール（例：`en-US`）の
Fluentリソースの結合セットは、Fluentバンドルとして知られています。

```fluent
typeck_address_of_temporary_taken = cannot take address of a temporary
```

上記の例では、`typeck_address_of_temporary_taken`はFluentメッセージの識別子であり、
英語の診断メッセージに対応します。他のFluentリソースを書くことができ、
これは別の言語のメッセージに対応します。したがって、各診断には少なくとも1つの
Fluentメッセージがあります。

```fluent
typeck_address_of_temporary_taken = cannot take address of a temporary
    .label = temporary value
```

慣例により、サブ診断の診断メッセージは、Fluentメッセージの「属性」として
指定されます（`.<attribute-name>`構文で示される追加の関連メッセージ）。
上記の例では、`label`は`typeck_address_of_temporary_taken`の属性であり、
この診断に追加されたラベルのメッセージに対応します。

診断メッセージは、多くの場合、ユーザーに表示されるメッセージに追加のコンテキストを
補間します。例えば、型や変数の名前など。Fluentメッセージへの追加コンテキストは、
診断への「引数」として提供されます。

```fluent
typeck_struct_expr_non_exhaustive =
    cannot create non-exhaustive {$what} using struct expression
```

上記の例では、Fluentメッセージは、存在することが期待される`what`という
引数を参照しています（引数が診断にどのように提供されるかについては、後で詳しく
説明します）。

Fluentとその構文の他の使用例については、[Fluent]ドキュメントを参照してください。

### メッセージ命名のガイドライン

通常、fluentはメッセージ名内の単語を区切るために`-`を使用します。ただし、
`_`もfluentによって受け入れられます。`_`はRustのユースケースによりよく適合するため、
Rust側の識別子も`_`を使用しているため、rustc内では、単語を区切るために`-`は
許可されず、代わりに`_`が推奨されます。唯一の例外は、
`-passes_see_issue`のようなメッセージ名の先頭の`-`です。

### 翻訳可能なメッセージを書くためのガイドライン

メッセージが異なる言語に翻訳可能であるためには、どの言語でも必要なすべての
情報を診断への引数として提供する必要があります（英語メッセージで必要な情報だけでなく）。

コンパイラチームが、異なる言語に翻訳するのに必要なすべての情報を持つ診断を
書く経験を積むにつれて、このページはより多くのガイダンスで更新されます。
今のところ、[Fluent]ドキュメントには、メッセージを異なるロケールに翻訳する
優れた例と、そのためにコードから提供する必要がある情報があります。

### コンパイル時検証と型付き識別子

rustcの`fluent_messages`マクロは、Fluentリソースのコンパイル時検証を実行し、
診断でFluentメッセージを参照しやすくするコードを生成します。

Fluentリソースのコンパイル時検証は、コンパイラをビルドする際に、
Fluentリソースからの解析エラーを出力し、無効なFluentリソースが
コンパイラでパニックを引き起こすのを防ぎます。コンパイル時検証は、
複数のFluentメッセージが同じ識別子を持つ場合にもエラーを出力します。

## 内部

rustcの診断内部のさまざまな部分は、翻訳をサポートするために変更されています。

### メッセージ

rustcの従来の診断API（例：`struct_span_err`または`note`）はすべて、
`DiagMessage`（または`SubdiagMessage`）に変換できる任意のメッセージを受け取ります。

[`rustc_error_messages::DiagMessage`]は、レガシーの翻訳不可能な
診断メッセージと翻訳可能なメッセージを表すことができます。翻訳不可能なメッセージは
単なる`String`です。翻訳可能なメッセージは、Fluentメッセージの識別子を持つ
`&'static str`です（時には属性名を持つ追加の`&'static str`と一緒に）。

`DiagMessage`は直接対話する必要はありません：
Fluentリソース内の各診断メッセージに対して`DiagMessage`定数が作成されるか、
または診断派生のマクロ生成コードで`DiagMessage`が作成されます。

`rustc_error_messages::SubdiagMessage`も同様で、レガシーの翻訳不可能な
診断メッセージまたはFluentメッセージへの属性の名前に対応できます。翻訳可能な
`SubdiagMessage`は、出力されるために`DiagMessage`（`DiagMessage::with_subdiagnostic_message`を
使用）と結合する必要があります（属性名だけでは、対応するメッセージ識別子がないため、
意味がありません。これが`DiagMessage`が提供するものです）。

`DiagMessage`と`SubdiagMessage`の両方は、文字列に変換できる任意の型に対して
`Into`を実装しており、これらを翻訳不可能な診断に変換します - これにより、
既存のすべての診断呼び出しが機能し続けます。

### 引数

メッセージコンテンツに補間されるFluentメッセージへの追加コンテキストは、
翻訳可能な診断に提供する必要があります。

診断には、この追加のコンテキストを診断に提供するために使用できる`set_arg`関数が
あります。

引数には、名前（例：前の例の「what」）と値の両方があります。
引数の値は、`DiagArgValue`型を使用して表されます。これは単なる文字列または数値です。
rustcの型は、文字列または数値への変換で`IntoDiagArg`を実装でき、
`Ty<'tcx>`のような一般的な型には既にそのような実装があります。

`set_arg`呼び出しは、診断派生によって透過的に処理されますが、
診断ビルダーAPIを使用する場合は手動で追加する必要があります。

### ロード

rustcは、デフォルトで使用され、別のロケールがメッセージを欠いている場合に使用される
`en-US`の「フォールバックバンドル」と、ユーザーが要求したプライマリfluentバンドルを
区別します。

診断エミッターは、フォールバックおよびプライマリfluentバンドルにアクセスするための
2つの関数を持つ`Emitter`トレイトを実装します（それぞれ`fallback_fluent_bundle`と
`fluent_bundle`）。

`Emitter`には、`fallback_fluent_bundle`と`fluent_bundle`の結果を使用して
`DiagMessage`の翻訳を実行するためのデフォルト実装を持つメンバー関数もあります。

rustc内のすべてのエミッターは、フォールバックFluentバンドルを遅延的にロードします。
エラーメッセージが最初に翻訳されるときにのみFluentリソースを読み取り、解析します
（パフォーマンス上の理由 - エラーが出力されない場合、これを行うのは意味がありません）。
`rustc_error_messages::fallback_fluent_bundle`は、エミッターに提供され、
`Emitter::fallback_fluent_bundle`への最初の呼び出しで評価される
`std::lazy::Lazy<FluentBundle>`を返します。

プライマリFluentバンドル（ユーザーが希望するロケール用）は、
`Emitter::fluent_bundle`によって返されることが期待されます。このバンドルは、
メッセージを翻訳する際に優先的に使用されます。プライマリバンドルがメッセージを
欠いているか、提供されていない場合にのみ、フォールバックバンドルが使用されます。

コンパイラと一緒に配布されるロケールバンドルはありませんが、
それらをロードするメカニズムが実装されています。

- `-Ztranslate-additional-ftl`を使用して、テスト目的で特定のリソースを
  プライマリバンドルとしてロードできます。
- `-Ztranslate-lang`には、言語識別子（`en-US`のようなもの）を提供でき、
  `$sysroot/share/locale/$locale/`ディレクトリで見つかったFluentリソースを
  ロードします（ユーザーが提供したsysrootと任意のsysroot候補の両方）。

プライマリバンドルは現在、遅延的にロードされておらず、要求された場合は、
エラーが発生するかどうかに関係なく、コンパイルの開始時にロードされます。
プライマリバンドルを遅延的にロードすることは、バンドルのロードが失敗しないと
仮定できれば可能です。要求されたロケールが欠けている場合、Fluentファイルが
不正な形式である場合、またはメッセージが複数のリソースで重複している場合、
バンドルのロードは失敗する可能性があります。

[Fluent]: https://projectfluent.org
[`compiler/rustc_borrowck/messages.ftl`]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_borrowck/messages.ftl
[`compiler/rustc_parse/messages.ftl`]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_parse/messages.ftl
[`rustc_error_messages::DiagMessage`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_error_messages/enum.DiagMessage.html
