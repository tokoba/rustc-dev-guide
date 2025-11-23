# `minicore` テスト補助：`core` スタブの使用

<!-- date-check: Oct 2025 -->

[`tests/auxiliary/minicore.rs`][`minicore`] は、ui/codegen/assembly テストスイートのためのテスト補助です。
クロスコンパイルされたターゲット向けにビルドする必要があるが、実行する必要がない/したくないテストに対して、`core` スタブを提供します。

<div class="warning">

[`minicore`] は `core` アイテムのみを対象としており、`std` や `alloc` アイテムは**明示的に対象外**であることに注意してください。これは、`core` アイテムがより広範なテストに適用できるためです。

</div>

テストで [`minicore`] を使用するには、`//@ add-minicore` ディレクティブを指定します。
次に、`#![feature(no_core)]` + `#![no_std]` + `#![no_core]` でテストをマークし、
`extern crate minicore` (edition 2015) または `use minicore` (edition 2018+) でクレートをテストにインポートします。

## 暗黙のコンパイラフラグ

これらのテストは `no_std` + `no_core` の性質上、`//@ add-minicore` は暗黙的にテストが `-C panic=abort` でビルドされることを意味し、それを必要とします。
**巻き戻しパニックはサポートされていません。**

テストはまた、アセンブリテストで CFI ディレクティブを保持するために `-C force-unwind-tables=yes` でビルドされます。

まとめると：`//@ add-minicore` は2つのコンパイラフラグを暗黙的に指定します：

1. `-C panic=abort`
2. `-C force-unwind-tables=yes`

## `core` スタブの追加

[`minicore`] スタブに欠けている `core` アイテムを見つけた場合、それが使用される可能性が高い場合、または既に複数のテストで必要とされている場合は、テスト補助に追加することを検討してください。

## `core` との同期を維持

`minicore` アイテムは `core` と同期を保つ必要があります。
`core` と `minicore` を使用したときの診断出力を一貫させるため、`diagnostic` 属性（例：`on_unimplemented`）は `minicore` で正確に複製する必要があります。

## `minicore` を使用するコード生成テストの例

```rust,no_run
//@ add-minicore
//@ revisions: meow bark
//@[meow] compile-flags: --target=x86_64-unknown-linux-gnu
//@[meow] needs-llvm-components: x86
//@[bark] compile-flags: --target=wasm32-unknown-unknown
//@[bark] needs-llvm-components: webassembly

#![crate_type = "lib"]
#![feature(no_core)]
#![no_std]
#![no_core]

extern crate minicore;
use minicore::*;

struct Meow;
impl Copy for Meow {} // ここでの `Copy` は `minicore` によって提供されます

// CHECK-LABEL: meow
#[unsafe(no_mangle)]
fn meow() {}
```

[`minicore`]: https://github.com/rust-lang/rust/tree/HEAD/tests/auxiliary/minicore.rs
