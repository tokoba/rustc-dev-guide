# HIR

HIR（「High-Level Intermediate Representation」：高レベル中間表現）は、rustc のほとんどで使用される主要な IR です。これは、パース、マクロ展開、名前解決の後に生成される抽象構文木（AST）のコンパイラフレンドリーな表現です（HIR がどのように作成されるかについては、[Lowering](./hir/lowering.md)を参照してください）。HIR の多くの部分は Rust の表面構文に非常に似ていますが、Rust の式の形式の一部がデシュガリングされている点が異なります。例えば、`for` ループは `loop` に変換され、HIR には現れません。これにより、HIR は通常の AST よりも分析に適しています。

この章では、HIR の主要な概念について説明します。

`-Z unpretty=hir-tree` フラグを rustc に渡すことで、コードの HIR 表現を表示できます：

```bash
cargo rustc -- -Z unpretty=hir-tree
```

また、`-Z unpretty=hir` オプションを使用して、元のソースコード式に近い HIR を生成することもできます：

```bash
cargo rustc -- -Z unpretty=hir
```

## アウトオブバンドストレージと `Crate` 型

HIR のトップレベルのデータ構造は [`Crate`] で、現在コンパイルされているクレートの内容を格納します（現在のクレートに対してのみ HIR を構築します）。AST ではクレートのデータ構造が基本的にルートモジュールを含むだけであるのに対し、HIR の `Crate` 構造には、より簡単にアクセスできるようにクレートの内容を整理するための多数のマップやその他のものが含まれています。

[`Crate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Crate.html

例えば、HIR 内の個々のアイテム（例：モジュール、関数、トレイト、impl など）の内容は、親ですぐにアクセスできるわけではありません。したがって、例えば、関数 `bar()` を含むモジュールアイテム `foo` がある場合：

```rust
mod foo {
    fn bar() { }
}
```

HIR では、モジュール `foo` の表現（[`Mod`] 構造体）は、`bar()` の **`ItemId`** `I` のみを持ちます。関数 `bar()` の詳細を取得するには、`items` マップで `I` を検索します。

[`Mod`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Mod.html

この表現の良い結果の1つは、これらのマップのキーと値のペアを反復処理することで、クレート内のすべてのアイテムを反復処理できることです（HIR 全体をトラバースする必要はありません）。トレイトアイテムや impl アイテム、および「ボディ」（後述）についても同様のマップがあります。

この方法で表現をセットアップするもう1つの理由は、インクリメンタルコンパイルとのより良い統合のためです。この方法では、[`&rustc_hir::Item`] にアクセスした場合（例：mod `foo` の場合）、関数 `bar()` の内容に即座にアクセスできません。代わりに、`bar()` の **id** にのみアクセスでき、その id が与えられた場合に `bar()` の内容を検索する関数を呼び出す必要があります。これにより、コンパイラは `bar()` のデータにアクセスしたことを観察し、依存関係を記録する機会を得ます。

[`&rustc_hir::Item`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Item.html

<a id="hir-id"></a>

## HIR の識別子

HIR は、共存してさまざまな目的を果たすさまざまな識別子を使用します。

- [`DefId`] は、名前が示すように、特定の定義、つまりトップレベルのアイテムを特定のクレート内で識別します。これは2つの部分で構成されます：定義が来るクレートを識別する [`CrateNum`] と、クレート内の定義を識別する [`DefIndex`]。[`HirId`] とは異なり、すべての式に対して [`DefId`] があるわけではないため、コンパイル間でより安定しています。

- [`LocalDefId`] は基本的に、現在のクレートから来ることが知られている [`DefId`] です。これにより、[`CrateNum`] 部分を削除し、型システムを使用してローカル定義のみが期待される関数に渡されることを保証できます。

- [`HirId`] は、現在のクレートの HIR 内のノードを一意に識別します。`owner` と `owner` 内で一意な `local_id` の2つの部分で構成されます。この組み合わせにより、インクリメンタルコンパイルに役立つより安定した値が得られます。[`DefId`] とは異なり、[`HirId`] は式などの[きめ細かいエンティティ][Node]を参照できますが、現在のクレート内にとどまります。

- [`BodyId`] は、現在のクレート内の HIR [`Body`] を識別します。現在、これは [`HirId`] のラッパーに過ぎません。HIR ボディの詳細については、[HIR 章][hir-bodies]を参照してください。

これらの識別子は、`TyCtxt` を介して相互に変換できます。

[`DefId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/def_id/struct.DefId.html
[`LocalDefId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/def_id/struct.LocalDefId.html
[`HirId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/struct.HirId.html
[`BodyId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.BodyId.html
[Node]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/enum.Node.html
[`CrateNum`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/def_id/struct.CrateNum.html
[`DefIndex`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/def_id/struct.DefIndex.html
[`Body`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Body.html
[hir-bodies]: ./hir.md#hir-bodies

## HIR 操作

ほとんどの場合、HIR を扱うときは、`TyCtxt` を介して行います。これには、`hir::map` モジュールで定義され、ほとんどが `hir_` プレフィックスを持つ多数のメソッドが含まれており、さまざまな種類の ID を変換したり、HIR ノードに関連付けられたデータを検索したりします。


例えば、[`LocalDefId`] があり、それを [`HirId`] に変換したい場合は、[`tcx.local_def_id_to_hir_id(def_id)`][local_def_id_to_hir_id] を使用できます。ローカルアイテムのみが HIR ノードを持つため、`DefId` ではなく `LocalDefId` が必要です。

[local_def_id_to_hir_id]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.local_def_id_to_hir_id

同様に、[`tcx.hir_node(n)`][hir_node] を使用して [`HirId`] のノードを検索できます。これは `Option<Node<'hir>>` を返します。ここで [`Node`] はマップで定義された列挙型です。これに一致させることで、`HirId` がどのようなノードを指していたかを判断し、データ自体へのポインタを取得できます。多くの場合、`n` がどのような種類のノードであるかがわかっています。例えば、`n` が何らかの HIR 式でなければならないことがわかっている場合は、[`tcx.hir_expect_expr(n)`][expect_expr] を使用できます。これは [`&hir::Expr`][Expr] を抽出して返し、`n` が実際に式でない場合はパニックします。

[hir_node]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.hir_node
[`Node`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/enum.Node.html
[expect_expr]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.expect_expr
[Expr]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Expr.html

最後に、[`tcx.parent_hir_node(n)`][parent_hir_node] のような呼び出しを介してノードの親を見つけることができます。

[parent_hir_node]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.parent_hir_node

## HIR ボディ

[`rustc_hir::Body`] は、関数/クロージャの本体や定数の定義など、何らかの実行可能なコードを表します。ボディは **owner** に関連付けられており、これは通常何らかのアイテム（例：`fn()` または `const`）ですが、クロージャ式（例：`|x, y| x + y`）である場合もあります。`TyCtxt` を使用して、特定の def-id に関連付けられたボディを見つけたり（[`hir_maybe_body_owned_by`]）、ボディの所有者を見つけたりできます（[`hir_body_owner_def_id`]）。

[`rustc_hir::Body`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Body.html
[`hir_maybe_body_owned_by`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.hir_maybe_body_owned_by
[`hir_body_owner_def_id`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.hir_body_owner_def_id
