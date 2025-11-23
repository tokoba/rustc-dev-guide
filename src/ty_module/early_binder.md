# `EarlyBinder` とパラメータのインスタンス化

ジェネリックパラメータ `T` を導入するアイテムが与えられたとき、`foo` の外部から `foo` 内部の型を参照するときはいつでも（つまり、戻り値の型や引数の型）、`foo` で定義されたジェネリックパラメータを処理するように注意する必要があります。例として：

```rust,ignore
fn foo<T, U>(a: T, _b: U) -> T { a }

fn main() {
    let c = foo::<i32, u128>(1, 2);
}
```

`main` を型チェックするとき、単純に `foo` の戻り値の型を見て、変数 `c` に型 `T` を割り当てることはできません。関数 `main` はジェネリックパラメータを定義しておらず、`T` はこのコンテキストでは完全に無意味です。より一般的に、アイテムがジェネリックパラメータを導入（バインド）するときはいつでも、外部からアイテム内部の型にアクセスするときは、ジェネリックパラメータを外部アイテムからの値でインスタンス化する必要があります。

rustc では、これを [`EarlyBinder`] 型を介して追跡します。`foo` の戻り値の型は `EarlyBinder<Ty>` として表現され、`Ty` にアクセスする唯一の方法は、`Ty` が使用している可能性のあるジェネリックパラメータの引数を提供することです。これは、バインダーを解除し、すべてのジェネリックパラメータを提供された引数で置き換えた内部値を返す [`EarlyBinder::instantiate`] メソッドを介して実装されます。

例に戻ると、`main` を型チェックするとき、`foo` の戻り値の型は `EarlyBinder(T/#0)` として表現されます。次に、ジェネリック引数に `i32, u128` を指定して関数を呼び出したため、args に `[i32, u128]` を指定して戻り値の型に `EarlyBinder::instantiate` を呼び出します。これにより、ローカル `c` の型として使用できるインスタンス化された戻り値の型 `i32` が生成されます。

いくつかの例を示します：

```rust,ignore
fn foo<T>() -> Vec<(u32, T)> { Vec::new() }
fn bar() {
    // インスタンス化する前の `foo` の戻り値の型は次のようになります：
    // `EarlyBinder(Adt(Vec, &[Tup(&[u32, T/#=0])]))`
    // 次に、`[u64]` でバインダーをインスタンス化し、次の型になります：
    // `Adt(Vec, &[Tup(&[u32, u64])])`
    let a = foo::<u64>();
}
```

```rust,ignore
struct Foo<A, B> {
    x: Vec<A>,
    ..
}

fn bar(foo: Foo<u32, f32>) {
    // インスタンス化する前の `foo` の `x` フィールドの型は次のようになります：
    // `EarlyBinder(Vec<A/#0>)`
    // 次に、`Foo` 構造体へのジェネリック引数である `[u32, f32]` で
    // バインダーをインスタンス化します。これにより、次の型になります：
    // `Vec<u32>`
    let y = foo.x;
}
```

コンパイラでは、この `instantiate` 呼び出しは [`FieldDef::ty`]（[src][field_def_ty_src]）で行われます。`bar` を型チェックする際のある時点で、`foo.x` の型を取得するために `FieldDef::ty(x, &[u32, f32])` を呼び出すことになります。

**インデックスに関する注意：** `Param` のインデックスが `EarlyBinder` がバインドするものと一致しない場合、それはバグです。例えば、インデックスが範囲外であるか、ライフタイムのインデックスが型パラメータに対応している場合などです。これらの種類のエラーは、アイテムによって導入されたジェネリックパラメータへの参照を、内部アイテムから名前付け可能であってはならないものを禁止するコンパイラの早い段階の名前解決中にキャッチされます。

[`FieldDef::ty`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.FieldDef.html#method.ty
[field_def_ty_src]: https://github.com/rust-lang/rust/blob/44d679b9021f03a79133021b94e6d23e9b55b3ab/compiler/rustc_middle/src/ty/mod.rs#L1421-L1426
[`EarlyBinder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.EarlyBinder.html
[`EarlyBinder::instantiate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.EarlyBinder.html#method.instantiate

---

前述のように、アイテムの _外部_ にいるときは、内部の値にアクセスする前にジェネリック引数で `EarlyBinder` をインスタンス化することが重要ですが、概念的にバインダーの内部にすでにいる場合のセットアップは少し異なります。

例：

```rust
impl<T> Trait for Vec<T> {
    fn foo(&self, b: Self) {}
}
```

`b` パラメータの型を表す `Ty` を構築するとき、内部にいる impl の `Self` の型を取得する必要があります。これは、impl の `DefId` で [`type_of`] クエリを呼び出すことで取得できますが、これは `EarlyBinder<Ty>` を返します。なぜなら、impl ブロックは、impl の外部にいる場合に解除する必要があるかもしれないジェネリックパラメータをバインドするからです。

`EarlyBinder` 型は、「すでにその内部にいる」ときにバインダーを解除するための [`instantiate_identity`] 関数を提供します。これは、事実上、`EarlyBinder::instantiate(GenericArgs::identity_for_item(..))` と書くよりもパフォーマンスが高いバージョンです。概念的には、これはルート宇宙のプレースホルダーでインスタンス化することによってバインダーを解除します（これが何を意味するかについては、次のいくつかの章で説明します）。しかし、実際には、単に変更を加えずに内部値を返すだけです。

[`type_of`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.type_of
[`instantiate_identity`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.EarlyBinder.html#method.instantiate_identity
