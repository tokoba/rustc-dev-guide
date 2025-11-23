# コヒーレンス

> 注：これは [@lcnr によるノート](https://github.com/rust-lang/rust/pull/121848)に基づいています

コヒーレンスチェックは、トレイト impl と固有 impl の両方が他のものと重複していることを検出するものです。
（リマインダー：[固有 impl](https://doc.rust-lang.org/reference/items/implementations.html#inherent-implementations) は、`impl MyStruct {}` のような具体的な型の impl です）

重複するトレイト impl は常にエラーを生成しますが、
重複する固有 impl は、同じ名前のメソッドを持つ場合にのみエラーになります。

重複のチェックは 2 つの部分に分かれています。まず、[重複チェック](#overlap-checks)があり、
これはコンパイラが現在知っているトレイトと固有実装間の重複を見つけます。

ただし、コヒーレンスは、現在未知であっても、他の impl が**存在する可能性がある**場合にもエラーになります。
これは、上流のクレートに後方互換性のある方法で追加される可能性のある impl と、
下流のクレートからの impl に影響します。
これはオーファンチェックと呼ばれます。

## 重複チェック

重複チェックは、固有 impl とトレイト impl の両方に対して実行されます。
これは実際には 2 つの別個の解析として行われる同じ重複チェックコードを使用します。
重複チェックは常に実装のペアを考慮し、それらを互いに比較します。

固有 impl ブロックの重複チェックは `fn check_item`（coherence/inherent_impls_overlap.rs 内）を通じて行われ、
そこで非常に明確に（少なくとも小さな `n` の場合）、チェックが impl 間で `n^2`
比較を実行することがわかります。

トレイトの場合、このチェックは現在[特殊化グラフ](traits/specialization.md)の構築の一部として行われ、
親と重複する特殊化 impl を処理しますが、これは将来変更される可能性があります。

どちらの場合も、すべての impl のペアが重複についてチェックされます。

重複は時々部分的に許可されます：

1. マーカートレイトの場合
2. [特殊化](traits/specialization.md)下で

しかし、通常は許可されません。

重複チェックには様々なモードがあります（[`OverlapMode`] を参照）。
重要なのは、明示的な負の impl チェックと暗黙的な負の impl チェックです。
どちらも重複が確実に不可能であることを証明しようとします。

[`OverlapMode`]: https://doc.rust-lang.org/beta/nightly-rustc/rustc_middle/traits/specialization_graph/enum.OverlapMode.html

### 明示的な負の impl チェック

このチェックは [`impl_intersection_has_negative_obligation`] で行われます。

このチェックは負のトレイト実装を見つけようとします。
例：

```rust
struct MyCustomErrorType;

// 両方ともあなた自身のクレート内
impl From<&str> for MyCustomErrorType {}
impl<E> From<E> for MyCustomErrorType where E: Error {}
```

この例では、次のようになります：
`MyCustomErrorType: From<&str>` と `MyCustomErrorType: From<?E>`、これにより `?E = &str` が得られます。

したがって、これら 2 つの実装は重複します。
ただし、libstd は `&str: !Error` を提供しており、したがって
`&str: Error` の正の実装が決して存在しないことを保証し、したがって重複はありません。

このタイプの負の impl チェックでは、明示的な負の実装が提供されている必要があることに注意してください。
これは現在安定していません。

[`impl_intersection_has_negative_obligation`]: https://doc.rust-lang.org/beta/nightly-rustc/rustc_trait_selection/traits/coherence/fn.impl_intersection_has_negative_obligation.html

### 暗黙的な負の impl チェック

このチェックは [`impl_intersection_has_impossible_obligation`] で行われ、
負のトレイト実装に依存せず、安定しています。

次のようなものがあるとしましょう

```rust
impl From<MyLocalType> for Box<dyn Error> {}  // あなた自身のクレート内
impl<E> From<E> for Box<dyn Error> where E: Error {} // std 内
```

これにより次のようになります：`Box<dyn Error>: From<MyLocalType>`、および `Box<dyn Error>: From<?E>`、
`?E = MyLocalType` が得られます。

あなたのクレートには `MyLocalType: Error` がなく、下流のクレートは `Error`（リモートトレイト）を `MyLocalType`（リモート型）に実装できません。
したがって、これら 2 つの impl は重複しません。
重要なのは、これは `impl !Error for MyLocalType` がなくても機能します。

[`impl_intersection_has_impossible_obligation`]: https://doc.rust-lang.org/beta/nightly-rustc/rustc_trait_selection/traits/coherence/fn.impl_intersection_has_impossible_obligation.html
