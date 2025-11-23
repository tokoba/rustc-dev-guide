# 曖昧/非曖昧な型と定数

HIRにおける型と定数の引数は、曖昧(ambig)または非曖昧(unambig)という2種類の位置に配置される可能性があります。曖昧な位置とは、型または定数のいずれかとして解析することが有効な位置であり、非曖昧な位置とは、1種類のみが有効に解析される位置です。

```rust
fn func<T, const N: usize>(arg: T) {
    //                           ^ 非曖昧な型の位置
    let a: _ = arg;
    //     ^ 非曖昧な型の位置

    func::<T, N>(arg);
    //     ^  ^
    //     ^^^^ 曖昧な位置

    let _: [u8; 10];
    //      ^^  ^^ 非曖昧な定数の位置
    //      ^^ 非曖昧な型の位置
}

```

曖昧な位置にあるほとんどの型/定数は、パース中に型または定数として明確に区別することができます。単一セグメントのパスは常にASTでは型として表現されますが、名前解決中に定数パラメータとして解決され、その後ast-lowering中に定数引数に下げられる可能性があります。lowering後も曖昧なままであるジェネリック引数は、パスセグメント内の推論されたジェネリック引数(`_`)のみです。例えば、`Foo<_>`では、`_`引数が推論された型引数なのか、推論された定数引数なのかは明確ではありません。

非曖昧な位置では、推論された引数は、それが型位置か定数位置かに応じて、[`hir::TyKind::Infer`][ty_infer]または[`hir::ConstArgKind::Infer`][const_infer]で表現されます。
曖昧な位置では、推論された引数は`hir::GenericArg::Infer`で表現されます。

これを素朴に実装すると、HIRの構造から見て、HIRで推論された型/定数が見つかる可能性のある場所が5つあることになります：

1. 非曖昧な型位置における`hir::TyKind::Infer`
2. 非曖昧な定数引数位置における`hir::ConstArgKind::Infer`
3. 曖昧な位置における[`GenericArg::Type(TyKind::Infer)`][generic_arg_ty]
4. 曖昧な位置における[`GenericArg::Const(ConstArgKind::Infer)`][generic_arg_const]
5. 曖昧な位置における[`GenericArg::Infer`][generic_arg_infer]

場所3と4は実際には遭遇することはありません。なぜなら、ジェネリック引数位置では常に`GenericArg::Infer`に下げるからです。

これにはいくつかの失敗モードがあります：

- `GenericArg::Infer`をチェックするビジターを書くが、`hir::TyKind/ConstArgKind::Infer`をチェックし忘れ、偶然にも曖昧な位置のinferのみを処理してしまう可能性がある。
- `hir::TyKind/ConstArgKind::Infer`をチェックするビジターを書くが、`GenericArg::Infer`をチェックし忘れ、偶然にも非曖昧な位置のinferのみを処理してしまう可能性がある。
- `GenericArg::Type/Const(TyKind/ConstArgKind::Infer)`と`GenericArg::Infer`をチェックするビジターを書くが、曖昧な位置で推論された型/定数を`GenericArg::Type/Const`として表現することは決してないことに気づかない可能性がある。
- `TyKind::Infer`のみをチェックし、`ConstArgKind::Infer`をチェックしないビジターを書き、推論された定数引数もあることを忘れてしまう可能性がある(その逆も同様)。

推論された型/定数を扱う際にHIRビジターを書くのを容易にし、エラーを減らすために、比較的複雑なシステムを採用しています：

1. コンパイラには、型または定数が非曖昧または曖昧な位置にある場合の異なる型、`hir::Ty<AmbigArg>`と`hir::Ty<()>`があります。[`AmbigArg`][ambig_arg]は居住不可能な型で、`TyKind`と`ConstArgKind`の`Infer`バリアントで使用され、曖昧な位置にある場合に選択的に「無効化」します。

2. HIRビジターの[`visit_ty`][visit_ty]と[`visit_const_arg`][visit_const_arg]メソッドは、型/定数の曖昧な位置バージョンのみを受け入れます。非曖昧な型/定数は、訪問プロセス中に曖昧な型/定数に暗黙的に変換され、`Infer`バリアントは専用の[`visit_infer`][visit_infer]メソッドによって処理されます。

これには多くの利点があります：

- `GenericArg::Type/Const`が推論された型/定数引数を表現できないことが明確である
- `visit_ty`と`visit_const_arg`の実装者は、推論された型/定数に遭遇することが決してないため、正しく動作するように見えるが、エッジケースを誤って処理するビジターを書くことが不可能になる
- `visit_infer`メソッドは、HIRにおける推論された型/定数の*すべて*のケースを処理するため、ビジターが専用の1つの場所で推論された型/定数を処理し、ケースを忘れないようにすることが容易になる

[ty_infer]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/enum.TyKind.html#variant.Infer
[const_infer]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/enum.ConstArgKind.html#variant.Infer
[generic_arg_ty]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/enum.GenericArg.html#variant.Type
[generic_arg_const]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/enum.GenericArg.html#variant.Const
[generic_arg_infer]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/enum.GenericArg.html#variant.Infer
[ambig_arg]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/enum.AmbigArg.html
[visit_ty]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/intravisit/trait.Visitor.html#method.visit_ty
[visit_const_arg]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/intravisit/trait.Visitor.html#method.visit_const_arg
[visit_infer]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/intravisit/trait.Visitor.html#method.visit_infer
