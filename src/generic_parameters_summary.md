# ジェネリックパラメータの定義

この章では、rustc がどのジェネリックパラメータが導入されているかを追跡する方法について説明します。例えば、`struct Foo<T>` が与えられた場合、rustc はどのように `Foo` が型パラメータ `T`（および他のジェネリックパラメータはない）を定義していることを追跡するのでしょうか。

これは、`for<'a>` 構文を介して導入されるジェネリックパラメータ（例：where 句や `fn` 型）を追跡する方法については*カバーしません*。これは [`Binder` に関する章][ch_binders]で説明されています。

# `ty::Generics`

アイテムによって導入されるジェネリックパラメータは、[`ty::Generics`] 構造体によって追跡されます。アイテムが親アイテムで定義されたジェネリックの使用を許可する場合があります。これは、`ty::Generics` 構造体が親アイテムを指定してそのジェネリックパラメータを継承するオプションフィールドを持つことで実現されます。例えば、次のコードが与えられた場合：

```rust,ignore
trait Trait<T> {
    fn foo<U>(&self);
}
```

`foo` に使用される `ty::Generics` には、`[U]` が含まれ、親は `Some(Trait)` です。`Trait` は、`[Self, T]` を含む `ty::Generics` を持ち、親は `None` です。

[`GenericParamDef`] 構造体は、`ty::Generics` リスト内の各個々のジェネリックパラメータを表すために使用されます。`GenericParamDef` 構造体には、ジェネリックパラメータに関する情報が含まれています。例えば、その名前、defid、どの種類のパラメータか（つまり、型、const、ライフタイム）などです。

`GenericParamDef` には、パラメータの位置を表す `u32` インデックスも含まれています（最も外側の親から開始）。これは、ジェネリックパラメータの使用を表すために使用される値です（詳細については、[型の表現に関する章][ch_representing_types]を参照してください）。

興味深いことに、`ty::Generics` は現在、アイテムで定義された_すべての_ジェネリックパラメータを含んでいるわけではありません。関数の場合、_early bound_ パラメータのみが含まれます。

[ch_representing_types]: ./ty.md
[`ty::Generics`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Generics.html
[`GenericParamDef`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/generics/struct.GenericParamDef.html
[ch_binders]: ./ty_module/binders.md
