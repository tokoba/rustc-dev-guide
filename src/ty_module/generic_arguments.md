# ADT とジェネリック引数

`ADT` という用語は「代数的データ型」の略で、Rust ではこれは struct、enum、または union を指します。

## ADT の表現

`MyStruct` が次のように定義されている場合の `MyStruct<u32>` のような型の例を考えてみましょう：

```rust,ignore
struct MyStruct<T> { x: u8, y: T }
```

型 `MyStruct<u32>` は `TyKind::Adt` のインスタンスになります：

```rust,ignore
Adt(&'tcx AdtDef, GenericArgs<'tcx>)
//  ------------  ---------------
//  (1)            (2)
//
// (1) `MyStruct` 部分を表す
// (2) `<u32>` または「置換」/ ジェネリック引数を表す
```

2 つの部分があります：

- [`AdtDef`][adtdef] は、型パラメータの値なしで struct/enum/union を参照します。この例では、これは引数 `u32` *なし* の `MyStruct` 部分です。（HIR では、struct、enum、union は異なる方法で表現されますが、`ty::Ty` では、それらはすべて `TyKind::Adt` を使用して表現されることに注意してください。）
- [`GenericArgs`] は、ジェネリックパラメータに置換される値のリストです。`MyStruct<u32>` の例では、`[u32]` のようなリストになります。ジェネリクスと置換については、もう少し詳しく説明します。

[adtdef]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.AdtDef.html
[`GenericArgs`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.GenericArgs.html

### **`AdtDef` と `DefId`**

ソースコードで定義された各型には、一意の `DefId` があります（[この章](../hir.md#identifiers-in-the-hir)を参照）。これには ADT とジェネリクスが含まれます。上記で与えた `MyStruct<T>` 定義には、2 つの `DefId` があります：`MyStruct` 用と `T` 用です。上記のコードは `u32` の新しい `DefId` を生成しないことに注意してください。なぜなら、そのコードでは定義されていない（参照されているだけ）からです。

`AdtDef` は、多くの便利なヘルパーメソッドを持つ `DefId` のラッパーのようなものです。`AdtDef` と `DefId` の間には、本質的に 1 対 1 の関係があります。[`tcx.adt_def(def_id)` クエリ][adtdefq]を使用して、`DefId` の `AdtDef` を取得できます。`'tcx` ライフタイムが示すように、`AdtDef` はすべてインターンされています。

[adtdefq]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.adt_def

## 質問：なぜ `AdtDef` の「内部」に置換しないのですか？

ジェネリック構造体を `(AdtDef, args)` で表現することを思い出してください。では、なぜこのスキームにこだわるのでしょうか？

代わりに選択できた別の方法は、すべての型がすでに置換されている、常に新しい完全に置換された形式の `AdtDef` を作成することです。これはより面倒がないように思えます。ただし、`(AdtDef, args)` スキームには、これに対していくつかの利点があります。

まず、`(AdtDef, args)` スキームには効率の勝利があります：

```rust,ignore
struct MyStruct<T> {
  ... 100個のフィールド ...
}

// やりたいこと: MyStruct<A> ==> MyStruct<B>
```

このような例では、`A` への 1 つの参照を `B` に置き換えるだけで、`MyStruct<A>` を `MyStruct<B>`（など）として非常に安価にインスタンス化できます。しかし、すべてのフィールドを熱心にインスタンス化した場合、`AdtDef` のすべてのフィールドを調べてすべての型を更新する必要があるため、より多くの作業になる可能性があります。

もう少し深く言うと、これは Rust の構造体が[*名目的*型][nominal]であることに対応しています – これは、それらが*名前*によって定義されることを意味します（そして、その内容はその名前の定義からインデックス付けされ、型自体の「内部」に運ばれるのではありません）。

[nominal]: https://en.wikipedia.org/wiki/Nominal_type_system

## `GenericArgs` 型

ジェネリック型 `MyType<A, B, …>` が与えられたとき、`MyType` のジェネリック引数のリストを保存する必要があります。

rustc では、これは [`GenericArgs`] を使用して行われます。`GenericArgs` は、ジェネリックアイテムのジェネリック引数のリストを表す [`GenericArg`] のスライスへの薄いポインタです。例えば、2 つの型パラメータ `K` と `V` を持つ `struct HashMap<K, V>` が与えられた場合、型 `HashMap<i32, u32>` を表すために使用される `GenericArgs` は `&'tcx [tcx.types.i32, tcx.types.u32]` で表現されます。

`GenericArg` は概念的には 3 つのバリアントを持つ `enum` で、1 つは型引数用、1 つは定数引数用、1 つはライフタイム引数用です。実際には、それは [`GenericArgKind`] によって表現され、[`GenericArg`] はそれを `GenericArgKind` に変換するメソッドを持つ、よりスペース効率の良いバージョンです。

実際の `GenericArg` 構造体は、型、ライフタイム、または定数をインターンされたポインタとして保存し、識別子を下位 2 ビットに保存します。`GenericArgs` 実装を具体的に扱っている場合を除き、一般的に `GenericArg` を直接扱う必要はなく、代わりに `GenericArg::unpack()` メソッドを介して取得可能な安全な [`GenericArgKind`](#genericargkind) 抽象化を利用する必要があります。

場合によっては `GenericArg` を構築する必要があります。これは `Ty/Const/Region::into()` または `GenericArgKind::pack` を介して行うことができます。

```rust,ignore
// ジェネリック引数をアンパックおよびパックする例。
fn deal_with_generic_arg<'tcx>(generic_arg: GenericArg<'tcx>) -> GenericArg<'tcx> {
    // 生の `GenericArg` をアンパックして安全に扱います。
    let new_generic_arg: GenericArgKind<'tcx> = match generic_arg.unpack() {
        GenericArgKind::Type(ty) => { /* ... */ }
        GenericArgKind::Lifetime(lt) => { /* ... */ }
        GenericArgKind::Const(ct) => { /* ... */ }
    };
    // `GenericArgKind` をパックしてジェネリック引数リストに保存します。
    new_generic_arg.pack()
}
```

[`GenericArg`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.GenericArg.html
[`GenericArgKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.GenericArgKind.html

すべてをまとめると：

```rust,ignore
struct MyStruct<T>(T);
type Foo = MyStruct<u32>
```

`Foo` 型エイリアスに書かれた `MyStruct<U>` については、次のように表現します：

- `MyStruct` の `AdtDef`（および対応する `DefId`）があります。
- リスト `[GenericArgKind::Type(Ty(u32))]` を含む `GenericArgs` があります
- そして最後に、上記の `AdtDef` と `GenericArgs` を持つ `TyKind::Adt` があります。
