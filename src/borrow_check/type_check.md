# MIR 型チェック

borrow check の重要なコンポーネントは [MIR 型チェック](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/type_check/index.html) です。このチェックは MIR を歩いて、他の言語で見られるような完全な「型チェック」を行います。この型チェックを行う過程で、プログラムに適用される領域制約も明らかにします。

TODO -- さらに詳しく説明する？多分？:)

## ユーザー型

MIR 型チェックの開始時に、本体内のすべての領域を新しい制約のない領域に置き換えます。
ただし、これにより次のプログラムを受け入れてしまいます:

```rust
fn foo<'a>(x: &'a u32) {
    let y: &'static u32 = x;
}
```

`y` の型のライフタイムを消去することで、それが `'static` であることを知らなくなり、ユーザーの意図を無視します。

これに対処するために、ユーザーが HIR 型チェック中に明示的に型を言及したすべての場所を [`CanonicalUserTypeAnnotations`][annot] として記憶します。

私たちが気にする 2 つの異なる注釈があります:

- 明示的な型アスクリプション。例えば、`let y: &'static u32` は `UserType::Ty(&'static u32)` になります。
- 明示的なジェネリック引数。例えば、`x.foo<&'a u32, Vec<String>>` は `UserType::TypeOf(foo_def_id, [&'a u32, Vec<String>])` になります。

HIR 型チェックからの領域推論が MIR typeck に影響を与えないようにしたいため、HIR から下げた直後にユーザー型を格納します。
これは、まだ推論変数が含まれている可能性があることを意味し、そのため**正規**ユーザー型注釈を使用しています。
すべての推論変数を存在束縛変数に置き換えます。
`let x: Vec<_>` のようなものは、`exists<T> UserType::Ty(Vec<T>)` になります。

`let Foo(x): Foo<&'a u32>` のようなパターンは、ユーザー型 `Foo<&'a u32>` を持ちますが、`x` の実際の型は `&'a u32` だけであるべきです。このために、[`UserTypeProjection`][proj] を使用します。

MIR では、ユーザー型を 2 つの若干異なる方法で扱います。

明示的な型注釈を持つパターン内の変数に対応する MIR ローカルが与えられた場合、そのローカルの型が [`UserTypeProjection`][proj] の型と等しいことを要求します。
これは [`LocalDecl`][decl] に直接格納されます。

スクルーティニー式の型も制約します。例えば、`let _: &'a u32 = x;` の `x` の型です。
ここで、`T_x` はユーザー型のサブタイプであればよいため、代わりに [`StatementKind::AscribeUserType`][stmt] を使用します。

MIR 型チェッカーは型と const 推論変数を直接扱わないため、ユーザー型を直接使用しないことに注意してください。代わりに、HIR 型チェッカーからの最終的な [`inferred_type`][inf] を格納します。MIR typeck 中、その領域を新しい nll 推論変数に置き換え、実際の `UserType` と関連付けて正しい領域制約を再び取得します。

MIR 型チェックの後、すべてのユーザー型注釈は破棄されます。もう必要ないためです。

[annot]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.CanonicalUserTypeAnnotation.html
[proj]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.UserTypeProjection.html
[decl]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.LocalDecl.html
[stmt]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.StatementKind.html#variant.AscribeUserType
[inf]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.CanonicalUserTypeAnnotation.html#structfield.inferred_ty
