# エフェクトとconst条件チェック

## `HostEffect` 述語

[`HostEffectPredicate`] は、`~const Tr` または `const Tr` 境界からの述語の一種です。トレイト参照と、境界に応じて `Maybe` または `Const` になる `constness` を持っています。`~const Tr`、すなわち `Maybe` 境界は、それらが存在するコンテキストに応じて異なる適用をされるため、通常の境界とは異なる動作をします。関数上の `T: Tr` のような通常のトレイト境界は、関数が呼び出されたときに証明され、関数内で仮定されるために [`predicates_of`] クエリ内で収集されますが、`T: ~const Tr` のような境界は通常のトレイト境界として動作し、`predicates_of` の結果に `T: Tr` を追加しますが、[`const_conditions`] クエリにも `HostEffectPredicate` を追加します。

一方、`T: const Tr` 境界はコンテキスト間で意味が変わらないため、`predicates_of` に `HostEffect(T: Tr, const)` が追加され、`const_conditions` には追加されません。

[`HostEffectPredicate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/predicate/struct.HostEffectPredicate.html
[`predicates_of`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.predicates_of
[`const_conditions`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.const_conditions

## `const_conditions` クエリ

`predicates_of` は、アイテムを使用するために証明する必要がある述語のセットを表します。例えば、以下の例で `foo` を使用するには：

```rust
fn foo<T>() where T: Default {}
```

`T` が `Default` を実装していることを証明できなければなりません。同様に、`const_conditions` は、アイテムを *const コンテキストで* 使用するために証明する必要がある述語のセットを表します。上記の例を `const` トレイト境界を使用するように調整すると：

```rust
const fn foo<T>() where T: ~const Default {}
```

この場合、`foo` は `const_conditions` クエリに `HostEffect(T: Default, maybe)` を取得します。これは、const コンテキストから `foo` を呼び出すためには、`T` が `Default` の const 実装を持っていることを証明する必要があることを示唆しています。

## `const_conditions` の強制

`const_conditions` は現在、さまざまな場所でチェックされています。

const コンテキスト（`const fn` と `const` アイテムを含む）からの HIR 内のすべての呼び出しは、呼び出している関数の `const_conditions` が成立することをチェックします。これは [`FnCtxt::enforce_context_effects`] で行われます。関数が参照されているだけで呼び出されていない場合はチェックしないことに注意してください。次のコードをコンパイルする必要があるためです：

```rust
const fn hi<T: ~const Default>() -> T {
    T::default()
}
const X: fn() -> u32 = hi::<u32>;
```

トレイト `impl` が well-formed であるためには、`impl` の環境からトレイトの `const_conditions` を証明できなければなりません。これは [`wfcheck::check_impl`] でチェックされます。

以下に例を示します：

```rust
const trait Bar {}
const trait Foo: ~const Bar {}
// `const_conditions` には `HostEffect(Self: Bar, maybe)` が含まれます

impl const Bar for () {}
impl const Foo for () {}
// ^ ここで impl が well-formed であるために `const_conditions` をチェックします
```

トレイト impl のメソッドは、実装しているトレイトのメソッドよりも厳しい境界を持ってはいけません。メソッドが互換性があるかをチェックするために、`impl` の述語とトレイトメソッドの述語を組み合わせたハイブリッド環境が構築され、impl メソッドの述語を証明しようとします。`const_conditions` についても同じことを行います：

```rust
const trait Foo {
    fn hi<T: ~const Default>();
}

impl<T: ~const Clone> Foo for Vec<T> {
    fn hi<T: ~const PartialEq>();
    // ^ `T: ~const Clone` と `T: ~const Default` が与えられても
    // `T: ~const PartialEq` を証明できないため、impl のメソッドが
    // トレイトのメソッドよりも厳しいことがわかります。
}
```

これらのチェックは [`compare_method_predicate_entailment`] で行われます。関連型について同じチェックを行う類似の関数は [`compare_type_predicate_entailment`] と呼ばれます。これらは両方とも、const コンテキストにあるときに `const_conditions` を考慮する必要があります。

MIR では、const チェックの一部として、呼び出されるアイテムの `const_conditions` が [`Checker::revalidate_conditional_constness`] で再検証されます。

[`compare_method_predicate_entailment`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_analysis/check/compare_impl_item/fn.compare_method_predicate_entailment.html
[`compare_type_predicate_entailment`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_analysis/check/compare_impl_item/fn.compare_type_predicate_entailment.html
[`FnCtxt::enforce_context_effects`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/fn_ctxt/struct.FnCtxt.html#method.enforce_context_effects
[`wfcheck::check_impl`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_analysis/check/wfcheck/fn.check_impl.html
[`Checker::revalidate_conditional_constness`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/check_consts/check/struct.Checker.html#method.revalidate_conditional_constness

## 関連型とトレイトの `explicit_implied_const_bounds`

関連型、opaque 型、スーパートレイトの境界、例えば：
```rust
trait Foo: ~const PartialEq {
    type X: ~const PartialEq;
}

fn foo() -> impl ~const PartialEq {
    // ^ 未実装の構文
}
```

これらの境界は異なる方法で表現されます。呼び出し元に対して証明する必要があり、定義内で仮定できる `const_conditions`（例：関数のトレイト境界）とは異なり、これらの境界は定義時に証明する必要があります（impl で、または opaque を返すときに）が、呼び出し元に対して仮定できます。これらの境界の非 const 版は [`explicit_item_bounds`] と呼ばれます。

これらの境界は、HIR 型チェックでは [`compare_impl_item::check_type_bounds`]、古いソルバーでは [`evaluate_host_effect_from_item_bounds`]、新しいソルバーでは [`consider_additional_alias_assumptions`] でチェックされます。

[`explicit_item_bounds`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.explicit_item_bounds
[`compare_impl_item::check_type_bounds`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_analysis/check/compare_impl_item/fn.check_type_bounds.html
[`evaluate_host_effect_from_item_bounds`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/traits/effects/fn.evaluate_host_effect_from_item_bounds.html
[`consider_additional_alias_assumptions`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_next_trait_solver/solve/assembly/trait.GoalKind.html#tymethod.consider_additional_alias_assumptions

## `HostEffectPredicate` の証明

`HostEffectPredicate` は[旧ソルバー][old solver]と[新しいトレイトソルバー][new trait solver]の両方で実装されています。一般的に、以下の条件のいずれかが満たされる場合、`HostEffect` 述語を証明できます：

* 述語が呼び出し元の境界から仮定できる場合
* 型がトレイトの `const` `impl` を持っており、*かつ* impl の const 条件が成立し、*かつ* トレイトの `explicit_implied_const_bounds` が成立する場合、または
* 型が const コンテキストでトレイトのビルトイン実装を持っている場合。例えば、`Fn` は const 条件が満たされている場合に関数アイテムによって実装される可能性があり、`Destruct` はコンパイル時に型がドロップできる場合に const コンテキストで実装されます。

[old solver]: https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_trait_selection/traits/effects.rs.html
[new trait solver]: https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_next_trait_solver/solve/effect_goals.rs.html
