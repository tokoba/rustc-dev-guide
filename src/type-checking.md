# 型チェック

[`hir_analysis`]クレートには、「型収集」のソース、および関連する機能の束が含まれています。関数の本体のチェックは[`hir_typeck`]クレートで実装されています。これらのクレートは、[型推論]と[トレイト解決]に大きく依存しています。

[`hir_analysis`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_analysis/index.html
[`hir_typeck`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/index.html
[型推論]: ./type-inference.md
[トレイト解決]: ./traits/resolution.md

## 型収集

型「収集」は、HIRで見つかった型（`hir::Ty`）を、ユーザーが書いた構文的なものを表すものから、コンパイラが使用する**内部表現**（`Ty<'tcx>`）に変換するプロセスです。同様の変換をwhere句や関数シグネチャの他の部分にも行います。

違いを理解するために、この関数を考えてみましょう：

```rust,ignore
struct Foo { }
fn foo(x: Foo, y: self::Foo) { ... }
//        ^^^     ^^^^^^^^^
```

これら2つのパラメータ`x`と`y`は、それぞれ同じ型を持っていますが、異なる`hir::Ty`ノードを持ちます。これらのノードは異なるスパンを持ち、もちろんパスを多少異なるようにエンコードします。しかし、`Ty<'tcx>`ノードに「収集」されると、まったく同じ内部型によって表現されます。

収集は、コンパイルされるクレート内のさまざまな関数、トレイト、およびその他のアイテムに関する情報を計算するための[クエリ]のバンドルとして定義されています。これらのクエリはそれぞれ、*プロシージャ間*のものに関係していることに注意してください。たとえば、関数定義の場合、収集は関数の型とシグネチャを解明しますが、関数の*本体*を何らかの方法で訪問したり、ローカル変数の型注釈を調べたりすることはありません（それは型*チェック*の仕事です）。

詳細については、[`collect`][collect]モジュールを参照してください。

[クエリ]: ./query.md
[collect]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_analysis/collect/index.html

**TODO**：実際に型チェックについて話す... [#1161](https://github.com/rust-lang/rustc-dev-guide/issues/1161)
