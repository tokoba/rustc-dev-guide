# エイリアスと正規化

## エイリアス

Rustには、ある「基礎となる」型と等しいと見なされる多くの型があります。
たとえば、固有関連型、トレイト関連型、自由型エイリアス（`type Foo = u32`）、
不透明型（`-> impl RPIT`）などです。このような型を「エイリアス」と見なし、
エイリアス型は[`TyKind::Alias`][tykind_alias]バリアントで表され、
エイリアスの種類は[`AliasTyKind`][aliaskind]列挙型で追跡されます。

正規化は、これらのエイリアス型を取り、それらが等しい基礎となる型に置き換えるプロセスです。
たとえば、ある型エイリアス`type Foo = u32`が与えられた場合、`Foo`を正規化すると`u32`が得られます。

エイリアスの概念は*型*に固有ではなく、概念は定数/const genericsにも適用されます。
ただし、現在、コンパイラでは、const エイリアスを「第一級の概念」として扱っていないため、
この章では主に型のコンテキストで物事を説明します（概念は問題なく転送されますが）。

[tykind_alias]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/enum.TyKind.html#variant.Alias
[aliaskind]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/enum.AliasTyKind.html

### Rigid、Ambiguous、Unnormalized エイリアス

エイリアスは「rigid」、「ambiguous」、または単に unnormalized のいずれかです。

型の「形状」が変更されない場合、型を rigid と見なします。たとえば、`Box`は rigid です。
正規化しても`Box`が`u32`になることはないためです。一方、`<vec::IntoIter<u32> as Iterator>::Item`は
rigid ではありません。`u32`に正規化できるためです。

エイリアスは、さらに正規化できない場合に rigid です。*rigid*エイリアスの具体的な例は、
`T: Iterator<Item = ...>`境界がなく、`T: Iterator`境界のみがある環境の`<T as Iterator>::Item`です。
```rust
fn foo<T: Iterator>() {
    // This alias is *rigid*
    let _: <T as Iterator>::Item;
}

fn bar<T: Iterator<Item = u32>>() {
    // This alias is *not* rigid as it can be normalized to `u32`
    let _: <T as Iterator>::Item;
}
```

エイリアスがまだ正規化できないが、[現在の環境](./typing_parameter_envs.md)で正規化可能になる可能性がある場合、
「ambiguous」エイリアスと見なします。これは、エイリアスにトレイトの実装方法を判断できない
推論変数が含まれている場合に発生する可能性があります。
```rust
fn foo<T: Iterator, U: Iterator>() {
    // This alias is considered to be "ambiguous"
    let _: <_ as Iterator>::Item;
}
```

これらを「ambiguous」エイリアスと呼ぶ理由は、これが rigid エイリアスかどうかが*あいまい*だからです。

`_: Iterator`トレイト impl のソースは*あいまい*（つまり不明）です。
`impl Iterator for u32`や`T: Iterator`トレイト境界など、いろいろなものがある可能性があります。まだわかりません。
`_: Iterator`が保持される理由に応じて、エイリアスは unnormalized エイリアスまたは rigid エイリアスである可能性があります。
これはどのようなエイリアスであるかが*あいまい*です。

最後に、エイリアスは単に unnormalized である可能性があります。`<Vec<u32> as IntoIterator>::Iter`は unnormalized エイリアスです。
すでに`std::vec::IntoIter<u32>`に正規化できますが、まだ行われていません。

---

FreeおよびInherentエイリアスは、名前を付けることもエイリアスの定義を解決したことを意味し、
エイリアスの基礎となる型を指定するため、rigid または ambiguous にはなり得ないことに注意してください。

### 発散エイリアス

エイリアスは、その定義が正規化する基礎となる非エイリアス型を指定しない場合、「発散」すると見なされます。
発散エイリアスの具体的な例：
```rust
type Diverges = Diverges;

trait Trait {
    type DivergingAssoc;
}
impl Trait for () {
    type DivergingAssoc = <() as Trait>::DivergingAssoc;
}
```
この例では、`Diverges`と`DivergingAssoc`はどちらも、自分自身と等しいと定義されている
発散型エイリアスの「自明な」ケースです。`Diverges`が正規化できる基礎となる型はありません。

