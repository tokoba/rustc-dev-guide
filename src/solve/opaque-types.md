# 新しいソルバーにおける不透明型

新しいソルバーにおける[不透明型]の処理方法は、古い実装とは異なります。
これは、新しいソルバーにおける動作の自己完結型の説明であるべきです。

[不透明型]: ../opaque-types-type-alias-impl-trait.md

## 不透明型はエイリアス型である

不透明型は、可能な限り他のエイリアス、特に関連型と同じように扱われます。動作の違いは可能な限り少なくする必要があります。

これは、隠蔽型に正規化でき、完全性に対して同じ要件を持つという点で、他のエイリアス型に非常に似ているため、望ましいです。
このように扱うことで、コードを共有することにより、型システムの複雑さも軽減されます。
不透明型を別々に扱う必要があると、より複雑なルールと新しい種類の
相互作用が発生します。暗黙的な負の
モードではそれらを他のエイリアスのように扱う必要があるため、モード間で大きな違いがあると複雑さも増します。

*未解決の質問：ここに代替アプローチがあるかもしれません。より制限された場所でインスタンス化することで、剛体型のようにより多く扱う可能性があります。コヒーレンス中は依然として通常の
エイリアスである必要があります*

### 不透明型の`normalizes-to`

[ソース][norm]

`normalizes-to`は、新しい
ソルバーにおけるエイリアスの1ステップ正規化動作を定義するために使用されます：`<<T as IdInner>::Assoc as IdOuter>::Assoc`は最初に`<T as IdInner>::Assoc`に
正規化され、次に`T`に正規化されます。これは、正規化される`AliasTy`と期待される
`Term`の両方を受け取ります。実際の正規化のために`normalizes-to`を使用するには、期待されるタームを単純に
制約されていない推論変数にすることができます。

定義スコープ内および暗黙的な負のコヒーレンスモード内の不透明型については、これは
常に2つのステップで行われます。定義スコープの外では、不透明型の`normalizes-to`は
常に`Err(NoSolution)`を返します。

まず、期待される型を隠蔽型として割り当てようとします。

暗黙的な負のコヒーレンスモードでは、これは現在、不透明型ストレージと相互作用せずに常に曖昧性を引き起こします。代わりに、すべての不透明型の「定義」を許可し、
最後にそれらの推論された型を破棄することもできます。これにより、コヒーレンス中に不透明型が複数回使用される場合の動作が変更されます：[例][coherence-example]

定義スコープ内では、まず、不透明の型引数と定数引数がすべてプレースホルダであるかをチェックします：[ソース][placeholder-ck]。このチェックが曖昧な場合は、
曖昧性を返し、失敗した場合は`Err(NoSolution)`を返します。このチェックは、
borrowckの最後にのみチェックされる領域を無視します。成功した場合は、続行します。

次に、不透明のジェネリック引数を、不透明型ストレージにすでにある任意の不透明型の引数と*意味的に*統一できるかどうかをチェックします。そうであれば、以前に保存された型とこの`normalizes-to`呼び出しの期待される型を統一します：[ソース][eq-prev][^1]。

そうでない場合は、期待される型を不透明型ストレージに挿入します：[ソース][insert-storage][^2]。
最後に、不透明のアイテム境界が期待される型に対して成り立つかをチェックします：
[ソース][item-bounds-ck]。

[norm]: https://github.com/rust-lang/rust/blob/384d26fc7e3bdd7687cc17b2662b091f6017ec2a/compiler/rustc_trait_selection/src/solve/normalizes_to/opaque_types.rs#L13
[coherence-example]: https://github.com/rust-lang/rust/blob/HEAD/tests/ui/type-alias-impl-trait/coherence/coherence_different_hidden_ty.rs
[placeholder-ck]: https://github.com/rust-lang/rust/blob/384d26fc7e3bdd7687cc17b2662b091f6017ec2a/compiler/rustc_trait_selection/src/solve/normalizes_to/opaque_types.rs#L33
[eq-prev]: https://github.com/rust-lang/rust/blob/384d26fc7e3bdd7687cc17b2662b091f6017ec2a/compiler/rustc_trait_selection/src/solve/normalizes_to/opaque_types.rs#L51-L59
[insert-storage]: https://github.com/rust-lang/rust/blob/384d26fc7e3bdd7687cc17b2662b091f6017ec2a/compiler/rustc_trait_selection/src/solve/normalizes_to/opaque_types.rs#L68
[item-bounds-ck]: https://github.com/rust-lang/rust/blob/384d26fc7e3bdd7687cc17b2662b091f6017ec2a/compiler/rustc_trait_selection/src/solve/normalizes_to/opaque_types.rs#L69-L74

[^1]: FIXME: argsがプレースホルダであることを要求し、領域が常に推論変数であることを考えると、これは理想的には一意の候補のみをもたらすはずです
[^2]: FIXME: なぜ期待される型が剛体であることをチェックするのか。

### 正規化可能なエイリアスのエイリアス境界を使用する

<https://github.com/rust-lang/trait-system-refactor-initiative/issues/77>

正規化可能なエイリアスに対して`AliasBound`候補を使用することは、一般的には不可能です。なぜなら、関連型は、
`ParamEnv`候補を介して正規化する際の結果の型よりも強い境界を持つことができるからです。

これらの候補は、正確な正規化戦略をユーザーに見えるように変更します。熱心に正規化するかどうかは、それ以外では
ほとんど観察できません。正規化する場所は、古いソルバーのサポートを削除した後に変更したいと思う可能性が高いため、これは望ましくありません。

## 不透明型はどこでも定義できる

定義スコープ内の不透明型は、単に型を関連付ける際やトレイトソルバー内など、どこでも定義できます。これにより、順序依存性と不完全性が削除されます。これがないと、ゴールの結果は、
不透明の最初の定義使用の前に不透明を使用してゴールを評価しようとするかどうかなど、微妙な理由により異なる可能性があります。

## 定義スコープ内の高階ランクの不透明型

これらはサポートされておらず、現在それらを定義しようとすると常にエラーになるはずです。

FIXME: 不透明型ストレージ内の不透明型を検索すると領域を統一できるようになったため、
不透明型がプレースホルダを参照していないことを熱心にチェックする必要があります。そうしないと、
プレースホルダをリークすることになります。

## メンバー制約

メンバー制約の処理は、新しいソルバーでは変わりません。そのための
[関連する既存の章][member-constraints]を参照してください。

[member-constraints]: ../borrow_check/region_inference/member_constraints.md

## 不透明型でメソッドを呼び出す

FIXME: 定義スコープ内でまだ制約されていない
不透明型でメソッドを呼び出すサポートを継続する必要があります。これを最善に行う方法は不明です。

```rust
use std::future::Future;
use futures::FutureExt;

fn go(i: usize) -> impl Future<Output = ()> + Send + 'static {
    async move {
        if i != 0 {
            // これは定義スコープ内で`impl Future<Output = ()>`を返しますが、
            // その不透明の具体的な型はこの時点ではわかりません。
            // 現在、不透明を既知の型として扱い、成功しますが、
            // 「健全に実装するのが最も簡単」という観点からは、これは
            // 曖昧であるべきです。
            go(i - 1).boxed().await;
        }
    }
}
```
