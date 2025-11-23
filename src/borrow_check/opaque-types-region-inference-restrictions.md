# 不透明型の領域推論の制限

この章では、不透明型の隠れた型を定義する際にジェネリック引数に課される様々な制限について説明します
`Opaque<'a, 'b, .., A, B, ..> := SomeHiddenType`。

これらの制限は、不透明型推論の最終ステップであるため、borrow checking で実装されています（[ソース][source-borrowck-opaque]）。

[source-borrowck-opaque]: https://github.com/rust-lang/rust/blob/435b5255148617128f0a9b17bacd3cc10e032b23/compiler/rustc_borrowck/src/region_infer/opaque_types.rs

## 背景: 型と const ジェネリック引数
型引数の場合、2 つの制限が必要です: 各型引数は (1) 型パラメータでなければならず、(2) ジェネリック引数の中で一意でなければなりません。
同じことが const 引数にも適用されます。

ケース (1) の例:
```rust
type Opaque<X> = impl Sized;

// `T` は型パラメータです。
// Opaque<T> := ();
fn good<T>() -> Opaque<T> {}

// `()` は型パラメータではありません。
// Opaque<()> := ();
fn bad() -> Opaque<()> {} //~ ERROR
```

ケース (2) の例:
```rust
type Opaque<X, Y> = impl Sized;

// `T` と `U` はジェネリック引数の中で一意です。
// Opaque<T, U> := T;
fn good<T, U>(t: T, _u: U) -> Opaque<T, U> { t }

// `T` がジェネリック引数に 2 回現れます。
// Opaque<T, T> := T;
fn bad<T>(t: T) -> Opaque<T, T> { t } //~ ERROR
```
**動機:** 最初のケース `Opaque<()> := ()` では、隠れた型は 2 つの異なる解釈と互換性があるため曖昧です: `Opaque<X> := X` と `Opaque<X> := ()`。
同様に、2 番目のケース `Opaque<T, T> := T` では、`Opaque<X, Y> := X` と解釈すべきか、`Opaque<X, Y> := Y` と解釈すべきかが曖昧です。
この曖昧さのため、両方のケースは無効な定義使用として拒否されます。

## 一意性の制限

各ライフタイム引数は引数リストの中で一意でなければならず、`'static` であってはなりません。
これは、型パラメータの場合と同様に、隠れた型推論での曖昧さを避けるためです。
例えば、以下の無効な定義使用 `Opaque<'static> := Inv<'static>` は、`Opaque<'x> := Inv<'static>` と `Opaque<'x> := Inv<'x>` の両方と互換性があります。

```rust
type Opaque<'x> = impl Sized + 'x;
type Inv<'a> = Option<*mut &'a ()>;

fn good<'a>() -> Opaque<'a> { Inv::<'static>::None }

fn bad() -> Opaque<'static> { Inv::<'static>::None }
//~^ ERROR
```

```rust
type Opaque<'x, 'y> = impl Trait<'x, 'y>;

fn good<'a, 'b>() -> Opaque<'a, 'b> {}

fn bad<'a>() -> Opaque<'a, 'a> {}
//~^ ERROR
```

**意味的なライフタイムの等価性:**
型パラメータと比較したライフタイムの 1 つの複雑さは、構文的に異なる 2 つのライフタイムが意味的に等しい可能性があることです。
したがって、ライフタイムが一意であることを検証する際には注意が必要です。

```rust
// これも無効です。なぜなら、`'a` は `'static` と*意味的に*等しいからです。
fn still_bad_1<'a: 'static>() -> Opaque<'a> {}
//~^ Should error!

// これも無効です。なぜなら、`'a` と `'b` は*意味的に*等しいからです。
fn still_bad_2<'a: 'b, 'b: 'a>() -> Opaque<'a, 'b> {}
//~^ Should error!
```

## 一意性ルールの例外

