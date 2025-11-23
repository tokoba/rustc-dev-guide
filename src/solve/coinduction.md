# 余帰納

トレイトソルバーは、ゴールを証明する際に余帰納を使用する場合があります。
余帰納はかなり微妙なので、独自の章を設けています。

## 余帰納と帰納

帰納では、有限の証明木に達するまで証明を再帰的に適用します。
`Vec<Vec<Vec<u32>>>: Debug`の例を考えると、次の木が得られます。

- `Vec<Vec<Vec<u32>>>: Debug`
  - `Vec<Vec<u32>>: Debug`
    - `Vec<u32>: Debug`
      - `u32: Debug`

この木は有限です。しかし、すべてのゴールが有限の証明木を持つわけではありません。
次の例を考えてみましょう：

```rust
struct List<T> {
    value: T,
    next: Option<Box<List<T>>>,
}
```

`List<T>: Send`が成り立つためには、すべてのフィールドも再帰的に`Send`を実装する必要があります。
これにより、次の証明木が得られます：

- `List<T>: Send`
  - `T: Send`
  - `Option<Box<List<T>>>: Send`
    - `Box<List<T>>: Send`
      - `List<T>: Send`
        - `T: Send`
        - `Option<Box<List<T>>>: Send`
          - `Box<List<T>>: Send`
            - ...

この木は無限に大きくなり、これがまさに余帰納が扱うものです。

> ゴールを**帰納的に**証明するには、その有限の証明木を提供する必要があります。
> ゴールを**余帰納的に**証明するには、提供される証明木が無限であっても構いません。

## なぜ余帰納は正しいのか

いくつかのトレイトゴールが成り立つかどうかをチェックする際、「この境界を満たす`impl`が存在するか」を問うています。ネストされたゴールの無限チェーンがある場合でも、使用すべき一意の`impl`があります。

## 余帰納の実装方法

実装は無限の木を構築しようとすることによって余帰納をチェックできませんが、無限のリソースを要するためです。しかし、余帰納をこの観点から考えることは依然として意味があります。

無限の木をチェックできないため、代わりに無限の証明木をもたらすことがわかっているパターンを探します。現在検出しているパターンは（正規）サイクルです。`T: Send`が`T: Send`に依存する場合、これが永遠に続くことは非常に明白です。

サイクルでは、キャッシングに注意する必要があります。領域と
推論変数の正規化のため、サイクルに遭遇しても無限の証明木が得られるとは限りません。
次の例を見てください：

```rust
trait Foo {}
struct Wrapper<T>(T);

impl<T> Foo for Wrapper<Wrapper<T>>
where
    Wrapper<T>: Foo
{}
```

`Wrapper<?0>: Foo`を証明することは、impl `impl<T> Foo for Wrapper<Wrapper<T>>`を使用し、これは
`?0`を`Wrapper<?1>`に制約し、次に`Wrapper<?1>: Foo`を必要とします。正規化により、これは
サイクルとして検出されます。

アイデアは、サイクルを検出するたびに*暫定的な結果*を返し、*暫定的な結果*がそのゴールの最終結果と等しくなるまでゴールを繰り返し
再試行することです。
制約のない`Yes`を結果として使用して開始し、再実行する必要がある場合は常に前のイテレーションの結果に更新します。

TODO: ここで詳しく説明する。余帰納サイクルに対してchalkと同じアプローチを使用しています。
帰納サイクルの扱いは現在、単に`Overflow`を返すことによって異なることに注意してください。
関連する章については、[関連する章][chalk]をchalk本で参照してください。

[chalk]: https://rust-lang.github.io/chalk/book/recursive/inductive_cycles.html

## 将来の作業

現在、自動トレイト、`Sized`、および`WF`ゴールのみを余帰納的と見なしています。
将来的には、ほとんどすべてのゴールを余帰納的にするつもりです。
まず、より多くの余帰納的証明を許可することがなぜ望ましいのかを説明しましょう。

### 再帰的データ型はすでに余帰納に依存しています

...トレイトソルバーでそれらを避ける傾向があるだけです。

```rust
enum List<T> {
    Nil,
    Succ(T, Box<List<T>>),
}

impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> Self {
        match self {
            List::Nil => List::Nil,
            List::Succ(head, tail) => List::Succ(head.clone(), tail.clone()),
        }
    }
}
```

このimplで`tail.clone()`を使用しています。このためには`Box<List<T>>: Clone`を証明する必要があり、
これには`List<T>: Clone`が必要ですが、これは現在チェックしているimplに依存しています。
[perfect derive]で行うように、その要件をimplの`where`句に追加することで、そのサイクルをトレイトソルバーに移動し、[エラーが発生します][ex1]。

### 再帰的データ型

プロジェクションを含む再帰的型について推論するには、余帰納も必要です。
たとえば、次は現在コンパイルに失敗しますが、有効であるべきです。

```rust
use std::borrow::Cow;
pub struct Foo<'a>(Cow<'a, [Foo<'a>]>);
```

