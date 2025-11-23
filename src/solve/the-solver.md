# ソルバー

[chalkの再帰的ソルバー][chalk]のドキュメントも参照してください。
この実装に非常に似ており、このアプローチの制限についても説明しています。

[chalk]: https://rust-lang.github.io/chalk/book/recursive.html

## 大まかなウォークスルー

ソルバーのエントリーポイントは`InferCtxtEvalExt::evaluate_root_goal`です。この
関数はルート`EvalCtxt`をセットアップし、次に`EvalCtxt::evaluate_goal`を呼び出して、
実際にトレイトソルバーに入ります。

`EvalCtxt::evaluate_goal`は、[正規化](./canonicalization.md)、キャッシング、
オーバーフロー、およびソルバーサイクルを処理します。それが完了すると、別のローカル`InferCtxt`を持つネストされた`EvalCtxt`を作成し、`EvalCtxt::compute_goal`を呼び出します。これは
「実際のソルバー動作」を担当します。`PredicateKind`にマッチし、それぞれに対して別の関数に委譲します。

`Vec<T>: Clone`のようなトレイトゴールの場合、`EvalCtxt::compute_trait_goal`は
`EvalCtxt::assemble_and_evaluate_candidates`を介してこのゴールを証明できるすべての可能な方法を収集する必要があります。各候補は、他の候補に推論制約をリークしないように、別の「プローブ」で処理されます。
次に、`EvalCtxt::merge_candidates`を介して組み立てられた候補をマージしようとします。

## 重要な概念と設計パターン

### `EvalCtxt::add_goal`

ネストされたゴールを証明するために、`EvalCtxt::compute_goal`を直接呼び出すのではなく、
代わりに`EvalCtxt::all_goal`でゴールを`EvalCtxt`に追加します。次に、すべてのネストされた
ゴールを`EvalCtxt::try_evaluate_added_goals`または
`EvalCtxt::evaluate_added_goals_and_make_canonical_response`のいずれかで一緒に証明します。これにより、
後のゴールからの推論制約を処理できます。

たとえば、`?x: Debug`と`(): ConstrainToU8<?x>`の両方をネストされたゴールとして持っている場合、
`?x: Debug`を証明することは最初は曖昧ですが、`(): ConstrainToU8<?x>`を証明した後は
`?x`を`u8`に制約し、`u8: Debug`を証明することは成功します。

### `TyKind`でのマッチング

ソルバーで型を遅延正規化するため、型と
定数が潜在的に非正規化されていると常に仮定する必要があります。これは、`TyKind`でのマッチングが簡単に不正確になる可能性があることを意味します。

正規化は2つの異なる方法で処理します。関連型を正規化する際に`Trait`ゴールを証明する際、
自己型に構造的に一致するかどうかに応じて候補を別々に組み立てます。自己型にマッチする候補は、
自己型を1レベル正規化する`EvalCtxt::assemble_candidates_after_normalizing_self_ty`を介して再帰する`EvalCtxt::assemble_candidates_via_self_ty`で処理されます。他のすべての場合で`TyKind`にマッチする必要がある場合、最初に
`EvalCtxt::try_normalize_ty`を使用して型を可能な限り正規化します。

### 高階ランクのゴール

ゴールが高階ランクの場合、例えば`for<'a> F: FnOnce(&'a ())`、`EvalCtxt::compute_goal`は
`'a`をプレースホルダで熱心にインスタンス化し、次に再帰的に
`F: FnOnce(&'!a ())`をネストされたゴールとして証明します。

### 選択の扱い

一部のゴールは複数の方法で証明できます。これらの場合、各オプションを
別の「プローブ」で試し、次に`EvalCtxt::try_merge_responses`を使用して結果のレスポンスをマージしようとします。レスポンスのマージに失敗した場合、代わりに`EvalCtxt::flounder`を使用して曖昧性を返します。一部のゴールでは、`EvalCtxt::try_merge_responses`が
失敗した場合に、一部の選択を他の選択よりも不完全に優先しようとします。

## さらに学ぶ

ソルバーはかなり自己完結型であるべきです。上記の情報が、コード自体を見る際に良い基盤を提供することを願っています。行き詰まった場合、またはいくつかの特異性や設計決定が不明確で、より良いコメントに値するか、ここで言及すべき場合は、Zulipで連絡してください。
