# 診断およびサブ診断構造体

rustcには、診断を作成するために使用できる3つの診断トレイトがあります：
`Diagnostic`、`LintDiagnostic`、および`Subdiagnostic`。

シンプルな診断の場合、派生実装を使用できます。例えば、`#[derive(Diagnostic)]`。
これらは、追加のサブ診断を追加するかどうかを決定するために多くのロジックを必要としない
シンプルな診断にのみ適しています。

診断がより複雑または動的な動作を必要とする場合、例えば条件付きでサブ診断を追加する、
レンダリングロジックをカスタマイズする、または実行時にメッセージを選択する場合、
対応するトレイト（`Diagnostic`、`LintDiagnostic`、または`Subdiagnostic`）を手動で実装する必要があります。
このアプローチは、より大きな柔軟性を提供し、シンプルで静的な構造を超える診断に推奨されます。

診断は異なる言語に翻訳でき、各診断には診断を一意に識別するスラッグがあります。

## `#[derive(Diagnostic)]`および`#[derive(LintDiagnostic)]`

以下に示す「フィールド already declared」診断の[定義][defn]を考えてみましょう：

```rust,ignore
#[derive(Diagnostic)]
#[diag(hir_analysis_field_already_declared, code = E0124)]
pub struct FieldAlreadyDeclared {
    pub field_name: Ident,
    #[primary_span]
    #[label]
    pub span: Span,
    #[label(previous_decl_label)]
    pub prev_span: Span,
}
```

`Diagnostic`は構造体と列挙型にのみ派生できます。
構造体に配置される属性は、列挙型の各バリアントに配置されます
（またはその逆）。各`Diagnostic`には、
構造体または各列挙型バリアントに適用される`#[diag(...)]`属性が1つ必要です。

エラーにエラーコード（例：「E0624」）がある場合、それは`code`サブ属性を使用して
指定できます。`code`の指定は必須ではありませんが、
`Diag`を使用する診断を`Diagnostic`を使用するように移植する場合、
コードがあった場合は保持する必要があります。

`#[diag(..)]`は、最初の位置引数としてスラッグ（`rustc_errors::fluent::*`内のアイテムへのパス）を
提供する必要があります。スラッグは診断を一意に識別し、
コンパイラがどのエラーメッセージを出力するかを知る方法でもあります（コンパイラのデフォルトロケール、
またはユーザーが要求したロケールで）。翻訳可能なエラーメッセージの書き方とスラッグアイテムの
生成方法の詳細については、[translation documentation](./translation.md)を参照してください。

この例では、「フィールド already declared」診断のFluentメッセージは
次のようになります：

```fluent
hir_analysis_field_already_declared =
    field `{$field_name}` is already declared
    .label = field already declared
    .previous_decl_label = `{$field_name}` first declared here
```

`hir_analysis_field_already_declared`は例のスラッグであり、
診断メッセージが続きます。

アノテーションのない`Diagnostic`のすべてのフィールドは、
Fluentメッセージで変数として利用できます。例えば、上記の例の`field_name`のように。
これが望ましくない場合、フィールドに`#[skip_arg]`のアノテーションを付けることができます。

フィールドに`#[primary_span]`属性を使用すると（型が`Span`の場合）、
診断のプライマリスパンを示し、診断のメインメッセージが表示されます。

診断は、プライマリメッセージだけでなく、多くの場合、
ラベル、ノート、ヘルプメッセージ、提案を含みます。これらはすべて
`Diagnostic`で指定できます。

`#[label]`、`#[help]`、`#[warning]`、`#[note]`はすべて、型が`Span`のフィールドに適用できます。
これらの属性のいずれかを適用すると、その`Span`で対応する
サブ診断が作成されます。これらの属性は、プライマリFluentメッセージに添付された
Fluent属性で診断メッセージを探します。この例では、`#[label]`は
`hir_analysis_field_already_declared.label`を探します（メッセージは「field already declared」です）。
同じタイプのサブ診断が複数ある場合、これらの属性は、
探す属性名である値を取ることもできます（例えば、この例の`previous_decl_label`）。

他の型は、`Diagnostic`派生で使用されるときに特別な動作を持ちます：

- `Option<T>`に適用された属性は、オプションが`Some(..)`の場合にのみ
  サブ診断を出力します。
- `Vec<T>`に適用された属性は、ベクトルの各要素に対して繰り返されます。