発散エイリアスが定義されたときに一般的にエラーを出そうとしますが、これは完全に「ベストエフォート」チェックです。
前の例では、定義は「十分に単純」で検出されるため、エラーが出力されます。
ただし、より複雑なケース、またはジェネリックパラメータの一部のインスタンス化のみが
発散エイリアスになるケースでは、エラーを出力しません。
```rust
trait Trait {
    type DivergingAssoc<U: Trait>;
}
impl<T: ?Sized> Trait for T {
    // This alias always diverges but we don't emit an error because
    // the compiler can't "see" that.
    type DivergingAssoc<U: Trait> = <U as Trait>::DivergingAssoc<U>;
}
```

最終的に、これは型システム内のエイリアスが非発散であるという保証がないことを意味します。
エイリアスは、特定のジェネリック引数に対してのみ発散する可能性があるため、
エイリアスが発散するかどうかは、完全に具体的な場合にのみわかります。
これは、codegenまたは const-evaluation も発散エイリアスを処理する必要があることを意味します。
```rust
trait Trait {
    type Diverges<U: Trait>;
}
impl<T: ?Sized> Trait for T {
    type Diverges<U: Trait> = <U as Trait>::Diverges<U>;
}

fn foo<T: Trait>() {
    let a: T::Diverges<T>;
}

fn main() {
    foo::<()>();
}
```
この例では、`foo::<()>`の codegen 中にのみ発散エイリアスからエラーが発生します。
`foo`への呼び出しが削除されると、コンパイルエラーは出力されません。

### 不透明型

不透明型は、比較的特別な種類のエイリアスであり、独自の章で説明されています：[不透明型](./opaque-types-type-alias-impl-trait.md)。

### Const エイリアス

型エイリアスとは異なり、const エイリアスは型システムで直接表されません。
代わりに、const エイリアスは常に const アイテムへのパス式を含む匿名本体です。
これは、型システムの唯一の「const エイリアス」が、未評価の匿名 const 本体であることを意味します。

そのため、`ConstKind::Alias(AliasCtKind::Projection/Inherent/Free, _)`はなく、
代わりに匿名定数を表すために使用される`ConstKind::Unevaluated`のみがあります。

```rust
fn foo<const N: usize>() {}

const FREE_CONST: usize = 1 + 1;

fn bar() {
    foo::<{ FREE_CONST }>();
    // The const arg is represented with some anonymous constant:
    // ```pseudo-rust
    // const ANON: usize = FREE_CONST;
    // foo::<ConstKind::Unevaluated(DefId(ANON), [])>();
    // ```
}
```

これは、const generics 機能が改善されるにつれて変更される可能性があります。
たとえば、`feature(associated_const_equality)`と`feature(min_generic_const_args)`はどちらも、
型と同様に（すべての const 引数をラップする匿名定数なしで）const エイリアスを処理する必要があります。

## 正規化とは

### 構造的正規化と深い正規化

正規化には、構造的（*浅い*とも呼ばれることがある）と深いという2つの形式があります。
構造的正規化は、型の「最も外側の」部分のみを正規化すると考えるべきです。
一方、深い正規化は、型内の*すべての*エイリアスを正規化します。

実際には、構造的正規化は、型の外側の層だけでなく、正規化される可能性がありますが、
この動作に依存すべきではありません。境界変数（`for<'a>`）を使用する
正規化できない非 rigid エイリアスは、どちらの種類の正規化でも正規化できません。

例として：概念的には、型`Vec<<u8 as Identity>::Assoc>`を構造的に正規化すると no-op になりますが、
深く正規化すると`Vec<u8>`が得られます。ただし、実際には構造的正規化でも`Vec<u8>`が得られますが、
繰り返しますが、これに依存すべきではありません。

