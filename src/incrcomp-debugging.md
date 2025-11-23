# 依存関係のデバッグとテスト

## 依存グラフのテスト

依存グラフに対してテストを書くには、さまざまな方法があります。最も簡単なメカニズムは、
`#[rustc_if_this_changed]`と`#[rustc_then_this_would_need]`アノテーションです。
これらは、[ui]テストで使用され、依存グラフに期待されるパスのセットが存在するかどうかをテストします。

[`tests/ui/dep-graph/dep-graph-caller-callee.rs`]: https://github.com/rust-lang/rust/blob/HEAD/tests/ui/dep-graph/dep-graph-caller-callee.rs
[ui]: tests/ui.html

例として、[`tests/ui/dep-graph/dep-graph-caller-callee.rs`]、または以下のテストを参照してください。

```rust,ignore
#[rustc_if_this_changed]
fn foo() { }

#[rustc_then_this_would_need(TypeckTables)] //~ ERROR OK
fn bar() { foo(); }
```

これは次のように読むべきです
> これ（`foo`）が変更された場合、これ（つまり`bar`）のTypeckTablesは変更される必要があります。

技術的には、テストは、この行に関連付けられた文字列「OK」をstderrに出力することが期待されます。

次の行を追加することもできます

```rust,ignore
#[rustc_then_this_would_need(TypeckTables)] //~ ERROR no path
fn baz() { }
```

その意味は
> `foo`が変更された場合、`baz`のTypeckTablesは変更される必要はありません。
> マクロはエラーを出力する必要があり、エラーメッセージには「no path」が含まれている必要があります。

`//~ ERROR OK`は、テストするRustコードの観点からはコメントですが、
テスト自体の観点からは意味があることを思い出してください。

## 依存グラフのデバッグ

### グラフのダンプ

コンパイラは、デバッグの楽しみのために依存グラフをダンプすることもできます。
そのためには、`-Z dump-dep-graph`フラグを渡します。グラフは、
現在のディレクトリの`dep_graph.{txt,dot}`にダンプされます。
`RUST_DEP_GRAPH`環境変数でファイル名を上書きできます。

ただし、多くの場合、完全な依存グラフは非常に圧倒的で、
特に役に立ちません。したがって、コンパイラはグラフをフィルタリングすることもできます。
3つの方法でフィルタリングできます。

1. 特定のノードセット（通常は単一のノード）から発生するすべてのエッジ。
2. 特定のノードセットに到達するすべてのエッジ。
3. 指定された開始ノードと終了ノードの間にあるすべてのエッジ。

フィルタリングするには、`RUST_DEP_GRAPH_FILTER`環境変数を使用します。
これは、次のいずれかのようになります。

```text
source_filter     // nodes originating from source_filter
-> target_filter  // nodes that can reach target_filter
source_filter -> target_filter // nodes in between source_filter and target_filter
```

`source_filter`と`target_filter`は、文字列の`&`区切りリストです。
ノードは、それらの文字列がすべてラベルに表示される場合、フィルターに一致すると見なされます。
したがって、例えば：

```text
RUST_DEP_GRAPH_FILTER='-> TypeckTables'
```

すべての`TypeckTables`ノードの先行ノードを選択します。ただし、通常は
特定のfnの`TypeckTables`ノードが必要なので、次のように書くかもしれません。

```text
RUST_DEP_GRAPH_FILTER='-> TypeckTables & bar'
```

これは、名前に`bar`を含む関数の`TypeckTables`ノードの先行ノードのみを選択します。

おそらく、`foo`を変更すると`bar`を再型チェックする必要があることがわかりますが、
そうする必要はないと思います。その場合、次のようにするかもしれません。

```text
RUST_DEP_GRAPH_FILTER='Hir & foo -> TypeckTables & bar'
```

これは、`Hir(foo)`から`TypeckTables(bar)`に至るすべてのノードをダンプします。
そこから（うまくいけば）誤ったエッジのソースを見ることができます。

### 誤ったエッジの追跡

依存グラフをダンプした後、存在すべきではないパスを見つけることがありますが、
それがどのように発生したかはよくわかりません。**コンパイラがデバッグアサーションでビルドされている場合、**
それを追跡するのに役立ちます。`RUST_FORBID_DEP_GRAPH_EDGE`環境変数を
フィルターに設定するだけです。依存グラフで作成されたすべてのエッジは、
そのフィルターに対してテストされます。一致する場合、`bug!`が報告されるので、
バックトレースを簡単に確認できます（`RUST_BACKTRACE=1`）。

これらのフィルターの構文は、前のセクションで説明したものと同じです。ただし、
このフィルターは、すべての**エッジ**に適用され、
前のセクションとは異なり、グラフ内のより長いパスを処理しないことに注意してください。

例：

`foo`の`Hir`から`bar`の型チェックへのパスがあり、そうあるべきではないと思います。
前のセクションで説明したように依存グラフをダンプし、`dep-graph.txt`を開いて
次のようなものを見ます。

```text
Hir(foo) -> Collect(bar)
Collect(bar) -> TypeckTables(bar)
```

その最初のエッジは怪しいと思います。そこで、
`RUST_FORBID_DEP_GRAPH_EDGE`を`Hir&foo -> Collect&bar`に設定し、再実行して、
バックトレースを観察します。 Voila、バグが修正されました！
