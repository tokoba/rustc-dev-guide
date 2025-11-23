# 暗黙の呼び出し元位置情報

[RFC 2091]で承認されたこの機能は、`Option::unwrap`、`Result::expect`、`Index::index` などの関数から開始されるパニック時に、呼び出し元の位置を正確に報告できるようにします。この機能は、関数用の[`#[track_caller]`][attr-reference]属性、[`caller_location`][intrinsic]組み込み関数、および安定化に適した[`core::panic::Location::caller`][wrapper]ラッパーを追加します。

## 動機となる例

次のプログラム例を見てみましょう：

```rust
fn main() {
    let foo: Option<()> = None;
    foo.unwrap(); // これは有用なパニックメッセージを生成するべきです！
}
```

Rust 1.42以前では、このような `unwrap()` のパニックはcoreの位置を出力していました：

```
$ rustc +1.41.0 example.rs; example.exe
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value',...core\macros\mod.rs:15:40
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace.
```

1.42以降では、はるかに役立つメッセージが得られます：

```
$ rustc +1.42.0 example.rs; example.exe
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value', example.rs:3:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

これらのエラーメッセージは、`panic!` の内部を変更して `core::panic::Location::caller` を利用するようにし、標準ライブラリの多数の `#[track_caller]` 注釈が呼び出し元情報を伝播することの組み合わせによって実現されます。

## 呼び出し元位置の読み取り

以前は、`panic!` は `file!()`、`line!()`、`column!()` マクロを使用してパニックが発生した場所を指す [`Location`] を構築していました。これらのマクロには、オーバーライドされた位置を与えることができなかったため、意図的に `panic!` を呼び出す関数は独自の位置を提供できず、実際のエラーの原因を隠していました。

内部的に、`panic!()` は現在 [`core::panic::Location::caller()`][wrapper] を呼び出して、展開された場所を見つけます。この関数自体は `#[track_caller]` で注釈され、rustcによって実装される [`caller_location`][intrinsic] コンパイラ組み込み関数をラップします。この組み込み関数は、`const` コンテキストでの動作の観点から説明するのが最も簡単です。

## `const` での呼び出し元位置

constコンテキストで呼び出し元の位置を返すには、主に2つのフェーズがあります：スタックを上って正しい位置を見つけることと、返すconst値を割り当てることです。

### 正しい `Location` を見つける

constコンテキストでは、組み込み関数が呼び出された場所から「スタックを上って歩き」、属性を持たない最初の関数呼び出しに到達するまで停止します。この歩行は [`InterpCx::find_closest_untracked_caller_location()`][const-find-closest] で行われます。

下から始めて、[`InterpCx::stack`][const-stack] 内のスタック[`Frame`][const-frame]を上向きに反復し、各 `Frame` からの [`Instance`][frame-instance] で [`InstanceKind::requires_caller_location`][requires-location] を呼び出します。`false` を返すものを見つけたら停止し、「最上位」の追跡された関数だった*前の*フレームのスパンを返します。

### 静的 `Location` の割り当て

`Span` を取得したら、`Location` のための静的メモリを割り当てる必要があります。これは [`TyCtxt::const_caller_location()`][const-location-query] クエリによって実行されます。内部的には [`InterpCx::alloc_caller_location()`][alloc-location] を呼び出し、一意の[メモリカインド][location-memory-kind]（`MemoryKind::CallerLocation`）になります。SSAコードジェネレーションバックエンドは、これらと同じ値のコードを出力でき、そこでもこのコードを使用します。

`Location` が静的メモリに割り当てられると、組み込み関数はそれへの参照を返します。

## `#[track_caller]` 呼び出し先のコード生成

追跡された関数とその呼び出し元の効率的なコードを生成するには、スタックを上って歩くことなく、組み込み関数の観点から同じ動作を提供する必要があります。アプローチを反転させます：スタックを下に成長させるときに、追跡された関数の呼び出しに追加の引数を渡すことで、組み込み関数が呼び出されたときにスタックを上って歩くのではなく、呼び出し元の位置をクエリできる場所ならどこでもその追加の引数を返すことができます。

追加する引数は `&'static core::panic::Location<'static>` 型です。参照が選択されたのは、不必要なコピーを避けるためです。ポインタは、執筆時点で `std::mem::size_of::<core::panic::Location>() == 24` の3分の1のサイズだからです。

