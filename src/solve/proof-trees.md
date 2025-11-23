# 証明木

トレイトソルバー自体は、ゴールが成り立つかどうかと必要な
制約のみを返しますが、証明しようとしている間に何が起こったかを知りたい場合もあります。トレイトソルバーは通常、残りのコンパイラによってブラックボックスとして扱われるべきですが、その内部を完全に無視することはできず、このためのインターフェースとして「証明木」を提供します。それらを使用するには、[`ProofTreeVisitor`]トレイトを実装します。
既存の実装を例として参照してください。最も注目すべき使用法は、[コヒーレンスエラーのためのintercrate曖昧性の原因][intercrate-ambig]、
[トレイトソルバーエラーの改善][solver-errors]、および
[クロージャシグネチャの熱心な推論][closure-sig]の計算です。

## 証明木の計算

トレイトソルバーは[正規化]を使用し、
各ネストされたゴールに対して完全に別の`InferCtxt`を使用します。診断とrustdocの自動トレイトの両方が、「ネストされたゴールへの覗き見」を正しく処理する必要があります。`Vec<Vec<?x>>: Debug`のようなゴールが与えられた場合、
`exists<T0> Vec<Vec<T0>>: Debug`に正規化し、そのゴールを
`Vec<Vec<?0>>: Debug`としてインスタンス化し、ネストされたゴール`Vec<?0>: Debug`を取得し、これを正規化して`exists<T0> Vec<T0>: Debug`を取得し、これを`Vec<?0>: Debug`としてインスタンス化し、次に
曖昧な`?0: Debug`ゴールが得られます。

証明木は、検索グラフに[`ProofTreeBuilder`]を渡すことによって計算されます。これは、
トレイトソルバーの評価ステップを木に変換します。推論変数またはプレースホルダを使用してデータを保存する際、データは、この計算中に作成されたすべての制約されていない推論変数のリストと共に正規化されます。
この[`CanonicalState`]は、証明木を歩く際に親推論コンテキストでインスタンス化され、推論変数のリストを使用して、この評価中に作成されたすべての
正規化された値を接続します。

## ソルバーのデバッグ

以前は、証明木を使用してソルバー実装をデバッグしようともしました。これには、
プログラム的に分析するのとは異なる設計要件があります。トレイトソルバーをデバッグする推奨
方法は、`tracing`を使用することです。トレイトソルバーは、一般的な「形状」には
`debug`トレーシングレベルのみを使用し、追加の詳細には`trace`を使用します。
したがって、`RUSTC_LOG=rustc_next_trait_solver=debug`は一般的なアウトラインを提供し、
より正確な情報が必要な場合は`RUSTC_LOG=rustc_next_trait_solver=trace`を使用できます。

[`ProofTreeVisitor`]: https://github.com/rust-lang/rust/blob/d6c8169c186ab16a3404cd0d0866674018e8a19e/compiler/rustc_trait_selection/src/solve/inspect/analyse.rs#L403
[`ProofTreeBuilder`]: https://github.com/rust-lang/rust/blob/d6c8169c186ab16a3404cd0d0866674018e8a19e/compiler/rustc_next_trait_solver/src/solve/inspect/build.rs#L40
[`CanonicalState`]: https://github.com/rust-lang/rust/blob/d6c8169c186ab16a3404cd0d0866674018e8a19e/compiler/rustc_type_ir/src/solve/inspect.rs#L31-L47
[intercrate-ambig]: https://github.com/rust-lang/rust/blob/d6c8169c186ab16a3404cd0d0866674018e8a19e/compiler/rustc_trait_selection/src/traits/coherence.rs#L742-L748
[solver-errors]: https://github.com/rust-lang/rust/blob/d6c8169c186ab16a3404cd0d0866674018e8a19e/compiler/rustc_trait_selection/src/solve/fulfill.rs#L343-L356
[closure-sig]: https://github.com/rust-lang/rust/blob/d6c8169c186ab16a3404cd0d0866674018e8a19e/compiler/rustc_hir_typeck/src/closure.rs#L333-L339
[正規化]: ./canonicalization.md