`#[help]`、`#[warning]`、`#[note]`は構造体自体にも適用でき、
その場合、フィールドに適用されたときと同じように機能しますが、
サブ診断には`Span`がありません。これらの属性は、`()`型のフィールドにも適用でき、
同じ効果があります。`Option`型と組み合わせると、
オプションの`#[note]`/`#[help]`/`#[warning]`サブ診断を表すために使用できます。

提案は、次の4つのフィールド属性のいずれかを使用して出力できます：

- `#[suggestion(slug, code = "...", applicability = "...")]`
- `#[suggestion_hidden(slug, code = "...", applicability = "...")]`
- `#[suggestion_short(slug, code = "...", applicability = "...")]`
- `#[suggestion_verbose(slug, code = "...", applicability = "...")]`

提案は、`Span`フィールドまたは`(Span, MachineApplicability)`フィールドに
適用する必要があります。他のフィールド属性と同様に、slugは
メッセージを含むFluent属性を指定し、デフォルトで`.suggestion`に相当します。
`code`は、置換として提案されるべきコードを指定し、
フォーマット文字列です（例：`{field_name}`は構造体の`field_name`フィールドの値に置き換えられます）。
Fluent識別子ではありません。`applicability`は、属性で適用可能性を指定するために使用できます。
フィールドの型に`Applicability`が含まれている場合は使用できません。

最終的に、`Diagnostic`派生は、次のような`Diagnostic`の実装を生成します：

```rust,ignore
impl<'a, G: EmissionGuarantee> Diagnostic<'a> for FieldAlreadyDeclared {
    fn into_diag(self, dcx: &'a DiagCtxt, level: Level) -> Diag<'a, G> {
        let mut diag = Diag::new(dcx, level, fluent::hir_analysis_field_already_declared);
        diag.set_span(self.span);
        diag.span_label(
            self.span,
            fluent::hir_analysis_label
        );
        diag.span_label(
            self.prev_span,
            fluent::hir_analysis_previous_decl_label
        );
        diag
    }
}
```

診断を定義したので、それを[使用する][use]方法は？非常に
簡単です。構造体のインスタンスを作成して、
`emit_err`（または`emit_warning`）に渡すだけです：

```rust,ignore
tcx.dcx().emit_err(FieldAlreadyDeclared {
    field_name: f.ident,
    span: f.span,
    prev_span,
});
```

### `#[derive(Diagnostic)]`および`#[derive(LintDiagnostic)]`のリファレンス

`#[derive(Diagnostic)]`および`#[derive(LintDiagnostic)]`は
次の属性をサポートします：

- `#[diag(slug, code = "...")]`
  - _構造体または列挙型バリアントに適用されます。_
  - _必須_
  - 診断に関連付けるテキストとエラーコードを定義します。
  - Slug（_必須_）
    - 診断を一意に識別し、そのFluentメッセージに対応します。
      必須。
    - `rustc_errors::fluent`内のアイテムへのパス。例えば、
      `rustc_errors::fluent::hir_analysis_field_already_declared`
      （属性では`rustc_errors::fluent`は暗黙的なので、
      `hir_analysis_field_already_declared`だけで十分です）。
    - [translation documentation](./translation.md)を参照してください。
  - `code = "..."`（_オプション_）
    - エラーコードを指定します。
- `#[note]`または`#[note(slug)]`（_オプション_）
  - _構造体または`Span`、`Option<()>`、`()`型の構造体フィールドに適用されます。_
  - ノートサブ診断を追加します。
  - 値は、ノートのメッセージのための`rustc_errors::fluent`内のアイテムへのパスです。
    - デフォルトで`.note`に相当します。
  - `Span`フィールドに適用される場合、スパン付きノートを作成します。
- `#[help]`または`#[help(slug)]`（_オプション_）
  - _構造体または`Span`、`Option<()>`、`()`型の構造体フィールドに適用されます。_
  - ヘルプサブ診断を追加します。
  - 値は、ノートのメッセージのための`rustc_errors::fluent`内のアイテムへのパスです。
    - デフォルトで`.help`に相当します。
  - `Span`フィールドに適用される場合、スパン付きヘルプを作成します。
- `#[label]`または`#[label(slug)]`（_オプション_）
  - _`Span`フィールドに適用されます。_
  - ラベルサブ診断を追加します。
  - 値は、ノートのメッセージのための`rustc_errors::fluent`内のアイテムへのパスです。
    - デフォルトで`.label`に相当します。
- `#[warning]`または`#[warning(slug)]`（_オプション_）
  - _構造体または`Span`、`Option<()>`、`()`型の構造体フィールドに適用されます。_
  - 警告サブ診断を追加します。
  - 値は、ノートのメッセージのための`rustc_errors::fluent`内のアイテムへのパスです。
    - デフォルトで`.warn`に相当します。