追跡されている関数への呼び出しを生成するとき、位置引数に [`FunctionCx::get_caller_location`][fcx-get] の値を渡します。

呼び出し元関数が追跡されている場合、`get_caller_location` は現在の呼び出し元の呼び出し元によって設定された [`FunctionCx::caller_location`][fcx-location] のローカルを返します。これらの場合、組み込み関数は実際には呼び出し元への引数で提供された参照を「返します」。

呼び出し元関数が追跡されていない場合、`get_caller_location` は現在の `Span` から `Location` 静的を割り当て、それへの参照を返します。

下向きにスタックを成長させるときに、複数の `FunctionCx` の `caller_location` フィールドを通じて単一の `&Location` 値を渡すことで、ループとして下から開始するのと同じ動作をより効率的に実現します。

### コード生成の例

この変換は実際にはどのように見えるでしょうか？新機能を使用する次の例を見てみましょう：

```rust
#![feature(track_caller)]
use std::panic::Location;

#[track_caller]
fn print_caller() {
    println!("called from {}", Location::caller());
}

fn main() {
    print_caller();
}
```

ここで `print_caller()` は引数を取らないように見えますが、次のようにコンパイルされます：

```rust
#![feature(panic_internals)]
use std::panic::Location;

fn print_caller(caller: &Location) {
    println!("called from {}", caller);
}

fn main() {
    print_caller(&Location::internal_constructor(file!(), line!(), column!()));
}
```

### 動的ディスパッチ

コードジェネレーションコンテキストでは、この情報をスタックに渡すために呼び出し先のABIを変更する必要がありますが、属性は明示的に関数の*型*を変更しません。ABIの変更は型チェックに対して透過的であり、すべての使用で健全である必要があります。

追跡された関数への直接呼び出しは、常に呼び出し先の完全なコードジェネレーションフラグを知っており、適切なコードを生成できます。間接的な呼び出し元はこの情報を持っておらず、呼び出す関数ポインタの型にエンコードされていないため、関数へのポインタを取るときは常に [`ReifyShim`] を生成します。このshimは間接呼び出しの実際の位置を報告することはできません（代わりに関数の定義サイトが報告されます）が、誤コンパイルを防ぎ、完全に安定化された型シグネチャを変更せずに、おそらく私たちができる最善のことです。

> *注：* 追跡された関数へのポインタを取るときは、常に [`ReifyShim`] を出力します。ここでの制約はコードジェネレーションコンテキストによって課されていますが、shimのMIR構築中には、constコンテキスト（shimを無視しても安全）またはコードジェネレーションコンテキスト（shimを無視するのは安全ではない）で呼び出されるかどうかはわかりません。わかっていたとしても、constとコードジェネレーションコンテキストからの結果は一致する必要があります。

## 属性

`#[track_caller]` 属性は、他のコードジェネレーション属性と一緒にチェックされ、関数が次の条件を満たすことを確認します：

* `"Rust"` ABIを持っている（例：`"C"` ではない）
* クロージャではない
* `#[naked]` ではない

使用が有効である場合、[`CodegenFnAttrsFlags::TRACK_CALLER`][attrs-flags] を設定します。このフラグは、constとコードジェネレーションの両方のコンテキストで正しい伝播を保証するために使用される [`InstanceKind::requires_caller_location`][requires-location] の戻り値に影響を与えます。

### トレイト

トレイトメソッドの実装に適用される場合、属性は通常の関数と同様に機能します。

トレイトメソッドのプロトタイプに適用される場合、属性はメソッドのすべての実装に適用されます。デフォルトのトレイトメソッド実装に適用される場合、属性はその実装*および*すべてのオーバーライドに効果を発揮します。

例：

```rust
#![feature(track_caller)]

macro_rules! assert_tracked {
    () => {{
        let location = std::panic::Location::caller();
        assert_eq!(location.file(), file!());
        assert_ne!(location.line(), line!(), "line should be outside this fn");
        println!("called at {}", location);
    }};
}

trait TrackedFourWays {
    /// すべての実装は `#[track_caller]` を継承します。
    #[track_caller]
    fn blanket_tracked();

    /// 実装者は自分自身に注釈を付けることができます。
    fn local_tracked();

    /// この実装は追跡されます（オーバーライドも同様です）。
    #[track_caller]
    fn default_tracked() {
        assert_tracked!();
    }

    /// この実装のオーバーライドは追跡されます（これも同様です）。
    #[track_caller]
    fn default_tracked_to_override() {
        assert_tracked!();
    }
}

