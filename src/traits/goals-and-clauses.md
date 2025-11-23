# ゴールと節

論理プログラミング用語では、**ゴール**は証明しなければならないものであり、
**節**は真であることがわかっているものです。[論理への降下](./lowering-to-logic.html)
章で説明されているように、Rust のトレイトソルバーは、遺伝的
ハロップ (HH) 節の拡張に基づいており、これは伝統的な Prolog ホーン節を
いくつかの新しい超能力で拡張したものです。

## ゴールと節のメタ構造

Rust のソルバーでは、**ゴール**と**節**は次の形式を持ちます
（2つの定義が互いに参照していることに注意してください）：

```text
Goal = DomainGoal           // 以下のセクションで定義
        | Goal && Goal
        | Goal || Goal
        | exists<K> { Goal }   // 存在量化
        | forall<K> { Goal }   // 全称量化
        | if (Clause) { Goal } // 含意
        | true                 // 自明に真なもの
        | ambiguous            // 証明不可能なもの

Clause = DomainGoal
        | Clause :- Goal     // Goal を証明できれば、Clause は真
        | Clause && Clause
        | forall<K> { Clause }

K = <type>     // 「種類」
    | <lifetime>
```

これらの種類のゴールの証明手続きは実際には非常に単純です。
本質的には、深さ優先探索の一種です。論文
["A Proof Procedure for the Logic of Hereditary Harrop Formulas"][pphhf]
が詳細を提供しています。

コードの観点から、これらの型は rustc の
[`rustc_middle/src/traits/mod.rs`][traits_mod] と、chalk の
[`chalk-ir/src/lib.rs`][chalk_ir] で定義されています。

[pphhf]: https://rust-lang.github.io/chalk/book/bibliography.html#pphhf
[traits_mod]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_middle/src/traits/mod.rs
[chalk_ir]: https://github.com/rust-lang/chalk/blob/master/chalk-ir/src/lib.rs

<a id="domain-goals"></a>

## ドメインゴール

*ドメインゴール*は、トレイト論理の原子です。上で示した定義で
見られるように、一般的なゴールは基本的にドメインゴールの組み合わせで
構成されています。

さらに、前に示した節の定義を少し平坦化すると、節は常に次の形式である
ことがわかります：

```text
forall<K1, ..., Kn> { DomainGoal :- Goal }
```

したがって、ドメインゴールは実際には節の LHS です。つまり、最も細かい
レベルでは、ドメインゴールはトレイトソルバーが最終的に証明しようとする
ものです。

<a id="trait-ref"></a>

システム内のドメインゴールのセットを定義するには、まずいくつかの
シンプルな定式化を導入する必要があります。**トレイト参照**は、
トレイトの名前と適切な入力セット P0..Pn で構成されます：

```text
TraitRef = P0: TraitName<P1..Pn>
```

したがって、例えば `u32: Display` はトレイト参照であり、`Vec<T>:
IntoIterator` もそうです。Rust の表面構文では、関連型バインディング
（`Vec<T>: IntoIterator<Item = T>`）など、いくつかの追加のものも
許可されていますが、これらはトレイト参照の一部ではないことに注意してください。

<a id="projection"></a>

**プロジェクション**は、関連アイテム参照とその入力 P0..Pm で
構成されます：

```text
Projection = <P0 as TraitName<P1..Pn>>::AssocItem<Pn+1..Pm>
```

これらを考えると、`DomainGoal` を次のように定義できます：

```text
DomainGoal = Holds(WhereClause)
            | FromEnv(TraitRef)
            | FromEnv(Type)
            | WellFormed(TraitRef)
            | WellFormed(Type)
            | Normalize(Projection -> Type)

WhereClause = Implemented(TraitRef)
            | ProjectionEq(Projection = Type)
            | Outlives(Type: Region)
            | Outlives(Region: Region)
```

`WhereClause` は、Rust ユーザーが Rust プログラムで実際に書くことができる
`where` 句を指します。この抽象化は、Rust で効果的に書けるドメインゴール
のみを扱いたい場合があるため、便宜上のみ存在します。

これらを1つずつ分解してみましょう。

#### Implemented(TraitRef)

例：`Implemented(i32: Copy)`

与えられたトレイトが与えられた入力型とライフタイムに対して実装されている
場合に真です。

#### ProjectionEq(Projection = Type)

例：`ProjectionEq<T as Iterator>::Item = u8`

与えられた関連型 `Projection` が `Type` と等しい場合。これは
正規化を使用するか、プレースホルダー関連型を使用して証明できます。
[Chalk Book の関連型に関するセクション][at]を参照してください。

#### Normalize(Projection -> Type)

例：`ProjectionEq<T as Iterator>::Item -> u8`

与えられた関連型 `Projection` が `Type` に[正規化][n]できる場合。

[Chalk Book の関連型に関するセクション][at]で説明されているように、
`Normalize` は `ProjectionEq` を意味しますが、その逆は成り立ちません。
一般に、`Normalize(<T as Trait>::Item -> U)` を証明するには、
`Implemented(T: Trait)` を証明する必要もあります。

[n]: https://rust-lang.github.io/chalk/book/clauses/type_equality.html#normalize
[at]: https://rust-lang.github.io/chalk/book/clauses/type_equality.html

#### FromEnv(TraitRef)

例：`FromEnv(Self: Add<i32>)`

