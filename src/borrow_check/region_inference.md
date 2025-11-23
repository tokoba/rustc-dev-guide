# 領域推論 (NLL)

MIR ベースの領域チェックコードは [the `rustc_mir::borrow_check` モジュール][nll] にあります。

[nll]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/index.html

MIR ベースの領域分析は、2 つの主要な関数で構成されています:

- [`replace_regions_in_mir`]。最初に呼び出され、2 つの仕事があります:
  - 第一に、関数のシグネチャ内に現れる領域のセットを見つけます（例えば、`fn foo<'a>(&'a u32) { ... }` の `'a`）。これらは「全称」または「自由」領域と呼ばれます -- 特に、関数本体で[自由に現れる][fvb]領域です。
  - 第二に、関数本体のすべての領域を新しい推論変数に置き換えます。これは、（現在）それらの領域が字句的領域推論の結果であり、あまり興味深くないためです。意図は -- 最終的には -- それらが「消去された領域」（つまり、情報がまったくない）になることです。なぜなら、字句的領域推論をまったく行わないためです。
- [`compute_regions`]。2 番目に呼び出されます: これは移動分析の結果を引数として与えられます。`replace_regions_in_mir` が導入したすべての推論変数の値を計算する仕事があります。
  - そのために、最初に [MIR type checker] を実行します。これは基本的には通常の型チェッカーですが、MIR に特化しています。もちろん、MIR は完全な Rust よりもはるかに単純です。MIR 型チェッカーを実行すると、領域変数間の様々な[制約][cp]が作成され、それらの潜在的な値と相互の関係を示します。
  - この後、[`RegionInferenceContext`] を作成し、その [`solve`] メソッドを呼び出すことで、[制約伝播][cp]を実行します。
  - [NLL RFC] も、かなり徹底的で（願わくば）読みやすいカバレッジを含んでいます。

