# 背景トピック

このセクションでは、本ガイドで登場する数多くの一般的なコンパイラ用語を取り上げます。Rust固有の文脈を提供しながら、一般的な定義を示すよう努めています。

<a id="cfg"></a>

## 制御フローグラフとは？

制御フローグラフ（CFG）はコンパイラで一般的に使用される用語です。フローチャートを使ったことがあれば、制御フローグラフの概念はとても馴染み深いものでしょう。これはプログラムの表現で、基礎となる制御フローを明確に露出させます。

制御フローグラフは、エッジで接続された**基本ブロック**の集合として構造化されています。基本ブロックの重要な考え方は、それが「一緒に」実行される文の集合であるということです - つまり、基本ブロックへ分岐すると、最初の文から始まり、その後残りのすべてを実行します。ブロックの最後でのみ、複数の場所への分岐の可能性があります（MIRでは、この最後の文を**ターミネーター**と呼びます）：

```mir
bb0: {
    statement0;
    statement1;
    statement2;
    ...
    terminator;
}
```

Rustで慣れ親しんでいる多くの式は、複数の基本ブロックにコンパイルされます。例えば、if文を考えてみましょう：

```rust,ignore
a = 1;
if some_variable {
    b = 1;
} else {
    c = 1;
}
d = 1;
```

これはMIRで4つの基本ブロックにコンパイルされます。テキスト形式では、次のようになります：

```mir
BB0: {
    a = 1;
    if some_variable {
        goto BB1;
    } else {
        goto BB2;
    }
}

BB1: {
    b = 1;
    goto BB3;
}

BB2: {
    c = 1;
    goto BB3;
}

BB3: {
    d = 1;
    ...
}
```

グラフ形式では、次のようになります：

```
                BB0
       +--------------------+
       | a = 1;             |
       +--------------------+
             /       \
  if some_variable   else
           /           \
     BB1  /             \  BB2
    +-----------+   +-----------+
    | b = 1;    |   | c = 1;    |
    +-----------+   +-----------+
            \          /
             \        /
              \ BB3  /
            +----------+
            | d = 1;   |
            | ...      |
            +----------+
```

制御フローグラフを使用する場合、ループは単にグラフ内のサイクルとして現れ、`break`キーワードはそのサイクルから出る経路に変換されます。

<a id="dataflow"></a>

## データフロー解析とは？

