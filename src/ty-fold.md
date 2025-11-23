<!-- date-check: may 2024 -->
# `TypeFoldable` と `TypeFolder`

[前の章][a previous chapter]で、バインダーのインスタンス化について議論しました。これには、バインドされた変数のすべての使用を見つけて置き換えるために、`Early(Binder)` の内部のすべてを調べることが含まれます。バインダーは、単なる `Ty` ではなく、任意の Rust 型 `T` をラップできます。では、`Early/Binder` 型に `instantiate` メソッドをどのように実装するのでしょうか？

答えは、いくつかのトレイトです：[`TypeFoldable`] と [`TypeFolder`]。

- `TypeFoldable` は、型情報を埋め込む型によって実装されます。これにより、`TypeFoldable` の内容を再帰的に処理して、それらに対して処理を行うことができます。
- `TypeFolder` は、`TypeFoldable` を処理している間に遭遇する型に対して何をしたいかを定義します。

例えば、`TypeFolder` トレイトには [`fold_ty`] メソッドがあり、型を入力として受け取り、結果として新しい型を返します。`TypeFoldable` は、自身に対して `TypeFolder` の `fold_foo` メソッドを呼び出し、`TypeFolder` にその内容（含まれている型、リージョンなど）へのアクセスを与えます。

これを、Rust で愛用されているイテレータコンビネータへの類推で考えることができます：

```rust,ignore
vec.iter().map(|e1| foo(e2)).collect()
//             ^^^^^^^^^^^^ `TypeFolder` に類似
//         ^^^ `TypeFoldable` に類似
```

要約すると：

- `TypeFolder` は「マップ」操作を定義するトレイトです。
- `TypeFoldable` は、型を埋め込むものによって実装されるトレイトです。

`subst` の場合、それが `TypeFolder` として実装されていることがわかります：[`ArgFolder`]。その実装を見ると、実際の置換が行われている場所がわかります。

ただし、実装が `super_fold_with` メソッドを呼び出していることにも気付くかもしれません。それは何でしょうか？これは `TypeFoldable` のメソッドです。次の `TypeFoldable` 型 `MyFoldable` を考えてみましょう：

```rust,ignore
struct MyFoldable<'tcx> {
  def_id: DefId,
  ty: Ty<'tcx>,
}
```

`TypeFolder` は、`MyFoldable` のフィールドの一部を新しい値に置き換えたいだけの場合、`MyFoldable` に対して `super_fold_with` を呼び出すことができます。代わりに `MyFoldable` 全体を別のものに置き換えたい場合は、`fold_with` を呼び出します（`TypeFoldable` の別のメソッド）。

ほとんどすべての場合、構造体全体を置き換えたくはありません。構造体内の `ty::Ty` のみを置き換えたいので、通常は `super_fold_with` を呼び出します。`MyFoldable` が持つ可能性のある典型的な実装は、次のようなことを行うかもしれません：

```rust,ignore
my_foldable: MyFoldable<'tcx>
my_foldable.subst(..., subst)

impl TypeFoldable for MyFoldable {
  fn super_fold_with(&self, folder: &mut impl TypeFolder<'tcx>) -> MyFoldable {
    MyFoldable {
      def_id: self.def_id.fold_with(folder),
      ty: self.ty.fold_with(folder),
    }
  }

  fn super_visit_with(..) { }
}
```

ここでは、`MyFoldable` のフィールドを処理し、*それら*に対して `fold_with` を呼び出すために `super_fold_with` を実装していることに注意してください。つまり、フォルダは `def_id` と `ty` を置き換える可能性がありますが、`MyFoldable` 構造体全体は置き換えません。

物事をまとめるためのもう 1 つの例を示します：`Vec<Vec<X>>` のような型があるとします。`ty::Ty` は次のようになります：`Adt(Vec, &[Adt(Vec, &[Param(X)])])`。`subst(X => u32)` を実行したい場合は、最初に全体的な型を見ます。外側レベルで行う置換がないことがわかるので、1 レベル下がり、`Adt(Vec, &[Param(X)])` を見ます。ここでもまだ行う置換がないので、再び下がります。今、`Param(X)` を見ています。これは置換できるので、`u32` に置き換えます。これ以上下がることはできないので、完了です。全体的な結果は `Adt(Vec, &[Adt(Vec, &[u32])])` です。

最後に言及すべきことが 1 つあります：`TypeFoldable` を折りたたむときに、ほとんどのことを変更したくないことがよくあります。型に到達したときにのみ何かをしたいのです。つまり、基本的にフィールドの `TypeFoldable` 実装に転送するだけの `TypeFoldable` 型が多数ある可能性があります。このような `TypeFoldable` の実装は、手で書くのが非常に退屈な傾向があります。このため、`#![derive(TypeFoldable)]` を使用できる `derive` マクロがあります。これは[ここ][here]で定義されています。

**`subst`** 置換の場合、[実際のフォルダ][actual folder]は、すでに言及したインデックス付けを行います。そこで `Folder` を定義し、`TypeFoldable` に対して `fold_with` を呼び出して自分自身を処理します。次に、[fold_ty] は、処理する各型を見て `ty::Param` を探し、それらを置換のリストから何かに置き換え、そうでなければ型を再帰的に処理するメソッドです。それを置き換えるには、[ty_for_param] を呼び出し、それがすることは、`Param` のインデックスで置換のリストにインデックスを付けるだけです。

[a previous chapter]: ty_module/instantiating_binders.md
[`TypeFoldable`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/trait.TypeFoldable.html
[`TypeFolder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/trait.TypeFolder.html
[`fold_ty`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/trait.TypeFolder.html#method.fold_ty
[`ArgFolder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/binder/struct.ArgFolder.html
[here]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_macros/src/type_foldable.rs
[actual folder]: https://github.com/rust-lang/rust/blob/75ff3110ac6d8a0259023b83fd20d7ab295f8dd6/src/librustc_middle/ty/subst.rs#L440-L451
[fold_ty]: https://github.com/rust-lang/rust/blob/75ff3110ac6d8a0259023b83fd20d7ab295f8dd6/src/librustc_middle/ty/subst.rs#L512-L536
[ty_for_param]: https://github.com/rust-lang/rust/blob/75ff3110ac6d8a0259023b83fd20d7ab295f8dd6/src/librustc_middle/ty/subst.rs#L552-L587
