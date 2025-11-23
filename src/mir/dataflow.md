# データフロー解析

MIRに取り組む場合、さまざまな種類の[データフロー解析][wiki]に頻繁に遭遇します。`rustc`は、データフローを使用して、初期化されていない変数を見つけたり、ジェネレータの`yield`ステートメント全体でどの変数が生きているかを判断したり、制御フローグラフの特定の時点でどの`Place`が借用されているかを計算したりします。データフロー解析は、現代のコンパイラの基本的な概念であり、この主題の知識は、将来の貢献者にとって役立ちます。

しかし、このドキュメントはデータフロー解析への一般的な入門ではありません。これは単に、`rustc`でこれらの解析を定義するために使用されるフレームワークの説明です。読者が、「転送関数」、「不動点」、「束」などの基本的な用語だけでなく、核心的なアイデアにも精通していることを前提としています。これらの用語に不慣れな場合、または簡単な復習が必要な場合は、Anders MøllerとMichael I. Schwartzbachによる[*Static Program Analysis*]は、優れた、無料で入手可能な教科書です。視聴覚学習を好む方のために、以前はYouTubeのGoethe University Frankfurtによる一連の短い講義を推奨していましたが、削除されています。コンテキストについては[このPR][pr-1295]を、代替講義については[このコメント][pr-1295-comment]を参照してください。

## データフロー解析の定義

データフロー解析は、[`Analysis`]トレイトによって定義されます。データフロー状態の型に加えて、このトレイトは、各ブロックの入口におけるその状態の初期値、および解析の方向（前方または後方）を定義します。データフロー解析のドメインは、適切に動作する`join`演算子を持つ[束][lattice]（厳密には半束）である必要があります。詳細については、[`lattice`]モジュールのドキュメント、および[`JoinSemiLattice`]トレイトを参照してください。

### 転送関数とエフェクト

`rustc`のデータフローフレームワークでは、基本ブロック内の各ステートメント（およびターミネータ）が独自の転送関数を定義できます。簡潔にするために、これらの個々の転送関数は「エフェクト」として知られています。各エフェクトはデータフロー順に連続して適用され、それらが一緒になって基本ブロック全体の転送関数を定義します。特定のターミネータの特定の出力エッジに対してエフェクトを定義することも可能です（例：`Call`ターミネータの`success`エッジに対する[`apply_call_return_effect`]）。総称して、これらは「エッジごとのエフェクト」と呼ばれます。

### 「Before」エフェクト

ドキュメントの注意深い読者は、各ステートメントとターミネータに実際には*2つ*の可能なエフェクトがあることに気付くかもしれません。「before」エフェクトとプレフィックスなし（または「プライマリ」）エフェクトです。「before」エフェクトは、**解析の方向に関係なく**、プレフィックスなしエフェクトの直前に適用されます。言い換えれば、後方解析は、前方解析と同様に、基本ブロックの転送関数を計算するときに「before」エフェクトを適用してから「プライマリ」エフェクトを適用します。

大部分の解析は、プレフィックスなしエフェクトのみを使用する必要があります：各ステートメントに複数のエフェクトがあると、消費者がどこを見るべきかを知ることが難しくなります。ただし、「before」バリアントは、代入ステートメントの右側のエフェクトを左側とは別に考慮する必要がある場合など、一部のシナリオで役立つ場合があります。

### 収束

解析は「不動点」に収束する必要があります。そうでないと、永遠に実行されます。不動点に収束することは、単に「平衡に達する」ことの別の言い方です。平衡に達するためには、解析はいくつかの法則に従う必要があります。従う必要がある法則の1つは、ボトム値[^bottom-purpose]が他の値と結合されると、2番目の値に等しくなることです。または、方程式として：

> *bottom* join *x* = *x*

別の法則は、解析が「トップ値」を持つ必要があることです。

> *top* join *x* = *top*

トップ値を持つことで、半束の高さが有限であることが保証され、上記で述べた法則により、データフロー状態がトップに達すると、それ以上変化しないことが保証されます（不動点はトップになります）。

[^bottom-purpose]: ボトム値の主な目的は、初期データフロー状態としてです。解析が開始される前に、各基本ブロックの入口状態はボトムに初期化されます。

## 簡単な例

このセクションでは、簡単なデータフロー解析の高レベルでの簡単な例を提供します。これは、知る必要があるすべてを説明するわけではありませんが、このページの残りの部分をより明確にするのに役立つことを願っています。