エイリアスを境界変数を使用するように変更すると、動作が異なります。
`Vec<for<'a> fn(<&'a u8 as Identity>::Assoc)>`は構造的に正規化されても変更されませんが、
深く正規化されると`Vec<for<'a> fn(&'a u8)>`になります。

### コア正規化ロジック

エイリアスを構造的に正規化することは、エイリアスをその定義で等しいと定義されているものに置き換えるよりも
少し微妙です。エイリアスを正規化した結果は、rigid 型または推論変数
（後で rigid 型に推論されます）のいずれかである必要があります。これを達成するために、2つのことを行います。

まず、ambiguous エイリアスを正規化するとき、そのままにするのではなく推論変数に正規化します。
これには2つの主な効果があります。
- 推論変数は rigid 型ではありませんが、常に rigid 型に推論されるため、
  正規化の結果が再び正規化される必要がないことを保証します
- 推論変数は、型が非 rigid であるすべてのケースで使用され、
  コンパイラの残りの部分が*両方の* ambiguous エイリアス*と*推論変数を処理する必要がないようにします

第二に、正規化がエイリアスの定義で指定された型を直接返すのではなく、
返す前にまず型を正規化します[^1]。正規化が冪等/呼び出し元がループで実行する必要がないようにこれを行います。

```rust
#![feature(lazy_type_alias)]

type Foo<T: Iterator> = Bar<T>;
type Bar<T: Iterator> = <T as Iterator>::Item;

fn foo() {
    let a_: Foo<_>;
}
```

この例では：
- `Foo<?x>`を正規化すると`Bar<?x>`になりますが、`Foo`が等しいと定義されている型のエイリアスを正規化したいです
- `Bar<?x>`を正規化すると`<?x as Iterator>::Item`になりますが、繰り返しますが、
  `Bar`が等しいと定義されている型のエイリアスを正規化したいです
- `<?x as Iterator>::Item`を正規化すると、`<?x as Iterator>::Item`が ambiguous エイリアスであるため、
  新しい推論変数`?y`が生成されます
- 最終的な結果は、`Foo<?x>`を正規化すると`?y`になることです

## 正規化の方法

型システムとのインターフェース時には、型を正規化するよう要求する必要があることがよくあります。
基礎となる正規化ロジックへのエントリポイントは多数あり、
各エントリポイントはコンパイラの特定の部分でのみ使用する必要があります。

