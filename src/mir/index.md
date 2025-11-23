# MIR（中レベルIR）

MIRは、Rustの_中レベル中間表現_です。これは[HIR](../hir.html)から構築されます。MIRは[RFC 1211]で導入されました。これは、特定のフロー依存の安全性チェック（特に借用チェッカー！）、および最適化とコード生成に使用される、Rustの根本的に簡略化された形式です。

MIRへの非常に高レベルの入門、および制御フローグラフや脱糖のようなそれが依存するコンパイラの概念に興味がある場合は、[MIRを紹介したrust-langブログ記事][blog]を楽しめるかもしれません。

[blog]: https://blog.rust-lang.org/2016/04/19/MIR.html

## MIRの紹介

MIRは[`compiler/rustc_middle/src/mir/`][mir]モジュールで定義されていますが、それを操作するコードの多くは[`compiler/rustc_mir_build`][mirmanip_build]、[`compiler/rustc_mir_transform`][mirmanip_transform]、および[`compiler/rustc_mir_dataflow`][mirmanip_dataflow]にあります。

[RFC 1211]: https://rust-lang.github.io/rfcs/1211-mir.html

MIRの主要な特性のいくつかは次のとおりです：

- [制御フローグラフ][cfg]に基づいています。
- ネストされた式はありません。
- MIRのすべての型は完全に明示的です。

[cfg]: ../appendix/background.html#cfg

## MIRの主要な用語

このセクションでは、MIRの主要な概念を紹介します。ここにまとめます：

- **基本ブロック**：制御フローグラフの単位で、次のもので構成されます：
  - **ステートメント：** 1つの後続を持つアクション
  - **ターミネータ：** 潜在的に複数の後続を持つアクション。常にブロックの最後にあります
  - （*基本ブロック*という用語に慣れていない場合は、[背景の章][cfg]を参照してください）
- **ローカル：** （少なくとも概念的には）スタックに割り当てられたメモリ位置、関数引数、ローカル変数、一時変数など。これらは、`_1`のように、先頭にアンダースコアが付いたインデックスで識別されます。戻り値を格納するために割り当てられた特別な「ローカル」（`_0`）もあります。
- **場所：** `_1`や`_1.f`のような、メモリ内の場所を識別する式。
- **Rvalue：** 値を生成する式。「R」は、これらが代入の「右辺」であることを表します。
  - **オペランド：** rvalueの引数で、定数（`22`など）または場所（`_1`など）のいずれかです。

簡単なプログラムをMIRに変換し、きれいに印刷された出力を読むことで、MIRがどのように構築されるかを感じることができます。実際、プレイグラウンドはMIRボタンを提供しているため、これは簡単です。このプログラムをプレイに入れて（または[このリンクをクリック][sample-play]）、上部の「MIR」ボタンをクリックしてみてください：

[sample-play]: https://play.rust-lang.org/?gist=30074856e62e74e91f06abd19bd72ece&version=stable&edition=2021

```rust
fn main() {
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
}
```

次のようなものが表示されるはずです：

```mir
// WARNING: This output format is intended for human consumers only
// and is subject to change without notice. Knock yourself out.
fn main() -> () {
    ...
}
```

これは`main`関数のMIR形式です。上記のリンクで示されるMIRは最適化されています。`StorageLive`のような一部のステートメントは、最適化で削除されます。これは、コンパイラが値がコード内でアクセスされないことに気付いたために発生します。最適化されていないMIRを表示するには、`rustc [filename].rs -Z mir-opt-level=0 --emit mir`を使用できます。これにはnightlyツールチェーンが必要です。


**変数宣言。** 少し掘り下げると、変数宣言の束から始まっていることがわかります。それらは次のようになります：

```mir
let mut _0: ();                      // return place
let mut _1: std::vec::Vec<i32>;      // in scope 0 at src/main.rs:2:9: 2:16
let mut _2: ();
let mut _3: &mut std::vec::Vec<i32>;
let mut _4: ();
let mut _5: &mut std::vec::Vec<i32>;
```

MIRの変数には名前がなく、`_0`や`_1`のようなインデックスがあることがわかります。また、ユーザーの変数（例：`_1`）と一時値（例：`_2`または`_3`）を混在させています。ユーザー定義変数は、それらに関連付けられたデバッグ情報があるため、区別できます（以下を参照）。

**ユーザー変数のデバッグ情報。** 変数宣言の下に、`_1`がユーザー変数を表すことを示す唯一のヒントがあります：
```mir
scope 1 {
    debug vec => _1;                 // in scope 1 at src/main.rs:2:9: 2:16
}
```
各`debug <Name> => <Place>;`アノテーションは、名前付きユーザー変数と、デバッガがその変数のデータを見つける場所（つまり、場所）を記述します。ここではマッピングは自明ですが、最適化により場所が複雑になったり、複数のユーザー変数が同じ場所を共有したりする可能性があります。さらに、クロージャキャプチャは同じシステムを使用して記述されるため、最適化がなくても複雑です。例：`debug x => (*((*_1).0: &T));`。