上記の一意性ルールの例外は、不透明型の定義の境界が、ライフタイムパラメータが別のパラメータまたは `'static` ライフタイムと等しいことを要求する場合です。
```rust
// 定義は `'x` が `'static` と等しいことを要求します。
type Opaque<'x: 'static> = impl Sized + 'x;

fn good() -> Opaque<'static> {}
```

**動機:** RPIT の一意性制限を実装しようとすると、[crater によって見つかった破損](https://github.com/rust-lang/rust/pull/112842#issuecomment-1610057887) が発生しました。
これは、このルールの例外によって緩和できます。
それ以外の場合に壊れるコードの例:
```rust
struct Type<'a>(&'a ());
impl<'a> Type<'a> {
    // `'b == 'a`
    fn do_stuff<'b: 'a>(&'b self) -> impl Trait<'a, 'b> {}
}
```

**なぜこれが正しいのか:** `Opaque<'a, 'a> := &'a str` のような定義使用の場合、
どちらの方法でも解釈できます — `Opaque<'x, 'y> := &'x str` または `Opaque<'x, 'y> := &'y str` として解釈でき、well-formedness ルールに従って `Opaque` のすべての使用が両方のパラメータが等しいことを保証するため、どちらでも問題ありません。

## 全称ライフタイムの制限

不透明型引数では、全称量化されたライフタイムのみが許可されます。
これには、ライフタイムパラメータとプレースホルダーが含まれます。

```rust
type Opaque<'x> = impl Sized + 'x;

fn test<'a>() -> Opaque<'a> {
    // `Opaque<'empty> := ()`
    let _: Opaque<'_> = ();
    //~^ ERROR
}
```

**動機:**
これにより、ライフタイムと型引数が一貫して動作するようになりますが、これはおまけにすぎません。
この制限の背後にある本当の理由は純粋に技術的なものです。[member constraints] アルゴリズムには基本的な制限があります:
不透明型定義 `Opaque<'?1> := &'?2 u8` に遭遇すると、メンバー制約 `'?2 member-of ['static, '?1]` が登録されます。
アルゴリズムが正しい選択を選ぶためには、選択領域 `['static, '?1]` 間の「outlives」関係の*完全な*セットが、領域推論を行う*前に*既に知られている必要があります。これは、各選択領域が次のいずれかである場合にのみ満たすことができます:
1. 全称領域、つまり `RegionKind::Re{EarlyParam,LateParam,Placeholder,Static}`。
なぜなら、全称領域間の関係は、明示的および暗黙の境界から、領域推論の前に完全に知られているからです。
1. または全称領域と「厳密に等しい」存在領域。
厳密なライフタイムの等価性は以下で定義され、ここで必要とされるのは、完全な領域推論の前に評価できる唯一の等価性のタイプであるためです。

