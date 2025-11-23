# 例：`rustc_driver`を使用した型チェック

[`rustc_driver`]を使用すると、コンパイルのさまざまな段階でRustコードとやり取りすることができます。

## 式の型を取得する

式の型を取得するには、[`after_analysis`]コールバックを使用して[`TyCtxt`]を取得します。

```rust
{{#include ../../examples/rustc-driver-interacting-with-the-ast.rs}}
```

[`after_analysis`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver/trait.Callbacks.html#method.after_analysis
[`rustc_driver`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver
[`TyCtxt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html