「スコープ」ブロック（例：`scope 1 { .. }`）は、ソースプログラムの字句構造（どの名前がいつスコープ内にあったか）を記述します。そのため、例えば、デバッガでコードをステップ実行している場合、`// in scope 0`でアノテーションされたプログラムの任意の部分には`vec`が欠けています。

**基本ブロック。** さらに読むと、最初の**基本ブロック**が表示されます（当然、表示すると少し異なる場合があり、一部のコメントは無視しています）：

```mir
bb0: {
    StorageLive(_1);
    _1 = const <std::vec::Vec<T>>::new() -> bb2;
}
```

基本ブロックは、一連の**ステートメント**と最終的な**ターミネータ**によって定義されます。この場合、1つのステートメントがあります：

```mir
StorageLive(_1);
```

このステートメントは、変数`_1`が「生きている」ことを示します。つまり、後で使用される可能性があることを意味します。これは、`StorageDead(_1)`ステートメントに遭遇するまで持続します。これは、変数`_1`の使用が終了したことを示します。これらの「ストレージステートメント」は、LLVMがスタック領域を割り当てるために使用されます。

ブロック`bb0`の**ターミネータ**は、`Vec::new`への呼び出しです：

```mir
_1 = const <std::vec::Vec<T>>::new() -> bb2;
```

ターミネータは、複数の後続を持つ可能性があるため、ステートメントとは異なります。つまり、制御が異なる場所に流れる可能性があります。`Vec::new`のような関数呼び出しは、アンワインドの可能性があるため、常にターミネータです。ただし、`Vec::new`の場合、アンワインドは実際には不可能であることがわかるため、後続ブロックは`bb2`の1つだけをリストします。

`bb2`を先読みすると、次のようになります：

```mir
bb2: {
    StorageLive(_3);
    _3 = &mut _1;
    _2 = const <std::vec::Vec<T>>::push(move _3, const 1i32) -> [return: bb3, unwind: bb4];
}
```

ここには2つのステートメントがあります：もう1つの`StorageLive`で、`_3`一時変数を導入し、次に代入：

```mir
_3 = &mut _1;
```

代入は一般に次の形式です：

```text
<Place> = <Rvalue>
```

場所は、`_3`、`_3.f`、または`*_3`のような式です。メモリ内の場所を示します。**Rvalue**は値を作成する式です：この場合、rvalueは可変借用式で、`&mut <Place>`のようになります。したがって、次のようなrvalueの文法を定義できます：

```text
<Rvalue>  = & (mut)? <Place>
          | <Operand> + <Operand>
          | <Operand> - <Operand>
          | ...

<Operand> = Constant
          | copy Place
          | move Place
```

この文法からわかるように、rvalueはネストできません。場所と定数のみを参照できます。さらに、場所を使用する場合、それを**コピー**しているか（場所が型`T`を持ち、`T: Copy`である必要があります）、**移動**しているか（任意の型の場所で機能します）を示します。したがって、例えば、Rustに`x = a + b + c`という式があった場合、それは2つのステートメントと1つの一時変数にコンパイルされます：

```mir
TMP1 = a + b
x = TMP1 + c
```

（[試してみてください][play-abc]が、オーバーフローチェックをスキップするにはリリースモードにしたいかもしれません。）

[play-abc]: https://play.rust-lang.org/?gist=1751196d63b2a71f8208119e59d8a5b6&version=stable

## MIRデータ型

MIRデータ型は、[`compiler/rustc_middle/src/mir/`][mir]モジュールで定義されています。前のセクションで言及された各主要概念は、かなり直接的な方法でRustの型にマップされます。

