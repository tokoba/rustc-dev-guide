# `Binder` のインスタンス化

[`EarlyBinder`] と同様に、[`Binder`] の内部にアクセスするときは、最初にバウンド変数を他の値に置き換えることによってそれを解除する必要があります。これは `EarlyBinder` とほぼ同じ理由です。`Binder` によって導入されたパラメータを参照する型は、そのバインダーの外では意味をなしません。例：

```rust,ignore
fn foo<'a>(a: &'a u32) -> &'a u32 {
    a
}
fn bar<T>(a: fn(&u32) -> T) -> T {
    a(&10)
}

fn main() {
    let higher_ranked_fn_ptr = foo as for<'a> fn(&'a u32) -> &'a u32;
    let references_bound_vars = bar(higher_ranked_fn_ptr);
}
```

この例では、型 `for<'a> fn(&'^0 u32) -> &'^0 u32` の引数を `bar` に提供しています。`T` が型 `&'^0 u32` に推論されることを許可したくありません。なぜなら、それはかなり無意味だからです（そして、ICE しなかった場合はおそらく健全ではありません。`main` は `'a` が何であるかを知らないので、借用チェッカーはライフタイム `'a` を持つ借用をどのように処理するでしょうか）。

`EarlyBinder` とは異なり、通常、ユーザーからの具体的な引数のセットで `Binder` をインスタンス化することはありません。つまり、`for<'a1, 'a2> fn(&'a1 u32, &'a2 u32)` への引数として `['b, 'static]` を使用します。代わりに、通常、推論変数またはプレースホルダーでバインダーをインスタンス化します。

## 推論変数でのインスタンス化

バインダーの可能なインスタンス化を推論しようとしているときに、推論変数でバインダーをインスタンス化します。例えば、高階関数ポインタを呼び出すか、高階 where 句を使用してバウンドを証明しようとします。例えば、上記の例からの `higher_ranked_fn_ptr` を `&10_u32` で呼び出す場合、次のようにします：

- 推論変数でバインダーをインスタンス化し、シグネチャ `fn(&'?0 u32) -> &'?0 u32)` を生成します
- 提供された引数 `&10_u32`（&'static u32）の型とシグネチャの型 `&'?0 u32` を等しくし、`'?0 = 'static` を推論します
- 提供された引数の型と fn ptr シグネチャの引数の型を正常に統一できたため、提供された引数は正しかったです

推論変数でのインスタンス化のもう 1 つの例として、`for<'a> T: Trait<'a>` where 句が与えられ、`T: Trait<'static>` が成立することを証明しようとしている場合、次のようにします：

- 推論変数でバインダーをインスタンス化し、where 句 `T: Trait<'?0>` を生成します
- ゴール `T: Trait<'static>` とインスタンス化された where 句を等しくし、`'?0 = 'static` を推論します
- `T: Trait<'static>` を `T: Trait<'?0>` と正常に統一できたため、ゴールは成立します

推論変数でバインダーをインスタンス化することは、[`InferCtxt`] の [`instantiate_binder_with_fresh_vars`] メソッドを使用して実行できます。バインダーの 1 つの特定のインスタンス化のみを気にする場合は、推論変数でバインダーをインスタンス化する必要があります。代わりにバインダーのすべての可能なインスタンス化について推論したい場合は、プレースホルダーを使用する必要があります。

## プレースホルダーでのインスタンス化

プレースホルダーは `Ty/ConstKind::Param`/`ReEarlyParam` に非常に似ています。それ自体にのみ等しい未知の型を表します。`Ty`/`Const` および `Region` はすべて [`Placeholder`] バリアントを持ち、[`Universe`] と [`BoundVar`] で構成されています。

`Universe` はプレースホルダーが発生したバインダーを追跡し、`BoundVar` はそのバインダーのどのパラメータにこのプレースホルダーが対応するかを追跡します。プレースホルダーの等価性は、宇宙が等しく `BoundVar` が等しいかどうかのみで決定されます。詳細については、[プレースホルダーと宇宙に関する章][ch_placeholders_universes]を参照してください。

他の rustc 開発者と話しているときや、`Debug` フォーマットされた `Ty`/`Const`/`Region` を見るときは、`Placeholder` はしばしば `'!UNIVERSE_BOUNDVARS` と書かれます。例えば、ある型 `for<'a> fn(&'a u32, for<'b> fn(&'b &'a u32))` が与えられ、両方のバインダーをインスタンス化した後（`InferCtxt` の `Universe` が以前に `U0` だったと仮定）、`&'b &'a u32` の型は `&'!2_0 &!1_0 u32` として表現されます。

