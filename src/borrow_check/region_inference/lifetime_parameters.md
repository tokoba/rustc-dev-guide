# 全称領域

「全称領域」は、コードが「名前付きライフタイム」を参照するために使用する名前です -- 例えば、ライフタイムパラメータと `'static`。名前は、そのようなライフタイムが「全称量化されている」という事実に由来します（つまり、それらのライフタイムのすべての値に対してコードが真であることを確認する必要があります）。領域推論中にライフタイムパラメータがどのように処理されるかについて少し議論する価値があります。次の例を考えてみましょう:

```rust,ignore
fn foo<'a, 'b>(x: &'a u32, y: &'b u32) -> &'b u32 {
  x
}
```

この例はコンパイルされることを意図していません。なぜなら、`x` を返していますが、これは型 `&'a u32` を持っていますが、シグネチャは `&'b u32` 値を返すことを約束しているからです。しかし、`'a` や `'b` のようなライフタイムが領域推論にどのように統合され、このエラーがどのように検出されるのでしょうか？

## 全称領域とそれらの相互関係

領域推論の早い段階で、最初に行うことの 1 つは、[`UniversalRegions`] 構造体を構築することです。この構造体は、特定の関数のスコープ内にある様々な全称領域を追跡します。また、[`UniversalRegionRelations`] 構造体も作成します。これは、それらの相互関係を追跡します。したがって、例えば `where 'a: 'b` がある場合、[`UniversalRegionRelations`] 構造体は `'a: 'b` が成立することが知られていることを追跡します（これは [`outlives`] 関数でテストできます）。

[`UniversalRegions`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/universal_regions/struct.UniversalRegions.html
[`UniversalRegionRelations`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/type_check/free_region_relations/struct.UniversalRegionRelations.html
[`outlives`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/type_check/free_region_relations/struct.UniversalRegionRelations.html#method.outlives

## すべてが領域変数です

NLL 領域推論の動作の重要な側面の 1 つは、**すべてのライフタイム**が番号付き変数として表現されることです。これは、使用する [`region_kind::RegionKind`] の唯一のバリアントが [`ReVar`] バリアントであることを意味します。これらの領域変数は、インデックスに基づいて 2 つの主要なカテゴリに分類されます:

[`region_kind::RegionKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/region_kind/enum.RegionKind.html
[`ReVar`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/region_kind/enum.RegionKind.html#variant.ReVar

- 0..N: 全称領域 -- ここで議論しているもの。この場合、コードは、宣言された関係を満たす変数のすべての値に対して正しくなければなりません。
- N..M: 存在領域 -- 領域推論器が*いくつかの*適切な値を見つけるタスクを負う推論変数。

実際、全称領域は、スコープに持ち込まれた場所に基づいてさらに細分化できます（[`RegionClassification`] 型を参照）。これらの細分化は、ここで議論されているトピックには重要ではありませんが、[クロージャ制約伝播](./closure_constraints.html) を考慮するときに重要になるため、そこで議論します。

[`RegionClassification`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/universal_regions/enum.RegionClassification.html#variant.Local

## 領域の値の要素としての全称ライフタイム

前述のように、各領域に対して推論する値は集合 `{E}` です。この集合の要素は、制御フローグラフ内のポイントである可能性がありますが、各全称ライフタイム `'a` に対応する要素 `end('a)` である可能性もあります。ある領域 `R0` の値に `end('a)` が含まれている場合、これは `R0` が呼び出し元の `'a` の終わりまで拡張しなければならないことを意味します。

## 全称領域の「値」

領域推論中、他の領域の値を計算するのと同じ方法で、各全称領域の値を計算します。この値は、効果的に、その全称領域の**下限** -- それが生存しなければならないものを表します。この値を使用してエラーをチェックする方法について説明します。

## 生存性と全称領域

すべての全称領域には、関数本体全体を含む初期生存性制約があります。これは、ライフタイムパラメータが呼び出し元で定義され、この特定の関数を呼び出す関数呼び出し全体を含まなければならないためです。さらに、各全称領域 `'a` は、その生存性制約に自分自身（つまり、`end('a)`）を含みます（つまり、`'a` は自分自身の終わりまで拡張しなければなりません）。コードでは、これらの生存性制約は [`init_free_and_bound_regions`] で設定されます。

[`init_free_and_bound_regions`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#method.init_free_and_bound_regions

## 全称領域の outlives 制約の伝播

では、このセクションの最初の例を考えてみましょう:

```rust,ignore
fn foo<'a, 'b>(x: &'a u32, y: &'b u32) -> &'b u32 {
  x
}
```

ここで、`x` を返すには `&'a u32 <: &'b u32` が必要で、これは outlives 制約 `'a: 'b` を引き起こします。デフォルトの生存性制約と組み合わせると、次のようになります:

```txt
'a live at {B, end('a)} // B は「関数本体」を表します
'b live at {B, end('b)}
'a: 'b
```

`'a: 'b` 制約を処理するとき、`end('b)` を `'a` の値に追加するため、`'a` の最終値は `{B, end('a), end('b)}` になります。

## エラーの検出

制約伝播が終了したら、ある全称領域 `'a` に要素 `end('b)` が含まれている場合、`'a: 'b` が関数の境界で宣言されている必要があるという制約を強制します。そうでない場合、この例のように、それはエラーです。このチェックは [`check_universal_regions`] 関数で行われ、すべての全称領域を反復し、その最終値を検査し、宣言された [`UniversalRegionRelations`] に対してテストします。

[`check_universal_regions`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#method.check_universal_regions
