# `ty` モジュール：型の表現

`ty` モジュールは、Rust コンパイラが型を内部的にどのように表現するかを定義します。また、コンパイラの中心的なデータ構造である *typing context*（`tcx` または `TyCtxt`）も定義します。

## `ty::Ty`

rustc が型をどのように表現するかについて話すとき、通常は `Ty` と呼ばれる型を指します。コンパイラには `Ty` に関するかなり多くのモジュールと型があります（[Ty ドキュメント][ty]）。

[ty]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/index.html

ここで指している特定の `Ty` は [`rustc_middle::ty::Ty`][ty_ty] であり、[`rustc_hir::Ty`][hir_ty] ではありません。この区別は重要なので、`ty::Ty` の詳細に入る前にまずこれについて議論します。

[ty_ty]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html
[hir_ty]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Ty.html

## `rustc_hir::Ty` と `ty::Ty` の比較

rustc の HIR は高レベル中間表現と考えることができます。これは AST（[この章](hir.md)を参照）とほぼ同じで、ユーザーが書いた構文を表しており、パースといくつかの *脱糖* の後に得られます。型の表現を持っていますが、実際にはユーザーが書いたもの、つまり、その型を表現するために書いたものをより反映しています。

対照的に、`ty::Ty` は型の意味論、つまりユーザーが書いたものの *意味* を表します。例えば、`rustc_hir::Ty` はユーザーがプログラムで `u32` という名前を 2 回使ったという事実を記録しますが、`ty::Ty` は両方の使用が同じ型を参照しているという事実を記録します。

**例：`fn foo(x: u32) → u32 { x }`**

この関数では、`u32` が 2 回出現しているのがわかります。それが同じ型であることは知っています。つまり、関数は引数を受け取り、同じ型の引数を返します。しかし、HIR の観点からは、プログラムの異なる場所に出現しているため、2 つの異なる型インスタンスがあることになります。つまり、2 つの異なる [`Span`s][span]（位置）を持っています。

[span]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/struct.Span.html

**例：`fn foo(x: &u32) -> &u32`**

さらに、HIR には省略された情報があるかもしれません。この型 `&u32` は不完全です。なぜなら、完全な Rust 型には実際にはライフタイムがありますが、それらのライフタイムを書く必要がなかったからです。また、情報を挿入する省略規則もあります。結果は `fn foo<'a>(x: &'a u32) -> &'a u32` のようになるかもしれません。

HIR レベルでは、これらのことは綴られておらず、全体像はかなり不完全であると言えます。しかし、`ty::Ty` レベルでは、これらの詳細が追加され、完全になります。さらに、`u32` のような特定の型に対しては正確に 1 つの `ty::Ty` を持ち、その `ty::Ty` は `rustc_hir::Ty` のような特定の使用ではなく、プログラム全体のすべての `u32` に対して使用されます。

要約は次のとおりです：

| [`rustc_hir::Ty`][hir_ty] | [`ty::Ty`][ty_ty] |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 型の *構文* を記述：ユーザーが書いたもの（いくつかの脱糖を伴う）。  | 型の *意味論* を記述：ユーザーが書いたものの意味。 |
| 各 `rustc_hir::Ty` は、プログラムの適切な場所に対応する独自のスパンを持ちます。 | ユーザーのプログラムの単一の場所に対応しません。 |
| `rustc_hir::Ty` にはジェネリクスとライフタイムがありますが、それらのライフタイムの一部は [`LifetimeKind::Implicit`][implicit] のような特別なマーカーです。 | `ty::Ty` は、ユーザーが省略した場合でも、ジェネリクスとライフタイムを含む完全な型を持ちます |
| `fn foo(x: u32) -> u32 { }` - `u32` の各使用を表す 2 つの `rustc_hir::Ty`、それぞれが独自の `Span` を持ち、`rustc_hir::Ty` は両方が同じ型であることを教えてくれません | `fn foo(x: u32) -> u32 { }` - プログラム全体の `u32` のすべてのインスタンスに対して 1 つの `ty::Ty`、そして `ty::Ty` は `u32` の両方の使用が同じ型を意味することを教えてくれます。 |
| `fn foo(x: &u32) -> &u32 { }` - 再び 2 つの `rustc_hir::Ty`。参照のライフタイムは、[`LifetimeKind::Implicit`][implicit] という特別なマーカーを使用して `rustc_hir::Ty` に表示されます。 | `fn foo(x: &u32) -> &u32 { }`- 単一の `ty::Ty`。`ty::Ty` には隠されたライフタイムパラメータがあります。 |

[implicit]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/enum.LifetimeKind.html#variant.Implicit

**順序**

