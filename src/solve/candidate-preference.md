# 候補の優先順位

`Trait`および`NormalizesTo`ゴールを証明する方法は複数あります。そのような各オプションは[`Candidate`]と呼ばれます。適用可能な候補が複数ある場合、一部の候補を他の候補よりも優先します。関連情報は[`CandidateSource`]に保存されます。

この優先順位は、不正確な推論や領域制約を引き起こす可能性があり、したがってコヒーレンス中は健全ではありません。このため、コヒーレンス中はすべての候補をマージしようとするだけです。

## `Trait`ゴール

トレイトゴールは、[`fn merge_trait_candidates`]で適用可能な候補をマージします。このドキュメントでは、現在の優先順位ルールを*なぜ*設定したのかを説明するための追加の詳細と参照を提供します。

### `CandidateSource::BuiltinImpl(BuiltinImplSource::Trivial))`

自明なビルトインimplは、well-formedな型に対して常に適用可能であることが知られているビルトインimplです。これは、存在する場合、別の候補を使用しても制約が少なくなることはないことを意味します。現在、`Sized`および`MetaSized`のimplのみを自明と見なしています。

これは、次のパターンのライフタイムエラーを防ぐために必要です

```rust
trait Trait<T>: Sized {}
impl<'a> Trait<u32> for &'a str {}
impl<'a> Trait<i32> for &'a str {}
fn is_sized<T: Sized>(_: T) {}
fn foo<'a, 'b, T>(x: &'b str)
where
    &'a str: Trait<T>,
{
    // `&'a str: Trait<T>` where境界を精緻化すると、
    // `&'a str: Sized` where境界が得られます。これを
    // ビルトインimplより優先したくありません。
    is_sized(x);
}
```

この優先順位は、ビルトインimplがパラメータ以外のwhere句に依存するネストされたゴールを持つ場合に不正確です

```rust
struct MyType<'a, T: ?Sized>(&'a (), T);
fn is_sized<T>() {}
fn foo<'a, T: ?Sized>()
where
    (MyType<'a, T>,): Sized,
    MyType<'static, T>: Sized,
{
    // where境界は自明ですが、タプルのビルトイン`Sized` implは
    // `MyType<'a, T>: Sized`を証明することを要求し、これはwhere句を使用することによってのみ
    // 証明でき、不要な`'static`制約を追加します。
    is_sized::<(MyType<'a, T>,)>();
    //~^ ERROR lifetime may not live long enough
}
```

### `CandidateSource::ParamEnv`

少なくとも1つの*非グローバル*な`ParamEnv`候補が存在すると、*すべての*`ParamEnv`候補を他の候補の種類よりも優先します。
where境界は、高階ランクでなく、ジェネリックパラメータを含まない場合、グローバルです。`'static`を含む場合があります。

where境界を他の候補よりも適用しようとするのは、ユーザーがwhere境界を最も制御しやすいため、候補の優先順位が不正確な場合に最も簡単に調整できるからです。

#### `Impl`候補よりも優先

これは、次の例で領域エラーを回避するために必要です

```rust
trait Trait<'a> {}
impl<T> Trait<'static> for T {}
fn impls_trait<'a, T: Trait<'a>>() {}
fn foo<'a, T: Trait<'a>>() {
    impls_trait::<'a, T>();
}
```

また、シャドウされたimplが現在曖昧なソルバーサイクルを引き起こす可能性があるため、これも必要です: [trait-system-refactor-initiative#76]。優先順位がない場合、where境界が領域制約を引き起こす場合に不完全性を避けるために、曖昧性エラーで失敗する必要があります。

```rust
trait Super {
    type SuperAssoc;
}

trait Trait: Super<SuperAssoc = Self::TraitAssoc> {
    type TraitAssoc;
}

impl<T, U> Trait for T
where
    T: Super<SuperAssoc = U>,
{
    type TraitAssoc = U;
}

fn overflow<T: Trait>() {
    // 精緻化された`Super<SuperAssoc = Self::TraitAssoc>` where境界を使用して、
    // `T: Trait`実装のwhere境界を証明できます。これは現在
    // オーバーフローを引き起こします。
    let x: <T as Trait>::TraitAssoc;
}
```

この優先順位は多くの問題を引き起こします。[#24066]を参照してください。問題のほとんどは、
where境界が型推論をガイドする場合でも、where境界をimplよりも優先することによって引き起こされます：

```rust
trait Trait<T> {
    fn call_me(&self, x: T) {}
}
impl<T> Trait<u32> for T {}
impl<T> Trait<i32> for T {}
fn bug<T: Trait<U>, U>(x: T) {
    x.call_me(1u32);
    //~^ ERROR mismatched types
}
```

ただし、where境界が推論をガイドしない場合にのみこの優先順位を適用しても、不正確なライフタイム制約が発生する可能性があります：

```rust
trait Trait<'a> {}
impl<'a> Trait<'a> for &'a str {}
fn impls_trait<'a, T: Trait<'a>>(_: T) {}
fn foo<'a, 'b>(x: &'b str)
where
    &'a str: Trait<'b>
{
    // `'b: 'x`で`&'x str: Trait<'b>`を証明する必要があります。
    impls_trait::<'b, _>(x);
    //~^ ERROR lifetime may not live long enough
}
```

#### `AliasBound`候補よりも優先

これは、次の例で領域エラーを回避するために必要です

```rust
trait Bound<'a> {}
trait Trait<'a> {
    type Assoc: Bound<'a>;
}

fn impls_bound<'b, T: Bound<'b>>() {}
fn foo<'a, 'b, 'c, T>()
where
    T: Trait<'a>,
    for<'hr> T::Assoc: Bound<'hr>,
{
    impls_bound::<'b, T::Assoc>();
    impls_bound::<'c, T::Assoc>();
}
```

不要な制約を引き起こす可能性もあります

```rust
trait Bound<'a> {}
trait Trait<'a> {
    type Assoc: Bound<'a>;
}

fn impls_bound<'b, T: Bound<'b>>() {}
fn foo<'a, 'b, T>()
where
    T: for<'hr> Trait<'hr>,
    <T as Trait<'b>>::Assoc: Bound<'a>,
{
    // `<T as Trait<'a>>::Assoc: Bound<'a>`のwhere境界を使用すると、
    // `<T as Trait<'a>>::Assoc`と環境からの
    // `<T as Trait<'b>>::Assoc`を不必要に等しくします。
    impls_bound::<'a, <T as Trait<'a>>::Assoc>();
    // `<T as Trait<'b>>::Assoc: Bound<'b>`の場合、where境界の自己型は
    // 一致しますが、トレイト境界の引数は一致しません。
    impls_bound::<'b, <T as Trait<'b>>::Assoc>();
}
```

#### グローバルwhere境界に優先順位を設定しない理由

グローバルwhere境界は、implによって完全に暗示されるか、満たすことができません。満たすことができない場合、何が起こっても実際には気にしません。where境界が完全に暗示される場合、implを使用してトレイトゴールを証明しても追加の制約は発生しません。トレイトゴールの場合、これは`'static`を使用するwhere境界にのみ有用です：

```rust
trait A {
    fn test(&self);
}

fn foo(x: &dyn A)
where
    dyn A + 'static: A, // この境界を使用するとライフタイムエラーになります。
{
    x.test();
}
```

より重要なことは、ここでimplを使用することで、関連型を正規化する際にグローバルwhere境界がimplをシャドウすることを防ぎます。グローバルwhere境界をimplよりも優先することによる既知の問題はありません。

#### グローバルwhere境界を引き続き考慮する理由

グローバルwhere境界が存在する場合でもimplを使用するだけなので、これらのグローバルwhere境界を完全に無視しない理由を疑問に思うかもしれません：非グローバルwhere境界からの推論ガイダンスを弱めるためにそれらを使用します。

非グローバルwhere境界がなければ、現在、適用可能なimplもあるにもかかわらず非グローバルwhere境界を優先します。非グローバルwhere境界を追加することで、この不必要な推論ガイダンスが無効になり、次のコンパイルが可能になります：

```rust
fn check<Color>(color: Color)
where
    Vec: Into<Color> + Into<f32>,
{
    let _: f32 = Vec.into();
    // グローバルな`Vec: Into<f32>`境界がなければ、
    // 非グローバルな`Vec: Into<Color>`境界を
    // 熱心に使用し、これが失敗します。
}

struct Vec;
impl From<Vec> for f32 {
    fn from(_: Vec) -> Self {
        loop {}
    }
}
```

### `CandidateSource::AliasBound`

エイリアス境界候補をimplよりも優先します。現在、この優先順位を使用して型推論をガイドし、次のコンパイルを可能にしています。個人的には、この優先順位が望ましいとは思いません 🤷

```rust
pub trait Dyn {
    type Word: Into<u64>;
    fn d_tag(&self) -> Self::Word;
    fn tag32(&self) -> Option<u32> {
        self.d_tag().into().try_into().ok()
        // `Self::Word: Into<?0>`を証明してから、
        // `?0`でメソッドを選択します。熱心な推論が必要です。
    }
}
```

```rust
fn impl_trait() -> impl Into<u32> {
    0u16
}

fn main() {
    // `x`には2つの可能な型があります：
    // - `impl Into<u32>`の「エイリアス境界」を使用した`u32`
    // - `impl<T> From<T> for T`を使用した`impl Into<u32>`、つまり`u16`
    //
    // 厳密には必要ではなく、驚くべきエラーを引き起こす可能性がある場合でも、
    // `x`の型を`u32`と推論します。
    let x = impl_trait().into();
    println!("{}", std::mem::size_of_val(&x));
}
```

この優先順位は、領域制約による曖昧性も回避します。これが実際に依存されているかどうかはわかりません。

```rust
trait Bound<'a> {}
impl<T> Bound<'static> for T {}
trait Trait<'a> {
    type Assoc: Bound<'a>;
}

fn impls_bound<'b, T: Bound<'b>>() {}
fn foo<'a, T: Trait<'a>>() {
    // これを`'a`または`'static`のどちらに推論すべきか。
    impls_bound::<'_, T::Assoc>();
}
```

### `CandidateSource::BuiltinImpl(BuiltinImplSource::Object(_))`

ビルトイントレイトオブジェクトimplをユーザー記述のimplよりも優先します。これは**健全ではなく**、将来的に削除されるべきです。詳細については、[#57893](https://github.com/rust-lang/rust/issues/57893)および[#141347](https://github.com/rust-lang/rust/pull/141347)を参照してください。

## `NormalizesTo`ゴール

正規化中の候補優先順位の動作は、[`fn assemble_and_merge_candidates`]で実装されています。

### Where境界はimplをシャドウする

関連アイテムの正規化は、対応するトレイトゴールが`ParamEnv`または`AliasBound`候補を介して証明されている場合、implを考慮しません。
これは、関連型を制約しないwhere境界の場合、関連型が*剛体*のままであることを意味します。

これは、implの適用による不要な領域制約を回避するために必要です。

```rust
trait Trait<'a> {
    type Assoc;
}
impl Trait<'static> for u32 {
    type Assoc = u32;
}

fn bar<'b, T: Trait<'b>>() -> T::Assoc { todo!() }
fn foo<'a>()
where
    u32: Trait<'a>,
{
    // 戻り値の型を正規化するとimplが使用され、
    // `T: Trait` where境界を証明するとwhere境界が使用され、
    // 異なる領域制約が発生します。
    bar::<'_, u32>();
}
```

### 常に`AliasBound`候補を考慮する

where境界が関連アイテムを指定しない場合、トレイトゴールが`ParamEnv`候補を介して証明された場合でも、エイリアスを剛体として扱う代わりに`AliasBound`候補を考慮します。

```rust
trait Super {
    type Assoc;
}
trait Bound {
    type Assoc: Super<Assoc = u32>;
}
trait Trait: Super {}

// 環境を精緻化すると、`T::Assoc: Super` where境界が得られます。
// このwhere境界は、`Super<Assoc = u32>`
// アイテム境界を介した正規化を妨げてはなりません。
fn heck<T: Bound<Assoc: Trait>>(x: <T::Assoc as Super>::Assoc) -> u32 {
    x
}
```

このようなエイリアスを使用すると、追加の領域制約が発生する可能性があります。[#133044]を参照してください。

```rust
trait Bound<'a> {
    type Assoc;
}
trait Trait {
    type Assoc: Bound<'static, Assoc = u32>;
}

fn heck<'a, T: Trait<Assoc: Bound<'a>>>(x: <T::Assoc as Bound<'a>>::Assoc) {
    // 関連型を正規化するには、`Bound<'static>`エイリアス境界を使用する代わりに
    // エイリアスを剛体に保つため、`T::Assoc: Bound<'static>`が必要です。
    drop(x);
}
```

### `ParamEnv`候補を`AliasBound`よりも優先

where境界が関連型を指定しない場合は`AliasBound`候補を使用しますが、指定する場合はwhere境界を優先します。
これは次の例で必要です：

```rust
// `I::IntoIterator: Iterator<Item = ()>`
// where境界を`I::Intoiterator: Iterator<Item = I::Item>`
// エイリアス境界よりも優先することを確認します。

trait Iterator {
    type Item;
}

trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
}

fn normalize<I: Iterator<Item = ()>>() {}

fn foo<I>()
where
    I: IntoIterator,
    I::IntoIter: Iterator<Item = ()>,
{
    // `I::IntoIterator: Iterator<Item = ()>`
    // where境界を`I::Intoiterator: Iterator<Item = I::Item>`
    // エイリアス境界よりも優先する必要があります。
    normalize::<I::IntoIter>();
}
```

### 常にwhere境界を考慮

トレイトゴールがimplを介して証明された場合でも、存在する場合は`ParamEnv`候補を優先します。

#### 「孤立した」where境界を優先

`fn check_type_bounds`でGATおよびRPITITのアイテム境界を正規化する際に、「孤立した」`Projection`句を`ParamEnv`に追加します。
これらの`ParamEnv`候補をimplおよび他のwhere境界よりも優先する必要があります。

```rust
#![feature(associated_type_defaults)]
trait Foo {
    // 以下のimplのために`i32: Baz<Self>`を証明できるはずです。
    // これには`Self::Bar<()>: Eq<i32>`が必要ですが、
    // `for<T> Self::Bar<T> = i32`を仮定するため、これは真です。
    type Bar<T>: Baz<Self> = i32;
}
trait Baz<T: ?Sized> {}
impl<T: Foo + ?Sized> Baz<T> for i32 where T::Bar<()>: Eq<i32> {}
trait Eq<T> {}
impl<T> Eq<T> for T {}
```

この優先順位が実際に必要なケースを完全には理解しておらず、楽しい方法でこれを悪用することもできていませんが 🤷

#### グローバルwhere境界をimplよりも優先

これは次のコンパイルに必要です。実際にこれに依存しているかどうかはわかりません 🤷

```rust
trait Id {
    type This;
}
impl<T> Id for T {
    type This = T;
}

fn foo<T>(x: T) -> <u32 as Id>::This
where
    u32: Id<This = T>,
{
    x
}
```

これは、正規化が追加の領域制約を引き起こす可能性があることを意味します。[#133044]を参照してください。

```rust
trait Trait {
    type Assoc;
}

impl Trait for &u32 {
    type Assoc = u32;
}

fn trait_bound<T: Trait>() {}
fn normalize<T: Trait<Assoc = u32>>() {}

fn foo<'a>()
where
    &'static u32: Trait<Assoc = u32>,
{
    trait_bound::<&'a u32>(); // ok、implを介して証明
    normalize::<&'a u32>(); // error、where境界を介して証明
}
```

[`Candidate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_next_trait_solver/solve/assembly/struct.Candidate.html
[`CandidateSource`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_next_trait_solver/solve/enum.CandidateSource.html
[`fn merge_trait_candidates`]: https://github.com/rust-lang/rust/blob/e3ee7f7aea5b45af3b42b5e4713da43876a65ac9/compiler/rustc_next_trait_solver/src/solve/trait_goals.rs#L1342-L1424
[`fn assemble_and_merge_candidates`]: https://github.com/rust-lang/rust/blob/e3ee7f7aea5b45af3b42b5e4713da43876a65ac9/compiler/rustc_next_trait_solver/src/solve/assembly/mod.rs#L920-L1003
[trait-system-refactor-initiative#76]: https://github.com/rust-lang/trait-system-refactor-initiative/issues/76
[#24066]: https://github.com/rust-lang/rust/issues/24066
[#133044]: https://github.com/rust-lang/rust/issues/133044
