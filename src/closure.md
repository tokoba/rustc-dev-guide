# クロージャキャプチャ推論

このセクションでは、rustc がクロージャを処理する方法について説明します。Rust のクロージャは
事実上、作成者のスタックフレームから使用する値（または使用する値への参照）を含む
構造体に「脱糖」されます。rustc は、
クロージャがどの値をどのように使用するかを把握して、共有参照、可変参照、
または移動による特定の変数をキャプチャするかどうかを決定する仕事があります。rustc はまた、
クロージャがクロージャトレイト（[`Fn`][fn]、
[`FnMut`][fn_mut]、または [`FnOnce`][fn_once]）のどれを実装できるかを把握する必要があります。

[fn]: https://doc.rust-lang.org/std/ops/trait.Fn.html
[fn_mut]:https://doc.rust-lang.org/std/ops/trait.FnMut.html
[fn_once]: https://doc.rust-lang.org/std/ops/trait.FnOnce.html

いくつかの例から始めましょう：

### 例 1

まず、次の例のクロージャがどのように脱糖されるかを見てみましょう：

```rust
fn closure(f: impl Fn()) {
    f();
}

fn main() {
    let x: i32 = 10;
    closure(|| println!("Hi {}", x));  // クロージャは x を読み取るだけです。
    println!("Value of x after return {}", x);
}
```

上記が `immut.rs` というファイルの内容だとしましょう。次のコマンドを使用して
`immut.rs` をコンパイルすると、[`-Z dump-mir=all`][dump-mir] フラグにより、
`rustc` が [MIR][mir] を生成し、`mir_dump` というディレクトリにダンプします。
```console
> rustc +stage1 immut.rs -Z dump-mir=all
```

[mir]: ./mir/index.md
[dump-mir]: ./mir/passes.md

このコマンドを実行すると、現在の作業ディレクトリに `mir_dump` という新しく生成されたディレクトリが表示されます。これにはいくつかのファイルが含まれます。
ファイル `rustc.main.-------.mir_map.0.mir` を見ると、他のものの中に、
この行も含まれていることがわかります：

```rust,ignore
_4 = &_1;
_3 = [closure@immut.rs:7:13: 7:36] { x: move _4 };
```

この章の MIR の例では、`_1` は `x` であることに注意してください。

ここで、最初の行 `_4 = &_1;` で、`mir_dump` は `x` が不変参照として
借用されたことを示しています。これは、クロージャが単に
`x` を読み取るだけなので、期待どおりです。

### 例 2

別の例：

```rust
fn closure(mut f: impl FnMut()) {
    f();
}

fn main() {
    let mut x: i32 = 10;
    closure(|| {
        x += 10;  // クロージャは x の値を変更します
        println!("Hi {}", x)
    });
    println!("Value of x after return {}", x);
}
```

```rust,ignore
_4 = &mut _1;
_3 = [closure@mut.rs:7:13: 10:6] { x: move _4 };
```
今回は、行 `_4 = &mut _1;` で、借用が可変借用に変更されたことがわかります。
十分に公平です！クロージャは `x` を 10 増加させます。

### 例 3

もう 1 つの例：

```rust
fn closure(f: impl FnOnce()) {
    f();
}

fn main() {
    let x = vec![21];
    closure(|| {
        drop(x);  // x をその後使用不可能にします。
    });
    // println!("Value of x after return {:?}", x);
}
```

```rust,ignore
_6 = [closure@move.rs:7:13: 9:6] { x: move _1 }; // bb16[3]: scope 1 at move.rs:7:13: 9:6
```
ここで、`x` はクロージャに直接移動され、その後のアクセスは許可されません。

## コンパイラでの推論

それでは、rustc コードに飛び込んで、これらすべての推論がコンパイラによってどのように行われるかを見てみましょう。

まず、これからの議論でかなり使用する用語を定義しましょう -
*upvar*。**upvar** は、クロージャが定義されている関数のローカル変数です。したがって、
上記の例では、**x** はクロージャへの upvar になります。*自由変数*とも呼ばれ、
クロージャのコンテキストにバインドされていないことを意味します。
[`compiler/rustc_passes/src/upvars.rs`][upvars] は、この目的のために *upvars_mentioned*
というクエリを定義しています。

[upvars]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_passes/upvars/index.html

