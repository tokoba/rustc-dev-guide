# rustcのメモリ管理

一般的にrustcは、メモリの管理方法にかなり注意を払おうとしています。
コンパイラはコンパイル全体で_大量の_データ構造を割り当てますが、
注意しないと、多くの時間と空間がかかります。

コンパイラがこれを管理する主な方法の1つは、[アリーナ][arena]と[インターン][interning]を使用することです。

[arena]: https://en.wikipedia.org/wiki/Region-based_memory_management
[interning]: https://en.wikipedia.org/wiki/String_interning

## アリーナとインターン

コンパイル中に大量のデータ構造が作成されるため、
パフォーマンス上の理由から、グローバルメモリプールから割り当てます。
それぞれは、長寿命の*アリーナ*から一度割り当てられます。
これは_アリーナアロケーション_と呼ばれます。
このシステムは、メモリの割り当て/割り当て解除を削減します。
また、型の等価性の簡単な比較も可能にします（型の詳細は[こちら](./ty.md)）。
インターンされた各型`X`に対して、[`PartialEq` for X][peqimpl]を実装したので、
ポインタを比較するだけで済みます。
[`CtxtInterners`]型には、インターンされた型のマップとアリーナ自体が含まれています。

[`CtxtInterners`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.CtxtInterners.html#structfield.arena
[peqimpl]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html#implementations

### 例：`ty::TyKind`

コンパイラで型を表す[`ty::TyKind`]を例に取ります
（詳細は[こちら](./ty.md)で読めます）。型を構築したいたびに、
コンパイラは素朴にバッファから割り当てません。代わりに、
その型がすでに構築されているかどうかを確認します。構築されている場合は、
以前と同じポインタを取得するだけで、そうでない場合は新しいポインタを作成します。
このスキーマでは、2つの型が同じかどうかを知りたい場合、
ポインタを比較するだけで済み、効率的です。[`ty::TyKind`]は
スタックに構築されるべきではなく、そうすると使用できなくなります。
常にこのアリーナから割り当て、常にインターンして一意になるようにします。

コンパイルの開始時に、バッファを作成し、型を割り当てる必要があるたびに、
このメモリバッファの一部を使用します。スペースが不足したら、別のスペースを取得します。
そのバッファのライフタイムは`'tcx`です。型はそのライフタイムに関連付けられているため、
コンパイルが終了すると、そのバッファに関連するすべてのメモリが解放され、
`'tcx`参照は無効になります。

型に加えて、割り当てることができる他のアリーナ割り当てデータ構造がいくつかあります。
これらはこのモジュールにあります。いくつかの例を次に示します。

- [`GenericArgs`]、[`mk_args`]で割り当て – これは型のスライスをインターンします。
多くの場合、ジェネリック引数に代入される値を指定するために使用されます
（例：`HashMap<i32, u32>`は、スライス`&'tcx [tcx.types.i32, tcx.types.u32]`として表されます）。
- [`TraitRef`]、通常は値で渡される – **トレイト参照**は、
  `Self`を含むさまざまな型パラメータと共に、トレイトへの参照で構成されます。
  `i32: Display`のようなものです（ここで、def-idは`Display`トレイトを参照し、
  argsには`i32`が含まれます）。`def-id`は、
  [`AdtDef and DefId`][adtdefid]セクションで詳しく定義および説明されていることに注意してください。
- [`Predicate`]は、トレイトシステムが証明する必要があるものを定義します（[traits]モジュールを参照）。

[`GenericArgs`]: ./ty_module/generic_arguments.md#the-genericargs-type
[adtdefid]: ./ty_module/generic_arguments.md#adtdef-and-defid
[`TraitRef`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.TraitRef.html
[`AdtDef` and `DefId`]: ./ty.md#adts-representation
[`def-id`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/def_id/struct.DefId.html
[`GenericArgs`]: ./generic_arguments.html#GenericArgs
[`mk_args`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.mk_args
[adtdefid]: ./ty_module/generic_arguments.md#adtdef-and-defid
[`Predicate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Predicate.html
[`TraitRef`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.TraitRef.html
[`ty::TyKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/sty/type.TyKind.html
[traits]: ./traits/resolution.md

## `tcx`とライフタイムの使用方法

型付けコンテキスト（`tcx`）は、コンパイラの中心的なデータ構造です。
これは、あらゆる種類のクエリを実行するために使用するコンテキストです。
構造体[`TyCtxt`]は、この共有コンテキストへの参照を定義します。

```rust,ignore
tcx: TyCtxt<'tcx>
//          ----
//          |
//          arena lifetime
```

ご覧のように、`TyCtxt`型はライフタイムパラメータを取ります。`'tcx`のような
ライフタイムを持つ参照を見ると、それはアリーナ割り当てデータ（または少なくとも
アリーナと同じくらい長く生きるデータ）を参照していることがわかります。

[`TyCtxt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html

### ライフタイムに関する注意

Rustコンパイラは、多くの大きなデータ構造
（[抽象構文木（AST）][ast]、[高レベル中間表現（`HIR`）][hir]、型システムなど）を含むかなり大きなプログラムであり、
そのため、不要なメモリ使用を最小限に抑えるために、アリーナと参照に大きく依存しています。
これは、人々がコンパイラにプラグインできる方法（つまり、[ドライバ](./rustc-driver/intro.md)）に現れます。
「プル」スタイル（`Iterator`トレイトを考えてください）ではなく、「プッシュ」スタイルAPI（コールバック）を好みます。

スレッドローカルストレージとインターンは、
多くの普及したライフタイムによる多くの人間工学的問題を防ぎながら、
重複を減らすためにコンパイラ全体で多く使用されます。
[`rustc_middle::ty::tls`][tls]モジュールは、これらのスレッドローカルにアクセスするために使用されますが、
めったに触れる必要はありません。

[ast]: ./ast-validation.md
[hir]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/index.html
[tls]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/tls/index.html