内部の `TraitRef` が真であると*仮定*されている場合、つまり、
スコープ内の where 句から導出できる場合に真です。

例えば、次の関数があるとします：

```rust
fn loud_clone<T: Clone>(stuff: &T) -> T {
    println!("cloning!");
    stuff.clone()
}
```

関数の本体内では、`FromEnv(T: Clone)` を持つことになります。スコープ内の
where 句はネストするため、impl 本体内の関数本体も impl 本体の where 句を
継承します。

これと次の規則は[暗黙の境界]を実装するために使用されます。降下に関する
セクションで見るように、`FromEnv(TraitRef)` は `Implemented(TraitRef)` を
意味しますが、その逆は成り立ちません。この区別は暗黙の境界にとって
重要です。

#### FromEnv(Type)

例：`FromEnv(HashSet<K>)`

内部の `Type` が整形式であると*仮定*されている場合、つまり、それが
関数または impl の入力型である場合に真です。

例えば、次のコードがあるとします：

```rust,ignore
struct HashSet<K> where K: Hash { ... }

fn loud_insert<K>(set: &mut HashSet<K>, item: K) {
    println!("inserting!");
    set.insert(item);
}
```

`HashSet<K>` は `loud_insert` 関数の入力型です。したがって、それが
整形式であると仮定するため、関数の本体内に `FromEnv(HashSet<K>)` を
持つことになります。降下に関するセクションで見るように、
`FromEnv(HashSet<K>)` は `Implemented(K: Hash)` を意味します。なぜなら、
`HashSet` 宣言が `K: Hash` という where 句で書かれているからです。
したがって、`loud_insert` 関数でその境界を繰り返す必要はありません：
それが真であると自動的に仮定します。

#### WellFormed(Item)

これらのゴールは、与えられたアイテムが*整形式*であることを意味します。

異なる種類のアイテムが整形式であることについて話すことができます：

* *型*、例えば `WellFormed(Vec<i32>)` は Rust では真ですが、
  `WellFormed(Vec<str>)` は真ではありません（`str` は `Sized` ではないため）。

* *TraitRefs*、例えば `WellFormed(Vec<i32>: Clone)`。

整形式性は[暗黙の境界]にとって重要です。特に、`loud_clone` の例で
`FromEnv(T: Clone)` を仮定することが問題ない理由は、`loud_clone` の
各呼び出しサイトで `WellFormed(T: Clone)` を検証するからです。
同様に、`loud_insert` の例で `FromEnv(HashSet<K>)` を仮定することが
問題ない理由は、`loud_insert` の各呼び出しサイトで
`WellFormed(HashSet<K>)` を検証するからです。

#### Outlives(Type: Region), Outlives(Region: Region)

例：`Outlives(&'a str: 'b)`、`Outlives('a: 'static)`

左側の与えられた型または領域が右側の領域よりも長生きする場合に真です。

<a id="coinductive"></a>

## 帰納的ゴール

システム内のほとんどのゴールは「帰納的」です。帰納的ゴールでは、
循環推論は許可されていません。この例の節を考えてみてください：

```text
    Implemented(Foo: Bar) :-
        Implemented(Foo: Bar).
```

帰納的に考えると、この節は役に立ちません：`Implemented(Foo: Bar)` を
証明しようとしている場合、再帰的に `Implemented(Foo: Bar)` を証明する
必要があり、そのサイクルは無限に続きます（トレイトソルバーはここで
終了し、`Implemented(Foo: Bar)` が真であることがわかっていないと
見なすだけです）。

しかし、いくつかのゴールは*余帰納的*です。簡単に言えば、これはサイクルが
OK であることを意味します。したがって、`Bar` が余帰納的トレイトである場合、
上記の規則は完全に有効であり、`Implemented(Foo: Bar)` が真であることを
示します。

*自動トレイト*は、Rust で余帰納的ゴールが使用される一例です。
`Send` トレイトを考えて、次の構造体があると想像してください：

```rust
struct Foo {
    next: Option<Box<Foo>>
}
```

自動トレイトのデフォルト規則は、そのフィールドの型が `Send` である場合、
`Foo` は `Send` であると言います。したがって、次のような規則があります

```text
Implemented(Foo: Send) :-
    Implemented(Option<Box<Foo>>: Send).
```

おそらく想像できるように、`Option<Box<Foo>>: Send` を証明すると、
`Foo: Send` を再び証明する必要が循環的に生じます。したがって、これは
サイクルに巻き込まれる例ですが、問題ありません。`Foo: Send` が
自分自身を参照していても、成り立つと*考えます*。

一般に、余帰納的トレイトは、固定された可能性のセットを列挙したいときに
Rust のトレイト解決で使用されます。自動トレイトの場合、与えられた
開始点から到達可能な型のセットを列挙しています（つまり、`Foo` は
`Option<Box<Foo>>` 型の値に到達でき、これは `Box<Foo>` 型の値に到達でき、
そして `Foo` 型に到達でき、そしてサイクルが完成します）。

自動トレイトに加えて、`WellFormed` 述語は余帰納的です。
これらは、[暗黙の境界]のセクションで説明されているように、同様の
「すべてのケースを列挙する」パターンを実現するために使用されます。


## 不完全な章

まだ書かれていないトピック：

* 証明手続きの詳細
* SLG 解決 - 否定的推論の導入
