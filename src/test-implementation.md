# `#[test]` 属性



多くのRustプログラマーは、`#[test]`という組み込みの属性に頼っています。関数にマークを付けて、次のようにいくつかのアサートを含めるだけです：


```rust,ignore
#[test]
fn my_test() {
    assert!(2+2 == 4);
}
```

このプログラムを `rustc --test` や `cargo test` でコンパイルすると、これや他のテスト関数を実行できる実行ファイルが生成されます。このテスト方法により、テストをコードと一緒に自然な形で配置できます。プライベートモジュール内にテストを置くこともできます：

```rust,ignore
mod my_priv_mod {
    fn my_priv_func() -> bool {}

    #[test]
    fn test_priv_func() {
        assert!(my_priv_func());
    }
}
```

プライベートアイテムは、外部のテストツールに公開する方法を心配することなく簡単にテストできます。これはRustのテストの人間工学の鍵です。しかし、意味論的にはかなり奇妙です。これらのテストが可視でない場合、どのような `main` 関数がこれらのテストを呼び出すのでしょうか？`rustc --test` は正確に何をしているのでしょうか？

`#[test]` は、コンパイラの[`rustc_ast`][rustc_ast]内の構文変換として実装されています。本質的には、クレートを3つのステップで書き換える洗練された[`macro`]です：

## ステップ1：再エクスポート

前述のように、テストはプライベートモジュール内に存在できるため、既存のコードを壊すことなくmain関数にそれらを公開する方法が必要です。そのために、[`rustc_ast`][rustc_ast]は`__test_reexports`と呼ばれるローカルモジュールを作成し、テストを再帰的に再エクスポートします。この展開により、上記の例は次のように変換されます：

```rust,ignore
mod my_priv_mod {
    fn my_priv_func() -> bool {}

    pub fn test_priv_func() {
        assert!(my_priv_func());
    }

    pub mod __test_reexports {
        pub use super::test_priv_func;
    }
}
```

これで、テストは`my_priv_mod::__test_reexports::test_priv_func`としてアクセスできます。より深いモジュール構造の場合、`__test_reexports`はテストを含むモジュールを再エクスポートするため、`a::b::my_test`のテストは`a::__test_reexports::b::__test_reexports::my_test`になります。このプロセスはかなり安全に見えますが、既存の`__test_reexports`モジュールがある場合はどうなるでしょうか？答え：何も起こりません。

説明するために、Rustの[抽象構文木][ast]が[識別子][Ident]をどのように表現するかを理解する必要があります。すべての関数、変数、モジュールなどの名前は文字列として保存されるのではなく、不透明な[Symbol][Symbol]として保存され、これは本質的に各識別子のID番号です。コンパイラは、必要に応じて（構文エラーを出力するときなど）Symbolの人間が読める名前を回復できる別のハッシュテーブルを保持しています。コンパイラが`__test_reexports`モジュールを生成するとき、識別子に対して新しい[Symbol][Symbol]を生成するため、コンパイラが生成した`__test_reexports`は手書きのものと名前を共有する可能性がありますが、[Symbol][Symbol]は共有しません。この技法は、コード生成中の名前の衝突を防ぎ、Rustの[`macro`]ハイジーンの基礎となっています。

## ステップ2：ハーネス生成

これで、クレートのルートからテストにアクセスできるようになったので、[`rustc_ast`][ast]を使用してそれらで何かをする必要があります。次のようなモジュールを生成します：

```rust,ignore
#[main]
pub fn main() {
    extern crate test;
    test::test_main_static(&[&path::to::test1, /*...*/]);
}
```

ここで`path::to::test1`は[`test::TestDescAndFn`][tdaf]型の定数です。

この変換はシンプルですが、テストが実際にどのように実行されるかについて多くの洞察を提供してくれます。テストは配列に集約され、`test_main_static`と呼ばれるテストランナーに渡されます。[`TestDescAndFn`][tdaf]が正確に何であるかについては後で説明しますが、今のところ重要なポイントは、Rustコアの一部である[`test`][test]と呼ばれるクレートがあり、テストのすべてのランタイムを実装しているということです。[`test`][test]のインターフェースは不安定なので、それと対話する唯一の安定した方法は`#[test]`マクロを介することです。

## ステップ3：テストオブジェクト生成

以前にRustでテストを書いたことがあるなら、テスト関数で利用できるいくつかのオプションの属性に精通しているかもしれません。たとえば、パニックが発生することを期待している場合、テストに`#[should_panic]`を注釈できます。次のようになります：

```rust,ignore
#[test]
#[should_panic]
fn foo() {
    panic!("intentional");
}
```

これは、テストが単純な関数以上のものであり、設定情報も持っていることを意味します。`test`はこの設定データを[`TestDesc`]と呼ばれる`struct`にエンコードします。クレート内の各テスト関数について、[`rustc_ast`][rustc_ast]はその属性を解析し、[`TestDesc`]インスタンスを生成します。次に、[`TestDesc`]とテスト関数を、予測可能な名前の[`TestDescAndFn`][tdaf] `struct`に結合します。これが[`test_main_static`]が操作するものです。
特定のテストについて、生成された[`TestDescAndFn`][tdaf]インスタンスは次のようになります：

```rust,ignore
self::test::TestDescAndFn{
  desc: self::test::TestDesc{
    name: self::test::StaticTestName("foo"),
    ignore: false,
    should_panic: self::test::ShouldPanic::Yes,
    allow_fail: false,
  },
  testfn: self::test::StaticTestFn(||
    self::test::assert_test_result(::crate::__test_reexports::foo())),
}
```

これらのテストオブジェクトの配列を構築したら、ステップ2で生成されたハーネスを介してテストランナーに渡されます。

## 生成されたコードの検査

`nightly`の`rustc`には、[`macro`]展開後のモジュールソースを出力するために使用できる`unpretty`という不安定なフラグがあります：

```bash
$ rustc my_mod.rs -Z unpretty=hir
```

[`macro`]: ./macro-expansion.md
[`TestDesc`]: https://doc.rust-lang.org/test/struct.TestDesc.html
[ast]: ./ast-validation.md
[Ident]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/symbol/struct.Ident.html
[rustc_ast]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_ast
[Symbol]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/symbol/struct.Symbol.html
[test]: https://doc.rust-lang.org/test/index.html
[tdaf]: https://doc.rust-lang.org/test/struct.TestDescAndFn.html
[`test_main_static`]: https://doc.rust-lang.org/test/fn.test_main_static.html
