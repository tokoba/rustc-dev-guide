# インタプリタ

インタプリタは、マシンコードにコンパイルせずにMIRを実行するための仮想マシンです。
通常、`tcx.const_eval_*`関数を介して呼び出されます。インタプリタは、
コンパイラ（コンパイル時関数評価、CTFE用）とツール[Miri](https://github.com/rust-lang/miri/)の間で
共有されています。Miriは同じ仮想マシンを使用して、（unsafe）Rustコードで
未定義動作を検出します。

定数から始める場合：

```rust
const FOO: usize = 1 << 12;
```

rustcは、定数が使用されるかメタデータに配置されるまで、
実際には何も呼び出しません。

次のような使用サイトがある場合：

```rust,ignore
type Foo = [u8; FOO - 42];
```

コンパイラは、型を使用するアイテム（ローカル、定数、関数引数、...）を
作成できるようにする前に、配列の長さを把握する必要があります。

（この場合は空の）パラメーター環境を取得するには、
`let param_env = tcx.param_env(length_def_id);`を呼び出すことができます。必要な`GlobalId`は

```rust,ignore
let gid = GlobalId {
    promoted: None,
    instance: Instance::mono(length_def_id),
};
```

`tcx.const_eval(param_env.and(gid))`を呼び出すと、配列長式の
MIRの作成がトリガーされます。MIRは次のようになります：

```mir
Foo::{{constant}}#0: usize = {
    let mut _0: usize;
    let mut _1: (usize, bool);

    bb0: {
        _1 = CheckedSub(const FOO, const 42usize);
        assert(!move (_1.1: bool), "attempt to subtract with overflow") -> bb1;
    }

    bb1: {
        _0 = move (_1.0: usize);
        return;
    }
}
```

評価前に、評価結果を保存するための仮想メモリ位置（この場合、本質的に
`vec![u8; 4]`または`vec![u8; 8]`）が作成されます。

評価の開始時、`_0`と`_1`は
`Operand::Immediate(Immediate::Scalar(ScalarMaybeUndef::Undef))`です。これは非常に
長い言い方です：[`Operand`]は、[インタプリタメモリ](#memory)のどこかに保存された
データ（`Operand::Indirect`）、または（最適化として）インラインで保存された
即値データのいずれかを表すことができます。そして[`Immediate`]は、単一の
（潜在的に未初期化の）[スカラー値][`Scalar`]（整数または細いポインタ）、
またはそれらの2つのペアのいずれかです。この場合、単一のスカラー値は（まだ）
初期化されて*いません*。

`_1`の初期化が呼び出されると、`FOO`定数の値が必要になり、
別の`tcx.const_eval_*`への呼び出しがトリガーされます。これはここでは示しません。
FOOの評価が成功した場合、`42`がその値`4096`から減算され、結果が
`Operand::Immediate(Immediate::ScalarPair(Scalar::Raw { data: 4054, .. },
Scalar::Raw { data: 0, .. })`として`_1`に保存されます。ペアの最初の部分は計算された値、
2番目の部分はオーバーフローが発生した場合にtrueになるboolです。`Scalar::Raw`は、
このスカラー値のサイズ（バイト単位）も保存します；ここでは省略しています。

次のステートメントは、そのbooleanが`0`であることをアサートします。アサーションが
失敗した場合、そのエラーメッセージはコンパイル時エラーを報告するために使用されます。

失敗しないため、`Operand::Immediate(Immediate::Scalar(Scalar::Raw {
data: 4054, .. }))`が、評価前に割り当てられた仮想メモリに保存されます。
`_0`は常にその場所を直接参照します。

評価が完了すると、戻り値は[`op_to_const`]によって[`Operand`]から
[`ConstValue`]に変換されます：前者の表現はconst評価*中*に必要なものに
向けられていますが、[`ConstValue`]は、const評価の結果を消費するコンパイラの
残りの部分のニーズによって形作られています。この変換の一部として、
スカラー値を持つ型の場合、結果の[`Operand`]が`Indirect`であっても、
即値の`ConstValue::Scalar(computed_value)`（通常の`ConstValue::ByRef`の代わりに）を
返します。これにより、結果の使用がより効率的かつより便利になります。`usize`のような
シンプルなものを取得するために、さらにクエリを実行する必要がないためです。

同じ定数の将来の評価は、実際にはインタプリタを呼び出さず、
キャッシュされた結果を使用するだけです。

[`Operand`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/interpret/operand/enum.Operand.html
[`Immediate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/interpret/enum.Immediate.html
[`ConstValue`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/consts/enum.ConstValue.html
[`Scalar`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/interpret/enum.Scalar.html
[`op_to_const`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/const_eval/eval_queries/fn.op_to_const.html

## データ構造

インタプリタの外部向けデータ構造は、
[rustc_middle/src/mir/interpret](https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_middle/src/mir/interpret)にあります。
これは主にエラー列挙型と[`ConstValue`]および[`Scalar`]型です。
`ConstValue`は、`Scalar`（単一の`Scalar`、つまり整数または細いポインタ）、
`Slice`（パターンマッチングに必要なバイトスライスと文字列を表すため）、
または`ByRef`（他のすべてに使用され、仮想割り当てを参照します）のいずれかです。
これらの割り当ては、`tcx.interpret_interner`のメソッドを介してアクセスできます。
`Scalar`は、一部の`Raw`整数またはポインタのいずれかです；
詳細については[次のセクション](#memory)を参照してください。

数値結果を期待している場合、`eval_usize`（`u64`として表現できないものでパニック）
または`try_eval_usize`を使用できます。これは、可能な場合は`Scalar`を生成する
`Option<u64>`になります。

## メモリ

任意の種類のポインタをサポートするために、インタプリタには、ポインタが指すことができる
「仮想メモリ」が必要です。これは[`Memory`]型で実装されています。最も単純なモデルでは、
すべてのグローバル変数、スタック変数、および動的割り当てが、そのメモリ内の
[`Allocation`]に対応します。（実際には、すべてのMIRスタック変数に割り当てを使用すると
非常に非効率的です；そのため、小さく、アドレスが取られないスタック変数には
`Operand::Immediate`があります。しかし、それは純粋な最適化です。）

このような`Allocation`は、基本的に、この割り当ての各バイトの値を保存する`u8`の
シーケンスです。（さらにいくつかの追加データがあります。以下を参照してください。）
すべての`Allocation`には、`Memory`内でグローバルに一意の`AllocId`が割り当てられます。
それにより、[`Pointer`]は、`AllocId`（割り当てを示す）と割り当てへのオフセット
（ポインタが割り当てのどのバイトを指しているかを示す）のペアで構成されます。
`Pointer`が単なる整数アドレスではないのは奇妙に見えるかもしれませんが、
const評価中に、割り当てが実際にどの整数アドレスに配置されるかを知ることができないことを
覚えておいてください -- したがって、`AllocId`をシンボリックベースアドレスとして使用します。
つまり、別個のオフセットが必要です。（余談ですが、実行時のポインタも
[単なる整数以上のもの](https://rust-lang.github.io/unsafe-code-guidelines/glossary.html#pointer-provenance)であることがわかります。）

これらの割り当ては、参照と生ポインタが何かを指すために存在します。
物事が割り当てられるグローバルな線形ヒープはありませんが、各割り当て
（ローカル変数、静的、または（将来の）ヒープ割り当て用）は、必要なサイズとまったく
同じサイズの独自の小さなメモリを取得します。したがって、ローカル変数`a`の割り当てへの
ポインタがある場合、（どれだけunsafeであっても）それを異なるローカル変数`b`への
ポインタに変更する可能性のある操作はありません。`a`でのポインタ演算は、
そのオフセットを変更するだけです；`AllocId`は同じままです。

ただし、これにより、`Pointer`を`Allocation`に保存したい場合に問題が発生します：
適切な長さの`u8`のシーケンスに変換できません！`AllocId`とオフセットを合わせると、
ポインタが「見える」べきサイズの2倍です。これが`Allocation`の`relocation`フィールドの
目的です：`Pointer`のバイトオフセットは、いくつかの`u8`として保存され、
その`AllocId`は帯域外で保存されます。2つは、`Pointer`がメモリから読み取られるときに
再組み立てされます。`Allocation`が必要とする他のビットの追加データは、
どのバイトが初期化されているかを追跡するための`undef_mask`です。

### グローバルメモリとエキゾチックな割り当て

`Memory`は評価中にのみ存在します；定数の最終値が計算されると破棄されます。
その定数にポインタが含まれている場合、それらは「インターン化」され、
`TyCtxt`の一部であるグローバルな「const eval memory」に移動されます。
これらの割り当ては、残りの計算中に保持され、最終出力にシリアライズされます
（依存クレートがそれらを使用できるようにするため）。

さらに、関数ポインタもサポートするために、`TyCtxt`のグローバルメモリには
「仮想割り当て」も含めることができます：`Allocation`の代わりに、これらには
`Instance`が含まれます。これにより、`Pointer`は通常のデータまたは関数のいずれかを
指すことができ、関数ポインタから生ポインタへのキャストを評価できるようにするために
必要です。

最後に、グローバルメモリで使用される[`GlobalAlloc`]型には、
特定の`const`または`static`アイテムを指す`Static`バリアントも含まれています。
これは、循環静的をサポートするために必要です。値のバイトをまだ知ることができないため、
`Allocation`をまだ持つことができない`static`への`Pointer`を持つ必要があります。

[`Memory`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/interpret/struct.Memory.html
[`Allocation`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/interpret/struct.Allocation.html
[`Pointer`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/interpret/struct.Pointer.html
[`GlobalAlloc`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/interpret/enum.GlobalAlloc.html

### ポインタ値対ポインタ型

インタプリタでよくある混乱の原因の1つは、ポインタ*値*であることとポインタ*型*を
持つことが完全に独立した特性であることです。「ポインタ値」とは、`Pointer`を含む
`Scalar::Ptr`を指し、したがってインタプリタの仮想メモリのどこかを指しています。
これは、いくつかの具体的な整数である`Scalar::Raw`とは対照的です。

ただし、`*const T`や`&T`のようなポインタまたは参照*型*の変数は、
ポインタ*値*を持つ必要はありません：整数をポインタにキャストまたは変換することで
取得できます。同様に、実際の割り当てへの参照を整数にキャストまたは変換すると、
整数*型*（`usize`）でポインタ*値*（`Scalar::Ptr`）になります。これは、
ポインタ値に対して除算などの整数演算を意味のある方法で実行できないため、
問題です。

## 解釈

定数評価へのメインエントリポイントは`tcx.const_eval_*`関数ですが、
[rustc_const_eval/src/const_eval](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/index.html)には、
`ConstValue`（`ByRef`またはその他）のフィールドにアクセスできる追加の関数があります。
コンパイルターゲット（現時点ではLLVMだけ）に翻訳する場合を除いて、
`Allocation`に直接アクセスする必要はありません。

インタプリタは、評価されている現在の定数用の仮想スタックフレームを作成することから
始まります。定数と引数のない関数の間には、本質的に違いはありません。
ただし、このガイドを書いている時点では、定数はローカル（名前付き）変数を
許可していません。

スタックフレームは、
[rustc_const_eval/src/interpret/eval_context.rs](https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_const_eval/src/interpret/eval_context.rs)の
`Frame`型で定義され、すべてのローカル変数メモリ（評価の開始時は`None`）を含みます。
各フレームは、ルート定数または`const fn`への後続の呼び出しのいずれかの評価を
参照します。別の定数の評価は単に`tcx.const_eval_*`を呼び出すだけで、
これは完全に新しい独立したスタックフレームを生成します。

フレームは単なる`Vec<Frame>`であり、ホラーな不正行為がunsafeコードを介して行われても、
`Frame`のメモリを実際に参照する方法はありません。参照できる唯一のメモリは
`Allocation`です。

インタプリタは、エラーを返すか、実行するステートメントがなくなるまで、
`step`メソッド（
[rustc_const_eval/src/interpret/step.rs](https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_const_eval/src/interpret/step.rs)内）を
呼び出します。各ステートメントは、ローカルまたはローカルによって参照される仮想メモリを
初期化または変更します。これには、他の定数または静的を評価する必要がある場合があり、
その場合は単に`tcx.const_eval_*`を再帰的に呼び出します。
