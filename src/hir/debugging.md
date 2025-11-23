# HIRのデバッグ

`-Z unpretty=hir`フラグを使用して、HIRの人間が読める表現を生成します。
cargoプロジェクトの場合、`cargo rustc -- -Z unpretty=hir`で実行できます。
この出力は、AST loweringの間にコードがどのように脱糖され、変換されたかを一目で確認する必要がある場合に役立ちます。

HIR内のデータの完全な`Debug`ダンプについては、`-Z unpretty=hir-tree`フラグを使用してください。
これは、コンパイラの視点からHIRの完全な構造を確認する必要がある場合に役立つ可能性があります。

`NodeId`または`DefId`をソースコードと関連付けようとしている場合は、
`-Z unpretty=expanded,identified`フラグが役立つ可能性があります。

TODO: 他に何かありますか？ [#1159](https://github.com/rust-lang/rustc-dev-guide/issues/1159)