Anders MøllerとMichael I. Schwartzbach著の[*Static Program Analysis*](https://cs.au.dk/~amoeller/spa/)は素晴らしいリソースです！

_データフロー解析_は、多くのコンパイラで一般的な静的解析の一種です。これは特定の解析ではなく、一般的な技法を表しています。

基本的な考え方は、[制御フローグラフ（CFG）](#cfg)を走査して、ある値が何であるかを追跡することができるということです。走査の最後で、ある主張が真であるか必ずしも真でないかを示すことができます（例：「この変数は初期化されている必要がある」）。`rustc`はMIRがすでにCFGであるため、MIRに対してデータフロー解析を行う傾向があります。

例えば、このスニペットで`x`が使用される前に初期化されていることを確認したいとします：

```rust,ignore
fn foo() {
    let mut x;

    if some_cond {
        x = 1;
    }

    dbg!(x);
}
```

このコードのCFGは次のようになります：

```txt
 +------+
 | Init | (A)
 +------+
    |   \
    |   if some_cond
  else    \ +-------+
    |      \| x = 1 | (B)
    |       +-------+
    |      /
 +---------+
 | dbg!(x) | (C)
 +---------+
```

データフロー解析を次のように行うことができます：`x`が初期化されているかどうかを示すフラグ`init`を開始します。CFGを歩く際にフラグを更新します。最後に、その値を確認できます。

まず、ブロック(A)では、変数`x`は宣言されていますが初期化されていないので、`init = false`です。ブロック(B)では、値を初期化するので、`x`が初期化されていることが分かります。したがって、(B)の最後では`init = true`です。

ブロック(C)が興味深いところです。2つの入力エッジがあることに注目してください。1つは(A)から、もう1つは(B)から、これは`some_cond`が真かどうかに対応します。しかし、それを知ることはできません！`some_cond`が常に真である場合もあり、その場合`x`は実際に常に初期化されています。また、`some_cond`がランダムな何か（例：時刻）に依存する場合もあり、その場合`x`は初期化されていない可能性があります。一般的に、静的に知ることはできません（[Riceの定理][rice]のため）。では、ブロック(C)での`init`の値はどうあるべきでしょうか？

[rice]: https://en.wikipedia.org/wiki/Rice%27s_theorem

一般的に、データフロー解析では、ブロックに複数の親がある場合（例の(C)のように）、そのデータフロー値はすべての親の何らかの関数になります（そしてもちろん、(C)で何が起こるか）。どの関数を使用するかは、実行している解析に依存します。

この場合、`x`が使用される前に初期化されている必要があることを確実に証明したいと考えています。これにより、保守的に`some_cond`が時々偽である可能性があると仮定することを強いられます。したがって、「マージ関数」は「and」です。つまり、(C)で`init = true`となるのは、(A)_および_(B)で`init = true`である場合（または(C)で`x`が初期化されている場合）です。しかし、これは当てはまりません。特に、(A)では`init = false`であり、(C)で`x`は初期化されていません。したがって、(C)では`init = false`です。「`x`は使用前に初期化されていない可能性がある」というエラーを報告できます。

データフロー解析については、確かにもっと多くのことが言えます。このトピックには、多くの理論を含む広範な研究文献が存在します。ここでは順方向解析のみを議論しましたが、逆方向データフロー解析も有用です。例えば、ブロック(A)から始めて順方向に移動するのではなく、`x`の使用から始めて逆方向に移動してその初期化を見つけることもできます。

<a id="quantified"></a>

## 「全称量化」とは？「存在量化」は？

数学では、述語は_全称量化_または_存在量化_される可能性があります：

- _全称_量化：
  - 述語は、すべての可能な入力に対して真である場合に成立します。
  - 従来の表記：∀x: P(x)。「すべてのxに対して、P(x)が成立する」と読みます。
- _存在_量化：
  - 述語は、それが真である任意の入力が存在する場合に成立します。つまり、単一の入力があればよいです。
  - 従来の表記：∃x: P(x)。「P(x)が成立するようなxが存在する」と読みます。

Rustでは、これらは型チェックとトレイト解決で登場します。例えば、

```rust,ignore
fn foo<T>()
```
この関数は、すべての型`T`に対して関数が適格型であると主張します：`∀ T: well_typed(foo)`。

別の例：

```rust,ignore
fn foo<'a>(_: &'a usize)
```
この関数は、任意のライフタイム`'a`（呼び出し元によって決定される）に対して、適格型であると主張します：`∀ 'a: well_typed(foo)`。

別の例：

```rust,ignore
fn foo<F>()
where for<'a> F: Fn(&'a u8)
```
この関数は、すべてのライフタイム`'a`に対して`F: Fn(&'a u8)`であるようなすべての型`F`に対して、適格型であると主張します：`∀ F: ∀ 'a: (F: Fn(&'a u8)) => well_typed(foo)`。

もう1つの例：

```rust,ignore
fn foo(_: dyn Debug)
```
この関数は、`Debug`を実装する何らかの型`T`が存在し、関数が適格型であると主張します：`∃ T:  (T: Debug) and well_typed(foo)`。

<a id="variance"></a>

## de Bruijnインデックスとは？

[De Bruijnインデックス][wikideb]は、整数のみを使用して、どの変数がどのバインダーにバインドされているかを表す方法です。これは元々ラムダ計算の評価で使用するために発明されました（詳細は[このWikipediaの記事][wikideb]を参照）。`rustc`では、de Bruijnインデックスを使用して[ジェネリック型を表現][sub]します。

[wikideb]: https://en.wikipedia.org/wiki/De_Bruijn_index
[sub]: ../ty_module/generic_arguments.md


クロージャにde Bruijnインデックスがどのように使用されるかの基本的な例を示します（ただし、`rustc`では実際にはこれを行いません！）：

```rust,ignore
|x| {
    f(x) // `x`のde Bruijnインデックスは1、なぜなら`x`は1レベル上でバインドされているから

    |y| {
        g(x, y) // `x`のインデックスは2、なぜなら2レベル上でバインドされているから
                // `y`のインデックスは1、なぜなら1レベル上でバインドされているから
    }
}
```

## 共変性と反変性とは？

[Rust Nomicon](https://doc.rust-lang.org/nomicon/subtyping.html)のサブタイピングの章を確認してください。

型チェッカーが変性をどのように処理するかについての詳細は、本ガイドの[変性](../variance.html)の章を参照してください。

<a id="free-vs-bound"></a>

## 「自由領域」または「自由変数」とは？「束縛領域」は？

プログラム変数の観点から、自由対束縛の概念を説明しましょう。これは最も馴染み深いものだからです。

- この式を考えてみましょう。これはクロージャを作成します：`|a, b| a + b`。
  ここで、`a + b`の`a`と`b`は、クロージャが呼び出されたときに与えられる引数を参照します。`a`と`b`はクロージャに**束縛されている**と言い、クロージャシグネチャ`|a, b|`は名前`a`と`b`の**バインダー**であると言います（なぜなら、内部の`a`や`b`への参照は、それが導入する変数を参照するからです）。
- この式を考えてみましょう：`a + b`。この式では、`a`と`b`は式の*外部*で定義されたローカル変数を参照します。これらの変数は式に**自由に現れる**と言います（つまり、それらは**自由**であり、**束縛されていない**（縛られていない））。

これで理解できました：変数がある式/文/その他に「自由に現れる」とは、その式/文/その他の外部で定義されたものを参照する場合です。同様に、式の「自由変数」を参照することもできます - これは単に「自由に現れる」変数の集合です。

では、これは領域とどう関係があるのでしょうか？まあ、類似の概念を型と領域に適用できます。例えば、型`&'a u32`では、`'a`は自由に現れます。しかし、型`for<'a> fn(&'a u32)`では、現れません。

# コンパイラに関するさらなる読み物

> 公式Discordの`mem`、`scottmcm`、`Levi`に推薦を、そして`tinaun`にさらなる推薦があった[Graydon Hoareのツイッタースレッド](https://web.archive.org/web/20181230012554/https://twitter.com/graydon_pub/status/1039615569132118016)へのリンクを投稿してくれたことに感謝します！
>
> その他の情報源：https://gcc.gnu.org/wiki/ListOfCompilerBooks
>
> 他に提案がある場合は、お気軽にissueまたはPRを開いてください。

## 書籍
- [Types and Programming Languages](https://www.cis.upenn.edu/~bcpierce/tapl/)
- [Programming Language Pragmatics](https://www.cs.rochester.edu/~scott/pragmatics/)
- [Practical Foundations for Programming Languages](https://www.cs.cmu.edu/~rwh/pfpl/)
- [Compilers: Principles, Techniques, and Tools, 2nd Edition](https://www.pearson.com/us/higher-education/program/Aho-Compilers-Principles-Techniques-and-Tools-2nd-Edition/PGM167067.html)
- [Garbage Collection: Algorithms for Automatic Dynamic Memory Management](https://www.cs.kent.ac.uk/people/staff/rej/gcbook/)
- [Linkers and Loaders](https://www.amazon.com/Linkers-Kaufmann-Software-Engineering-Programming/dp/1558604960) （この無料版もありますが、リンクしていたバージョンは現在オフラインのようです。）
- [Advanced Compiler Design and Implementation](https://www.goodreads.com/book/show/887908.Advanced_Compiler_Design_and_Implementation)
- [Building an Optimizing Compiler](https://www.goodreads.com/book/show/2063103.Building_an_Optimizing_Compiler)
- [Crafting Interpreters](http://www.craftinginterpreters.com/)

## コース
- [University of Oregon Programming Languages Summer School archive](https://www.cs.uoregon.edu/research/summerschool/archives.html)

## Wiki
- [Wikipedia](https://en.wikipedia.org/wiki/List_of_programming_languages_by_type)
- [Esoteric Programming Languages](https://esolangs.org/wiki/Main_Page)
- [Stanford Encyclopedia of Philosophy](https://plato.stanford.edu/index.html)
- [nLab](https://ncatlab.org/nlab/show/HomePage)

## その他の論文とブログ投稿
- [Programming in Martin-Löf's Type Theory](https://www.cse.chalmers.se/research/group/logic/book/)
- [Polymorphism, Subtyping, and Type Inference in MLsub](https://dl.acm.org/doi/10.1145/3093333.3009882)