この問題は少なくとも2015年から知られています。詳細については
[#23714](https://github.com/rust-lang/rust/issues/23714)を参照してください。

### 明示的にチェックされた暗黙の境界

implをチェックする際、implヘッダーの型がwell-formedであると仮定します。
これは、implをインスタンス化する際に、実際にそうであることを証明する必要があることを意味します。
[#100051](https://github.com/rust-lang/rust/issues/100051)は、これが当てはまらないことを示しています。
これを修正するには、implヘッダーの型に対して`WF`述語を追加する必要があります。
すべてのトレイトに対する余帰納がなければ、これは`core`さえ壊します。

```rust
trait FromResidual<R> {}
trait Try: FromResidual<<Self as Try>::Residual> {
    type Residual;
}

struct Ready<T>(T);
impl<T> Try for Ready<T> {
    type Residual = Ready<()>;
}
impl<T> FromResidual<<Ready<T> as Try>::Residual> for Ready<T> {}
```

`FromResidual`のimplがwell-formedであることをチェックする際、次のサイクルが発生します：

implは、`<Ready<T> as Try>::Residual`と`Ready<T>`がwell-formedである場合にwell-formedです。

- `wf(<Ready<T> as Try>::Residual)`には
- `Ready<T>: Try`が必要で、これはスーパートレイトのために
- `Ready<T>: FromResidual<Ready<T> as Try>::Residual>`を必要とし、**implの暗黙の境界のために**
- `wf(<Ready<T> as Try>::Residual)` :tada: **サイクル**

### より多くのゴールに余帰納を拡張する際の問題

余帰納を拡張する際に注意すべき追加の問題がいくつかあります。
ここでの問題は、現在のソルバーには関係ありません。

#### 暗黙のスーパートレイト境界

トレイトシステムは現在、スーパートレイト（たとえば`trait Trait: SuperTrait`）を

1) `Trait`を実装するすべての型に対して`SuperTrait`が成り立つ必要がある、
および2) `Trait`が成り立つ場合は`SuperTrait`を仮定する、という方法で扱います。

1)を証明する際に2)に依存することは健全ではありません。これは、余帰納サイクルの場合にのみ観察できます。サイクルがなければ、2)に依存する場合は常に、
使用されたimpl `Trait`に対して2)に依存せずに1)も証明している必要があります。

```rust
trait Trait: SuperTrait {}

impl<T: Trait> Trait for T {}

// 余帰納の現在の設定を維持すると、これがコンパイルされることになります。ああ :<
fn sup<T: SuperTrait>() {}
fn requires_trait<T: Trait>() { sup::<T>() }
fn generic<T>() { requires_trait::<T>() }
```

これは本質的に余帰納に固有のものではなく、それによって健全でなくなる既存の特性です。

##### 可能な解決策

これを解決する最も簡単な方法は、2)を完全に削除し、常に
トレイトソルバーの外で`T: Trait`を`T: Trait`と`T: SuperTrait`に精緻化することです。
これにより1)も削除できますが、トレイトに対して通常の
`where`境界を証明する必要があるため、それは単なる追加作業です。

1)をチェックする際に2)の循環的な使用を無効にする方法を想像することもできますが、
少なくとも私自身 - @lcnr - のアイデアはすべて、合理的であるには複雑すぎます。

#### `normalizes_to`ゴールと進捗

`normalizes_to`ゴールは、`<T as Trait>::Assoc`が
ある`U`に正規化される要件を表します。これは事実上、最初に`<T as Trait>::Assoc`を正規化し、次に
結果の型を`U`と等しくすることによって達成されます。各プロジェクションは正確に1つの型に正規化されるべきなので、マッピングであるべきです。単に無限の証明木を許可することによって、次の動作が得られます：

```rust
trait Trait {
    type Assoc;
}

impl Trait for () {
    type Assoc = <() as Trait>::Assoc;
}
```

`normalizes_to(<() as Trait>::Assoc, Vec<u32>)`を計算すると、implを解決し、
関連型`<() as Trait>::Assoc`を取得します。次に、それを期待される型と等しくし、
`normalizes_to(<() as Trait>::Assoc, Vec<u32>)`を再びチェックすることになります。
これは永遠に続き、無限の証明木になります。

これは、`<() as Trait>::Assoc`が他の任意の型と等しいことを意味し、健全ではありません。

##### これを解決する方法

**警告：これは微妙で間違っている可能性があります**

トレイトゴールとは異なり、`normalizes_to`は*生産的*である必要があります[^1]。`normalizes_to`ゴール
は、プロジェクションが剛体型コンストラクタに正規化されると生産的になります。
したがって、`<() as Trait>::Assoc`が`Vec<<() as Trait>::Assoc>`に正規化されることは生産的です。

`normalizes_to`ゴールには2種類のネストされたゴールがあります。プロジェクションを実際に
正規化するために必要なネストされた要件と、正規化されたプロジェクションと
期待される型の間の等式です。等式のみが生産的である必要があります。証明木の分岐は、有限であるか、少なくとも1つの`normalizes_to`を含む場合に生産的です。ここで、エイリアスは剛体型コンストラクタに解決されます。

あるいは、`normalizes_to`の等価分岐を常に帰納的として扱うこともできます。
サイクルは無限型をもたらすはずであり、それはとにかくサポートされておらず、codegenの深い正規化時にオーバーフローするだけです。

実験と例：<https://hackmd.io/-8p0AHnzSq2VAE6HE_wX-w?view>

別の要約の試み。

- プロジェクション等式では、rhsを制約することで進捗する必要があります
- サイクルは、等化中に少なくとも1回は正規化後にlhsに剛体型がある場合にのみ問題ありません
- `normalizes_to`の再帰的な`eq`呼び出しの外側のサイクルは常に問題ありません

[^1]: 関連：<https://coq.inria.fr/refman/language/core/coinductive.html#top-level-definitions-of-corecursive-functions>

[perfect derive]: https://smallcultfollowing.com/babysteps/blog/2022/04/12/implied-bounds-and-perfect-derive
[ex1]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=0a9c3830b93a2380e6978d6328df8f72