遅延呼び出しに加えて、クロージャを通常の関数と区別するもう 1 つのことは、
upvar を使用できることです。これは、周囲のコンテキストからこれらの upvar を
借用します。したがって、コンパイラは upvar の借用タイプを決定する必要があります。コンパイラは
不変借用タイプを割り当てることから始め、使用に基づいて必要に応じて制限を緩和します（つまり、
**不変**から**可変**から**移動**に変更します）。上記の例 1 では、
クロージャは変数を印刷にのみ使用し、何も変更しないため、
`mir_dump` で upvar `x` の借用タイプが不変であることがわかります。しかし、例 2 では、
クロージャは `x` を変更し、ある値で増加させます。この変更により、コンパイラは
最初に `x` を不変参照タイプとして割り当てていましたが、可変参照として調整する必要があります。
同様に、3 番目の例では、クロージャはベクトルをドロップするため、これには変数
`x` をクロージャに移動する必要があります。借用の種類に応じて、クロージャは
適切なトレイトを実装する必要があります：不変借用の場合は `Fn` トレイト、可変借用の場合は `FnMut`、
移動セマンティクスの場合は `FnOnce`。

クロージャに関連するコードのほとんどは
[`compiler/rustc_hir_typeck/src/upvar.rs`][upvar] ファイルにあり、データ構造は
[`compiler/rustc_middle/src/ty/mod.rs`][ty] ファイルで宣言されています。

[upvar]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/upvar/index.html
[ty]:https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/index.html

さらに進む前に、rustc コードベースを通じて制御フローを調べる方法について説明しましょう。
特にクロージャの場合、`RUSTC_LOG` 環境変数を以下のように設定し、
出力をファイルに収集します：

```console
> RUSTC_LOG=rustc_hir_typeck::upvar rustc +stage1 -Z dump-mir=all \
    <.rs file to compile> 2> <file where the output will be dumped>
```

これは stage1 コンパイラを使用し、
`rustc_hir_typeck::upvar` モジュールの `debug!` ログを有効にします。

もう 1 つのオプションは、lldb または gdb を使用してコードをステップ実行することです。

1. `rust-lldb build/host/stage1/bin/rustc test.rs`
2. lldb で：
    1. `b upvar.rs:134`  // upvar.rs ファイルの特定の行にブレークポイントを設定
    2. `r`  // ブレークポイントに到達するまでプログラムを実行

[`upvar.rs`][upvar] から始めましょう。このファイルには、クロージャのソースをウォークし、
借用、変更、または移動された各 upvar に対してコールバックを呼び出す [`euv::ExprUseVisitor`]
と呼ばれるものがあります。

[`euv::ExprUseVisitor`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/expr_use_visitor/struct.ExprUseVisitor.html

```rust
fn main() {
    let mut x = vec![21];
    let _cl = || {
        let y = x[0];  // 1.
        x[0] += 1;  // 2.
    };
}
```

上記の例では、ビジターは、マークされた行 1 と 2 に対して、共有借用に対して 1 回と
可変借用に対してもう 1 回、2 回呼び出されます。また、何が借用されたかも教えてくれます。

コールバックは [`Delegate`] トレイトを実装することによって定義されます。
[`InferBorrowKind`][ibk] 型は `Delegate` を実装し、
各 upvar に対してどのキャプチャモードが必要だったかを記録するマップを保持します。キャプチャのモードは、
`ByValue`（移動）または `ByRef`（借用）です。`ByRef` 借用の場合、可能な
[`BorrowKind`] は、[`compiler/rustc_middle/src/ty/mod.rs`][middle_ty] で定義されているように、
`ImmBorrow`、`UniqueImmBorrow`、`MutBorrow` です。

[`BorrowKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/enum.BorrowKind.html
[middle_ty]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/index.html

`Delegate` はいくつかの異なるメソッド（異なるコールバック）を定義します：
変数の*移動*のための **consume**、何らかの種類の*借用*
（共有または可変）のための **borrow**、そして何かの*割り当て*を見たときの **mutate**。

これらのコールバックはすべて、共通の引数 *cmt* を持ちます。これは Category、
Mutability、Type の略で、
[`compiler/rustc_hir_typeck/src/expr_use_visitor.rs`][cmt] で定義されています。コード
コメントから借用すると、"`cmt` は値がどこで発生し、どのように配置されているか、
および値が格納されているメモリの可変性を示す完全な分類です"。コールバック（consume、borrow など）に基づいて、
関連する `adjust_upvar_borrow_kind_for_<something>` を呼び出し、
`cmt` を渡します。借用タイプが調整されると、それをテーブルに保存します。
基本的に、各クロージャに対してどの借用が行われたかを示します。

```rust,ignore
self.tables
    .borrow_mut()
    .upvar_capture_map
    .extend(delegate.adjust_upvar_captures);
```

[`Delegate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/expr_use_visitor/trait.Delegate.html
[ibk]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/upvar/struct.InferBorrowKind.html
[cmt]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/expr_use_visitor/index.html
