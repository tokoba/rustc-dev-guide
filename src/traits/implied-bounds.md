# 暗黙の境界

明示的なアノテーションを避けるために、現在、暗黙の領域境界を追加しています。
例えば、`fn foo<'a, T>(x: &'a T)` は、それを指定しなくても `T: 'a` が
成り立つことを自由に仮定できます。

暗黙の境界には2つの種類があります：明示的と暗黙的です。明示的な暗黙の境界は
関連アイテムの `fn predicates_of` に追加されますが、暗黙的なものは...
まあ...暗黙的に処理されます。

## 明示的な暗黙の境界

明示的な暗黙の境界は [`fn inferred_outlives_of`] で計算されます。ADT と
遅延型エイリアスのみが明示的な暗黙の境界を持ち、これらは
[`fn inferred_outlives_crate`] クエリの不動点アルゴリズムを介して計算されます。

クレート内のすべての ADT のすべてのフィールドに対して
[`fn insert_required_predicates_to_be_wf`] を使用します。この関数は、
別の実装を使用して、フィールドの各コンポーネントの outlives 境界を計算します。

ADT、トレイトオブジェクト、および関連型の場合、最初に必要な述語は
[`fn check_explicit_predicates`] で計算されます。これは単に
`fn explicit_predicates_of` を精緻化せずに使用します。

領域述語は [`fn insert_outlives_predicate`] を介して追加されます。この関数は
outlives 述語を受け取り、それを分解し、outlived 領域が領域パラメータである
場合にのみ、コンポーネントを明示的な述語として追加します。
[`'static` 要件は追加しません][nostatic]。

 [`fn inferred_outlives_of`]: https://github.com/rust-lang/rust/blob/5b8bc568d28b2e922290c9a966b3231d0ce9398b/compiler/rustc_hir_analysis/src/outlives/mod.rs#L20
 [`fn inferred_outlives_crate`]: https://github.com/rust-lang/rust/blob/5b8bc568d28b2e922290c9a966b3231d0ce9398b/compiler/rustc_hir_analysis/src/outlives/mod.rs#L83
 [`fn insert_required_predicates_to_be_wf`]: https://github.com/rust-lang/rust/blob/5b8bc568d28b2e922290c9a966b3231d0ce9398b/compiler/rustc_hir_analysis/src/outlives/implicit_infer.rs#L89
 [`fn check_explicit_predicates`]: https://github.com/rust-lang/rust/blob/5b8bc568d28b2e922290c9a966b3231d0ce9398b/compiler/rustc_hir_analysis/src/outlives/implicit_infer.rs#L238
 [`fn insert_outlives_predicate`]: https://github.com/rust-lang/rust/blob/5b8bc568d28b2e922290c9a966b3231d0ce9398b/compiler/rustc_hir_analysis/src/outlives/utils.rs#L15
 [nostatic]: https://github.com/rust-lang/rust/blob/5b8bc568d28b2e922290c9a966b3231d0ce9398b/compiler/rustc_hir_analysis/src/outlives/utils.rs#L159-L165

## 暗黙的な暗黙の境界

バインダー内の含意をまだ処理できないため、impl や関数の outlives 要件を
明示的な述語として単純に追加することはできません。

### 仮定として暗黙的な暗黙の境界を使用する

これらの境界は、影響を受けるアイテム自体の `ParamEnv` には追加されません。
字句領域解決の場合、[`fn OutlivesEnvironment::from_normalized_bounds`] を
使用して追加されます。同様に、MIR borrowck 中は、
[`fn UniversalRegionRelationsBuilder::add_implied_bounds`] を使用して追加します。

[MIR borrowck では関数シグネチャと impl ヘッダーの暗黙の境界を追加します][mir]。
MIR borrowck 以外では、[`fn assumed_wf_types`] クエリによって返される型の
outlives 要件を追加します。

暗黙の境界に対する仮定された outlives 制約は、
[`fn implied_outlives_bounds`] クエリを使用して計算されます。これは直接
[`fn wf::obligations` から必要な outlives 境界を抽出します][boundsfromty]。

MIR borrowck は正規化された型と正規化されていない型の両方の outlives 制約を
追加しますが、字句領域解決は[正規化されていない型のみを使用します][notnorm]。

[`fn OutlivesEnvironment::from_normalized_bounds`]: https://github.com/rust-lang/rust/blob/8239a37f9c0951a037cfc51763ea52a20e71e6bd/compiler/rustc_infer/src/infer/outlives/env.rs#L50-L55
[`fn UniversalRegionRelationsBuilder::add_implied_bounds`]: https://github.com/rust-lang/rust/blob/5b8bc568d28b2e922290c9a966b3231d0ce9398b/compiler/rustc_borrowck/src/type_check/free_region_relations.rs#L316
[mir]: https://github.com/rust-lang/rust/blob/91cae1dcdcf1a31bd8a92e4a63793d65cfe289bb/compiler/rustc_borrowck/src/type_check/free_region_relations.rs#L258-L332
[`fn assumed_wf_types`]: https://github.com/rust-lang/rust/blob/5b8bc568d28b2e922290c9a966b3231d0ce9398b/compiler/rustc_ty_utils/src/implied_bounds.rs#L21
[`fn implied_outlives_bounds`]: https://github.com/rust-lang/rust/blob/5b8bc568d28b2e922290c9a966b3231d0ce9398b/compiler/rustc_traits/src/implied_outlives_bounds.rs#L18C4-L18C27
[boundsfromty]: https://github.com/rust-lang/rust/blob/5b8bc568d28b2e922290c9a966b3231d0ce9398b/compiler/rustc_trait_selection/src/traits/query/type_op/implied_outlives_bounds.rs#L95-L96
[notnorm]: https://github.com/rust-lang/rust/blob/91cae1dcdcf1a31bd8a92e4a63793d65cfe289bb/compiler/rustc_trait_selection/src/traits/engine.rs#L227-L250

### 暗黙的な暗黙の境界を証明する

暗黙的な暗黙の境界は `fn predicates_of` に含まれていないため、
それらが実際に成り立つことを別途確認する必要があります。一般的には、
`WellFormed` 述語を発行することで、使用されるすべての型が整形式であることを
チェックすることでこれを処理します。

impl をインスタンス化するときに `WellFormed` 述語を発行することはできません。
これは、現在しばしば帰納的なトレイトソルバーサイクルになるからです。
また、それらのバインダーからの暗黙の境界が不足しているため、高階領域を
含む制約も発行しません。

これにより、複数の健全性の問題が発生します：
- サブタイピングを使用することによる：[#25860]
- 高階トレイト境界のスーパートレイトアップキャスティングを使用することによる：[#84591]
- impl を使用するときに射影を正規化できるが、impl をチェックするときには
  正規化できないことによる：[#100051]

[#25860]: https://github.com/rust-lang/rust/issues/25860
[#84591]: https://github.com/rust-lang/rust/issues/84591
[#100051]: https://github.com/rust-lang/rust/issues/100051