<!-- date-check: May 2025 -->
追加の複雑さとして、コンパイラは現在、古いトレイトソルバーから新しいトレイトソルバーへの
移行を進めています。この移行の一環として、コンパイラでの正規化へのアプローチは大幅に変更されており、
一部の正規化エントリポイントは「古いソルバーのみ」であり、新しいソルバーが安定したら
長期的に削除される予定です。
移行は、Github の [WG-trait-system-refactor](https://github.com/rust-lang/rust/labels/WG-trait-system-refactor) ラベルで追跡できます。

コンパイラのさまざまな正規化エントリポイントの概要は次のとおりです。
- `infcx.at.structurally_normalize`
- `infcx.at.(deeply_)?normalize`
- `infcx.query_normalize`
- `tcx.normalize_erasing_regions`
- `traits::normalize_with_depth(_to)`
- `EvalCtxt::structurally_normalize`

### トレイトソルバーの外側

[`InferCtxt`][infcx]型は、分析中に正規化する「主な」方法を公開します：
[`normalize`][normalize]、[`deeply_normalize`][deeply_normalize]、[`structurally_normalize`][structurally_normalize]。
これらの関数は、[`FnCtxt`][fcx]や[`ObligationCtxt`][ocx]などのさまざまな`InferCtxt`ラッパー型で
ラップされて再公開されることが多く、一部の引数または戻り値の一部を自動的に処理するために
APIの微調整が行われます。

#### 構造的`InferCtxt`正規化

[`infcx.at.structurally_normalize`][structurally_normalize]は、推論変数と領域を処理できる
構造的正規化を公開します。型の種類を調べるときは一般的に使用する必要があります。

HIR Typeck 内には、関連する正規化メソッド- [`fcx.structurally_resolve`][structurally_resolve]があります。
これは、解決される型が未解決の推論変数である場合にエラーを出します。
新しいソルバーが有効になっている場合、型を構造的に正規化しようともします。

このため、HIR typeck には、型が最初に`normalize`を介して正規化される
（古いソルバーでのみ正規化）パターンがあり、次に`structurally_resolve`される
（新しいソルバーでのみ正規化）パターンがあります。このパターンは、
HIR typeck 中に`structurally_normalize`を呼び出すよりも優先されるべきです。
`structurally_resolve`は、`structurally_normalize`がゴールを評価しないのに対し、
ゴールを評価して推論の進行を試みるためです。

#### 深い`InferCtxt`正規化

##### `infcx.at.(deeply_)?normalize`

`InferCtxt`で深く正規化するには、`normalize`と`deeply_normalize`の2つの方法があります。
この理由は、`normalize`は古いソルバーでのみ使用される「レガシー」正規化エントリポイントであるのに対し、
`deeply_normalize`は長期的に深く正規化する方法であることを意図しているためです。
これらのメソッドはどちらも領域を処理できます。

新しいソルバーが安定すると、`infcx.at.normalize`関数は削除され、
すべてが新しい深いまたは構造的正規化メソッドに移行されます。このため、
`normalize`関数は新しいソルバーでは no-op であり、
古いソルバーが正規化を必要とするが新しいソルバーは必要としない場合にのみ適しています。

`deeply_normalize`を使用すると、ambiguous エイリアス[^2]に遭遇したときにエラーが出力されます。
*すべての* ambiguous エイリアスを推論変数に正規化することをサポートすることは不可能だからです[^3]。
`deeply_normalize`は一般的に、ambiguous エイリアスに遭遇することが予想されない場合にのみ使用する必要があります。
たとえば、アイテムシグネチャの型を処理する場合などです。

##### `infcx.query_normalize`

[`infcx.query_normalize`][query_norm]はほとんど使用されず、`normalize_erasing_regions`とほぼ同じ制限があります
（推論変数を処理できない、診断サポートなし）。主な違いは、ライフタイム情報を保持することです。
このため、ライフタイム消去クエリをキャッシュする方がより効率的であるため、
`normalize_erasing_regions`はほぼすべての状況でより良い選択です。

実際には、`query_normalize`は borrow checker での正規化に使用され、
`infcx.normalize`のパフォーマンス最適化として他の場所で使用されます。
新しいソルバーが安定すると、新しいソルバーの正規化実装は十分にパフォーマンスが高いため、
パフォーマンス回帰にならないため、`query_normalize`をコンパイラから削除できることが期待されます。

##### `tcx.normalize_erasing_regions`

[`normalize_erasing_regions`][norm_erasing_regions]は一般的に、型システム分析を行っていない
コンパイラの部分で使用されます。この正規化エントリポイントは、
推論変数、ライフタイム、または診断を処理しません。
Lintsとcodegenは、通常、完全に推論されたエイリアスで作業しており、
適格であると仮定できる（または少なくともエラーを出す責任がない）ため、
このエントリポイントを多用します。

[query_norm]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/infer/at/struct.At.html#method.query_normalize
[norm_erasing_regions]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.normalize_erasing_regions
[normalize]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/infer/at/struct.At.html#method.normalize
[deeply_normalize]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/traits/normalize/trait.NormalizeExt.html#tymethod.deeply_normalize
[structurally_normalize]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/traits/trait.StructurallyNormalizeExt.html#tymethod.structurally_normalize_ty
[infcx]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/infer/struct.InferCtxt.html
[fcx]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/fn_ctxt/struct.FnCtxt.html
[ocx]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/traits/struct.ObligationCtxt.html
[structurally_resolve]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/fn_ctxt/struct.FnCtxt.html#method.structurally_resolve_type

### トレイトソルバーの内側

[`traits::normalize_with_depth(_to)`][norm_with_depth]と[`EvalCtxt::structurally_normalize`][eval_ctxt_structural_norm]は、
トレイトソルバー（それぞれ古いものと新しいもの）の内部でのみ使用されます。
これは、正規化が各トレイトソルバーによってどのように実装されるかの内部への
事実上の生のエントリポイントです。他の正規化エントリポイントは、
ゴールサイクルと再帰深度を正しく処理しないため、トレイトソルビングの内部から使用できません。

[norm_with_depth]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/traits/normalize/fn.normalize_with_depth.html
[eval_ctxt_structural_norm]:  https://doc.rust-lang.org/nightly/nightly-rustc/rustc_next_trait_solver/solve/struct.EvalCtxt.html#method.structurally_normalize_term

## いつ/どこで正規化するか（古いソルバー vs 新しいソルバー）

古いソルバーと新しいソルバーの間の大きな変更の1つは、エイリアスを正規化することが期待される時期へのアプローチです。

### 古いソルバー

すべての型は、できるだけ早く正規化されることが期待されるため、
型システムで遭遇するすべての型は、rigid または推論変数（後で rigid 項に推論される）のいずれかです。

具体例として：エイリアスの等価性は、それらが rigid であると仮定し、
エイリアスのジェネリック引数を再帰的に等価にすることによって実装されます。

### 新しいソルバー

すべての型には、ambiguous または unnormalized エイリアスが含まれている可能性があると予想されます。
エイリアスを正規化する必要がある操作が実行されるたびに、
そのロジックがエイリアスを正規化する責任があります
（これは、`ty.kind()`で一致させるには、ほぼ常に最初に構造的に正規化する必要があることを意味します）。

具体例として：エイリアスの等価性は、エイリアス自体の正規化を処理できるように、
カスタムゴールカインド（[`PredicateKind::AliasRelate`][aliasrelate]）によって実装されます。
これは、等価にされるすべてのエイリアス型が rigid であると仮定する代わりに行われます。

このアプローチにもかかわらず、パフォーマンス/簡潔さのために[writeback][writeback]中に
まだ深く正規化するため、MIRの型はまだ深く正規化されていると仮定できます。

[aliasrelate]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.PredicateKind.html#variant.AliasRelate
[writeback]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/writeback/index.html

---

古いソルバーの正規化アプローチに関するいくつかの主な問題があり、
新しいソルバーで物事を変更する動機となりました。

### 正規化呼び出しの欠落

正規化呼び出しが欠落していることが頻繁に発生し、
すべてがすでに正規化されていることを期待するAPIに unnormalized 型を渡すことになりました。
unnormalized エイリアスを rigid として扱うと、エイリアスが互いに等しいと見なされないか、
unnormalized エイリアスのジェネリック引数を等価にすることから驚くべき推論ガイダンスが得られるなど、
あらゆる種類の奇妙なエラーが発生します。

### パラメータ環境の正規化

もう1つの問題は、古いソルバーでは`ParamEnv`を正しく正規化できなかったことです。
正規化自体が正しい結果を与えるために正規化された`ParamEnv`を期待するためです。
詳細については、`ParamEnv`に関する章を参照してください：[`Typing/ParamEnv`s: すべての境界の正規化](./typing_parameter_envs.md#normalizing-all-bounds)

### 高ランク型の正規化できない非 rigid エイリアス

`for<'a> fn(<?x as Trait<'a>::Assoc>)`のような型が与えられた場合、
古いソルバーの正規化アプローチでこれを正しく処理することはできません。

`for<'a> fn(?y)`に正規化し、`for<'a> <?x as Trait<'a>>::Assoc -> ?y`を正規化するゴールを登録すると、
`<?x as Trait<'a>>::Assoc`が`&'a u32`に正規化されるケースでエラーが発生します。
推論変数`?y`は、`for<'a>`バインダーをインスタンス化するときに作成されたプレースホルダーよりも
低い[ユニバース]にあります。

エイリアスを unnormalized のままにすることも間違っています。
古いソルバーはすべてのエイリアスが rigid であることを期待しているためです。
これは、新しいソルバーがコヒーレンスで安定する前の健全性バグでした：
[relating projection substs is unsound during coherence](https://github.com/rust-lang/rust/issues/102048)。

最終的に、これは値内のすべてのエイリアスが rigid であることを保証することが
常に可能であるとは限らないことを意味します。

[universe]: borrow_check/region_inference/placeholders_and_universes.md#what-is-a-universe
[deeply_normalize]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/traits/normalize/trait.NormalizeExt.html#tymethod.deeply_normalize

## 発散エイリアスの使用の処理

発散エイリアスは、ambiguous エイリアスと同様に、推論変数に正規化されます。
発散エイリアスを正規化すると、トレイトソルバーサイクルが発生するため、
古いソルバーでは常にエラーになります。新しいソルバーでは、
現在のコンテキストですべてのゴールが保持される必要がある場合にのみエラーになります。
たとえば、HIR typeck 中に発散エイリアスを正規化すると、
両方のソルバーでエラーになります。

エイリアス well-formedness は、エイリアスが発散しないことを要求しません[^4]。
これは、エイリアスが well-formed であることをチェックすることが、
発散エイリアスに対してエラーを出すのに十分ではないことを意味します。
実際にエイリアスを正規化しようとする必要があります。

発散エイリアスのエラーが正規化の副作用であることは、
実際にエラーを出すかどうかが非常に*恣意的*であることを意味し、
古いソルバーと新しいソルバーでは、今は正規化する場所が少ないため異なります。

発散エイリアスが恣意的にエラーを引き起こすことの「問題」の例：
```rust
trait Trait {
    type Diverges<D: Trait>;
}

impl<T> Trait for T {
    type Diverges<D: Trait> = D::Diverges<D>;
}

struct Bar<T: ?Sized = <u8 as Trait>::Diverges<u8>>(Box<T>);
```

この例では、発散エイリアスが使用されていますが、
ジェネリックパラメータのデフォルトを明示的に正規化しないため、
エラーを出しません。`?Sized`オプトアウトが削除されると、
`<u8 as Trait>::Diverges<u8>: Sized`ゴールを正規化することになるため、
エラーが出力されます。これは、副作用として発散エイリアスに関するエラーになります。

Const エイリアスは、型エイリアスとはここで少し異なります。
const エイリアスの well-formedness は、それらが正常に評価できることを要求します
（[`ConstEvaluatable`][const_evaluatable]ゴール経由）。これは、
const 引数の well-formedness を単純にチェックするだけで、
評価に失敗した場合にエラーを出すのに十分であることを意味します。
これが型エイリアスにも意味があるかどうか、または
const エイリアスが well-formedness のためにこれを要求するのをやめるべきかどうかは、
やや不明確です[^5]。

[^1]: 新しいソルバーでは、これは暗黙的に行われます

[^2]: バインダー内の ambiguous エイリアスの処理方法には、古いソルバーと新しいソルバーの間で微妙な違いがあります。古いソルバーでは、高ランク型内の一部の ambiguous エイリアスでエラーを出さないのに対し、新しいソルバーは正しくエラーを出します。

[^3]: バインダー内の ambiguous エイリアスは推論変数に正規化できません。これについては後で詳しく説明します。

[^4]: エイリアスが非発散であることをチェックすることは、完全に具体的になるまで実行できないため、codegen/const-evaluation の前にエイリアスが well-formed であることをチェックできないか、エイリアスがモノモーフィゼーション後に well-formed から非 well-formed になることを意味します。

[^5]: Const エイリアスがこれを行うのをやめた場合、確かに型エイリアスより*健全性が低い*ことはありません

[const_evaluatable]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.ClauseKind.html#variant.ConstEvaluatable