主要なMIRデータ型は[`Body`]です。これには、単一の関数のデータが含まれています（「昇格された定数」のMirのサブインスタンスとともに、[それらについては以下で読むことができます](#promoted)）。

- **基本ブロック**：基本ブロックは、フィールド[`Body::basic_blocks`][basicblocks]に格納されています。これは、[`BasicBlockData`]構造体のベクターです。誰も直接基本ブロックを参照しません：代わりに、このベクトルへの[newtype化][newtype'd]インデックスである[`BasicBlock`]値を渡します。
- **ステートメント**は、型[`Statement`]によって表されます。
- **ターミネータ**は、[`Terminator`]によって表されます。
- **ローカル**は、[newtype化][newtype'd]インデックス型[`Local`]によって表されます。ローカル変数のデータは、[`Body::local_decls`][localdecls]ベクトルにあります。また、戻り値を表す特別な「ローカル」を識別する特別な定数[`RETURN_PLACE`]もあります。
- **場所**は、構造体[`Place`]によって識別されます。いくつかのフィールドがあります：
  - `_1`のようなローカル変数
  - **プロジェクション**。これは、基本場所から「投影される」フィールドやその他のものです。これらは、[newtype化][newtype'd]型[`ProjectionElem`]によって表されます。したがって、例えば、場所`_1.f`はプロジェクションで、`f`が「プロジェクション要素」であり、`_1`が基本パスです。`*_1`もプロジェクションで、`*`は[`ProjectionElem::Deref`]要素で表されます。
- **Rvalue**は、列挙型[`Rvalue`]によって表されます。
- **オペランド**は、列挙型[`Operand`]によって表されます。

## 定数の表現

コードがMIR段階に達すると、定数は一般に2つの形式で提供されます：*MIR定数*（[`mir::Constant`]）と*型システム定数*（[`ty::Const`]）。MIR定数はオペランドとして使用されます：`x + CONST`では、`CONST`はMIR定数です。同様に、`x + 2`では、`2`はMIR定数です。型システム定数は型システムで使用され、特に配列長に使用されますが、const genericsにも使用されます。

一般に、両方の種類の定数は「未評価」または「すでに評価済み」である可能性があります。未評価定数は、この結果を計算するために評価する必要があるものの`DefId`を単に格納します。評価済み定数（「値」）はすでに計算されています。それらの表現は、型システム定数とMIR定数で異なります：MIR定数は`mir::ConstValue`に評価されます。型システム定数は`ty::ValTree`に評価されます。

型システム定数には、const genericsをサポートするためのいくつかのバリアントがあります：ローカルconst genericsパラメータを参照でき、推論の対象となります。さらに、`mir::Constant::Ty`バリアントを使用すると、任意の型システム定数をMIR定数として使用できます。これは、const genericsパラメータがオペランドとして使用されるときはいつでも発生します。

### MIR定数値

一般に、MIR定数値（`mir::ConstValue`）は、ユーザーが書いた定数を評価することによって計算されました。この[const評価](../const-eval.md)は、個々のバイトの観点から非常に低レベルの結果表現を生成します。これを「間接」定数（`mir::ConstValue::Indirect`）と呼びます。値はメモリ内に格納されているためです。

ただし、すべてをメモリ内に格納すると、非常に非効率的です。したがって、`mir::ConstValue`には、特定の単純で一般的な値をより効率的に表すことができるいくつかの他のバリアントがあります。特に、Rustでリテラルとして直接記述できるすべてのもの（整数、浮動小数点数、文字、ブール値、だけでなく`"string literals"`および`b"byte string literals"`）には、メモリ内表現の完全なオーバーヘッドを回避する最適化されたバリアントがあります。

### ValTree

評価済み型システム定数は「valtree」です。`ty::ValTree`データ構造により、次のものを表現できます。

* 配列、
* 多くの構造体、
* タプル、
* 列挙型、および
* ほとんどのプリミティブ。

この表現の最も重要なルールは、すべての値が一意に表現される必要があることです。言い換えれば：特定の値は、1つの特定の方法でのみ表現可能である必要があります。例えば：2つの整数の配列を`ValTree`として表現する方法は1つだけです：`Branch([Leaf(first_int), Leaf(second_int)])`。理論的には`[u32; 2]`は`u64`にエンコードできるため、単に`Leaf(bits_of_two_u32)`になる可能性がありますが、それは`ValTree`の合法的な構築ではありません（そして、そうすることは非常に複雑なので、誰もがそうしたくなる可能性は低いです）。

これらのルールは、一部の値が表現可能でないことも意味します。型レベルの定数には`union`を含めることはできません。アクティブなバリアントが不明であるため、どのように表現すべきかが明確ではないからです。同様に、生のポインタを表現する方法はありません。アドレスはコンパイル時に不明であり、それらについて仮定を立てることはできないためです。一方、参照は表現*できます*。参照の等価性はその値の等価性として定義されるため、アドレスを無視して、バッキング値だけを見ます。参照のポインタ値がコンパイル時に観察可能でないことを確認する必要があります。したがって、`&42`は`42`とまったく同じようにエンコードします。valtreeからMIR定数値への変換は、実際の間接参照を再導入する必要があります。コード生成時、アドレスは複数の使用間で重複排除される場合とされない場合があり、完全に任意の最適化の選択に依存します。

結果として、`ValTree`のすべてのデコードは、最初に型をマッチングし、それに依存して決定を下すことによって行われる必要があります。値自体は、それに属する型なしでは有用な情報を提供しません。

<a id="promoted"></a>

### 昇格された定数

const-eval WGの[昇格に関するドキュメント](https://github.com/rust-lang/const-eval/blob/master/promotion.md)を参照してください。


[mir]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/index.html
[mirmanip_build]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_build/index.html
[mirmanip_transform]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_transform/index.html
[mirmanip_dataflow]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/index.html
[`Body`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Body.html
[newtype'd]: ../appendix/glossary.html#newtype
[basicblocks]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Body.html#structfield.basic_blocks
[`BasicBlock`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.BasicBlock.html
[`BasicBlockData`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.BasicBlockData.html
[`Statement`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Statement.html
[`Terminator`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/terminator/struct.Terminator.html
[`Local`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Local.html
[localdecls]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Body.html#structfield.local_decls
[`RETURN_PLACE`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/constant.RETURN_PLACE.html
[`Place`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Place.html
[`ProjectionElem`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.ProjectionElem.html
[`ProjectionElem::Deref`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.ProjectionElem.html#variant.Deref
[`Rvalue`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.Rvalue.html
[`Operand`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.Operand.html
[`mir::Constant`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.Const.html
[`ty::Const`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Const.html
