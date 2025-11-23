# rustcでのシリアライゼーション

rustcはコンパイル中にさまざまなデータを[シリアライズ]およびデシリアライズする必要があります。具体的には：

- 主にクエリ出力で構成される「クレートメタデータ」は、バイナリ形式からライブラリクレートをコンパイルするときに出力される`rlib`ファイルと`rmeta`ファイルにシリアライズされます。これらの`rlib`ファイルと`rmeta`ファイルは、そのライブラリに依存するクレートによってデシリアライズされます。
- 特定のクエリ出力は、[インクリメンタルコンパイルの結果を永続化する][persist incremental compilation results]ためにバイナリ形式でシリアライズされます。
- [`CrateInfo`]は、`-Z no-link`フラグが使用されたときに`JSON`にシリアライズされ、`-Z link-only`フラグが使用されたときに`JSON`からデシリアライズされます。

[`CrateInfo`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/struct.CrateInfo.html
[persist incremental compilation results]: queries/incremental-compilation-in-detail.md#the-real-world-how-persistence-makes-everything-complicated
[シリアライズ]: https://en.wikipedia.org/wiki/Serialization

## `Encodable`トレイトと`Decodable`トレイト

[`rustc_serialize`]クレートは、シリアライズ可能な型のための2つのトレイトを定義しています：

```rust,ignore
pub trait Encodable<S: Encoder> {
    fn encode(&self, s: &mut S) -> Result<(), S::Error>;
}

pub trait Decodable<D: Decoder>: Sized {
    fn decode(d: &mut D) -> Result<Self, D::Error>;
}
```

また、整数型、浮動小数点型、`bool`、`char`、`str`など、さまざまな標準ライブラリの一般的な[プリミティブ型](https://doc.rust-lang.org/std/#primitives)に対する実装も定義しています。

これらの型から構築される型の場合、`Encodable`と`Decodable`は通常[derives]によって実装されます。これらは、デシリアライゼーションを構造体やenumのフィールドに転送する実装を生成します。構造体の場合、それらの実装は次のようになります：

```rust,ignore
#![feature(rustc_private)]
extern crate rustc_serialize;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

struct MyStruct {
    int: u32,
    float: f32,
}

impl<E: Encoder> Encodable<E> for MyStruct {
    fn encode(&self, s: &mut E) -> Result<(), E::Error> {
        s.emit_struct("MyStruct", 2, |s| {
            s.emit_struct_field("int", 0, |s| self.int.encode(s))?;
            s.emit_struct_field("float", 1, |s| self.float.encode(s))
        })
    }
}

impl<D: Decoder> Decodable<D> for MyStruct {
    fn decode(s: &mut D) -> Result<MyStruct, D::Error> {
        s.read_struct("MyStruct", 2, |d| {
            let int = d.read_struct_field("int", 0, Decodable::decode)?;
            let float = d.read_struct_field("float", 1, Decodable::decode)?;

            Ok(MyStruct { int, float })
        })
    }
}
```
[`rustc_serialize`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_serialize/index.html

## アリーナに割り当てられた型のエンコードとデコード

rustcには多数の[アリーナに割り当てられた型][arena allocated types]があります。これらの型をデシリアライズするには、それらが割り当てられる必要があるアリーナへのアクセスが必要です。[`TyDecoder`]トレイトと[`TyEncoder`]トレイトは、[`TyCtxt`]へのアクセスを可能にする[`Decoder`]と[`Encoder`]のサブトレイトです。

`arena`に割り当てられた型を含む型は、その[`Encodable`]および[`Decodable`]実装の型パラメータをこれらのトレイトで制約できます。たとえば：

```rust,ignore
impl<'tcx, D: TyDecoder<'tcx>> Decodable<D> for MyStruct<'tcx> {
    /* ... */
}
```

[`TyEncodable`]および[`TyDecodable`] [derive マクロ][derives]は、このような実装に展開されます。

実際の`arena`に割り当てられた型のデコードは、[orphan rules]のために一部の実装を書けないため、より複雑です。これを回避するために、[`RefDecodable`]トレイトが[`rustc_middle`]で定義されています。これは任意の型に実装できます。`TyDecodable`マクロは参照をデコードするために`RefDecodable`を呼び出しますが、さまざまなジェネリックコードは、型が特定のデコーダーで実際に`Decodable`である必要があります。

インターン化された型の場合、[`ty::Predicate`]のような新しい型ラッパーを使用し、手動で`Encodable`と`Decodable`を実装する方が、`RefDecodable`を手動で実装するよりも簡単かもしれません。

[`Decodable`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_serialize/trait.Decodable.html
[`Decoder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_serialize/trait.Decoder.html
[`Encodable`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_serialize/trait.Encodable.html
[`Encoder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_serialize/trait.Encoder.html
[`RefDecodable`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/codec/trait.RefDecodable.html
[`rustc_middle`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/index.html
[`ty::Predicate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/predicate/struct.Predicate.html
[`TyCtxt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html
[`TyDecodable`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_macros/derive.TyDecodable.html
[`TyDecoder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/codec/trait.TyDecoder.html
[`TyEncodable`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_macros/derive.TyEncodable.html
[`TyEncoder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/codec/trait.TyEncoder.html
[arena allocated types]: memory.md
[derives]: #derive-macros
[orphan rules]:https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules

## Deriveマクロ

[`rustc_macros`]クレートは、`Decodable`と`Encodable`の実装を支援するためのさまざまなderiveを定義しています。

- `Encodable`および`Decodable`マクロは、すべての`Encoders`と`Decoders`に適用される実装を生成します。これらは、[`rustc_middle`]に依存しないクレート、または`TyEncoder`を実装しない型によってシリアライズされる必要があるクレートで使用する必要があります。
- [`MetadataEncodable`]および[`MetadataDecodable`]は、[`rustc_metadata::rmeta::encoder::EncodeContext`]および[`rustc_metadata::rmeta::decoder::DecodeContext`]によるデコードのみを許可する実装を生成します。これらは、[`rustc_metadata::rmeta::`]`Lazy*`を含む型に使用されます。
- `TyEncodable`と`TyDecodable`は、任意の`TyEncoder`または`TyDecoder`に適用される実装を生成します。これらは、クレートメタデータおよび/またはインクリメンタルキャッシュでのみシリアライズされる型に使用する必要があります。これは`rustc_middle`のほとんどのシリアライズ可能な型に該当します。

[`MetadataDecodable`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_macros/derive.MetadataDecodable.html
[`MetadataEncodable`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_macros/derive.MetadataEncodable.html
[`rustc_macros`]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_macros
[`rustc_metadata::rmeta::`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/rmeta/index.html
[`rustc_metadata::rmeta::decoder::DecodeContext`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/rmeta/decoder/struct.DecodeContext.html
[`rustc_metadata::rmeta::encoder::EncodeContext`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/rmeta/encoder/struct.EncodeContext.html
[`rustc_middle`]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_middle

## 短縮形

`Ty`は深く再帰的になる可能性があるため、各`Ty`を素朴にエンコードすると、クレートメタデータが非常に大きくなります。これを処理するために、各`TyEncoder`は、型をシリアライズした出力内の場所のキャッシュを持っています。エンコードされる型がキャッシュにある場合、通常のように型をシリアライズする代わりに、書き込まれるファイル内のバイトオフセットがエンコードされます。`ty::Predicate`にも同様のスキームが使用されています。

## `LazyValue<T>`

クレートメタデータは、`TyCtxt<'tcx>`が作成される前に最初にロードされるため、一部のデシリアライゼーションはメタデータの最初のロードから延期される必要があります。[`LazyValue<T>`]型は、`T`がシリアライズされたクレートメタデータ内の（相対的な）オフセットをラップします。また、いくつかのバリアントもあります：[`LazyArray<T>`]および[`LazyTable<I, T>`]。

`LazyArray<[T]>`および`LazyTable<I, T>`型は、`Lazy<Vec<T>>`および`Lazy<HashMap<I, T>>`に対していくつかの機能を提供します：

- `LazyArray<T>`を`Iterator`から直接エンコードすることができ、最初に`Vec<T>`に収集する必要はありません。
- `LazyTable<I, T>`へのインデックスアクセスは、読み取られているエントリ以外のエントリをデコードする必要はありません。

**注意**：`LazyValue<T>`は、最初にデシリアライズされた後、その値をキャッシュしません。代わりに、クエリシステム自体がこれらの結果をキャッシュする主な方法です。

[`LazyArray<T>`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/rmeta/struct.LazyValue.html
[`LazyTable<I, T>`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/rmeta/struct.LazyValue.html
[`LazyValue<T>`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/rmeta/struct.LazyValue.html

## 特殊化

いくつかの型、特に`DefId`は、異なる`Encoder`に対して異なる実装を持つ必要があります。これは現在、アドホックな特殊化によって処理されています。たとえば、`DefId`には`Encodable<E>`の`default`実装と、`Encodable<CacheEncoder>`の特殊化された実装があります。
