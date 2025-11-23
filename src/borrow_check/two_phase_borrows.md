# 二相借用

二相借用は、`vec.push(vec.len())` のようなネストされたメソッド呼び出しを可能にする、より許容的なバージョンの可変借用です。そのような借用は、最初に「予約」フェーズで共有借用として機能し、後で完全な可変借用に「アクティブ化」できます。

特定の暗黙的な可変借用のみが二相にできます。ソースコード内の `&mut` または `ref mut` は決して二相借用ではありません。二相借用を生成するケースは次のとおりです:

1. 可変参照レシーバーでメソッドを呼び出すときの autoref 借用。
2. 関数引数での可変再借用。
3. オーバーロードされた複合代入演算子での暗黙的な可変借用。

いくつかの例を示します:

```rust,edition2018
// ソースコード内

// ケース 1:
let mut v = Vec::new();
v.push(v.len());
let r = &mut Vec::new();
r.push(r.len());

// ケース 2:
std::mem::replace(r, vec![1, r.len()]);

// ケース 3:
let mut x = std::num::Wrapping(2);
x += x;
```

二相借用を表示するのに十分に展開します:

```rust,ignore
// ケース 1:
let mut v = Vec::new();
let temp1 = &two_phase v;
let temp2 = v.len();
Vec::push(temp1, temp2);
let r = &mut Vec::new();
let temp3 = &two_phase *r;
let temp4 = r.len();
Vec::push(temp3, temp4);

// ケース 2:
let temp5 = &two_phase *r;
let temp6 = vec![1, r.len()];
std::mem::replace(temp5, temp6);

// ケース 3:
let mut x = std::num::Wrapping(2);
let temp7 = &two_phase x;
let temp8 = x;
std::ops::AddAssign::add_assign(temp7, temp8);
```

借用が二相になることができるかどうかは、型チェック後の [`AutoBorrow`] のフラグで追跡され、その後 MIR 構築中に [`BorrowKind`] に[変換]されます。

各二相借用は、1 回だけ使用される一時変数に割り当てられます。したがって、次のように定義できます:

* 一時変数が割り当てられるポイントは、二相借用の*予約*ポイントと呼ばれます。
* 一時変数が使用されるポイント（実質的に常に関数呼び出し）は、*アクティブ化*ポイントと呼ばれます。

アクティブ化ポイントは [`GatherBorrows`] ビジターを使用して見つけられます。[`BorrowData`] は、借用の予約とアクティブ化ポイントの両方を保持します。

[`AutoBorrow`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/adjustment/enum.AutoBorrow.html
[変換]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_build/thir/cx/expr/trait.ToBorrowKind.html#method.to_borrow_kind
[`BorrowKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.BorrowKind.html
[`GatherBorrows`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/borrow_set/struct.GatherBorrows.html
[`BorrowData`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/borrow_set/struct.BorrowData.html

## 二相借用のチェック

二相借用は、次の例外を除いて、可変借用であるかのように扱われます:

1. MIR 内のすべての位置で、この位置でアクティブ化される二相借用があるかどうかを[チェック]します。生きている二相借用が位置でアクティブ化される場合、二相借用と競合する借用がないことをチェックします。
2. 予約ポイントで、競合する生きている*可変*借用がある場合はエラーになります。競合する共有借用がある場合は lint します。
3. 予約とアクティブ化ポイントの間、二相借用は共有借用として機能します。MIR グラフの [`Dominators`] を使用して、そのようなポイントにいるかどうかを（[`is_active`] で）判断します。
4. アクティブ化ポイントの後、二相借用は可変借用として機能します。

[チェック]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/struct.MirBorrowckCtxt.html#method.check_activations
[`Dominators`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_data_structures/graph/dominators/struct.Dominators.html
[`is_active`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/path_utils/fn.is_active.html