**厳密なライフタイムの等価性:**
2 つのライフタイムが厳密に等しいとは、それらの間に双方向の outlives 制約がある場合を言います。NLL の用語では、これはライフタイムが同じ [SCC] の一部であることを意味します。
重要なことは、このタイプの等価性は完全な領域推論の前に評価できることです（ただし、もちろん制約収集の後です）。
もう 1 つのタイプの等価性は、厳密に等しくなくても、領域推論が 2 つのライフタイム変数に同じ値を与える場合です。
違いを混同していた方法については、[#113971] を参照してください。

[#113971]: https://github.com/rust-lang/rust/issues/113971
[SCC]: https://en.wikipedia.org/wiki/Strongly_connected_component
[member constraints]: ./region_inference/member_constraints.md

**「領域を一度だけモジュロ」制限との相互作用**
上記の例では、シグネチャの不透明型は `Opaque<'a>` であり、無効な定義使用の不透明型は `Opaque<'empty>` であることに注意してください。
提案された MiniTAIT プラン、すなわち [「領域を一度だけモジュロ」][#116935] ルールでは、これを既に許可していません。
「全称ライフタイム」制限が「MiniTAIT」制限から論理的に従うため、冗長になるように見えるかもしれませんが、ライフタイムの等価性とクロージャに関するその後の関連する議論は引き続き関連しています。

[#116935]: https://github.com/rust-lang/rust/pull/116935


## クロージャの制限

不透明型がクロージャ/コルーチン/inline-const 本体で定義されている場合、クロージャの「外部」である全称ライフタイムは不透明型引数で許可されません。
外部領域は [`RegionClassification::External`][source-external-region] で定義されています

[source-external-region]: https://github.com/rust-lang/rust/blob/caf730043232affb6b10d1393895998cb4968520/compiler/rustc_borrowck/src/universal_regions.rs#L201.

例:（これは現在の nightly でたまたまコンパイルされますが、より実用的な例は、既に混乱を招くエラーで拒否されています。）
```rust
type Opaque<'x> = impl Sized + 'x;

fn test<'a>() -> Opaque<'a> {
    let _ = || {
        // `'a` はクロージャの外部です
        let _: Opaque<'a> = ();
        //~^ Should be an error!
    };
    ()
}
```

**動機:**
クロージャ本体では、外部ライフタイムは「全称」ライフタイムとして分類されていますが、それらの間の関係が事前にわかっていないという点で、存在ライフタイムのように振る舞います。代わりに、それらの値は存在ライフタイムと同じように推論され、要件は親 fn に伝播されます。これは、上記で説明したように、メンバー制約アルゴリズムを壊します:
> アルゴリズムが正しい選択を選ぶためには、選択領域 `['static, '?1]` 間の outlives 関係の完全なセットが、領域推論を行う前に既に知られている必要があります

詳細を説明する例:

```rust
type Opaque<'x, 'y> = impl Sized;

//
fn test<'a, 'b>(s: &'a str) -> impl FnOnce() -> Opaque<'a, 'b> {
    move || { s }
    //~^ ERROR hidden type for `Opaque<'_, '_>` captures lifetime that does not appear in bounds
}

// 上記のクロージャ本体は次のように脱糖されます:
fn test::{closure#0}(_upvar: &'?8 str) -> Opaque<'?6, '?7> {
    return _upvar
}

// ここで、`['?8, '?6, ?7]` はクロージャの*外部*の全称ライフタイムです。
// クロージャの*内部*では、それらの間に既知の関係はありません。
// しかし、親 fn では `'?6: '?8` が知られています。
//
// 不透明定義 `Opaque<'?6, '?7> := &'8 str` に遭遇すると、
// メンバー制約アルゴリズムは `?8 = '?6` を安全に行うのに十分な情報を持っていません。
// このため、適切なメッセージでエラーを出します:
// "hidden type captures lifetime that does not appear in bounds"。
```

これらの制限がなければ、エラーメッセージは混乱を招き、さらに重要なことに、メンバー制約がクロージャで非常に壊れているため、将来的に壊れる可能性の高いコードを受け入れるリスクがあります。

**出力型:**
これが実世界のコードで問題を引き起こす最も一般的なシナリオは、クロージャ/async ブロックの出力型だと思います。クロージャと async ブロックの間には不一致があることに注意する価値があります。これはこの問題をさらに示しており、[`replace_opaque_types_with_inference_vars` のハック][source-replace-opaques] に起因しています。これは future にのみ適用されます。

[source-replace-opaques]: https://github.com/rust-lang/rust/blob/9cf18e98f82d85fa41141391d54485b8747da46f/compiler/rustc_hir_typeck/src/closure.rs#L743

```rust
type Opaque<'x> = impl Sized + 'x;
fn test<'a>() -> impl FnOnce() -> Opaque<'a> {
    // クロージャの出力型は Opaque<'a> です
    // -> 隠れた型定義はクロージャの*内部*で発生します
    // -> 拒否されます。
    move || {}
    //~^ ERROR expected generic lifetime parameter, found `'_`
}
```
```rust
use std::future::Future;
type Opaque<'x> = impl Sized + 'x;
fn test<'a>() -> impl Future<Output = Opaque<'a>> {
    // async ブロックの出力型はユニット `()` です
    // -> 隠れた型定義は親 fn で発生します
    // -> 受け入れられます。
    async move {}
}
```