[cp]: ./region_inference/constraint_propagation.md
[fvb]: ../appendix/background.md#free-vs-bound
[`replace_regions_in_mir`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/nll/fn.replace_regions_in_mir.html
[`compute_regions`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/nll/fn.compute_regions.html
[`RegionInferenceContext`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html
[`solve`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#method.solve
[NLL RFC]: https://rust-lang.github.io/rfcs/2094-nll.html
[MIR type checker]: ./type_check.md

## 全称領域

[`UniversalRegions`] 型は、ある MIR `DefId` に対応する_全称_領域のコレクションを表します。これは [`replace_regions_in_mir`] でインスタンス化され、すべての領域を新しい推論変数に置き換えるときに構築されます。[`UniversalRegions`] には、与えられた MIR 内のすべての自由領域のインデックスと、それらの間で保持されることが_既知_の関係（例えば、暗黙の境界、where 句など）が含まれます。

例えば、次の関数の MIR が与えられた場合:

```rust
fn foo<'a>(x: &'a u32) {
    // ...
}
```

`'a` の全称領域と `'static` の全称領域を作成します。クロージャを処理するための複雑さもあるかもしれませんが、今のところそれらは無視します。

TODO: これらの領域が_どのように_計算されるかについて書く。

[`UniversalRegions`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/universal_regions/struct.UniversalRegions.html

<a id="region-variables"></a>

## 領域変数

領域の値は**集合**と考えることができます。この集合には、領域が有効な MIR 内のすべてのポイントと、この領域によって生存される領域が含まれます（例えば、`'a: 'b` の場合、`end('b)` は `'a` の集合に含まれます）; この集合のドメインを `RegionElement` と呼びます。コードでは、すべての領域の値は [the `rustc_borrowck::region_infer` モジュール][ri] で維持されます。
各領域について、その値に存在する要素を格納する集合を維持します（これを効率的にするために、各種の要素にインデックス `RegionElementIndex` を与え、スパース bitsets を使用します）。

[ri]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_borrowck/src/region_infer

領域要素の種類は次のとおりです:

- MIR 制御フローグラフ内の各**[`location`]**: location は基本ブロックとインデックスのペアです。これは、そのインデックスを持つステートメント（またはインデックスが `statements.len()` に等しい場合はターミネータ）への**エントリ時**のポイントを識別します。
- 各全称領域 `'a` に対して要素 `end('a)` があり、呼び出し元（または呼び出し元の呼び出し元など）の制御フローグラフの一部に対応します。
- 同様に、この関数が返った後のプログラム実行の残りに対応する要素 `end('static)` があります。
- 各プレースホルダー領域 `!1` に対して要素 `!1` があります。これは（直感的に）他の要素の未知の集合に対応します -- プレースホルダーの詳細については、セクション [placeholders and universes](region_inference/placeholders_and_universes.md) を参照してください。

## 制約

領域の値を推論する前に、領域に関する制約を収集する必要があります。完全な制約セットは [制約伝播に関するセクション][cp] で説明されていますが、最も一般的な制約の 2 種類は次のとおりです:

1. Outlives 制約。これらは、ある領域が別の領域よりも長く生存する制約です（例えば、`'a: 'b`）。Outlives 制約は [MIR type checker] によって生成されます。
2. 生存性制約。各領域は、使用できるポイントで生きている必要があります。

## 推論の概要

では、領域の内容をどのように計算するのでしょうか？このプロセスを_領域推論_と呼びます。高レベルのアイデアはかなりシンプルですが、いくつかの詳細に注意する必要があります。

高レベルのアイデア: 生存性制約から必ず含まれる必要がある MIR の位置で各領域を開始します。そこから、型チェッカーから計算されたすべての outlives 制約を使用して、制約を_伝播_します: 各領域 `'a` について、`'a: 'b` の場合、`'b` のすべての要素を `'a` に追加します。`end('b)` も含みます。これはすべて [`propagate_constraints`] で行われます。

次に、エラーをチェックします。最初に [`check_type_tests`] を呼び出して、型テストが満たされていることをチェックします。これは `T: 'a` のような制約をチェックします。次に、全称領域が「大きすぎない」ことをチェックします。これは [`check_universal_regions`] を呼び出すことで行われます。これは、各領域 `'a` について、`'a` に要素 `end('b)` が含まれている場合、`'a: 'b` が既に成立していることを知っている必要があることをチェックします（例えば、where 句から）。これを既に知らない場合、それはエラーです...まあ、ほぼです。クロージャに対するいくつかの特別な処理があり、後で説明します。

### 例

次の例を考えてみましょう:

```rust,ignore
fn foo<'a, 'b>(x: &'a usize) -> &'b usize {
    x
}
```

明らかに、これは `'a` が `'b` より長く生存するかどうかがわからないため、コンパイルされるべきではありません（そうでない場合、戻り値は dangling 参照になる可能性があります）。

少し戻りましょう。いくつかの自由推論変数を導入する必要があります（[`replace_regions_in_mir`] で行われるように）。この例では生成される正確な領域を使用していませんが、（願わくば）アイデアを伝えるのに十分です。

```rust,ignore
fn foo<'a, 'b>(x: &'a /* '#1 */ usize) -> &'b /* '#3 */ usize {
    x // '#2, location L1
}
```

いくつかの記法: `'#1`、`'#3`、`'#2` は、引数、戻り値、式 `x` の全称領域を表します。さらに、式 `x` の位置を `L1` と呼びます。

したがって、生存性制約を使用して次の開始点を取得できます:

領域  | 内容
--------|----------
'#1     |
'#2     | `L1`
'#3     | `L1`

次に、outlives 制約を使用して各領域を拡張します。具体的には、`'#2: '#3` がわかっているので...

領域  | 内容
--------|----------
'#1     | `L1`
'#2     | `L1, end('#3) // '#3 の内容と end('#3) を追加`
'#3     | `L1`

... および `'#1: '#2` なので ...

領域  | 内容
--------|----------
'#1     | `L1, end('#2), end('#3) // '#2 の内容と end('#2) を追加`
'#2     | `L1, end('#3)`
'#3     | `L1`

次に、領域が大きすぎないかをチェックする必要があります（この場合、チェックする型テストはありません）。`'#1` には `end('#3)` が含まれていますが、`'a: 'b` を言う `where` 句や暗黙の境界がありません...それはエラーです！

### いくつかの詳細

[`RegionInferenceContext`] 型には、推論を行うために必要なすべての情報が含まれています。これには、[`replace_regions_in_mir`] からの全称領域と、各領域に対して計算される制約が含まれます。これは、生存性制約を計算した直後に構築されます。

構造体のいくつかのフィールドは次のとおりです:

- [`constraints`]: すべての outlives 制約が含まれます。
- [`liveness_constraints`]: すべての生存性制約が含まれます。
- [`universal_regions`]: [`replace_regions_in_mir`] によって返される `UniversalRegions` が含まれます。
- [`universal_region_relations`]: 全称領域について真であることが知られている関係が含まれます。例えば、`'a: 'b` という where 句がある場合、その関係は実装の borrow check 中に真であると仮定されます（呼び出し元でチェックされます）。したがって、`universal_region_relations` には `'a: 'b` が含まれます。
- [`type_tests`]: 推論後にチェックする必要がある型に関するいくつかの制約が含まれます（例えば、`T: 'a`）。
- [`closure_bounds_mapping`]: クロージャからクロージャの作成者に領域制約を伝播するために使用されます。

[`constraints`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#structfield.constraints
[`liveness_constraints`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#structfield.liveness_constraints
[`location`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Location.html
[`universal_regions`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#structfield.universal_regions
[`universal_region_relations`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#structfield.universal_region_relations
[`type_tests`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#structfield.type_tests
[`closure_bounds_mapping`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#structfield.closure_bounds_mapping

TODO: 他のフィールドについて議論する必要がありますか？SCC についてはどうですか？

さて、`RegionInferenceContext` を構築したので、推論を行うことができます。これは、コンテキストで [`solve`] メソッドを呼び出すことで行われます。これは、上記で議論したように、[`propagate_constraints`] を呼び出し、結果の型テストと全称領域をチェックする場所です。

[`propagate_constraints`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#method.propagate_constraints
[`check_type_tests`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#method.check_type_tests
[`check_universal_regions`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#method.check_universal_regions
