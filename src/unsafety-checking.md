# Unsafetyチェック

Rustの特定の式はメモリ安全性を侵害する可能性があるため、`unsafe`ブロックまたは関数内にある必要があります。コンパイラは、unsafe操作に対応するunsafeブロックが使用されていない場合にも警告します。

## 概要

unsafetyチェックは[`check_unsafety`]モジュールにあります。これは、関数とそのすべてのクロージャおよびインライン定数の[THIR]をウォークスルーします。unsafeコンテキストを追跡し続けます：`unsafe`ブロックに入ったかどうか。unsafe操作が`unsafe`ブロックの外で使用されている場合、エラーが報告されます。unsafe操作がunsafeブロックで使用されている場合、そのブロックは[未使用unsafeリント](#the-unused_unsafe-lint)のために使用済みとしてマークされます。

unsafetyチェックには型情報が必要なので、HIRで行うこともできます。型チェックの結果、THIR、またはMIRを使用して行うことができます。THIRが選ばれているのは、HIRよりも考慮すべきケースが少ないからです。たとえば、unsafe関数呼び出しとunsafeメソッド呼び出しは、THIRでは同じ表現を持っています。このチェックはMIRで行われません。安全性チェックは制御フローに依存しないため、MIRを使用する必要がなく、MIRは一部の式に対してHIRほど正確なスパンを持っていないからです。

ほとんどのunsafe操作は、THIRで`ExprKind`をチェックし、引数の型をチェックすることで識別できます。たとえば、生ポインタの逆参照は、引数が生ポインタ型を持つ`ExprKind::Deref`に対応します。

unsafeなUnionフィールドアクセスを探すのはもう少し複雑です。unionのフィールドへの書き込みは安全だからです。チェッカーは、割り当て式の左辺を訪問しているときを追跡し、unionフィールドがそこに直接現れることを許可し、他のすべてのケースでエラーを報告します。unionフィールドアクセスはパターンでも発生する可能性があるため、それらもウォークする必要があります。

もう1つの複雑な安全性チェックは、レイアウト制約のある構造体（[`NonNull`]など）のフィールドへの書き込みに対するものです。これらは、借用または割り当て式を探し、借用または割り当てされるサブ式を別のビジターで訪問することによって見つけられます。

[THIR]: ./thir.md
[`check_unsafety`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_build/check_unsafety/index.html
[`NonNull`]: https://doc.rust-lang.org/std/ptr/struct.NonNull.html

## unused_unsafeリント

unused_unsafeリントは、削除できる`unsafe`ブロックを報告します。unsafetyチェッカーは、unsafeを必要とする操作を見つけるたびに記録します。次のいずれかの場合にリントが報告されます：

- `unsafe`ブロックにunsafe操作が含まれていない
- `unsafe`ブロックが別のunsafeブロック内にあり、外側のブロックが未使用と見なされていない

```rust
#![deny(unused_unsafe)]
let y = 0;
let x: *const u8 = core::ptr::addr_of!(y);
unsafe { // このブロックに対してリントが報告されます
    unsafe {
        let z = *x;
    }
    let safe_expr = 123;
}
unsafe {
    unsafe { // このブロックに対してリントが報告されます
        let z = *x;
    }
    let unsafe_expr = *x;
}
```

## `unsafe`に関連する他のチェック

[Unsafeトレイト]を実装するには`unsafe impl`が必要で、このチェックは[coherence]の一部として行われます。`unsafe_code`リントは、unsafeブロック、関数、実装、および特定のunsafe属性を検索するastのlintパスとして実行されます。

[Unsafeトレイト]: https://doc.rust-lang.org/reference/items/traits.html#unsafe-traits
[coherence]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_hir_analysis/src/coherence/unsafety.rs
