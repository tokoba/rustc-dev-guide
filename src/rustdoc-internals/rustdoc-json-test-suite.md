# `rustdoc-json`テストスイート

このページは、rustdocの[json出力]をテストする`rustdoc-json`という名前のテストスイートについて具体的に説明します。
rustdocをテストするために使用される他のテストスイートについては、[§Rustdocテストスイート](../tests/compiletest.md#rustdoc-test-suites)を参照してください。

テストはcompiletestで実行され、通常の[ディレクティブ](../tests/directives.md)のセットにアクセスできます。
ここで頻繁に使用されるディレクティブは次のとおりです：

- [`//@ aux-build`][aux-build]で依存関係を持つ。
- `//@ edition: 2021`（または他のエディション）。
- `//@ compile-flags: --document-hidden-items`で[プライベートアイテムをドキュメント化]を有効にする。

各クレートのjson出力は、2つのプログラムによってチェックされます：[jsondoclint](#jsondocck)と[jsondocck](#jsondocck)。

## jsondoclint

[jsondoclint]は、すべての[`Id`]が`index`（または`paths`）に存在することをチェックします。
これにより、ぶら下がった[`Id`]がないことを確認します。

<!-- TODO: 他にもいくつかのことを行いますか？
また、どのように機能するかについても説明してください
 -->

## jsondocck

[jsondocck]は、コメントで与えられたディレクティブを処理して、出力の値が期待されるものであることをアサートします。
その点で[htmldocck](./rustdoc-test-suite.md)に似ています。

クエリ言語として[JSONPath]を使用します。これはパスを取得し、そのパスが一致するとされる値の*リスト*を返します。

### ディレクティブ

- `//@ has <path>`: `<path>`が存在することをチェックします。つまり、少なくとも1つの値に一致します。
- `//@ !has <path>`: `<path>`が存在しないことをチェックします。つまり、0個の値に一致します。
- `//@ has <path> <value>`: `<path>`が存在し、一致する値の少なくとも1つが指定された`<value>`と等しいことをチェックします。
- `//@ !has <path> <value>`: `<path>`が存在するが、一致する値のいずれも指定された`<value>`と等しくないことをチェックします。
- `//@ is <path> <value>`: `<path>`が正確に1つの値に一致し、それが指定された`<value>`と等しいことをチェックします。
- `//@ is <path> <value> <value>...`: `<path>`が指定されたすべての`<value>`に正確に一致することをチェックします。
   ここでは順序は関係ありません。
- `//@ !is <path> <value>`: `<path>`が正確に1つの値に一致し、その値が指定された`<value>`と等しくないことをチェックします。
- `//@ count <path> <number>`: `<path>`が`<number>`個の値に一致することをチェックします。
- `//@ set <name> = <path>`: `<path>`が正確に1つの値に一致することをチェックし、その値を`<name>`という変数に格納します。

これらは[`directive.rs`]で定義されています。

### 値

値はJSON値または変数のいずれかです。

- JSON値はJSONリテラルです。たとえば、`true`、`"string"`、`{"key": "value"}`です。
  これらは、1つの値として処理されるために、`'`を使用してクォートする必要があることがよくあります。[§引数の分割](#argument-spliting)を参照してください。
- 変数を使用して、1つのパスに値を格納し、後のクエリで使用できます。
  これらは`//@ set <name> = <path>`ディレクティブで設定され、`$<name>`でアクセスされます。

  ```rust
  //@ set foo = $some.path
  //@ is $.some.other.path $foo
  ```

### 引数の分割

ディレクティブへの引数は、POSIXシェルエスケープを実装する[shlex]クレートを使用して分割されます。
これは、[ディレクティブ](#directives)への`<path>`と`<value>`の両方の引数に、
頻繁に空白と引用符の両方があるためです。

`<path>`が`$.index[?(@.docs == "foo")].some.field`で、値が`"bar"` [^why_quote]の`@ is`を使用するには、次のように書きます：

```rust
//@ is '$.is[?(@.docs == "foo")].some.field' '"bar"'
```

[^why_quote]: 値はshlex分割の*後*に`"bar"`である必要があります。なぜなら、
    JSON文字列値である必要があるからです。

[json出力]: https://doc.rust-lang.org/nightly/rustdoc/unstable-features.html#json-output
[jsondocck]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/jsondocck
[jsondoclint]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/jsondoclint
[aux-build]: ../tests/compiletest.md#building-auxiliary-crates
[`Id`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustdoc_json_types/struct.Id.html
[プライベートアイテムをドキュメント化]: https://doc.rust-lang.org/nightly/rustdoc/command-line-arguments.html#--document-private-items-show-items-that-are-not-public
[`directive.rs`]: https://github.com/rust-lang/rust/blob/HEAD/src/tools/jsondocck/src/directive.rs
[shlex]: https://docs.rs/shlex/1.3.0/shlex/index.html
[JSONPath]: https://www.rfc-editor.org/rfc/rfc9535.html