/// この実装は `default_tracked` のデフォルト実装を使用し、
/// `default_tracked_to_override` の独自実装を提供します。
impl TrackedFourWays for () {
    fn blanket_tracked() {
        assert_tracked!();
    }

    #[track_caller]
    fn local_tracked() {
        assert_tracked!();
    }

    fn default_tracked_to_override() {
        assert_tracked!();
    }
}

fn main() {
    <() as TrackedFourWays>::blanket_tracked();
    <() as TrackedFourWays>::default_tracked();
    <() as TrackedFourWays>::default_tracked_to_override();
    <() as TrackedFourWays>::local_tracked();
}
```

## 背景/履歴

広く言えば、この機能の目標は、安定性保証を破ることなく、エンドユーザーのソースの変更を必要とせず、プラットフォーム固有のデバッグ情報に依存せず、ユーザー定義型が同じエラー報告の利点を持つことを妨げることなく、一般的なRustエラーメッセージを改善することです。

これらのパニックの出力を改善することは、少なくとも2016年半ばからの提案の目標でした（詳細については、承認されたRFCの[実行不可能な代替案][non-viable alternatives]を参照してください）。RFC 2091が承認されるまでにさらに2年かかりました。この機能の設計の多くの[根拠][rationale]は、いくつかの以前の提案に関する議論を通じて発見されました。

元のRFCの設計は、当時コンパイラ内で大幅なリファクタリングなしに実装できる実装に限定されていました。しかし、RFCの承認と実際の実装作業の間の1年半で、[改訂された設計][revised design]が提案され、トラッキングissueに書き上げられました。それを実装する過程で、関数のMIRの引数の数を変更せずに実装が可能であることも発見されました。これにより、後の段階が簡素化され、トレイトでの使用が可能になりました。

RFCの実装戦略はトレイトを容易にサポートできなかったため、セマンティクスは当初指定されていませんでした。それ以来、著者とレビュアーにとって最も正しいと思われるパスに従って実装されました。

[RFC 2091]: https://github.com/rust-lang/rfcs/blob/master/text/2091-inline-semantic.md
[attr-reference]: https://doc.rust-lang.org/reference/attributes/codegen.html#the-track_caller-attribute
[intrinsic]: https://doc.rust-lang.org/nightly/core/intrinsics/fn.caller_location.html
[wrapper]: https://doc.rust-lang.org/nightly/core/panic/struct.Location.html#method.caller
[non-viable alternatives]: https://github.com/rust-lang/rfcs/blob/master/text/2091-inline-semantic.md#non-viable-alternatives
[rationale]: https://github.com/rust-lang/rfcs/blob/master/text/2091-inline-semantic.md#rationale
[revised design]: https://github.com/rust-lang/rust/issues/47809#issuecomment-443538059
[attrs-flags]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/middle/codegen_fn_attrs/struct.CodegenFnAttrFlags.html#associatedconstant.TRACK_CALLER
[`ReifyShim`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/enum.InstanceKind.html#variant.ReifyShim
[`Location`]: https://doc.rust-lang.org/core/panic/struct.Location.html
[const-find-closest]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/interpret/struct.InterpCx.html#method.find_closest_untracked_caller_location
[requires-location]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/instance/enum.InstanceKind.html#method.requires_caller_location
[alloc-location]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/interpret/struct.InterpCx.html#method.alloc_caller_location
[fcx-location]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/mir/struct.FunctionCx.html#structfield.caller_location
[const-location-query]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.const_caller_location
[location-memory-kind]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/interpret/enum.MemoryKind.html#variant.CallerLocation
[const-frame]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/interpret/struct.Frame.html
[const-stack]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/interpret/struct.InterpCx.html#structfield.stack
[fcx-get]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/mir/struct.FunctionCx.html#method.get_caller_location
[frame-instance]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/interpret/struct.Frame.html#structfield.instance
