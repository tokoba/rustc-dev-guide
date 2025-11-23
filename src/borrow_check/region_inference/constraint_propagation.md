# 制約伝播

領域推論の主な作業は**制約伝播**で、これは [`propagate_constraints`] 関数で行われます。NLL で使用される制約には 3 種類あり、それらの制約を 1 つずつ「レイヤー化」することで、`propagate_constraints` の動作を説明します（それぞれは他のものからかなり独立しています）:

- 生存性制約（`R live at E`）、生存性から生じます;
- outlives 制約（`R1: R2`）、サブタイピングから生じます;
- [メンバー制約][m_c]（`member R_m of [R_c...]`）、impl Trait から生じます。

[`propagate_constraints`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#method.propagate_constraints
[m_c]: ./member_constraints.md

この章では、制約伝播の「核心」を説明し、生存性制約と outlives 制約の両方をカバーします。

## 記法と高レベルの概念

概念的に、領域推論は「不動点」計算です。いくつかの制約セット `{C}` が与えられ、各領域 `R` を要素のセット `{E}` にマッピングする値のセット `Values: R -> {E}` を計算します（領域要素に関する詳細については [こちら][riv] を参照してください）:

- 最初に、各領域は空の集合にマッピングされます。したがって、すべての領域 `R` について `Values(R) = {}` です。
- 次に、不動点に達するまで制約を繰り返し処理します:
  - 各制約 C について:
    - 制約を満たすために必要に応じて `Values` を更新します

[riv]: ../region_inference.md#region-variables

簡単な例として、生存性制約 `R live at E` がある場合、`Values(R) = Values(R) union {E}` を適用して制約を満たすことができます。同様に、outlives 制約 `R1: R2` がある場合、`Values(R1) = Values(R1) union Values(R2)` を適用できます。
（メンバー制約はより複雑で、[このセクション][m_c] で説明します。）

しかし、実際には、もう少し賢いです。制約をループで適用する代わりに、制約を分析して適用する正しい順序を把握できるため、最終結果を見つけるために各制約を 1 回だけ適用すれば済みます。

同様に、実装では、`Values` セットは `scc_values` フィールドに格納されますが、*領域*ではなく*強連結成分*（SCC）によってインデックス化されます。SCC は、冗長なストレージと計算を大幅に回避する最適化です。これらは outlives 制約のセクションで説明されます。

## 生存性制約

**生存性制約**は、型に領域 R が含まれる変数がある [point] P で生きているときに発生します。これは単に、R の値がポイント P を含む必要があることを意味します。生存性制約は MIR 型チェッカーによって計算されます。

[point]: ../../appendix/glossary.md#point

生存性制約 `R live at E` は、`E` が `Values(R)` のメンバーである場合に満たされます。したがって、そのような制約を `Values` に「適用」するには、`Values(R) = Values(R) union {E}` を計算するだけです。

生存性の値は型チェックで計算され、作成時に `liveness_constraints` 引数で領域推論に渡されます。
これらは `R live at E` のような個別の制約としては表現されていません; 代わりに、領域変数ごとに（[`LivenessValues`] 型の）（スパース）bitset を格納します。このようにして、各生存性制約に 1 ビットしか必要ありません。

[`LivenessValues`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/values/struct.LivenessValues.html

言及する価値があることの 1 つ: すべてのライフタイムパラメータは常に関数本体全体で生きていると見なされます。これは、それらが*呼び出し元の*実行の一部に対応するためで、その実行には明らかにこの関数で過ごす時間が含まれます。なぜなら、呼び出し元は私たちが返るのを待っているからです。

## Outlives 制約

outlives 制約 `'a: 'b` は、`'a` の値が `'b` の値の**スーパーセット**でなければならないことを示します。つまり、outlives 制約 `R1: R2` は、`Values(R1)` が `Values(R2)` のスーパーセットである場合に満たされます。したがって、そのような制約を `Values` に「適用」するには、`Values(R1) = Values(R1) union Values(R2)` を計算するだけです。

これから次のことが観察されます: `R1: R2` と `R2: R1` がある場合、`R1 = R2` でなければなりません。同様に、次のようなものがある場合:

```txt
R1: R2
R2: R3
R3: R4
R4: R1
```

すると、`R1 = R2 = R3 = R4` になります。これをすぐに説明するように、これを利用してものを大幅に高速化します。

コードでは、outlives 制約のセットは、[`OutlivesConstraintSet`] 型のパラメータで作成時に領域推論コンテキストに与えられます。制約セットは基本的に `'a: 'b` 制約のリストです。

### outlives 制約グラフと SCC

outlives 制約をより効率的に扱うために、それらは[グラフの形式に変換][graph-fn]されます。グラフのノードは領域変数（`'a`、`'b`）で、各制約 `'a: 'b` はエッジ `'a -> 'b` を誘導します。この変換は、推論コンテキストを作成する [`RegionInferenceContext::new`] 関数で行われます。

[`OutlivesConstraintSet`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/constraints/struct.OutlivesConstraintSet.html
[graph-fn]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/constraints/struct.OutlivesConstraintSet.html#method.graph
[`RegionInferenceContext::new`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#method.new

グラフ表現を使用すると、サイクルを探すことで等しくなければならない領域を検出できます。つまり、次のような制約がある場合:

```txt
'a: 'b
'b: 'c
'c: 'd
'd: 'a
```

これは、要素 `'a...'d` を含むグラフ内のサイクルに対応します。

したがって、領域の値を伝播する最初のことの 1 つは、制約グラフ内の**強連結成分**（SCC）を計算することです。結果は [`constraint_sccs`] フィールドに格納されます。その後、`constraint_sccs.scc(r)` を呼び出すことで、領域 `r` が属する SCC を簡単に見つけることができます。

SCC の観点から作業することで、より効率的になります: 単一の SCC の一部である領域のセット `'a...'d` がある場合、それらの値を別々に計算/格納する必要はありません。すべて等しくなければならないため、**SCC に対して** 1 つの値を格納するだけで済みます。

領域推論コードを見ると、多くのフィールドが SCC の観点から定義されていることがわかります。例えば、[`scc_values`] フィールドは各 SCC の値を格納します。特定の領域 `'a` の値を取得するには、最初に領域が属する SCC を把握し、次にその SCC の値を見つけます。

[`constraint_sccs`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#structfield.constraint_sccs
[`scc_values`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/region_infer/struct.RegionInferenceContext.html#structfield.scc_values

SCC を計算するとき、どの領域が各 SCC のメンバーであるかを把握するだけでなく、それらの間のエッジも把握します。したがって、例えば、次の outlives 制約のセットを考えてみましょう:

```txt
'a: 'b
'b: 'a

'a: 'c

'c: 'd
'd: 'c
```

ここには 2 つの SCC があります: S0 には `'a` と `'b` が含まれ、S1 には `'c` と `'d` が含まれます。しかし、これらの SCC は独立していません: `'a: 'c` であるため、`S0: S1` も同様です。つまり -- `S0` の値は `S1` の値のスーパーセットでなければなりません。重要なことの 1 つは、この SCC のグラフが常に DAG であることです -- つまり、サイクルを持つことはありません。これは、すべてのサイクルが SCC 自体を形成するために削除されたためです。

### SCC への生存性制約の適用

型チェッカーから入ってくる生存性制約は領域の観点から表現されます -- つまり、`Liveness: R -> {E}` のようなマップがあります。しかし、最終結果を SCC の観点から表現したいです -- 和を取るだけで、これらの生存性制約を非常に簡単に統合できます:

```txt
for each region R:
  let S be the SCC that contains R
  Values(S) = Values(S) union Liveness(R)
```

領域推論器では、このステップは [`RegionInferenceContext::new`] で行われます。

### outlives 制約の適用

SCC の DAG を計算したら、それを使用して計算全体を構造化します。2 つの SCC 間にエッジ `S1 -> S2` がある場合、`Values(S1) >= Values(S2)` が成立する必要があることを意味します。したがって、`S1` の値を計算するには、最初に各後継 `S2` の値を計算します。次に、それらの値をすべて結合します。準イテレータ風の記法を使用すると:

```txt
Values(S1) =
  s1.successors()
    .map(|s2| Values(s2))
    .union()
```

コードでは、この作業は [`propagate_constraints`] 関数で開始され、すべての SCC を反復します。各 SCC `S1` について、最初にその後継の値を計算してその値を計算します。SCC は DAG を形成するため、サイクルについて心配する必要はありませんが、特定の SCC を既に処理したかどうかを追跡するためのセットを保持する必要があります。各後継 `S2` について、`S2` の値を計算したら、それらの要素を `S1` の値に結合できます。（ただし、このプロセスでは、[高ランクプレースホルダー](./placeholders_and_universes.html) を適切に処理するように注意する必要があります。`S1` の値には既に生存性制約が含まれていることに注意してください。それらは [`RegionInferenceContext::new`] で追加されたためです。

そのプロセスが完了すると、すべての生存性および outlives 制約を考慮した `S1` の「最小値」が得られます。ただし、プロセスを完了するには、[メンバー制約][m_c] も考慮する必要があります。これについては [後のセクション][m_c] で説明します。
