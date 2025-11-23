# 例：`rustc_interface`を使用した診断情報の取得

[`rustc_interface`]を使用すると、通常stderrに出力される診断情報を
インターセプトすることができます。

## 診断情報の取得

コンパイラから診断情報を取得するには、
診断情報をバッファに出力するように[`rustc_interface::Config`]を設定し、
各アイテムに対して[`rustc_hir_typeck::typeck`]を実行します。

```rust
{{#include ../../examples/rustc-interface-getting-diagnostics.rs}}
```

[`rustc_interface`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/index.html
[`rustc_interface::Config`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/interface/struct.Config.html
[`rustc_hir_typeck::typeck`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/fn.typeck.html