HIR は AST から直接構築されるため、`ty::Ty` が生成される前に発生します。HIR が構築された後、基本的な型推論と型チェックが行われます。型推論中に、すべてのものの `ty::Ty` が何であるかを把握し、何かの型があいまいかどうかもチェックします。`ty::Ty` は、すべてが期待される型を持っていることを確認しながら、型チェックに使用されます。[`hir_ty_lowering` モジュール][hir_ty_lowering]は、`rustc_hir::Ty` を `ty::Ty` に下げる責任があるコードが配置されている場所です。使用される主なルーチンは `lower_ty` です。これは型チェックフェーズ中に発生しますが、「この関数はどのような引数型を期待していますか？」のような質問をしたいコンパイラの他の部分でも発生します。

[hir_ty_lowering]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_analysis/hir_ty_lowering/index.html

**意味論が 2 つの `Ty` インスタンスを駆動する方法**

HIR は、最小限のことを仮定する型情報の観点と考えることができます。2 つのものが同じものであることが証明されるまで、異なるものと仮定します。言い換えれば、それらについてより少ないことを知っているので、それらについてより少ないことを仮定すべきです。

それらは構文的に 2 つの文字列です：行 N 列 20 の `"u32"` と行 N 列 35 の `"u32"`。それらがまだ同じであることは知りません。したがって、HIR ではそれらが異なるかのように扱います。後で、それらが意味的に同じ型であることを判断し、それが使用する `ty::Ty` です。

別の例を考えてみましょう：`fn foo<T>(x: T) -> u32`。誰かが `foo::<u32>(0)` を呼び出すとします。これは、`T` と `u32`（この呼び出しで）が実際には同じ型であることが判明することを意味するので、最終的には同じ `ty::Ty` になりますが、異なる `rustc_hir::Ty` があります。（ただし、これは少し単純化されています。型チェック中に、関数を一般的にチェックし、まだ `u32` とは異なる `T` を持ちます。後で、コード生成を行うときは、常に各関数の「単相化」（完全に置換された）バージョンを処理し、したがって `T` が何を表すか（具体的にはそれが `u32` であること）を知ります。）

もう 1 つの例を示します：

```rust
mod a {
    type X = u32;
    pub fn foo(x: X) -> u32 { 22 }
}
mod b {
    type X = i32;
    pub fn foo(x: X) -> i32 { x }
}
```

ここで、型 `X` はコンテキストによって異なります。`rustc_hir::Ty` を見ると、両方の場合で `X` がエイリアスであることがわかります（ただし、名前解決を介して異なるエイリアスにマッピングされます）。しかし、`ty::Ty` シグネチャを見ると、`fn(u32) -> u32` または `fn(i32) -> i32`（型エイリアスが完全に展開された状態）になります。

## `ty::Ty` の実装

[`rustc_middle::ty::Ty`][ty_ty] は実際には [`Interned<WithCachedTypeInfo<TyKind>>`][tykind] のラッパーです。一般的に `Interned` は無視できます。基本的に明示的にアクセスすることはありません。常に `Ty` 内に隠され、`Deref` 実装やメソッドを介してスキップします。`TyKind` は、多くの異なる Rust 型（例：プリミティブ、参照、代数的データ型、ジェネリクス、ライフタイムなど）を表すバリアントを持つ大きな enum です。`WithCachedTypeInfo` には、`flags` や `outer_exclusive_binder` のようないくつかのキャッシュされた値があります。これらは効率のための便利なハックであり、知りたいかもしれない型に関する情報を要約しますが、ここではそれほど重要ではありません。最後に、[`Interned`](./memory.md) により、`ty::Ty` を薄いポインターのような型にすることができます。これにより、等価性の安価な比較を行うことができ、インターンの他の利点もあります。

[tykind]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html

## 型の割り当てと操作

新しい型を割り当てるには、[`Ty`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html) で定義されたさまざまな `new_*` メソッドを使用できます。これらの名前は、主にさまざまな種類の型に対応しています。例：

```rust,ignore
let array_ty = Ty::new_array_with_const_len(tcx, ty, count);
```

これらのメソッドはすべて `Ty<'tcx>` を返します – 返されるライフタイムは、この `tcx` がアクセスできるアリーナのライフタイムであることに注意してください。型は常に正規化されインターンされます（したがって、まったく同じ型を 2 回割り当てることはありません）。

また、`tcx` 自体のフィールドにアクセスすることで、さまざまな一般的な型を見つけることもできます：`tcx.types.bool`、`tcx.types.char` など。（詳細については [`CommonTypes`] を参照してください。）

