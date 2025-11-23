# Drop Check

一般的に、ローカル変数が使用される際には、その型が well-formed であることが必要です。これには、ローカル変数の where 境界の証明と、それに使用されるすべての領域が有効であることが求められます。

この規則の唯一の例外は、値がスコープ外になったときに暗黙的にドロップする場合です。これは値が有効である必要はありません:

```rust
fn main() {
    let x = vec![];
    {
        let y = String::from("I am temporary");
        x.push(&y);
    }
    // `x` はここでスコープ外になり、`y` への参照が無効化された後です。
    // これは `x` をドロップする際に、その型が well-formed ではないことを意味します。
    // なぜなら、有効でない領域が含まれているからです。
}
```

これは、値をドロップする際に死んだ領域にアクセスしようとしない場合にのみ健全です。これをチェックするため、値の型が drop-live であることを要求します。
その要件は `fn dropck_outlives` で計算されます。

このセクションの残りの部分では、領域パラメータが有効であることを要求する型について、次の型定義を使用します:

```rust
struct PrintOnDrop<'a>(&'a str);
impl<'a> Drop for PrintOnDrop<'_> {
    fn drop(&mut self) {
        println!("{}", self.0);
    }
}
```

## 値がドロップされる方法

その核心において、型 `T` の値は「drop glue」を実行することでドロップされます。drop glue はコンパイラによって生成され、最初に `<T as Drop>::drop` を呼び出し、次に再帰的に所有する値の drop glue を呼び出します。

- `T` が明示的な `Drop` 実装を持つ場合、`<T as Drop>::drop` を呼び出します。
- `T` が `Drop` を実装しているかどうかに関係なく、`T` が*所有する*すべての値に再帰します:
    - 参照、生ポインタ、関数ポインタ、関数アイテム、トレイトオブジェクト[^traitobj]、およびスカラーは何も所有しません。
    - タプル、スライス、配列は要素を所有していると見なされます。長さゼロの配列の場合、要素型の値を所有しません。
    - ADT のすべてのフィールド（すべてのバリアントの）は所有されていると見なされます。enum の場合、すべてのバリアントを考慮します。ここでの例外は `ManuallyDrop<U>` で、これは `U` を所有していると見なされません。
      `PhantomData<U>` も何も所有しません。
      クロージャとジェネレータは、キャプチャされた upvar を所有します。

型が drop glue を持つかどうかは [`fn Ty::needs_drop`](https://github.com/rust-lang/rust/blob/320b412f9c55bf480d26276ff0ab480e4ecb29c0/compiler/rustc_middle/src/ty/util.rs#L1086-L1108) によって返されます。

### ローカル変数の部分的なドロップ

`Drop` 自体を実装していない型の場合、残りをドロップする前に値の一部を部分的に移動することもできます。この場合、まだ移動されていない値の drop glue のみが呼び出されます。例えば:

```rust
fn main() {
    let mut x = (PrintOnDrop("third"), PrintOnDrop("first"));
    drop(x.1);
    println!("second")
}
```

MIR の構築中、*型がドロップを必要とする限り*、ローカル変数がスコープ外になったときにドロップされる可能性があると仮定します。変数の正確な drop glue の計算は、borrowck の**後**、`ElaborateDrops` パスで行われます。これは、ローカル変数の一部が以前にドロップされた場合でも、dropck はこの値が有効であることを要求することを意味します。これは、ローカル変数を完全に移動した場合でも当てはまります。

```rust
fn main() {
    let mut x;
    {
        let temp = String::from("I am temporary");
        x = PrintOnDrop(&temp);
        drop(x);
    }
} //~ ERROR `temp` does not live long enough.
```

borrowck の前にある程度の drop elaboration を追加することは可能であるはずで、この例をコンパイルできるようになります。const チェックの前に drop elaboration を移動する不安定な機能があります:
[#73255](https://github.com/rust-lang/rust/issues/73255)。borrowck の前にある程度の drop elaboration を行うための機能ゲートは存在しませんが、関連する [MCP](https://github.com/rust-lang/compiler-team/issues/558) があります。

[^traitobj]: トレイトオブジェクトは、vtable によって提供される `drop_in_place` を直接使用する組み込みの `Drop` 実装を持つと考えることができます。この `Drop` 実装は、すべてのジェネリックパラメータが有効であることを要求します。

### `dropck_outlives`

実行する「生存性」の計算には 2 つの異なるものがあります:

* 値 `v` が位置 `L` で*use-live*であるのは、後で「使用」される可能性がある場合です; ここでの*使用*は基本的に*ドロップ*ではないすべてのものです
* 値 `v` が位置 `L` で*drop-live*であるのは、後でドロップされる可能性がある場合です

値が*use-live*である場合、その型全体が `L` で有効でなければなりません。*drop-live*である場合、dropck によって要求されるすべての領域が `L` で有効でなければなりません。MIR でドロップされる値は*places*です。

型 `T` に対して `dropck_outlives` によって計算される制約は、その型に生成される drop glue と密接に一致します。drop glue とは異なり、`dropck_outlives` は所有される値自体ではなく、所有される値の型を考慮します。値型 `T` の場合:

- `T` が明示的な `Drop` を持つ場合、すべてのジェネリック引数が有効であることを要求します。ただし、`#[may_dangle]` でマークされている場合は完全に無視されます
- `T` が明示的な `Drop` を持つかどうかに関係なく、`T` が*所有する*すべての型に再帰します
    - 参照、生ポインタ、関数ポインタ、関数アイテム、トレイトオブジェクト[^traitobj]、およびスカラーは何も所有しません。
    - タプル、スライス、配列は要素型を所有していると見なされます。**配列の場合、現在長さがゼロかどうかをチェックしていません**。
    - ADT のすべてのフィールド（すべてのバリアントの）は所有されていると見なされます。ここでの例外は `ManuallyDrop<U>` で、これは `U` を所有していると見なされません。**`PhantomData<U>` は `U` を所有していると見なされます**。
    - クロージャとジェネレータは、キャプチャされた upvar を所有します。

太字でマークされたセクションは、`dropck_outlives` が `Ty::needs_drop` によって無視される型を所有していると見なすケースです。含まれるローカル変数の `Ty::needs_drop` が `true` を返した場合にのみ、`dropck_outlives` に依存します。これは、型がより大きなローカル変数に含まれているかどうかによって、生存性の要件が変わる可能性があることを意味します。**これは一貫性がなく、修正されるべきです: [配列の例](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=8b5f5f005a03971b22edb1c20c5e6cbe) と [`PhantomData` の例](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=44c6e2b1fae826329fd54c347603b6c8)。**[^core]

これらの不整合を修正する 1 つの方法は、MIR の構築をより悲観的にすることです。おそらく `Ty::needs_drop` を弱くするか、または代わりに `dropck_outlives` をより正確にして、有効である必要がある領域を少なくすることです。

[^core]: これは [#110288](https://github.com/rust-lang/rust/issues/110288) と [RFC 3417](https://github.com/rust-lang/rfcs/pull/3417) の核心的な仮定です。
