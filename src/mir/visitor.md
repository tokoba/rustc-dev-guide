# MIRビジター

MIRビジターは、MIRを走査し、何かを探したり、変更を加えたりするための便利なツールです。ビジタートレイトは[`rustc_middle::mir::visit`モジュール][m-v]で定義されています。2つあり、単一のマクロを介して生成されます：`Visitor`（`&Mir`を操作し、共有参照を返します）と`MutVisitor`（`&mut Mir`を操作し、可変参照を返します）。

[m-v]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/visit/index.html

ビジターを実装するには、ビジターを表す型を作成する必要があります。通常、この型は、MIRを処理する際に必要な状態を「保持」したいと考えます：

```rust,ignore
struct MyVisitor<...> {
    tcx: TyCtxt<'tcx>,
    ...
}
```

次に、その型に対して`Visitor`または`MutVisitor`トレイトを実装します：

```rust,ignore
impl<'tcx> MutVisitor<'tcx> for MyVisitor {
    fn visit_foo(&mut self, ...) {
        ...
        self.super_foo(...);
    }
}
```

上記のように、impl内で、`visit_foo`メソッド（例：`visit_terminator`）のいずれかをオーバーライドして、`foo`が見つかったときに実行されるコードを書くことができます。`foo`の内容を再帰的に歩きたい場合は、`super_foo`メソッドを呼び出します。（注：`super_foo`をオーバーライドしたくはありません。）

非常に簡単なビジターの例は、[`LocalFinder`]にあります。`visit_local`メソッドを実装することで、このビジターは、並べ替えの候補となるローカル変数を識別します。

[`LocalFinder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_transform/prettify/struct.LocalFinder.html

## トラバーサル

ビジターに加えて、[`rustc_middle::mir::traversal`モジュール][t]には、MIR CFGを[さまざまな標準順序][traversal]（例：事前順序、逆事後順序など）で歩くための便利な関数が含まれています。

[t]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/traversal/index.html
[traversal]: https://en.wikipedia.org/wiki/Tree_traversal