- `#[suggestion{,_hidden,_short,_verbose}(slug, code = "...", applicability = "...")]`
  （_オプション_）
  - _`(Span, MachineApplicability)`または`Span`フィールドに適用されます。_
  - 提案サブ診断を追加します。
  - Slug（_必須_）
    - `rustc_errors::fluent`内のアイテムへのパス。例えば、
      `rustc_errors::fluent::hir_analysis_field_already_declared`
      （属性では`rustc_errors::fluent`は暗黙的なので、
      `hir_analysis_field_already_declared`だけで十分です）。すべてのメッセージの
      Fluent属性は、そのモジュールのトップレベルアイテムとして存在します（したがって、
      `hir_analysis_message.attr`は単に`attr`です）。
    - [translation documentation](./translation.md)を参照してください。
    - デフォルトで`rustc_errors::fluent::_subdiag::suggestion`（または
    - Fluentでは`.suggestion`）。
  - `code = "..."`/`code("...", ...)`（_必須_）
    - 置換として提案されるコードを示す1つまたは複数のフォーマット文字列。
      複数の値は、複数の可能な置換を意味します。
  - `applicability = "..."`（_オプション_）
    - `machine-applicable`、`maybe-incorrect`、
      `has-placeholders`、または`unspecified`のいずれかである必要がある文字列。
- `#[subdiagnostic]`
  - _`Subdiagnostic`を実装する型に適用されます（`#[derive(Subdiagnostic)]`から）。_
  - サブ診断構造体によって表されるサブ診断を追加します。
- `#[primary_span]`（_オプション_）
  - _`Subdiagnostic`の`Span`フィールドに適用されます。`LintDiagnostic`には使用されません。_
  - 診断のプライマリスパンを示します。
- `#[skip_arg]`（_オプション_）
  - _任意のフィールドに適用されます。_
  - フィールドが診断引数として提供されるのを防ぎます。

## `#[derive(Subdiagnostic)]`

コンパイラでは、適用可能な場合に特定のサブ診断をエラーに条件付きで追加する
関数を書くことが一般的です。多くの場合、これらのサブ診断は、
全体的な診断ができなくても、診断構造体を使用して表すことができます。
この状況では、`Subdiagnostic`派生を使用して、部分的な診断（例：ノート、ラベル、ヘルプ、
または提案）を構造体として表すことができます。

以下に示す「expected return type」ラベルの[定義][subdiag_defn]を考えてみましょう：

```rust
#[derive(Subdiagnostic)]
pub enum ExpectedReturnTypeLabel<'tcx> {
    #[label(hir_analysis_expected_default_return_type)]
    Unit {
        #[primary_span]
        span: Span,
    },
    #[label(hir_analysis_expected_return_type)]
    Other {
        #[primary_span]
        span: Span,
        expected: Ty<'tcx>,
    },
}
```

`Diagnostic`と同様に、`Subdiagnostic`は構造体または列挙型に派生できます。
構造体に配置される属性は、列挙型の各バリアントに配置されます
（またはその逆）。各`Subdiagnostic`には、
構造体または各バリアントに適用される次のいずれかの属性が必要です：

- `#[label(..)]`ラベルを定義するため
- `#[note(..)]`ノートを定義するため
- `#[help(..)]`ヘルプを定義するため
- `#[warning(..)]`警告を定義するため
- `#[suggestion{,_hidden,_short,_verbose}(..)]`提案を定義するため

上記のすべては、最初の位置引数としてスラッグ（`rustc_errors::fluent::*`内のアイテムへのパス）を
提供する必要があります。スラッグは診断を一意に識別し、
コンパイラがどのエラーメッセージを出力するかを知る方法でもあります（コンパイラのデフォルトロケール、
またはユーザーが要求したロケールで）。翻訳可能なエラーメッセージの書き方とスラッグアイテムの
生成方法の詳細については、[translation documentation](./translation.md)を参照してください。

この例では、「expected return type」ラベルのFluentメッセージは
次のようになります：

```fluent
hir_analysis_expected_default_return_type = expected `()` because of default return type

hir_analysis_expected_return_type = expected `{$expected}` because of return type
```

フィールドに`#[primary_span]`属性を使用すると（型が`Span`の場合）、
サブ診断のプライマリスパンを示します。プライマリスパンは、
ラベルまたは提案に必要であり、スパンレスにはできません。