プレースホルダーの宇宙が `0` の場合、デバッグ出力から完全に省略されます。つまり、`!0_2` は `!2` として出力されます。ただし、実際にはこれはめったに起こりません。なぜなら、バインダーをプレースホルダーでインスタンス化するときに `InferCtxt` の宇宙を増やすため、通常、遭遇可能な最も低い宇宙のプレースホルダーは `U1` のものだからです。

`Binder` は、`InferCtxt` の [`enter_forall`] メソッドを介してプレースホルダーでインスタンス化できます。コンパイラが、バインダーの 1 つの具体的なインスタンス化ではなく、バインダーの任意の可能なインスタンス化を気にする必要がある場合はいつでも使用する必要があります。

注：この章の最初の例では、ローカル変数が型 `&'^0 u32` を持つように推論すべきではないと述べました。このコードは、宇宙を介してコンパイルが防止されます（リンクされた章で説明されているとおり）

### なぜ `RePlaceholder` と `ReBound` の両方があるのですか？

これらの両方のバリアントがあるのはなぜか疑問に思うかもしれません。結局のところ、`Placeholder` に保存されているデータは、`ReBound` のそれと事実上同等です：どのバインダーかを追跡する何か、およびパラメータを追跡するインデックス、`Binder` が導入しました。

この主な理由は、`Bound` がバウンド変数のより構文的な表現であるのに対し、`Placeholder` はより意味的な表現であるためです。具体例として：

```rust
impl<'a> Other<'a> for &'a u32 { }

impl<T> Trait for T
where
    for<'a> T: Other<'a>,
{ ... }

impl<T> Bar for T
where
    for<'a> &'a T: Trait
{ ... }
```

これらのトレイト実装が与えられたとき、`u32: Bar` は成立すべきではありません。`&'a u32` は、借用のライフタイムとトレイトのライフタイムが等しい場合にのみ `Other<'a>` を実装します。ただし、`ReBound` のみを使用し、プレースホルダーを持っていなかった場合、そのトレイトバウンドが成立すると誤って信じる可能性があります。これを説明するために、rustc がプレースホルダーを持っていなかった世界で `u32: Bar` を証明しようとする例を見てみましょう：

- `u32: Bar` を証明しようとします
- `impl<T> Bar for T` impl を見つけます。`EarlyBinder` を `u32` でインスタンス化することになります（注：これは、最初に推論変数でバインダーをインスタンス化してから `u32` に推論するため、_正確に_正確ではありませんが、その区別はここではそれほど重要ではありません）
- impl には where 句 `for<'a> &'^0 T: Trait` があります。早期バインダーを `u32` でインスタンス化したため、実際には `for<'a> &'^0 u32: Trait` を証明する必要があります
- `impl<T> Trait for T` impl を見つけます。`EarlyBinder` を `&'^0 u32` でインスタンス化することになります
- where 句 `for<'a> T: Other<'^0>` があります。早期バインダーを `&'^0 u32` でインスタンス化したため、実際には `for<'a> &'^0 u32: Other<'^0>` を証明する必要があります
- `impl<'a> Other<'a> for &'a u32` を見つけます。この impl は、借用のライフタイムとトレイトのライフタイムが両方とも `'^0` であるため、バウンドを証明するのに十分です

この最終結果は正しくありません。なぜなら、2 つの別々のバインダーが独自のジェネリックパラメータを導入していたため、トレイトバウンドは `for<'a1, 'a2> &'^1 u32: Other<'^0>` のようなものになるはずだったからです。これは `impl<'a> Other<'a> for &'a u32` によって満たされて*いません*。

理論的にはこれを動作させることができますが、現在のセットアップよりも非常に複雑でより複雑になります。次のことをしなければなりません：

- `Bound` ty/const/region で `Binder`/`EarlyBinder` をインスタンス化するたびに、バウンド変数を「書き直し」て、より高い `DebruijnIndex` を持つようにします
- 推論変数をバウンド変数に推論するとき、そのバウンド変数が推論変数の作成後に入力されたバインダーからのものである場合、変数の `DebruijnIndex` を下げる必要があります。
- 推論変数がどのバインダーの内部で作成されたかを個別に追跡し、また、パラメータを名前付けできる最も内側のバインダーも追跡します（現在は後者のみを追跡する必要があります）
- 推論変数を解決するとき、infcx の現在のバインダー深度に応じてバウンド変数を書き直します
- おそらくもっと（このリストを書いている間にアイテムが追加され続けたので、これが網羅的であると考えるのは naive に思えます）

基本的に、この複雑さのすべては、`Bound` ty/const/regions が、パラメータを導入するバインダーとその使用の間にある他の `Binder` の数に応じて、`Binder` の特定のパラメータに対して異なる表現を持っているためです。例えば、次のコードが与えられたとします：