[`CommonTypes`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.CommonTypes.html

<!-- N.B: This section is linked from the type comparison internal lint. -->
## 型の比較

型はインターンされているため、`==` を使用して効率的に等価性を比較することができます – ただし、これはハッシュして重複を探している場合を除いて、やりたいことではほとんどありません。これは、Rust では特に推論が関与すると、同じ型を表現する複数の方法があるためです。

例えば、型 `{integer}`（`ty::Infer(ty::IntVar(..))`、整数リテラル `0` のような整数推論変数）と `u8`（`ty::UInt(..)`）は、互いに割り当て可能かどうかをテストするときに等しいものとして扱われることがよくあります（これは診断コードでは一般的な操作です）。しかし、それらに対する `==` は `false` を返します。なぜなら、それらは異なる型だからです。

2 つの型を正しく比較する最も簡単な方法には、推論コンテキスト（`infcx`）が必要です。それがある場合は、`infcx.can_eq(param_env, ty1, ty2)` を使用して、型を等しくできるかどうかを確認できます。これは通常、診断中にチェックしたいことであり、2 つの型が互いに割り当て可能かどうかなどの質問に関係しており、コンパイラの型チェックレイヤーで同一に表現されているかどうかではありません。

推論コンテキストで作業するときは、型内の潜在的な推論変数が実際にその推論コンテキストに属していることを確認するように注意する必要があります。すでに推論コンテキストにアクセスできる関数内にいる場合は、そうあるべきです。具体的には、HIR 型チェックまたは MIR 借用チェック中です。

もう 1 つの考慮事項は正規化です。2 つの型は実際には同じかもしれませんが、1 つは関連型の背後にあります。それらを正しく比較するには、最初に型を正規化する必要があります。これは主に HIR 型チェック中および `TyCtxt` クエリからのすべての型（例えば `tcx.type_of()` から）に関する懸念です。

型チェック中に `FnCtxt` または `ObligationCtxt` が利用可能な場合は、型を正規化するためにそれらで `.normalize(ty)` を使用する必要があります。型チェック後、診断コードは `tcx.normalize_erasing_regions(ty)` を使用できます。

`Ty` で `==` を使用しても問題ない場合もあります。これは例えば、遅延リントまたは単相化後の場合です。型チェックが完了しているため、すべての推論変数が解決され、すべてのリージョンが消去されています。これらの場合、推論変数や正規化が問題にならないことがわかっている場合は、リントを `#[allow]` または `#[expect]` することをお勧めします。

診断コードが推論コンテキストにアクセスできない場合は、どこかで利用可能な場合（型チェック中など）、関数呼び出しを介してスレッド化する必要があります。

推論コンテキストがまったく利用できない場合は、[type-inference] で説明されているように作成できます。しかし、これは関係する型（例えば、`tcx.type_of()` のようなクエリから来た場合）が [`fresh_args_for_item`] を使用して新しい推論変数で実際に置換されている場合にのみ有用です。これは、「任意の `T` に対する `Vec<T>` を `Vec<u32>` と統一できますか？」のような質問に答えるために使用できます。

[type-inference]: ./type-inference.md#creating-an-inference-context
[`fresh_args_for_item`]: https://doc.rust-lang.org/beta/nightly-rustc/rustc_infer/infer/struct.InferCtxt.html#method.fresh_substs_for_item

## `ty::TyKind` バリアント

注：`TyKind` は関数型プログラミングの概念である *Kind* ではありません。

コンパイラで `Ty` を操作するときは、型の種類に対してマッチすることが一般的です：

```rust,ignore
fn foo(x: Ty<'tcx>) {
  match x.kind {
    ...
  }
}
```

`kind` フィールドは `TyKind<'tcx>` 型で、これはコンパイラ内のすべての異なる種類の型を定義する enum です。

> 注：型推論中に型の `kind` フィールドを検査することは危険です。推論変数やその他の考慮事項があるか、または型がまだ知られておらず、後で知られるようになる場合があります。

`TyKind` enum には多くのバリアントがあり、その[ドキュメント][tykind]を見ることで確認できます。いくつかのサンプルを示します：

- [**代数的データ型（ADT）**][kindadt] [*代数的データ型*][wikiadt]は `struct`、`enum`、または `union` です。内部的には、`struct`、`enum`、`union` は実際には同じ方法で実装されています：それらはすべて [`ty::TyKind::Adt`][kindadt] です。基本的にはユーザー定義型です。これらについては後で詳しく説明します。
- [**Foreign**][kindforeign] `extern type T` に対応します。
- [**Str**][kindstr] 型 str です。ユーザーが `&str` と書いたとき、`Str` はその型の `str` 部分を表現する方法です。
- [**Slice**][kindslice] `[T]` に対応します。
- [**Array**][kindarray] `[T; n]` に対応します。
- [**RawPtr**][kindrawptr] `*mut T` または `*const T` に対応します。
- [**Ref**][kindref] `Ref` は安全な参照を表します、`&'a mut T` または `&'a T`。`Ref` には、参照が参照する型である `Ty<'tcx>` のようないくつかの関連部分があります。`Region<'tcx>` は参照のライフタイムまたはリージョンであり、`Mutability` は参照が可変かどうかです。
- [**Param**][kindparam] 型パラメータ（例：`Vec<T>` の `T`）を表します。
- [**Error**][kinderr] より良い診断を印刷できるように、どこかで型エラーを表します。これについては後で詳しく説明します。
- [**その他多数**...][kindvars]

[wikiadt]: https://en.wikipedia.org/wiki/Algebraic_data_type
[kindadt]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html#variant.Adt
[kindforeign]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html#variant.Foreign
[kindstr]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html#variant.Str
[kindslice]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html#variant.Slice
[kindarray]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html#variant.Array
[kindrawptr]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html#variant.RawPtr
[kindref]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html#variant.Ref
[kindparam]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html#variant.Param
[kinderr]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html#variant.Error
[kindvars]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/ty_kind/enum.TyKind.html#variants

## インポート規則

厳密なルールはありませんが、`ty` モジュールは次のように使用される傾向があります：

```rust,ignore
use ty::{self, Ty, TyCtxt};
```

特に、非常に一般的であるため、`Ty` および `TyCtxt` 型は直接インポートされます。他の型は、明示的な `ty::` プレフィックスで参照されることがよくあります（例：`ty::TraitRef<'tcx>`）。ただし、一部のモジュールは、より大きなまたはより小さな名前のセットを明示的にインポートすることを選択します。

## 型エラー

ユーザーが型エラーを起こしたときに生成される `TyKind::Error` があります。この型を伝播し、それに起因する他のエラーを抑制して、カスケードするコンパイラエラーメッセージでユーザーを圧倒しないようにするというアイデアです。

`TyKind::Error` には **重要な不変条件** があります。コンパイラは、エラーがユーザーに **既に報告されている** ことを **知っている** 場合を除き、`Error` を生成すべきではありません。これは通常、(a) そこで報告したばかりか、(b) 既存の Error 型を伝播している（その場合、その Error 型が生成されたときにエラーが報告されているはずです）ためです。

この不変条件を維持することが重要です。なぜなら、`Error` 型の全体的なポイントは、他のエラーを抑制することだからです – つまり、それらを報告しません。実際にユーザーにエラーを発行せずに `Error` 型を生成すると、後のエラーが抑制される可能性があり、コンパイルが誤って成功する可能性があります！

時には 3 番目のケースがあります。エラーが報告されたと信じていますが、コンパイルの早い段階で報告されたと信じており、ローカルではありません。その場合、[`delayed_bug`] または [`span_delayed_bug`] で「遅延バグ」を作成できます。これにより、コンパイルがエラーを生成することを期待しているというメモが作成されます – ただし、コンパイルが成功する場合は、コンパイラバグレポートがトリガーされます。

[`delayed_bug`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.DiagCtxt.html#method.delayed_bug
[`span_delayed_bug`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.DiagCtxt.html#method.span_delayed_bug

安全性を高めるため、[`rustc_middle::ty`][ty] の外部で `TyKind::Error` 値を生成することは実際には不可能です。`TyKind::Error` には、他の場所で構築可能にするのを防ぐプライベートメンバーがあります。代わりに、[`Ty::new_error`][terr] または [`Ty::new_error_with_message`][terrmsg] メソッドを使用する必要があります。これらのメソッドは、`ErrorGuaranteed` を受け取るか、`Error` 種類のインターンされた `Ty` を返す前に `span_delayed_bug` を呼び出します。すでに [`span_delayed_bug`] を使用する予定だった場合は、代わりにスパンとメッセージを [`ty_error_with_message`][terrmsg] に渡して、冗長な遅延バグを回避できます。

[terr]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html#method.new_error
[terrmsg]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html#method.new_error_with_message

## `TyKind` バリアントの簡略構文

`Ty` のデバッグ出力を見ているとき、または単にコンパイラ内の異なる型について話しているときに、有効な Rust ではないが、型に関する内部情報を簡潔に表現するために使用される構文に遭遇することがあります。以下は、さまざまな構文が実際に何を意味するかを示すクイックリファレンスチートシートです。これらはより詳しく後の章でカバーされるべきです。

- ジェネリックパラメータ：`{name}/#{index}` 例：`T/#0`、ここで `index` はジェネリックパラメータのリストでの位置に対応します
- 推論変数：`?{id}` 例：`?x`/`?0`、ここで `id` は推論変数を識別します
- バインダーからの変数：`^{binder}_{index}` 例：`^0_x`/`^0_2`、ここで `binder` と `index` はどのバインダーからどの変数が参照されているかを識別します
- プレースホルダー：`!{id}` または `!{id}_{universe}` 例：`!x`/`!0`/`!x_2`/`!0_2`、指定された宇宙内の一意の型を表します。宇宙が `0` の場合、しばしば省略されます
