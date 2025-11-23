# Langアイテム

コンパイラには、特定のプラグ可能な操作があります。つまり、言語にハードコードされていないが、
ライブラリに実装されており、それが存在することをコンパイラに伝える特別なマーカーがある機能です。
マーカーは属性`#[lang = "..."]`であり、`...`のさまざまな値、
つまりさまざまな「langアイテム」があります。

多くのlangアイテムは、`add`（`trait core::ops::Add`）や
`future_trait`（`trait core::future::Future`）など、
1つの賢明な方法でのみ実装できます。他のものは、特定の目標を達成するために
オーバーライドできます。たとえば、バイナリのエントリポイントを制御できます。

langアイテムによって提供される機能には、次のものがあります。

- トレイトによるオーバーロード可能な演算子：`==`、`<`、逆参照（`*`）、`+`などの
  演算子に対応するトレイトはすべて、langアイテムでマークされています。
  これらの特定の4つは、それぞれ`eq`、`ord`、`deref`、`add`です。
- パニックとスタック巻き戻し：`eh_personality`、`panic`、
  `panic_bounds_checks` langアイテム。
- コンパイラが使用する型のプロパティを示すために使用される`std::marker`のトレイト。
  langアイテム`send`、`sync`、`copy`。
- `core::marker`にある分散インジケータに使用される特別なマーカー型。
  langアイテム`phantom_data`。

Langアイテムはコンパイラによって遅延ロードされます。たとえば、`Box`を使用しない場合、
`exchange_malloc`と`box_free`の関数を定義する必要はありません。`rustc`は、
アイテムが必要だが、現在のクレートまたはそれが依存するクレートで見つからない場合、
エラーを出力します。

ほとんどのlangアイテムは`core`ライブラリによって定義されていますが、
`#![no_std]`で実行可能ファイルをビルドしようとしている場合、
通常`std`によって提供されるいくつかのlangアイテムを定義する必要があります。

## 言語アイテムの取得

[`tcx.lang_items()`]を呼び出すことで、langアイテムを取得できます。

`trait Sized {}`言語アイテムを取得する小さな例を次に示します。

```rust
// Note that in case of `#![no_core]`, the trait is not available.
if let Some(sized_trait_def_id) = tcx.lang_items().sized_trait() {
    // do something with `sized_trait_def_id`
}
```

`sized_trait()`は、`DefId`自体ではなく、`Option`を返すことに注意してください。
これは、langアイテムが標準ライブラリで定義されているため、誰かが
`#![no_core]`（または一部のlangアイテムでは`#![no_std]`）でコンパイルした場合、
langアイテムが存在しない可能性があるためです。
次のいずれかを実行できます。

- 続行するためにlangアイテムが必要な場合は、ハードエラーを発生させます（これはユーザーコードで
  発生する可能性があるため、パニックしないでください）。
- `DefId`で行う予定だったことを省略するだけで、限定された機能で続行します。

[`tcx.lang_items()`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.lang_items

## すべての言語アイテムのリスト

次の場所で言語アイテムを見つけることができます。
- コンパイラドキュメントの網羅的なリファレンス：[`rustc_hir::LangItem`]
- ripgrepを使用してソースの場所を含む自動生成されたリスト：`rg '#\[.*lang =' library/`

言語アイテムは明示的に不安定であり、新しいリリースで変更される可能性があることに注意してください。

[`rustc_hir::LangItem`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/lang_items/enum.LangItem.html