```rust
fn foo<T>()
where
    for<'a> T: Trait<'a, for<'b> fn(&'b T, &'a u32)>
{ ... }
```

その where 句は次のように書かれます：
`for<'a> T: Trait<'^0, for<'b> fn(&'^0 T, &'^1_0 u32)>`
`'a` パラメータへの 2 つの参照があるにもかかわらず、それらは両方とも異なる方法で表現されています：`^0` と `^1_0`。これは、後者の使用が内部関数ポインタ型の 2 番目の `Binder` の下にネストされているためです。

これは、パラメータの使用サイトではなく、現在の `InferCtxt` に固有の `Universe` であるため、この制限を持たない `Placeholder` ty/const/regions とは対照的です。

既存の `Placeholder` で `EarlyBinder` をインスタンス化し、推論変数を統一することは簡単に可能です。なぜなら、`Placeholder` がどのコンテキストにあっても、同じ表現を持つからです。上記の高階 where 句のバインダーをインスタンス化した場合の例として、次のように表現されます：
`T: Trait<'!1_0, for<'b> fn(&'^0 T, &'!1_0 u32)>`
`'a` の両方の使用の `RePlaceholder` 表現は、1 つが別の `Binder` の下にあるにもかかわらず同じです。

次に、関数ポインタのバインダーをインスタンス化すると、次のような型が得られます：
`fn(&'!2_0 T, ^'!1_0 u32)`
`'b` パラメータの `RePlaceholder` は、そのバインダーが `'a` のバインダーの後にインスタンス化されたという事実を追跡するために、より高い宇宙にあります。

## `ReLateParam` でのインスタンス化

[型の表現に関する章][representing-types]で議論されたように、`RegionKind` には、ジェネリックパラメータを表現するための 2 つのバリアントがあります：`ReLateParam` と `ReEarlyParam`。`ReLateParam` は概念的には、常にルート宇宙（`U0`）にある `Placeholder` です。これは、関数/クロージャの遅延バウンドパラメータをそれらの内部からインスタンス化するときに使用されます。その実際の表現は、`ReEarlyParam` と `RePlaceholder` の両方とは比較的異なります：

- 遅延バウンドジェネリックパラメータを導入したアイテムの `DefId`
- ジェネリックパラメータの `DefId` とその名前（`Symbol` を介して）を指定する [`BoundRegionKind`]、またはこのプレースホルダーが `Fn`/`FnMut` クロージャの自己借用の匿名ライフタイムを表していることを示します。`BrAnon` のバリアントもありますが、これは `ReLateParam` には使用されません。

例えば、次のコードが与えられたとします：

```rust,ignore
impl Trait for Whatever {
    fn foo<'a>(a: &'a u32) -> &'a u32 {
        let b: &'a u32 = a;
        b
    }
}
```

関数本体の型 `&'a u32` のライフタイム `'a` は次のように表現されます：

```
ReLateParam(
    {impl#0}::foo,
    BoundRegionKind::BrNamed({impl#0}::foo::'a, "'a")
)
```

関数の内部から関数の遅延バウンドジェネリックパラメータを参照するこの特定のケースでは、これは `hir_ty_lowering` 中に暗黙的に行われ、どこかで `Binder` をインスタンス化するときに明示的に行われるのではありません。ただし、場合によっては、`ReLateParam` で `Binder` を明示的にインスタンス化します。

一般的に、関数/クロージャの遅延バウンドパラメータの `Binder` があり、概念的にバインダーの内部にすでにいる場合は、[`liberate_late_bound_regions`] を使用して `ReLateParam` でインスタンス化します。これにより、この操作は `Binder` の `EarlyBinder` の `instantiate_identity` に相当します。

具体例として、型チェックしている関数のシグネチャにアクセスすることは、`EarlyBinder<Binder<FnSig>>` として表現されます。これらのバインダーの「内部」にすでにいるため、`instantiate_identity` に続いて `liberate_late_bound_regions` を呼び出します。

[`liberate_late_bound_regions`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.liberate_late_bound_regions
[representing-types]: param_ty_const_regions.md
[`BoundRegionKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/enum.BoundRegionKind.html
[`enter_forall`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/infer/struct.InferCtxt.html#method.enter_forall
[ch_placeholders_universes]: ../borrow_check/region_inference/placeholders_and_universes.md
[`instantiate_binder_with_fresh_vars`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/infer/struct.InferCtxt.html#method.instantiate_binder_with_fresh_vars
[`InferCtxt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/infer/struct.InferCtxt.html
[`EarlyBinder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.EarlyBinder.html
[`Binder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.Binder.html
[`Placeholder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Placeholder.html
[`Universe`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.UniverseIndex.html
[`BoundVar`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.BoundVar.html
