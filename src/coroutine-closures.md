# 非同期クロージャ/"coroutine-closures"

機能の一般的な動機を理解するには、[RFC 3668](https://rust-lang.github.io/rfcs/3668-async-closures.html)をお読みください。これは非常に技術的でやや「縦断的な」章です。理想的には、これを分割して関連するすべての章に散りばめるべきですが、非同期クロージャを*全体的に*理解する目的で、ここに一つの章としてまとめました。

## コルーチンクロージャ -- 技術的な深掘り

コルーチンクロージャは、非同期クロージャの一般化であり、コルーチンを返すクロージャ式の特別な構文です。特に、クロージャのupvarsからキャプチャすることが許可されているものです。

今のところ、使用可能なコルーチンクロージャの唯一の種類は非同期クロージャであり、非同期クロージャをサポートすることがこのPRの範囲です。最終的には`gen || {}`などをサポートする可能性があり、このドキュメントで説明されている問題や興味深い点のほとんどは、一般的なすべてのコルーチンクロージャに適用されます。

コードがやや一般的であるため、このドキュメントは「非同期クロージャ」と「コルーチンクロージャ」を入れ替えて呼ぶことがあります。非同期クロージャによって返されるfutureは、一般的に「コルーチン」または「子コルーチン」と呼ばれます。

### HIR

非同期クロージャ(および将来的には`gen`などの他のコルーチンフレーバー)は、HIRでは`hir::Closure`として表現されます。
`hir::Closure`のclosure-kindは`ClosureKind::CoroutineClosure(_)`[^k1]であり、これは非同期ブロックをラップします。非同期ブロックもHIRでは`hir::Closure`として表現されます。
非同期ブロックのclosure-kindは`ClosureKind::Closure(CoroutineKind::Desugared(_, CoroutineSource::Closure))`[^k2]です。

[^k1]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_ast_lowering/src/expr.rs#L1147>

[^k2]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_ast_lowering/src/expr.rs#L1117>

`async fn`のように、非同期クロージャの本体を下げる際には、クロージャのすべての引数を本体に無条件に移動する必要があります。これは`lower_coroutine_body_with_moved_arguments`[^l1]によって処理されます。この関数の唯一の注目すべき癖は、生成する非同期ブロックのキャプチャ種類が`CaptureBy::ByRef`[^l2]であることです。後で*クロージャ引数*のすべてをby-valueでキャプチャするよう強制します[^l3]が、*全体の*非同期ブロックが`async move`であるかのように動作させたくありません。それでは非同期クロージャの自己借用の目的が損なわれるからです。

[^l1]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_ast_lowering/src/item.rs#L1096-L1100>

[^l2]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_ast_lowering/src/item.rs#L1276-L1279>

[^l3]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_hir_typeck/src/upvar.rs#L250-L256>

### `rustc_middle::ty`表現

実装をほぼ将来互換性のあるもの(つまり、`gen || {}`や`async gen || {}`など)に保つために、このセクションのほとんどは非同期クロージャを「コルーチンクロージャ」と呼びます。

このPRが導入する主なものは、`CoroutineClosure`[^t1]と呼ばれる新しい`TyKind`と、typeckとborrowckの他の関連する列挙型の対応するバリアント(`UpvarArgs`、`DefiningTy`、`AggregateKind`)です。

[^t1]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_type_ir/src/ty_kind.rs#L163-L168>

既存の`TyKind::Closure`を一般化するのではなく、新しい`TyKind`を導入します。これは、型の表現上の大きな違いによるものです。`CoroutineClosure`と通常のクロージャの主な違いは、まずコルーチンクロージャのジェネリクスの「展開された」表現である`CoroutineClosureArgsParts`を検査することで探ることができます。

#### クロージャとの類似点

クロージャのように、`parent_args`、`closure_kind_ty`、`tupled_upvars_ty`があります。これらは、クロージャの対応物と同じものを表します。つまり: クロージャが定義された本体から継承されたジェネリクス、クロージャの最大「呼び出し能力」(つまり、`FnOnce`のように呼び出すために消費する必要があるか、by-refで呼び出すことができるか)、クロージャ自体のキャプチャされたupvarsです。

#### シグネチャ

従来のクロージャは、クロージャのシグネチャを表すために使用される`fn_sig_as_fn_ptr_ty`を持っています。対照的に、コルーチンクロージャのシグネチャは、やや「爆発的な」方法で保存されます。コルーチンクロージャは、どの`AsyncFn*`トレイトで呼び出されるかに応じて*2つの*シグネチャを持つためです(以下のセクションを参照)。

概念的には、コルーチンクロージャは、by-refで呼び出されるかby-moveで呼び出されるかに応じて、いくつかの異なるシグネチャ型を含んでいると考えることができます。

これらのシグネチャの両方を便利に再作成するために、`signature_parts_ty`は、このコルーチンクロージャによって返されるコルーチンのすべての関連部分を保存します。このシグネチャパーツ型は、一般的に`fn(tupled_inputs, resume_ty) -> (return_ty, yield_ty)`の形式を持ちます。ここで、`resume_ty`、`return_ty`、`yield_ty`は、それぞれコルーチンクロージャによって返される*コルーチン*の対応する型です[^c1]。

[^c1]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_type_ir/src/ty_kind/closure.rs#L221-L229>

コンパイラは主に`CoroutineClosureSignature`型[^c2]を扱います。これは、上記の`fn()`ポインタ型から関連する型を抽出して作成され、コルーチンクロージャが最終的に返す*コルーチン*を構築するために使用できるメソッドを公開します。

[^c2]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_type_ir/src/ty_kind/closure.rs#L362>

#### `Coroutine`戻り型を構築するために持ち運ぶ必要があるデータ

シグネチャに保存されたデータに加えて、戻すべき`TyKind::Coroutine`を構築するには、コルーチンの「witness」も保存する必要があります。

それでは、返されるコルーチンのupvarsはどうでしょうか?まあ、`AsyncFnOnce`(つまり、call-by-move)の場合、これは単にコルーチンが返すのと同じupvarsです。しかし、`AsyncFnMut`/`AsyncFn`の場合、コルーチンクロージャから返されるコルーチンは、与えられた「environment」ライフタイム[^c3]でコルーチンクロージャからデータを借用します。これは、`AsyncFnMut`/`AsyncFn`呼び出しシグネチャの`&self`ライフタイム[^c4]、および`ByRef`のGATライフタイム[^c5]に対応します。

[^c3]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_type_ir/src/ty_kind/closure.rs#L447-L455>

[^c4]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/library/core/src/ops/async_function.rs#L36>

[^c5]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/library/core/src/ops/async_function.rs#L30>

#### 実際にコルーチン戻り型を取得する

コルーチンクロージャが返す`Coroutine`を最も簡単に構築するには、`CoroutineClosureArgs`から取得できる`CoroutineClosureSignature`の`to_coroutine_given_kind_and_upvars`[^helper]ヘルパーを使用できます。

[^helper]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_type_ir/src/ty_kind/closure.rs#L419>

その関数への引数のほとんどは、`CoroutineArgs`から取得できるコンポーネントですが、`goal_kind: ClosureKind`は例外で、これは渡された`ClosureKind`に基づいてどのフレーバーのコルーチンを返すかを制御します - つまり、`ClosureKind::Fn | ClosureKind::FnMut`の場合はby-refコルーチンを準備し、`ClosureKind::FnOnce`の場合はby-moveコルーチンを準備します。

### トレイト階層

`Fn*`トレイトの並列階層を導入します。導入の動機は、ブログ投稿:[Async Closures](https://hackmd.io/@compiler-errors/async-closures)でカバーされています。

現在安定しているすべての呼び出し可能型(つまり、クロージャ、関数アイテム、関数ポインタ、および`dyn Fn*`トレイトオブジェクト)は、何らかの出力型`Fut`に対して`Fn*() -> Fut`を実装し、`Fut`が`Future<Output = T>`を実装している場合、自動的に`AsyncFn*() -> T`を実装します[^tr1]。

[^tr1]: <https://github.com/rust-lang/rust/blob/7c7bb7dc017545db732f5cffec684bbaeae0a9a0/compiler/rustc_next_trait_solver/src/solve/assembly/structural_traits.rs#L404-L409>

非同期クロージャは、その本体が許す限り`AsyncFn*`を実装します。つまり、upvarsを消費または変更する方法で使用する場合、`AsyncFn`および`AsyncFnMut`を実装するかどうかに影響を与える可能性があります...

#### レンディング

将来的には、`AsyncFn*`をより一般的な`LendingFn*`トレイトのセットに移行する可能性があります。ただし、今日のコンパイラで`LendingFn`を人間工学的に使用する能力を制限する具体的な技術実装の詳細があります。これらは以下に関連しています:

- クロージャシグネチャ推論。
- 高ランクトレイト境界の制限。
- エラーメッセージの欠点。

これらの制限に加えて、基礎となるトレイトは非同期クロージャと非同期`Fn`トレイト境界のユーザーエクスペリエンスに影響を与えないという事実により、今のところ`AsyncFn*`を使用することにしました。最終的にこれらのより一般的なトレイトに移行できるようにするために、正確な`AsyncFn*`トレイト定義(関連型を含む)は実装の詳細として残されています。

#### 非同期クロージャは通常の`Fn*`トレイトをいつ実装しますか?

上記では「通常の」呼び出し可能型が`AsyncFn*`を実装できることを述べましたが、逆の質問が存在します:「非同期クロージャも`Fn*`を実装できますか?」答えは「有効な場合」です。つまり、`AsyncFn`/`AsyncFnMut`から返されるはずのコルーチンが、実際には親のコルーチンクロージャから「貸し出された」upvarsを持っていない場合です。

詳細な答えについては、以下の「フォローアップ: いつ...」セクションを参照してください。完全な答えは、ほとんどの非同期クロージャが「うまく動作する」ことを保証するために使用される、非常に興味深く、願わくば徹底的なヒューリスティックを説明しています。

### 2つの本体の物語

非同期クロージャが`AsyncFn`/`AsyncFnMut`で呼び出された場合、クロージャから借用するコルーチンを返します。ただし、`AsyncFnOnce`を介して呼び出された場合、そのクロージャを消費し、現在ドロップされているデータから借用するコルーチンを返すことはできません。

この制限を回避するために、by-refで呼び出すことができるコルーチンクロージャで`AsyncFnOnce::call_once`を呼び出すための別個のby-move MIR本体を合成します。

この本体は、「通常の」コルーチンクロージャを呼び出すことで返されるコルーチンと同じように動作しますが、異なるupvarsのセットを持っているという点が異なります。親のコルーチンクロージャからのキャプチャを子コルーチンに*移動*する必要があるためです。

#### by-move本体の合成

コルーチンクロージャによって返されるコルーチンのby-move本体にアクセスしたい場合、`coroutine_by_move_body_def_id`[^b1]クエリを介してアクセスできます。

[^b1]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_mir_transform/src/coroutine/by_move_body.rs#L1-L70>

このクエリは、コルーチンのMIR本体をコピーし、本体のセマンティクスを保持するために追加のデリファレンスとフィールド投影[^b2]を挿入することにより、新しいMIR本体を合成します。

[^b2]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_mir_transform/src/coroutine/by_move_body.rs#L131-L195>

新しいdef idを合成したため、このクエリはMIR本体の他の多くの関連クエリをフィードすることも担当しています。このクエリは、コルーチンの*ビルド済み*mirで動作するため、`mir_promoted`クエリ中に`ensure()`されます[^b3]。

[^b3]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_mir_transform/src/lib.rs#L339-L342>

### クロージャシグネチャ推論

非同期クロージャのクロージャシグネチャ推論アルゴリズムは、「従来の」クロージャの推論アルゴリズムよりも少し複雑です。クロージャのように、関連する可能性のあるすべての句を反復処理します(渡された期待型の場合)[^deduce1]。

シグネチャを抽出するために、2つの状況を考慮します:
- `AsyncFnOnce::Output`を持つ投影述語。これを使用して、クロージャの入力と出力型を抽出します。これは、`F: AsyncFn*() -> T`境界があった状況に対応します[^deduce2]。
- `FnOnce::Output`を持つ投影述語。これを使用して入力を抽出します。出力については、関連する`Future::Output`投影述語を探すことで出力を推測しようとします。これは、`F: Fn*() -> T, T: Future<Output = U>`境界があった状況に対応します。[^deduce3]
  - `Future`境界がない場合は、単に出力に新しい推論変数を使用します。これは、`Option::map`のようなコンビネータ関数に非同期クロージャを渡すことができる場合に対応します。[^deduce4]

[^deduce1]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_hir_typeck/src/closure.rs#L345-L362>

[^deduce2]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_hir_typeck/src/closure.rs#L486-L487>

[^deduce3]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_hir_typeck/src/closure.rs#L517-L534>

[^deduce4]: <https://github.com/rust-lang/rust/blob/5ca0e9fa9b2f92b463a0a2b0b34315e09c0b7236/compiler/rustc_hir_typeck/src/closure.rs#L575-L590>

後者のケースをサポートするのは、単に、ファーストクラスの`AsyncFn*`トレイトが利用可能になる前に設計されたAPIを呼び出す場合でも、ユーザーが`async || {}`構文を簡単にドロップインできるようにするためです。

#### その種類が推論される前にクロージャを呼び出す

typeckの終わりまでコルーチンクロージャの「kind」(つまり、最大呼び出しモード: `AsyncFnOnce`/`AsyncFnMut`/`AsyncFn`)の計算を延期します[^call1]。ただし、typeckの終わりより前にそのコルーチンクロージャを呼び出せるようにしたいので、その前にコルーチンクロージャの戻り型を考え出す必要があります。

[^call1]: <https://github.com/rust-lang/rust/blob/705cfe0e966399e061d64dd3661bfbc57553ed87/compiler/rustc_hir_typeck/src/callee.rs#L169-L210>

戻り型がどの`Fn*`トレイトで呼び出されるかによって変わらない通常のクロージャとは異なり、コルーチンクロージャは、呼び出しに使用される`AsyncFn*`トレイトのフレーバーに応じて、実際に異なるコルーチン型を返します*。*

具体的には、返されるコルーチンのdef-idは変更されませんが、upvars[^call2](親のコルーチンクロージャから借用または移動される)とコルーチンkind[^call3]は呼び出しモードに依存します。

[^call2]: <https://github.com/rust-lang/rust/blob/705cfe0e966399e061d64dd3661bfbc57553ed87/compiler/rustc_type_ir/src/ty_kind/closure.rs#L574-L576>

[^call3]: <https://github.com/rust-lang/rust/blob/705cfe0e966399e061d64dd3661bfbc57553ed87/compiler/rustc_type_ir/src/ty_kind/closure.rs#L554-L563>

`AsyncFnKindHelper`トレイトを導入します。これにより、「このコルーチンクロージャはこの呼び出しモードをサポートしていますか」[^helper1]という質問をトレイトゴールを介して延期でき、「この呼び出しモードのタプル化されたupvarsは何ですか」[^helper2]という質問を関連型を介して延期できます。これは、upvar分析中に計算された入力型をupvarsまたは「by ref」upvarsのいずれかに追加することで計算できます。

[^helper1]: <https://github.com/rust-lang/rust/blob/7c7bb7dc017545db732f5cffec684bbaeae0a9a0/library/core/src/ops/async_function.rs#L135-L144>

[^helper2]: <https://github.com/rust-lang/rust/blob/7c7bb7dc017545db732f5cffec684bbaeae0a9a0/library/core/src/ops/async_function.rs#L146-L154>

#### では、なぜ?

これは少し回りくどく複雑に見えます。そして、確かにそうです。しかし、「何もしない」代替案を考えてみましょう - 代わりに、upvar分析まですべての`AsyncFn*`ゴールを曖昧としてマークすることができます。その時点で、返すコルーチンのupvarsに正確に何を入れるかがわかります。しかし、これは実際にはプログラムの推論に*非常に*有害です。なぜなら、次のようなプログラムが有効でなくなるからです:

```rust,ignore
let c = async || -> String { .. };
let s = c().await;
// ^^^ `<{c} as AsyncFn>::call()`をコルーチンに投影できない場合、`.await`内の`IntoFuture::into_future`呼び出しは停止し、`s`の型は推論変数として制約されないままになります。
s.as_bytes();
// ^^^ つまり、コルーチンクロージャの待機された戻り値に対してメソッドを呼び出すことができません...まったく!
```

そのため、*代わりに*、このエイリアス(この場合、投影: `AsyncFnKindHelper::Upvars<'env, ...>`)を使用して*タプル化されたupvars*の計算を遅らせ、その場所に何かを配置できるようにしながら、引き続き`TyKind::Coroutine`(剛体型)を返すことができます。そして、ビルトイントレイト(この場合、`Future`)を正常に確認できます。`Future`実装はupvarsにまったく依存しないためです。

### Upvar分析

大部分において、コルーチンクロージャとその子コルーチンのupvar分析は、通常のupvar分析のように進行します。ただし、非同期クロージャの特殊な性質を考慮するために、いくつかの興味深い部分があります:

#### すべての入力を強制的にキャプチャする

async fnのように、すべての入力引数がキャプチャされます。これらの入力すべてを明示的にmoveによってキャプチャするよう強制します[^f1]。これにより、非同期クロージャによって返される将来のコルーチンが、入力が本体で*使用*されているかどうかに依存しなくなります。これは興味深いsemver hazardをもたらす可能性があります。

[^f1]: <https://github.com/rust-lang/rust/blob/7c7bb7dc017545db732f5cffec684bbaeae0a9a0/compiler/rustc_hir_typeck/src/upvar.rs#L250-L259>

#### by-refキャプチャの計算

`AsyncFn`/`AsyncFnMut`をサポートするコルーチンクロージャの場合、コルーチンクロージャのキャプチャとその子コルーチンの関係も計算する必要があります。具体的には、コルーチンクロージャがupvarを`move`してキャプチャする場合がありますが、コルーチンはそのupvarを借用するだけです。

子コルーチンのすべてのキャプチャを調べ、対応する親コルーチンクロージャのキャプチャと比較することにより、「`coroutine_captures_by_ref_ty`」を計算します[^br1]。この`coroutine_captures_by_ref_ty`は、`for<'env> fn() -> captures...`型として表現されます。追加のバインダーライフタイムは、`AsyncFn::async_call`または`AsyncFnMut::async_call_mut`を呼び出す際の「`&self`」ライフタイムを表します。実際にメソッドを呼び出す際に、後でそのバインダーをインスタンス化します。

[^br1]: <https://github.com/rust-lang/rust/blob/7c7bb7dc017545db732f5cffec684bbaeae0a9a0/compiler/rustc_hir_typeck/src/upvar.rs#L375-L471>

親コルーチンクロージャからのすべてのby-refキャプチャが「レンディング」借用になるわけではないことに注意してください。詳細については、以下の**フォローアップ: 非同期クロージャが通常の`Fn*`トレイトを実装するのはいつですか?**セクションを参照してください。これは、コルーチンクロージャが`Fn*`トレイトファミリーを実装することが許可されるかどうかに密接に影響します。

#### By-move本体 + `FnOnce`の癖

クロージャのupvar分析が、コルーチンクロージャの子コルーチンに対して緩すぎるupvarsを推論し、最終的にborrow-checkerエラーになるいくつかの状況があります。これは例を通じて最もよく示されます。たとえば、次のように与えられた場合:

```rust
fn force_fnonce<T: async FnOnce()>(t: T) -> T { t }

let x = String::new();
let c = force_fnonce(async move || {
    println!("{x}");
});
```

`x`はコルーチンクロージャに移動されますが、返されるコルーチンは`&x`を借用するだけです。ただし、`force_fnonce`はコルーチンクロージャを`AsyncFnOnce`に強制するため、これは*レンディング*ではありません。by-moveでキャプチャを強制する必要があります[^bm1]。

同様に:

```rust
let x = String::new();
let y = String::new();
let c = async move || {
    drop(y);
    println!("{x}");
};
```

`x`はコルーチンクロージャに移動されますが、返されるコルーチンは`&x`を借用するだけです。ただし、`y`もキャプチャしてドロップするため、コルーチンクロージャは`AsyncFnOnce`に強制されます。`x`のキャプチャもby-moveで強制する必要があります。この特定の状況を判断するために、前の例とは異なり、コルーチンkindのclosure-kindがまだ制約されていないため、コルーチンクロージャの本体を分析して、すべてのupvarsがどのように使用されているかを確認し、「consuming」方法で使用されているかどうかを判断する必要があります - つまり、`FnOnce`に強制するかどうか[^bm2]。

[^bm1]: <https://github.com/rust-lang/rust/blob/7c7bb7dc017545db732f5cffec684bbaeae0a9a0/compiler/rustc_hir_typeck/src/upvar.rs#L211-L248>

[^bm2]: <https://github.com/rust-lang/rust/blob/7c7bb7dc017545db732f5cffec684bbaeae0a9a0/compiler/rustc_hir_typeck/src/upvar.rs#L532-L539>

#### フォローアップ: 非同期クロージャが通常の`Fn*`トレイトを実装するのはいつですか?

まず第一に、すべての非同期クロージャは`FnOnce`を実装します。*少なくとも1回*は常に呼び出すことができるためです。

`Fn`/`FnMut`の場合、詳細な答えは関連する質問に答えることを含みます: コルーチンクロージャはレンディングですか?もしそうなら、非レンディングの`Fn`/`FnMut`トレイトを実装できません。

コルーチンクロージャがそのupvarsを*貸し出す*必要がある場合を判断することは、`should_reborrow_from_env_of_parent_coroutine_closure`ヘルパー関数[^u1]で実装されています。具体的には、これは2つの場所で発生する必要があります:

[^u1]: <https://github.com/rust-lang/rust/blob/7c7bb7dc017545db732f5cffec684bbaeae0a9a0/compiler/rustc_hir_typeck/src/upvar.rs#L1818-L1860>

1. 親クロージャが所有するデータを借用していますか?親キャプチャがby-moveであるかどうかをチェックすることで、これが当てはまるかどうかを判断できます。ただし、デリファレンス投影を適用する場合を除きます。これは、by-moveでキャプチャした参照を再借用していることを意味します。

```rust
let x = &1i32; // このライフタイムを`'1`と呼びましょう。
let c = async move || {
    println!("{:?}", *x);
    // 内部コルーチンはby-refで借用しますが、`*x`のみをキャプチャしています。
    // `x`ではないため、内部クロージャは`'1`のデータを再借用することが許可されます。
};
```

2. コルーチンが親キャプチャから可変的に借用している場合、その可変借用は親*または*元のupvarに対する借用よりも長く生きることはできません。したがって、常に親コルーチンクロージャのenvのライフタイムで子キャプチャを借用する必要があります。

```rust
let mut x = 1i32;
let c = async || {
    x = 1;
    // 親は`x`を何らかの`&'1 mut i32`で借用します。
    // ただし、`c()`を呼び出すと、暗黙的に
    // `AsyncFnMut::async_call_mut`のシグネチャに対して自動参照します。そのライフタイムを`'call`と呼びましょう。
    // `&'call mut &'1 mut i32`が再借用できる最大値は`&'call mut i32`であるため、
    // 内部コルーチンはコルーチンクロージャのライフタイムでキャプチャする必要があります。
};
```

これらのケースのいずれかが当てはまる場合、親コルーチンクロージャのenvのライフタイムで借用をキャプチャする必要があります。幸いなことに、この関数が正しくない場合でも、プログラムは健全ではありません。借用チェックを行い、この関数から行われた選択を検証するためです - 唯一の副作用は、ユーザーが不要なborrowckエラーを受け取る可能性があることです。

### インスタンス解決

コルーチンクロージャのclosure-kindが`FnOnce`の場合、その`AsyncFnOnce::call_once`および`FnOnce::call_once`実装はコルーチンクロージャの本体に解決され[^res1]、返されるコルーチンの`Future::poll`は子クロージャの本体に解決されます。

[^res1]: <https://github.com/rust-lang/rust/blob/705cfe0e966399e061d64dd3661bfbc57553ed87/compiler/rustc_ty_utils/src/instance.rs#L351>

コルーチンクロージャのclosure-kindが`FnMut`/`Fn`の場合、同じことが`AsyncFn`と返されるコルーチンの対応する`Future`実装に適用されます。[^res1]ただし、MIRシムを使用して`AsyncFnOnce::call_once`/`FnOnce::call_once`[^res2]の実装、および存在する場合は`Fn::call`/`FnMut::call_mut`インスタンス[^res3]を生成します。

[^res2]: <https://github.com/rust-lang/rust/blob/705cfe0e966399e061d64dd3661bfbc57553ed87/compiler/rustc_ty_utils/src/instance.rs#L341-L349>

[^res3]: <https://github.com/rust-lang/rust/blob/705cfe0e966399e061d64dd3661bfbc57553ed87/compiler/rustc_ty_utils/src/instance.rs#L312-L326>

これは`ConstructCoroutineInClosureShim`[^i1]によって表されます。`receiver_by_ref` boolは、これが`Fn::call`/`FnMut::call_mut`のインスタンスである場合にtrueになります。[^i2]これらすべてのインスタンスが返すコルーチンは、この時点までに合成したby-move本体に対応します。[^i3]

[^i1]: <https://github.com/rust-lang/rust/blob/705cfe0e966399e061d64dd3661bfbc57553ed87/compiler/rustc_middle/src/ty/instance.rs#L129-L134>

[^i2]: <https://github.com/rust-lang/rust/blob/705cfe0e966399e061d64dd3661bfbc57553ed87/compiler/rustc_middle/src/ty/instance.rs#L136-L141>

[^i3]: <https://github.com/rust-lang/rust/blob/07cbbdd69363da97075650e9be24b78af0bcdd23/compiler/rustc_middle/src/ty/instance.rs#L841>

### 借用チェック

非同期クロージャの借用チェックは非常に簡単であることがわかりました。新しい`DefiningTy::CoroutineClosure`[^bck1]バリアントを追加し、borrowckにコルーチンクロージャのシグネチャを生成する方法を教えた後[^bck2]、borrowckは完全に正常に進行します。

注意すべき点の1つは、by-moveコルーチン用に作成する合成本体を借用チェックしないことです。構築上(およびそれが派生したby-refコルーチン本体の妥当性から)、それは有効でなければならないためです。

[^bck1]: <https://github.com/rust-lang/rust/blob/705cfe0e966399e061d64dd3661bfbc57553ed87/compiler/rustc_borrowck/src/universal_regions.rs#L110-L115>

[^bck2]: <https://github.com/rust-lang/rust/blob/7c7bb7dc017545db732f5cffec684bbaeae0a9a0/compiler/rustc_borrowck/src/universal_regions.rs#L743-L790>
