# エディション

本章では、rustcにおけるエディションサポートの仕組みについて概要を説明します。
ここでは、エディションとは何か（[エディションガイド]を参照）について理解していることを前提としています。

[エディションガイド]: https://doc.rust-lang.org/edition-guide/

## エディションの定義

`--edition` CLIフラグは、クレートに使用するエディションを指定します。
これは[`Session::edition`]からアクセスできます。
クレートのエディションをチェックするための[`Session::at_least_rust_2021`]のような便利な関数がありますが、グローバルセッションをチェックするかスパンをチェックするかについて注意する必要があります。以下の[エディションハイジーン]を参照してください。

`at_least_rust_20xx`便利メソッドの代わりに、[`Edition`]型は範囲チェックを行うための比較もサポートしています。例えば、`span.edition() >= Edition::Edition2021`のようになります。

[`Session::edition`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/struct.Session.html#method.edition
[`Session::at_least_rust_2021`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/struct.Session.html#method.at_least_rust_2021
[`Edition`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/edition/enum.Edition.html

### 新しいエディションの追加

新しいエディションの追加は、主に[`Edition`]列挙型にバリアントを追加してから、壊れたすべてのものを修正することに関わります。例として[#94461](https://github.com/rust-lang/rust/pull/94461)を参照してください。

### 機能とエディションの安定性

[`Edition`]列挙型は、エディションが安定しているかどうかを定義します。
安定していない場合、それを有効にするには`-Zunstable-options` CLIオプションを渡す必要があります。

新しい機能を追加する際、将来のエディションで安定性を扱う方法として2つのオプションを選択できます：

- `span.at_least_rust_20xx()`のようにスパンのエディションをチェックするだけです（[エディションハイジーン]を参照）、または[`Session::edition`]を使用します。これは、機能が利用可能であることを示すために、エディション自体の安定性に暗黙的に依存します。
- 新しい動作を[フィーチャーゲート]の後ろに配置します。

比較的単純な変更の場合、現在のエディションのみをチェックするだけで十分な場合があります。
ただし、大きな言語変更の場合は、フィーチャーゲートを作成することを検討してください。
フィーチャーゲートを使用することには、いくつかの利点があります：

- フィーチャーゲートにより、新しい機能の作業と実験が容易になります。
- `#![feature(…)]`属性が使用されたときに、新しい機能が有効になっていることが明確になります。
- エディションのテストが容易になり、まだ完成していない機能が完成して準備ができているエディション固有の機能のテストを妨げないようになります。
- 機能がエディションから切り離されるため、機能の準備ができたときに、チームが次のエディションに機能を追加するかどうかを意図的に決定しやすくなります。

機能が完成して準備ができたら、フィーチャーゲートを削除できます（コードはスパンまたは`Session`エディションをチェックして有効かどうかを判断するだけで済みます）。

機能チェックを行うためのいくつかの異なるオプションがあります：

- 非常に実験的な機能で、エディションに関与する可能性がある、または関与しない可能性がある場合、`tcx.features().my_feature`のような通常のフィーチャーゲートを実装し、当面はエディションを無視することができます。

- エディションに関与する*可能性がある*実験的な機能の場合、`tcx.features().my_feature && span.at_least_rust_20xx()`でゲートを実装する必要があります。
  これには、ユーザーが`#![feature(my_feature)]`を指定する必要があり、準備ができてエディション内で受け入れられた他のエディション機能のテストを妨げないようにします。

- エディションの一部であることが確実に決まった実験的な機能の場合、`tcx.features().my_feature || span.at_least_rust_20xx()`でゲートを実装するか、機能チェックを完全に削除して`span.at_least_rust_20xx()`のみをチェックします。

複数の場所で機能ゲートを行う必要がある場合は、単一の関数にチェックを配置して、更新する場所が1つだけになるようにすることを検討してください。例えば：

```rust,ignore
// Edition 2021のdisjoint closure capturesの例

fn enable_precise_capture(tcx: TyCtxt<'_>, span: Span) -> bool {
    tcx.features().capture_disjoint_fields || span.rust_2021()
}
```

リントと安定性の詳細については、以下の[リントと安定性](#lints-and-stability)を参照してください。

[フィーチャーゲート]: ../feature-gates.md

## エディションのパース

ほとんどの場合、レクサーはエディションに依存しません。
[`Lexer`]内では、トークンをエディション固有の動作に基づいて変更できます。
例えば、`c"foo"`のようなC文字列リテラルは、2021より前のエディションでは複数のトークンに分割されます。
これは、2021エディションの予約済みプレフィックスなどが処理される場所でもあります。

エディション固有のパースは比較的まれです。1つの例は`async fn`で、トークンのスパンをチェックして2015エディションかどうかを判断し、その場合はエラーを出力します。
これは、構文がすでに無効である場合にのみ実行できます。

パーサーでエディションチェックを行う必要がある場合、通常はトークンのエディションを確認する必要があります。[エディションハイジーン]を参照してください。
まれに、[`ParseSess::edition`]からグローバルエディションをチェックする必要がある場合があります。

ほとんどのエディション固有のパース動作は、パーサーではなく[マイグレーションリント]で処理されます。
これは、構文の*変更*がある場合（新しい構文とは対照的に）に適切です。
これにより、古い構文は以前のエディションで引き続き機能します。
リントは、動作の変更をチェックします。
古いエディションでは、リントパスは新しいエディションへの移行を支援するためにマイグレーションリントを出力する必要があります。
新しいエディションでは、コードは代わりに`emit_err`でハードエラーを出力する必要があります。
例えば、非推奨の`start...end`パターン構文は、2021より前のエディションで[`ellipsis_inclusive_range_patterns`]リントを出力し、2021では`emit_err`メソッドを介してハードエラーになります。

[`Lexer`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/lexer/struct.Lexer.html
[`ParseSess::edition`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/parse/struct.ParseSess.html#structfield.edition
[`ellipsis_inclusive_range_patterns`]: https://doc.rust-lang.org/nightly/rustc/lints/listing/warn-by-default.html#ellipsis-inclusive-range-patterns

### キーワード

新しいキーワードは、エディション境界を越えて導入できます。
これは、[`Symbol::is_used_keyword_conditional`]のような関数によって実装され、キーワードの定義方法の順序に依存します。

新しいキーワードが導入されると、[`keyword_idents`]リントを更新して、自動マイグレーションがキーワードを識別子として使用している可能性のあるコードを移行できるようにする必要があります（[`KeywordIdents`]を参照）。
考慮すべき代替案は、使用されている位置がそれを区別するのに十分であれば、キーワードを弱いキーワードとして実装することです。

考慮すべき追加のオプションは、[RFC 3101]で導入された`k#`プレフィックスです。
これにより、キーワードが導入されたエディション*より前の*エディションでキーワードを使用できます。
これは現在実装されていません。

[`Symbol::is_used_keyword_conditional`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/symbol/struct.Symbol.html#method.is_used_keyword_conditional
[`keyword_idents`]: https://doc.rust-lang.org/nightly/rustc/lints/listing/allowed-by-default.html#keyword-idents
[`KeywordIdents`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/builtin/struct.KeywordIdents.html
[RFC 3101]: https://rust-lang.github.io/rfcs/3101-reserved_prefixes.html

### エディションハイジーン

[エディションハイジーン]: #edition-hygiene

スパンは、スパンが来たクレートのエディションでマークされています。
これがユーザーにとって何を意味するかについては、エディションガイドの[マクロハイジーン]を参照してください。

通常、グローバル`Session`エディションを見るのではなく、トークンスパンからエディションを使用する必要があります。
例えば、`sess.at_least_rust_2021()`の代わりに`span.edition().at_least_rust_2021()`を使用します。
これにより、マクロがクレート間で使用される場合に正しく動作するようになります。

[マクロハイジーン]: https://doc.rust-lang.org/nightly/edition-guide/editions/advanced-migrations.html#macro-hygiene

## リント

リントは、エディションと対話するためのいくつかの異なるオプションをサポートしています。
リントは、新しいエディションへの[マイグレーション][マイグレーションリント]をサポートするために使用される*将来互換性のないエディションマイグレーションリント*にすることができます。
あるいは、リントは[エディション固有](#edition-specific-lints)にすることができ、特定のエディションからデフォルトレベルが変更されます。

### マイグレーションリント

[マイグレーションリント]: #migration-lints

*マイグレーションリント*は、プロジェクトをあるエディションから次のエディションに移行するために使用されます。
これらは、`MachineApplicable`[サジェスチョン](../diagnostics.md#suggestions)で実装され、コードを書き換えて**前のエディションと次のエディションの両方で正常にコンパイルされる**ようにします。
例えば、[`keyword_idents`]リントは、新しいキーワードと競合する識別子を取得し、raw識別子構文を使用して競合を回避します（例えば、`async`を`r#async`に変更する）。

マイグレーションリントは、リント宣言で[`FutureIncompatibilityReason::EditionError`]または[`FutureIncompatibilityReason::EditionSemanticsChange`]の[将来互換性のないオプション](../diagnostics.md#future-incompatible-lints)で宣言する必要があります：

```rust,ignore
declare_lint! {
    pub KEYWORD_IDENTS,
    Allow,
    "detects edition keywords being used as an identifier",
    @future_incompatible = FutureIncompatibleInfo {
        reason: FutureIncompatibilityReason::EditionError(Edition::Edition2018),
        reference: "issue #49716 <https://github.com/rust-lang/rust/issues/49716>",
    };
}
```

このように宣言すると、リントは適切な`rust-20xx-compatibility`リントグループに自動的に追加されます。
ユーザーが`cargo fix --edition`を実行すると、cargoは`--force-warn rust-20xx-compatibility`フラグを渡して、エディション移行中にこれらすべてのリントが表示されるようにします。
Cargoは`--cap-lints=allow`も渡すため、他のリントがエディション移行を妨げることはありません。

サンプルコードが正しいエディションを設定していることを確認してください。サンプルは前のエディションを示し、マイグレーション警告がどのように見えるかを示す必要があります。例えば、2024マイグレーション用のこのリントは、2021のサンプルを示しています：

```rust,ignore
declare_lint! {
    /// The `keyword_idents_2024` lint detects ...
    ///
    /// ### Example
    ///
    /// ```rust,edition2021
    /// #![warn(keyword_idents_2024)]
    /// fn gen() {}
    /// ```
    ///
    /// {{produces}}
}
```

マイグレーションリントは、デフォルトで`Allow`または`Warn`にすることができます。
`Allow`の場合、ユーザーは通常、エディション移行を手動で行っている場合や移行中に問題がある場合を除き、この警告を表示しません。
ほとんどのマイグレーションリントは`Allow`です。

デフォルトで`Warn`の場合、すべてのエディションのユーザーにこの警告が表示されます。
変更を皆に認識してもらうことが重要であり、すべてのエディションでコードを更新するよう人々を促したい場合にのみ、`Warn`を使用してください。
多くのプロジェクトに影響を与える新しい警告がデフォルトで警告になると、非常に破壊的で不満を招く可能性があることに注意してください。
エディションが安定してから数年後に、`Allow`を`Warn`に切り替えることを検討するかもしれません。
これは、新しいエディションに更新していない比較的少数の遅延者にのみ表示されます。

[`FutureIncompatibilityReason::EditionError`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint_defs/enum.FutureIncompatibilityReason.html#variant.EditionError
[`FutureIncompatibilityReason::EditionSemanticsChange`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint_defs/enum.FutureIncompatibilityReason.html#variant.EditionSemanticsChange

### エディション固有のリント

リントは、特定のエディションから異なるレベルになるようにマークできます。
リント宣言では、`@edition`マーカーを使用します：

```rust,ignore
declare_lint! {
    pub SOME_LINT_NAME,
    Allow,
    "my lint description",
    @edition Edition2024 => Warn;
}
```

ここでは、`SOME_LINT_NAME`は2024より前のすべてのエディションで`Allow`をデフォルトとし、その後`Warn`になります。

これは一般的に控えめに使用する必要があります。他のオプションがあるためです：

- エディションに関係のない小さな影響のスタイル変更は、すべてのエディションでリントを`Warn`にするだけで済みます。人々に物事を書く別の方法を採用してもらいたい場合は、すべてのプロジェクトに表示されるようにコミットしてください。

  多くのプロジェクトに影響を与える新しい警告がデフォルトで警告になると、非常に破壊的で不満を招く可能性があることに注意してください。

- 新しいスタイルを新しいエディションでハードエラーに変更し、[マイグレーションリント]を使用してプロジェクトを新しいスタイルに自動的に変換します。例えば、[`ellipsis_inclusive_range_patterns`]は2021でハードエラーになり、以前のすべてのエディションで警告します。

  これらは、エディションが安定した後に追加することはできないことに注意してください。

- マイグレーションリントも時間とともに変更できます。
  例えば、マイグレーションリントはデフォルトで`Allow`として開始できます。
  移行を実行する人々の場合、自動的に新しいコードに更新されます。
  その後、数年後、リントは以前のエディションで`Warn`にすることができます。

  例えば、[`anonymous_parameters`]は2018エディションのマイグレーションリント（2018ではハードエラー）で、以前のエディションではデフォルトで`Allow`でした。
  その後、3年後、以前のすべてのエディションで`Warn`に変更され、すべてのユーザーにスタイルが段階的に廃止されているという警告が表示されるようになりました。
  最初から警告だった場合、多くのプロジェクトに影響を与え、非常に破壊的だったでしょう。
  エディションの一部にすることで、ほとんどのユーザーは最終的に新しいエディションに更新し、移行によって処理されました。
  `Warn`に切り替えることは、更新しなかった少数の遅延者にのみ影響を与えました。

[`anonymous_parameters`]: https://doc.rust-lang.org/nightly/rustc/lints/listing/warn-by-default.html#anonymous-parameters

### リントと安定性

リントは不安定としてマークできます。これは、新しいエディション機能を開発していて、マイグレーションリントをテストしたい場合に役立ちます。
フィーチャーゲートは、次のようにリントの宣言で指定できます：

```rust,ignore
declare_lint! {
    pub SOME_LINT_NAME,
    Allow,
    "my cool lint",
    @feature_gate = sym::my_feature_name;
}
```

その後、リントはユーザーが適切な`#![feature(my_feature_name)]`を持っている場合にのみ発火します。
移行のテストをするcraterの実行を行う時が来たら、フィーチャーゲートを削除する必要があることに注意してください。

あるいは、フィーチャーゲートなしで今後の不安定なエディションに対してallow-by-default[マイグレーションリント]を実装できます。
技術的には、ユーザーはエディションが安定する前にリントを有効にできるかもしれませんが、ほとんどの人は新しいリントが存在することに気づかず、何も中断したり破壊したりすることはありません。

### イディオムリント

2018エディションでは、`rust-2018-idioms`リントグループの下に「イディオムリント」の概念がありました。
この概念は、`rust-2018-compatibility`リントグループの下の強制マイグレーションとは別の異なるリントグループの下に新しい慣用的なスタイルを持つことで、人々が特定のエディション変更にオプトインする方法について柔軟性を与えることでした。

全体として、このアプローチはあまりうまく機能せず、将来イディオムグループを使用する可能性は低いです。

## 標準ライブラリの変更

### プレリュード

各エディションには、標準ライブラリの特定のプレリュードが付属しています。
これらは、[`core::prelude`]および[`std::prelude`]の通常のモジュールとして実装されています。
プレリュードに新しいアイテムを追加できますが、これはユーザーの既存のコードと競合する可能性があることに注意してください。
通常、競合を回避するために既存のコードを移行するには、[マイグレーションリント]を使用する必要があります。
例えば、[`rust_2021_prelude_collisions`]は、2021の新しいトレイトとの競合を処理するために使用されます。

[`core::prelude`]: https://doc.rust-lang.org/core/prelude/index.html
[`std::prelude`]: https://doc.rust-lang.org/std/prelude/index.html
[`rust_2021_prelude_collisions`]: https://doc.rust-lang.org/nightly/rustc/lints/listing/allowed-by-default.html#rust-2021-prelude-collisions

### カスタマイズされた言語動作

通常、標準ライブラリに破壊的な変更を加えることはできません。
まれに、チームは動作の変更がこのルールを破るのに十分重要であると決定する場合があります。
欠点は、古いシグネチャまたは動作と新しいシグネチャまたは動作をいつ使用するかを区別できるように、コンパイラで特別な処理が必要になることです。

1つの例は、配列の[`into_iter()`][into-iter]のメソッド解決の変更です。
これは、`IntoIterator`トレイトの`#[rustc_skip_array_during_method_dispatch]`属性で実装され、エディションに基づいてコンパイラに代替トレイト解決の選択肢を検討するよう指示します。

別の例は、[`panic!`マクロの変更][panic-macro]です。
これには、複数のpanicマクロを定義し、組み込みのpanicマクロ実装が適切な展開方法を決定する必要がありました。
これには、panicマクロの使用を検出するために`rustc_diagnostic_item`属性が必要な、[`non_fmt_panics`][マイグレーションリント]も含まれていました。

一般的に、非常に価値の高い状況を除いて、これらの特殊なケースは避けることをお勧めします。

[into-iter]: https://doc.rust-lang.org/nightly/edition-guide/rust-2021/IntoIterator-for-arrays.html
[panic-macro]: https://doc.rust-lang.org/nightly/edition-guide/rust-2021/panic-macro-consistency.html

### 標準ライブラリエディションの移行

標準ライブラリ自体のエディションを更新するには、おおよそ次のプロセスが含まれます：

- 新しく安定したエディションがbetaに到達し、ブートストラップコンパイラが更新されるまで待ちます。
- マイグレーションリントを適用します。これは、一部のコードが外部サブモジュール[^std-submodules]にあり、標準ライブラリが条件付きコンパイルを多用しているため、複雑なプロセスになる可能性があります。また、標準ライブラリ自体で`cargo fix --edition`を実行することは実用的ではない場合があります。1つのアプローチは、各クレートの先頭に各リントごとに`#![warn(...)]`を個別に追加し、`./x check library`を実行し、マイグレーションを適用し、`#![warn(...)]`を削除して、各マイグレーションを個別にコミットすることです。完全なカバレッジを得るには、多くの異なるターゲットで`./x check`を`--target`とともに実行する必要があります（そうしないと、CIが通過するまでに数日または数週間かかる可能性があります）[^ed-docker]。その他のヒントについては、[上級マイグレーションガイド]も参照してください。
  - [`backtrace-rs`]にマイグレーションを適用します。[2024の例](https://github.com/rust-lang/backtrace-rs/pull/700)。これは、crates.ioで独立して公開されているため、クレート自体のエディションは更新せず、そうしないと最小Rustバージョンが制限されることに注意してください。エディションが更新されるまで、リグレッションを回避するために、いくつかの`#![deny()]`属性を追加することを検討してください。
  - [`stdarch`]にマイグレーションを適用し、エディションとフォーマットを更新します。[2024の例](https://github.com/rust-lang/stdarch/pull/1710)。
  - backtraceとstdarchサブモジュールを更新するPRを投稿し、それらがマージされるまで待ちます。
  - 標準ライブラリクレートにマイグレーションリントを適用し、エディションを更新します。`core`から始めて、一度に1つのクレートで作業することをお勧めします。[2024の例](https://github.com/rust-lang/rust/pull/138162)。

[^std-submodules]: これは将来的に変更され、これらのサブモジュールを`rust-lang/rust`に取り込むことが期待されます。
[^ed-docker]: また、さまざまなターゲットに対して多くのテストを行う必要があり、ここで[dockerテスト](../tests/docker.md)が役立ちます。

[上級マイグレーションガイド]: https://doc.rust-lang.org/nightly/edition-guide/editions/advanced-migrations.html
[`backtrace-rs`]: https://github.com/rust-lang/backtrace-rs/
[`stdarch`]: https://github.com/rust-lang/stdarch/

## エディションの安定化

エディションチームが承認を与えた後、エディションを安定化するプロセスはおおよそ次のとおりです：

- [`LATEST_STABLE_EDITION`]を更新します。
- [`Edition::is_stable`]を更新します。
- エディションを番号で参照するドキュメントを探して更新します：
  - [`--edition`フラグ](https://github.com/rust-lang/rust/blob/HEAD/src/doc/rustc/src/command-line-arguments.md#--edition-specify-the-edition-to-use)
  - [Rustdoc属性](https://github.com/rust-lang/rust/blob/HEAD/src/doc/rustdoc/src/write-documentation/documentation-tests.md#attributes)
- `//@ edition`ヘッダーを使用するテストをクリーンアップして、`-Zunstable-options`フラグを削除し、実際に安定していることを確認します。注：これは理想的には自動化すべきです。[#133582]を参照してください。
- 変更されるテストをblessします。
- `lint-docs`を新しいエディションにデフォルト設定するように更新します。

[2024の例](https://github.com/rust-lang/rust/pull/133349)を参照してください。

[`LATEST_STABLE_EDITION`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/edition/constant.LATEST_STABLE_EDITION.html
[`Edition::is_stable`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/edition/enum.Edition.html#method.is_stable
[#133582]: https://github.com/rust-lang/rust/issues/133582
