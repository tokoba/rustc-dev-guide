# パラメータ `Ty`/`Const`/`Region`

ジェネリックアイテムの内部では、スコープ内のジェネリックパラメータを使用する型を書くことができます。例えば、`fn foo<'a, T>(_: &'a Vec<T>)` です。この特定のケースでは、`&'a Vec<T>` 型は内部的に次のように表現されます：
```
TyKind::Ref(
  RegionKind::LateParam(DefId(foo), DefId(foo::'a), "'a"),
  TyKind::Adt(Vec, &[TyKind::Param("T", 0)])
)
```

ジェネリックパラメータの使用を表現する 3 つの別々の方法があります：
- [`TyKind::Param`]/[`ConstKind::Param`]/[`RegionKind::EarlyParam`] 早期バウンドジェネリックパラメータ用（注：すべての型および定数パラメータは早期バウンドと見なされます。詳細については、[早期 vs 遅延バウンドパラメータに関する章][ch_early_late_bound]を参照してください）
- [`TyKind::Bound`]/[`ConstKind::Bound`]/[`RegionKind::Bound`] 高階バウンドまたは高階型によって導入されたパラメータへの参照用、つまり `for<'a> fn(&'a u32)` または `for<'a> T: Trait<'a>`。これは [`Binder` に関する章][ch_binders]で議論されています。
- [`RegionKind::LateParam`] 遅延バウンドライフタイムパラメータ用。`LateParam` は [`Binder` のインスタンス化に関する章][ch_instantiating_binders]で議論されています。

この章では、`TyKind::Param`、`ConstKind::Param`、および `RegionKind::EarlyParam` のみを扱います。

## Ty/Const パラメータ

`TyKind::Param` と `ConstKind::Param` は同一に実装されているため、このセクションでは簡略化のために `TyKind::Param` のみを参照します。ただし、ここでのすべてが `ConstKind::Param` にも当てはまることに留意してください

各 `TyKind::Param` には 2 つのものが含まれています：パラメータの名前とインデックス。

`TyKind::Param` の使用の次の具体例を参照してください：
```rust,ignore
struct Foo<T>(Vec<T>);
```
`Vec<T>` 型は `TyKind::Adt(Vec, &[GenericArgKind::Type(Param("T", 0))])` として表現されます。

名前はいくらか自明です。それは型パラメータの名前です。型パラメータのインデックスは、スコープ内のジェネリックパラメータのリストでのその順序を示す整数です（注：これには、パラメータが定義されているアイテムよりも外側のスコープのアイテムで定義されたパラメータも含まれます）。次の例を考えてみましょう：

```rust,ignore
struct Foo<A, B> {
  // A はインデックス 0 を持ちます
  // B はインデックス 1 を持ちます

  .. // いくつかのフィールド
}
impl<X, Y> Foo<X, Y> {
  fn method<Z>() {
    // ここでは、X、Y、Z がすべてスコープ内にあります
    // X はインデックス 0 を持ちます
    // Y はインデックス 1 を持ちます
    // Z はインデックス 2 を持ちます
  }
}
```

具体的には、パラメータが定義されているアイテムの `ty::Generics` が与えられたとき、インデックスが `2` の場合、ルート `parent` から開始して、3 番目に導入されるパラメータになります。例えば、上記の例では、`Z` はインデックス `2` を持ち、`impl` ブロックから開始して 3 番目に導入されるジェネリックパラメータです。

インデックスは `Ty` を完全に定義し、コンパイルしているコードについて推論するために重要な `TyKind::Param` の唯一の部分です。

一般的に、名前は気にせず、インデックスのみを使用します。名前は診断およびデバッグログに含まれています。そうでなければ、出力を理解することが非常に困難になるためです。つまり、`Vec<Param(0)>: Sized` 対 `Vec<T>: Sized` です。デバッグ出力では、パラメータ型はしばしば `{name}/#{index}` として出力されます。例えば、関数 `foo` で `Vec<T>` をデバッグ出力すると、`Vec<T/#0>` と書かれます。

代替表現は名前のみを持つことですが、インデックスを使用する方がより効率的です。なぜなら、ジェネリック引数でジェネリックパラメータをインスタンス化するときに `GenericArgs` にインデックスを付けることができるからです。そうでなければ、`GenericArgs` を `HashMap<Symbol, GenericArg>` として保存し、ジェネリックアイテムを使用するたびにハッシュマップルックアップを行う必要があります。

理論的には、インデックスは、同じ名前を使用する複数の異なるパラメータを持つことも可能にします。例えば、`impl<A> Foo<A> { fn bar<A>() { .. } }`。シャドウイングに対する規則はこれを困難にしますが、それらの言語規則は将来変わる可能性があります。

### ライフタイムパラメータ

`Ty`/`Const` の単一の `Param` バリアントとは対照的に、ライフタイムにはリージョンパラメータを表現するための 2 つのバリアントがあります：[`RegionKind::EarlyParam`] と [`RegionKind::LateParam`]。この理由は、関数が[早期および遅延バウンドパラメータ][ch_early_late_bound]を区別するためです。これは前の章で議論されています（リンクを参照）。

`RegionKind::EarlyParam` は `Ty/Const` の `Param` バリアントと同一に構造化されています。それは単に `u32` インデックスと `Symbol` です。非関数アイテムで定義されたライフタイムパラメータには、常に `ReEarlyParam` を使用します。関数の場合、早期バウンドパラメータには `ReEarlyParam` を使用し、遅延バウンドパラメータには `ReLateParam` を使用します。`Ty` および `Const` パラメータと同様に、しばしばそれらを `'SYMBOL/#INDEX` としてデバッグフォーマットします。例えば、次を参照してください：

```rust,ignore
// この関数は、そのシグネチャが次のように表現されます：
//
// ```
// fn(
//     T/#2,
//     Ref('a/#0, Ref(ReLateParam(...), u32))
// ) -> Ref(ReLateParam(...), u32)
// ```
fn foo<'a, 'b, T: 'a>(one: T, two: &'a &'b u32) -> &'b u32 {
    ...
}
```

`RegionKind::LateParam` は、[バインダーのインスタンス化に関する章][ch_instantiating_binders]でさらに議論されています。

[ch_early_late_bound]: ../early_late_parameters.md
[ch_binders]: ./binders.md
[ch_instantiating_binders]: ./instantiating_binders.md
[`BoundRegionKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/enum.BoundRegionKind.html
[`RegionKind::EarlyParam`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.RegionKind.html#variant.ReEarlyParam
[`RegionKind::LateParam`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.RegionKind.html#variant.ReLateParam
[`ConstKind::Param`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.ConstKind.html#variant.Param
[`TyKind::Param`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.TyKind.html#variant.Param
[`TyKind::Bound`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.TyKind.html#variant.Bound
[`ConstKind::Bound`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.ConstKind.html#variant.Bound
[`RegionKind::Bound`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.RegionKind.html#variant.ReBound
