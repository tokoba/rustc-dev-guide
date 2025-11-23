# トレイト解決（新）

この章では、[`rustc_trait_selection/solve`][solve]にある新しいWIPソルバーでトレイト解決がどのように機能するかを説明します。[現在のソルバー](../traits/resolution.md)および[chalkソルバー](../traits/chalk.md)のドキュメントも自由に参照してください。

## コア概念

トレイトシステムの目標は、与えられたトレイト境界が満たされているかどうかをチェックすることです。
最も顕著なのは、潜在的にジェネリックな関数の本文を型チェックする際です。
例：

```rust
fn uses_vec_clone<T: Clone>(x: Vec<T>) -> (Vec<T>, Vec<T>) {
    (x.clone(), x)
}
```
ここで、`x.clone()`への呼び出しは、`T: Clone`が真であるという仮定の下で、`Vec<T>`が`Clone`を実装することを証明する必要があります。`T: Clone`を仮定できるのは、
この関数の呼び出し側によって証明されるためです。

「`T: Clone`という仮定の下で`Vec<T>: Clone`を証明する」という概念は[`Goal`]と呼ばれます。
`Vec<T>: Clone`と`T: Clone`の両方は[`Predicate`]を使用して表されます。他の
述語、最も顕著なのは関連アイテムの等価性境界もあります：`<Vec<T> as IntoIterator>::Item == T`。
すべてのリストについては、`PredicateKind`列挙型を参照してください。`Goal`は、証明する必要がある`predicate`と、
この述語が成り立つ必要がある`param_env`として表されます。

与えられたゴールに対して各可能な[`Candidate`]が適用されるかどうかをチェックすることで、ゴールを証明します。
再帰的にそのネストされたゴールを証明します。例を含む可能な候補のリストについては、
[`CandidateSource`]を参照してください。最も重要な候補は、`Impl`候補、つまりユーザーによって書かれたトレイト実装、および`ParamEnv`候補、つまり現在の環境での仮定です。

上記の例を見ると、`Vec<T>: Clone`を証明するために、まず
`impl<T: Clone> Clone for Vec<T>`を使用します。このimplを使用するには、ネストされた
ゴール`T: Clone`が成り立つことを証明する必要があります。これは、`ParamEnv`からの仮定`T: Clone`を使用でき、
ネストされたゴールがありません。したがって、`Vec<T>: Clone`は成り立ちます。

トレイトソルバーは、[`CanonicalResponse`]として成功、曖昧性、またはエラーを返すことができます。
成功と曖昧性の場合、推論と領域制約も返します。

[solve]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/solve/index.html
[`Goal`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_infer/infer/canonical/ir/solve/struct.Goal.html
[`Predicate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Predicate.html
[`Candidate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_next_trait_solver/solve/assembly/struct.Candidate.html
[`CandidateSource`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_infer/infer/canonical/ir/solve/enum.CandidateSource.html
[`CanonicalResponse`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/traits/solve/type.CanonicalResponse.html