アノテーションのない型/バリアントのすべてのフィールドは、
Fluentメッセージで変数として利用できます。これが望ましくない場合、
フィールドに`#[skip_arg]`のアノテーションを付けることができます。

`Diagnostic`と同様に、`Subdiagnostic`は`Option<T>`および
`Vec<T>`フィールドをサポートします。

提案は、型/バリアントで次の4つの属性のいずれかを使用して出力できます：

- `#[suggestion(..., code = "...", applicability = "...")]`
- `#[suggestion_hidden(..., code = "...", applicability = "...")]`
- `#[suggestion_short(..., code = "...", applicability = "...")]`
- `#[suggestion_verbose(..., code = "...", applicability = "...")]`

提案には、フィールドに`#[primary_span]`を設定する必要があり、
次のサブ属性を持つことができます：

- 最初の位置引数は、メッセージを含むFluent属性に対応する
  `rustc_errors::fluent`内のアイテムへのパスを指定し、
  デフォルトで`.suggestion`に相当します。
- `code`は、置換として提案されるべきコードを指定し、
  フォーマット文字列です（例：`{field_name}`は構造体の`field_name`フィールドの値に置き換えられます）。
  Fluent識別子ではありません。
- `applicability`は、属性で適用可能性を指定するために使用できます。
  フィールドの型に`Applicability`が含まれている場合は使用できません。

適用可能性は、`#[applicability]`属性を使用して
（`Applicability`型の）フィールドとして指定することもできます。

最終的に、`Subdiagnostic`派生は、次のような`Subdiagnostic`の実装を生成します：

```rust
impl<'tcx> Subdiagnostic for ExpectedReturnTypeLabel<'tcx> {
    fn add_to_diag(self, diag: &mut rustc_errors::Diagnostic) {
        use rustc_errors::{Applicability, IntoDiagArg};
        match self {
            ExpectedReturnTypeLabel::Unit { span } => {
                diag.span_label(span, rustc_errors::fluent::hir_analysis_expected_default_return_type)
            }
            ExpectedReturnTypeLabel::Other { span, expected } => {
                diag.set_arg("expected", expected);
                diag.span_label(span, rustc_errors::fluent::hir_analysis_expected_return_type)
            }
        }
    }
}
```

定義されると、サブ診断は、診断の`subdiagnostic`関数に渡すか
（[例][subdiag_use_1]と[例][subdiag_use_2]）、
診断構造体の`#[subdiagnostic]`アノテーション付きフィールドに割り当てることで使用できます。

### 引数の共有と分離

サブ診断は、情報をレンダリングする前に、独自の引数（つまり、構造体内の特定のフィールド）を`Diag`構造体に追加します。
`Diag`構造体はメイン診断からの引数も保存するため、サブ診断はメイン診断からの引数も使用できます。