プログラム内の特定の時点までに`mem::transmute`が呼び出された可能性があるかどうかを見つける簡単な解析を行いたいとしましょう。解析ドメインは、これまでに`transmute`が呼び出されたかどうかを記録する単なる`bool`になります。ボトム値は`false`です。なぜなら、デフォルトでは`transmute`は呼び出されていないからです。トップ値は`true`です。なぜなら、`transmute`が呼び出されたと判断するとすぐに解析が完了するからです。結合演算子は、ブールORまたは（`||`）演算子です。ANDではなくORを使用する理由は、次のケースのためです：

```rust
# unsafe fn example(some_cond: bool) {
let x = if some_cond {
    std::mem::transmute::<i32, u32>(0_i32) // transmuteが呼び出されました！
} else {
    1_u32 // transmuteは呼び出されていません
};

// この時点でtransmuteは呼び出されましたか？保守的に「はい」と近似します。
// これがOR演算子を使用する理由です。
println!("x: {}", x);
# }
```

## データフロー解析の結果の検査

解析を構築したら、`iterate_to_fixpoint`を呼び出す必要があります。これは、各ブロックの入口における不動点でのデータフロー状態を含む`Results`を返します。`Results`を取得したら、CFGの任意の時点で不動点でのデータフロー状態を検査できます。いくつかの場所（例：各`Drop`ターミネータ）での状態のみが必要な場合は、[`ResultsCursor`]を使用します。*すべて*の場所での状態が必要な場合は、[`ResultsVisitor`]の方が効率的です。

```text
                         Analysis
                            |
                            | iterate_to_fixpoint()
                            |
                         Results
                         /     \
 into_results_cursor(…) /       \  visit_with(…)
                       /         \
               ResultsCursor  ResultsVisitor
```

例えば、次のコードは[`ResultsVisitor`]を使用しています...


```rust,ignore
// `MyVisitor`が`ResultsVisitor<FlowState = MyAnalysis::Domain>`を実装していると仮定...
let mut my_visitor = MyVisitor::new();

// RPOの各ブロック内のすべての場所の不動点状態を検査します。
let results = MyAnalysis::new()
    .iterate_to_fixpoint(tcx, body, None);
results.visit_with(body, &mut my_visitor);`
```

一方、このコードは[`ResultsCursor`]を使用しています：

```rust,ignore
let mut results = MyAnalysis::new()
    .iterate_to_fixpoint(tcx, body, None);
    .into_results_cursor(body);

// 各`Drop`ターミネータの直前の不動点状態を検査します。
for (bb, block) in body.basic_blocks().iter_enumerated() {
    if let TerminatorKind::Drop { .. } = block.terminator().kind {
        results.seek_before_primary_effect(body.terminator_loc(bb));
        let state = results.get();
        println!("state before drop: {:#?}", state);
    }
}
```

### Graphviz図

データフロー解析の結果が期待どおりでない場合、それらを視覚化すると役立つことがよくあります。これは、[MIRのデバッグ]で説明されている`-Z dump-mir`フラグを使用して行うことができます。`-Z dump-mir=F -Z dump-mir-dataflow`から始めます。ここで、`F`は「all」または興味のあるMIR本体の名前のいずれかです。

これらの`.dot`ファイルは`mir_dump`ディレクトリに保存され、ファイル名の一部として解析の[`NAME`]（例：`maybe_inits`）が含まれます。各視覚化は、各ブロックの入口と出口での完全なデータフロー状態、および各ステートメントとターミネータで発生する変更を表示します。以下の例を参照してください：

![A graphviz diagram for a dataflow analysis](../img/dataflow-graphviz-example.png)

["gen-kill" problems]: https://en.wikipedia.org/wiki/Data-flow_analysis#Bit_vector_problems
[*Static Program Analysis*]: https://cs.au.dk/~amoeller/spa/
[MIRのデバッグ]: ./debugging.md
[`Analysis`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/trait.Analysis.html
[`GenKillAnalysis`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/trait.GenKillAnalysis.html
[`JoinSemiLattice`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/lattice/trait.JoinSemiLattice.html
[`NAME`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/trait.Analysis.html#associatedconstant.NAME
[`ResultsCursor`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/struct.ResultsCursor.html
[`ResultsVisitor`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/trait.ResultsVisitor.html
[`apply_call_return_effect`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/trait.Analysis.html#tymethod.apply_call_return_effect
[`into_engine`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/trait.Analysis.html#method.into_engine
[`lattice`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/lattice/index.html
[pr-1295]: https://github.com/rust-lang/rustc-dev-guide/pull/1295
[pr-1295-comment]: https://github.com/rust-lang/rustc-dev-guide/pull/1295#issuecomment-1118131294
[lattice]: https://en.wikipedia.org/wiki/Lattice_(order)
[wiki]: https://en.wikipedia.org/wiki/Data-flow_analysis#Basic_principles