ただし、`#[derive(Subdiagnostic)]`を実装してサブ診断をメイン診断に追加する場合、
[rust-lang/rust#142724](https://github.com/rust-lang/rust/pull/142724)で導入された次のルールが
引数（つまり、Fluentメッセージで使用される変数）の処理に適用されます：

**サブ診断間の引数の分離**：
サブ診断によって設定された引数は、そのサブ診断のレンダリング中にのみ利用可能です。
サブ診断がレンダリングされた後、導入されたすべての引数はメイン診断から復元されます。
これにより、複数のサブ診断が互いの引数スコープを汚染しないことが保証されます。
例えば、`Vec<Subdiag>`を使用する場合、同じ引数を繰り返し追加します。

**サブとメイン診断間の同じ引数のオーバーライド**：
サブ診断がメイン診断に既に存在する引数と同じ名前の引数を設定する場合、
両方がまったく同じ値でない限り、実行時にエラーを報告します。
これには2つの利点があります：

- メイン診断の引数がサブ診断の属性に表示される柔軟性を保持します。
例えば、サブ診断に`#[suggestion(code = "{new_vis}")]`属性がありますが、`new_vis`はメイン診断構造体のフィールドです。
- メイン診断または他のサブ診断に必要な引数の誤った上書きまたは削除を防ぎます。

これらのルールは、サブ診断によって注入された引数が、名前の衝突が存在する場合でも、独自のレンダリングに厳密にスコープされることを保証します。
メイン診断の引数は、サブ診断ロジックの影響を受けません。
さらに、サブ診断は、必要に応じて同じ名前のメイン診断からの引数にアクセスできます。

### `#[derive(Subdiagnostic)]`のリファレンス

`#[derive(Subdiagnostic)]`は次の属性をサポートします：

- `#[label(slug)]`、`#[help(slug)]`、`#[warning(slug)]`、または`#[note(slug)]`
  - _構造体または列挙型バリアントに適用されます。構造体/列挙型バリアント属性と相互排他的です。_
  - _必須_
  - 型をラベル、ヘルプ、またはノートを表すものとして定義します。
  - Slug（_必須_）
    - 診断を一意に識別し、そのFluentメッセージに対応します。
      必須。
    - `rustc_errors::fluent`内のアイテムへのパス。例えば、
      `rustc_errors::fluent::hir_analysis_field_already_declared`
      （属性では`rustc_errors::fluent`は暗黙的なので、
      `hir_analysis_field_already_declared`だけで十分です）。
    - [translation documentation](./translation.md)を参照してください。
- `#[suggestion{,_hidden,_short,_verbose}(slug, code = "...", applicability = "...")]`
  - _構造体または列挙型バリアントに適用されます。構造体/列挙型バリアント属性と相互排他的です。_
  - _必須_
  - 型を提案を表すものとして定義します。
  - Slug（_必須_）
    - `rustc_errors::fluent`内のアイテムへのパス。例えば、
      `rustc_errors::fluent::hir_analysis_field_already_declared`
      （属性では`rustc_errors::fluent`は暗黙的なので、
      `hir_analysis::field_already_declared`だけで十分です）。すべてのメッセージの
      Fluent属性は、そのモジュールのトップレベルアイテムとして存在します（したがって、
      `hir_analysis_message.attr`は単に`hir_analysis::attr`です）。
    - [translation documentation](./translation.md)を参照してください。
    - デフォルトで`rustc_errors::fluent::_subdiag::suggestion`（または
    - Fluentでは`.suggestion`）。
  - `code = "..."`/`code("...", ...)`（_必須_）
    - 置換として提案されるコードを示す1つまたは複数のフォーマット文字列。
      複数の値は、複数の可能な置換を意味します。
  - `applicability = "..."`（_オプション_）
    - _フィールドの`#[applicability]`と相互排他的です。_
    - 値は提案の適用可能性です。
    - 次のいずれかである必要がある文字列：
      - `machine-applicable`
      - `maybe-incorrect`
      - `has-placeholders`
      - `unspecified`
- `#[multipart_suggestion{,_hidden,_short,_verbose}(slug, applicability = "...")]`
  - _構造体または列挙型バリアントに適用されます。構造体/列挙型バリアント属性と相互排他的です。_
  - _必須_
  - 型をマルチパート提案を表すものとして定義します。
  - Slug（_必須_）：`#[suggestion]`を参照
  - `applicability = "..."`（_オプション_）：`#[suggestion]`を参照
- `#[primary_span]`（ラベルと提案には_必須_；それ以外は_オプション_；マルチパート提案には適用不可）
  - _`Span`フィールドに適用されます。_
  - サブ診断のプライマリスパンを示します。
- `#[suggestion_part(code = "...")]`（_必須_；マルチパート提案にのみ適用可能）
  - _`Span`フィールドに適用されます。_
  - スパンをマルチパート提案の一部として示します。
  - `code = "..."`（_必須_）
    - 値は、置換として提案されるコードを示すフォーマット文字列です。
- `#[applicability]`（_オプション_；（シンプルおよびマルチパート）提案にのみ適用可能）
  - _`Applicability`フィールドに適用されます。_
  - 提案の適用可能性を示します。
- `#[skip_arg]`（_オプション_）
  - _任意のフィールドに適用されます。_
  - フィールドが診断引数として提供されるのを防ぎます。

[defn]: https://github.com/rust-lang/rust/blob/6201eabde85db854c1ebb57624be5ec699246b50/compiler/rustc_hir_analysis/src/errors.rs#L68-L77
[use]: https://github.com/rust-lang/rust/blob/f1112099eba41abadb6f921df7edba70affe92c5/compiler/rustc_hir_analysis/src/collect.rs#L823-L827

[subdiag_defn]: https://github.com/rust-lang/rust/blob/f1112099eba41abadb6f921df7edba70affe92c5/compiler/rustc_hir_analysis/src/errors.rs#L221-L234
[subdiag_use_1]: https://github.com/rust-lang/rust/blob/f1112099eba41abadb6f921df7edba70affe92c5/compiler/rustc_hir_analysis/src/check/fn_ctxt/suggestions.rs#L670-L674
[subdiag_use_2]: https://github.com/rust-lang/rust/blob/f1112099eba41abadb6f921df7edba70affe92c5/compiler/rustc_hir_analysis/src/check/fn_ctxt/suggestions.rs#L704-L707
