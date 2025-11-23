# マクロ展開

Rustには非常に強力なマクロシステムがあります。前の章では、
パーサーがマクロを展開のために（一時的な[プレースホルダー]を使用して）どのように保留するかを見ました。
この章では、クレートのマクロが展開されていない（またはコンパイルエラー）完全な
[*抽象構文木*（AST）][ast]を得るまで、これらのマクロを反復的に展開するプロセスについて説明します。

[ast]: ./ast-validation.md
[placeholders]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/placeholders/index.html

まず、マクロ出力を展開してASTに統合するアルゴリズムについて説明します。次に、
ハイジーンデータの収集方法を見ていきます。最後に、
さまざまなタイプのマクロの展開の詳細を見ていきます。

以下で説明する多くのアルゴリズムとデータ構造は、[`rustc_expand`]にあり、
基本的なデータ構造は[`rustc_expand::base`][base]にあります。

また、`cfg`と`cfg_attr`は他のマクロとは特別に扱われ、
[`rustc_expand::config`][cfg]で処理されることにも注意してください。

[`rustc_expand`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/index.html
[base]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/index.html
[cfg]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/config/index.html

## 展開とAST統合

まず、展開はクレートレベルで行われます。クレートの生のソースコードが与えられると、
コンパイラは、すべてのマクロが展開され、すべてのモジュールがインライン化された、
巨大なASTを生成します。このプロセスの主なエントリポイントは、
[`MacroExpander::fully_expand_fragment`][fef]メソッドです。いくつかの例外を除いて、
クレート全体でこのメソッドを使用します
（エッジケース展開の問題の詳細については、以下の[「Eager Expansion」](#eager-expansion)を参照してください）。

[`rustc_builtin_macros`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_builtin_macros/index.html
[reb]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/build/index.html

高レベルでは、[`fully_expand_fragment`][fef]は反復で機能します。
未解決のマクロ呼び出し（つまり、定義がまだ見つかっていないマクロ）のキューを保持します。
キューからマクロを繰り返し取り出して、解決し、展開し、統合し直します。
反復で進展しない場合、これはコンパイルエラーを表します。以下は[アルゴリズム][original]です。

[fef]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/expand/struct.MacroExpander.html#method.fully_expand_fragment
[original]: https://github.com/rust-lang/rust/pull/53778#issuecomment-419224049

1. 未解決のマクロの`queue`を初期化します。
2. `queue`が空になるまで（または進展しない場合、これはエラーです）繰り返します。
   1. 部分的に構築されたクレートのインポートを可能な限り[解決](./name-resolution.md)します。
   2. 部分的に構築されたクレートからできるだけ多くのマクロ[`Invocation`s][inv]
      （`fn`のような、属性、派生）を収集し、キューに追加します。
   3. 最初の要素をデキューして、解決を試みます。
   4. 解決された場合：
      1. [`TokenStream`]またはASTを消費し、[`TokenStream`]または[`AstFragment`]
         （マクロの種類による）を生成するマクロの展開関数を実行します。
         （[`TokenStream`]は[`TokenTree`s][tt]のコレクションであり、
         それぞれはトークン（句読点、識別子、またはリテラル）または
         区切られたグループ（`()`/`[]`/`{}`内のもの）です）。
         - この時点で、マクロ自体についてすべてを知っており、
           グローバルデータでそのプロパティを埋めるために[`set_expn_data`]を
           呼び出すことができます。つまり、[`ExpnId`]に関連付けられた[hygiene]データです
           （以下の[Hygiene][hybelow]を参照）。
      2. そのASTの断片を、現在存在するが部分的に構築されたASTに統合します。
         これは本質的に「トークンのような塊」が適切に確定したASTとサイドテーブルになる場所です。
         次のように行われます。
         - マクロがトークンを生成する場合（例：procマクロ）、ASTに解析します。
           これは解析エラーを生成する可能性があります。
         - 展開中に、[`SyntaxContext`]s（階層2）を作成します
           （以下の[Hygiene][hybelow]を参照）。
         - これらの3つのパスは、マクロから新しく展開されたすべてのASTフラグメントで
           次々に行われます。
           - [`NodeId`]sは[`InvocationCollector`]によって割り当てられます。
             これは、この新しいAST断片から新しいマクロ呼び出しも収集し、
             キューに追加します。
           - [「Defパス」][defpath]が作成され、[`DefId`]sが
             [`DefCollector`]によって割り当てられます。
           - 名前は、[`BuildReducedGraphVisitor`]によって
             モジュール（リゾルバーの観点から）に配置されます。
      3. 単一のマクロを展開してその出力を統合した後、
         [`fully_expand_fragment`][fef]の次の反復に進みます。
   5. 解決されない場合：
      1. マクロをキューに戻します。
      2. 次の反復に進みます...

[`AstFragment`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/expand/enum.AstFragment.html
[`BuildReducedGraphVisitor`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/build_reduced_graph/struct.BuildReducedGraphVisitor.html
[`DefCollector`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/def_collector/struct.DefCollector.html
[`DefId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/def_id/struct.DefId.html
[`ExpnId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.ExpnId.html
[`InvocationCollector`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/expand/struct.InvocationCollector.html
[`NodeId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/node_id/struct.NodeId.html
[`set_expn_data`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.LocalExpnId.html#method.set_expn_data
[`SyntaxContext`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.SyntaxContext.html
[`TokenStream`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/tokenstream/struct.TokenStream.html
[defpath]: hir.md#identifiers-in-the-hir
[hybelow]: #hygiene-and-hierarchies
[hygiene]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/index.html
[inv]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/expand/struct.Invocation.html
[tt]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/tokenstream/enum.TokenTree.html

### エラー回復

反復で進展しない場合、コンパイルエラーに到達しました
（例：未定義のマクロ）。失敗（つまり、未解決のマクロまたはインポート）から
診断を生成する意図で回復を試みます。
失敗回復は、未解決のマクロを[`ExprKind::Err`][err]に展開することで行われ、
`rustc`が元の失敗だけでなくより多くのエラーを報告できるように、
最初のエラーを超えてコンパイルを続行できるようにします。

[err]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/ast/enum.ExprKind.html#variant.Err

### 名前解決

名前解決がここに関与していることに注意してください。上記のアルゴリズムでは、
インポートとマクロ名を解決する必要があります。これは、
[`rustc_resolve::macros`][mresolve]で行われ、マクロパスを解決し、
これらの解決を検証し、さまざまなエラー（例：「見つからない」、「見つかったが不安定」、
「xが期待されたが、yが見つかった」）を報告します。ただし、
他の名前はまだ解決しようとしません。これは後で行われます。
章[Name Resolution](./name-resolution.md)で説明します。

[mresolve]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/macros/index.html

### Eager Expansion

_Eager expansion_とは、マクロ呼び出し自体の前にマクロ呼び出しの引数を展開することを意味します。
これは、リテラルを期待するいくつかの特別な組み込みマクロに対してのみ実装されています。
これらのマクロの一部で最初に引数を展開すると、よりスムーズなユーザー体験が得られます。
例として、次のことを考えてください。

```rust,ignore
macro bar($i: ident) { $i }
macro foo($i: ident) { $i }

foo!(bar!(baz));
```

遅延展開は、最初に`foo!`を展開します。eager-expansionは、最初に`bar!`を展開します。

Eager-expansionは、Rustの一般的に利用可能な機能ではありません。
eager-expansionをより一般的に実装することは困難なので、
ユーザー体験のために、いくつかの特別な組み込みマクロに対して実装します。
組み込みマクロは、[`rustc_builtin_macros`]で実装されており、
標準ライブラリのインポートの注入やテストハーネスの生成など、
他の初期のコード生成機能と共に実装されています。
ASTフラグメントを構築するための追加のヘルパーが[`rustc_expand::build`][reb]にあります。
Eager-expansionは一般的に、遅延（通常）展開が行うことのサブセットを実行します。
これは、クレート全体ではなく（通常のように）、クレートの一部のみで
[`fully_expand_fragment`][fef]を呼び出すことによって行われます。

### その他のデータ構造

展開と統合に関与する他の注目すべきデータ構造は次のとおりです。
- [`ResolverExpand`] - クレートの依存関係を壊すために使用される`trait`。
  これにより、[`rustc_resolve`]とほぼすべてが[`rustc_ast`]に依存しているにもかかわらず、
  [`rustc_ast`]でリゾルバーサービスを使用できます。
- [`ExtCtxt`]/[`ExpansionData`] - さまざまな中間展開インフラストラクチャデータを保持します。
- [`Annotatable`] - 属性ターゲットになり得るASTの断片。
  マクロによって生成できるが属性で注釈を付けることができない型とパターンを除いて、
  [`AstFragment`]とほぼ同じです。
- [`MacResult`] - 「ポリモーフィック」ASTフラグメント。
  [`AstFragmentKind`]（つまり、アイテム、式、パターンなど）に応じて、
  異なる[`AstFragment`]に変換できるもの。

[`AstFragment`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/expand/enum.AstFragment.html
[`rustc_ast`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/index.html
[`rustc_resolve`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/index.html
[`ResolverExpand`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/trait.ResolverExpand.html
[`ExtCtxt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/struct.ExtCtxt.html
[`ExpansionData`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/struct.ExpansionData.html
[`Annotatable`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/enum.Annotatable.html
[`MacResult`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/trait.MacResult.html
[`AstFragmentKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/expand/enum.AstFragmentKind.html

## ハイジーンと階層

C/C++プリプロセッサマクロを使用したことがある場合、
いくつかの厄介でデバッグが困難な落とし穴があることをご存知でしょう！
たとえば、次のCコードを考えてみましょう。

```c
#define DEFINE_FOO struct Bar {int x;}; struct Foo {Bar bar;};

// Then, somewhere else
struct Bar {
    ...
};

DEFINE_FOO
```

ほとんどの人はこのようなCを書くことを避けます。そして正当な理由があります。
コンパイルされません。マクロによって定義された`struct Bar`は、
コードで定義された`struct Bar`と名前が衝突します。次の例も考えてみましょう。

```c
#define DO_FOO(x) {\
    int y = 0;\
    foo(x, y);\
    }

// Then elsewhere
int y = 22;
DO_FOO(y);
```

問題がわかりますか？`foo(22, 0)`の呼び出しを生成したかったのですが、
代わりに`foo(0, 0)`を取得しました。マクロが独自の`y`を定義したためです！

これらは両方とも_マクロハイジーン_の問題の例です。_ハイジーン_は、
_マクロ内で_定義された名前の処理方法に関連しています。特に、
衛生的なマクロシステムは、マクロ内で導入された名前によるエラーを防ぎます。
Rustマクロは衛生的であり、上記のようなバグを書くことを許可しません。

高レベルでは、Rustコンパイラ内のハイジーンは、
名前が導入および使用されるコンテキストを追跡することによって達成されます。
次に、そのコンテキストに基づいて名前を明確にできます。
マクロシステムの将来の反復により、
マクロ作成者はそのコンテキストを使用してより大きな制御を得ることができます。
たとえば、マクロ作成者は、マクロが呼び出されたコンテキストに新しい名前を導入したい場合があります。
または、マクロ作成者は、マクロ内でのみ使用する変数を定義している可能性があります
（つまり、マクロの外部には表示されないようにする必要があります）。

[code_dir]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_expand/src/mbe
[code_mp]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/mbe/macro_parser
[code_mr]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/mbe/macro_rules
[code_parse_int]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/mbe/macro_parser/struct.TtParser.html#method.parse_tt
[parsing]: ./the-parser.html

コンテキストはASTノードに添付されます。マクロによって生成されたすべてのASTノードには、
コンテキストが添付されています。さらに、一部の脱糖構文など、
コンテキストが添付された他のノードがある場合があります
（マクロ以外で展開されたノードは、以下で説明するように、
「ルート」コンテキストを持つだけと見なされます）。
コンパイラ全体で、[`rustc_span::Span`s][span]を使用してコードの場所を参照します。
この構造体には、後で見るように、ハイジーン情報も添付されています。

[span]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/struct.Span.html

マクロの呼び出しと定義はネストできるため、
ノードの構文コンテキストは階層である必要があります。たとえば、
マクロを展開し、生成された出力に別のマクロ呼び出しまたは定義がある場合、
構文コンテキストはネストを反映する必要があります。

ただし、実際には、異なる目的のために追跡したいコンテキストのタイプが
いくつかあることがわかります。したがって、1つではなく_3つ_の
展開階層があり、それらが一緒になってクレートのハイジーン情報を構成します。

これらの階層はすべて、展開のチェーン内の個々の要素を識別するために、
ある種の「マクロID」を必要とします。このIDは[`ExpnId`]です。すべてのマクロは、
新しいマクロ呼び出しを発見するときに0から連続して割り当てられた整数IDを受け取ります。
すべての階層は、それ自体の親である[`ExpnId::root`][rootid]から始まります。

[`rustc_span::hygiene`][hy]クレートには、すべてのハイジーン関連のアルゴリズム
（[`Resolver::resolve_crate_root`][hacks]のいくつかのハックを除く）と、
グローバルデータに保持されているハイジーンと展開に関連する構造が含まれています。

実際の階層は[`HygieneData`][hd]に格納されています。これは、
任意の[`Ident`]からコンテキストなしでアクセスできる、
ハイジーンと展開情報を含むグローバルなデータです。


[`ExpnId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.ExpnId.html
[rootid]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.ExpnId.html#method.root
[hd]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.HygieneData.html
[hy]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/index.html
[hacks]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/struct.Resolver.html#method.resolve_crate_root
[`Ident`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/symbol/struct.Ident.html

### 展開順序階層

最初の階層は、展開の順序を追跡します。
つまり、マクロ呼び出しが別のマクロの出力にある場合です。

ここで、階層の子は「最も内側の」トークンになります。
[`ExpnData`]構造体自体には、グローバルデータを介して利用可能な
マクロ定義とマクロ呼び出しの両方からのプロパティのサブセットが含まれています。
[`ExpnData::parent`][edp]は、この階層の子から親へのリンクを追跡します。

[`ExpnData`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.ExpnData.html
[edp]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.ExpnData.html#structfield.parent

例えば：

```rust,ignore
macro_rules! foo { () => { println!(); } }

fn main() { foo!(); }
```

このコードでは、最終的に生成されるASTノードは、階層
`root -> id(foo) -> id(println)`を持ちます。

### マクロ定義階層

2番目の階層は、マクロ定義の順序を追跡します。
つまり、1つのマクロを展開しているときに、その出力に別のマクロ定義が明らかにされる場合です。
これは、他の2つの階層よりも少し厄介で複雑です。

[`SyntaxContext`][sc]は、IDを介してこの階層のチェーン全体を表します。
[`SyntaxContextData`][scd]には、指定された[`SyntaxContext`][sc]に関連付けられたデータが含まれています。
ほとんどの場合、さまざまな方法でそのチェーンをフィルタリングした結果のキャッシュです。
[`SyntaxContextData::parent`][scdp]がここでの子から親へのリンクであり、
[`SyntaxContextData::outer_expns`][scdoe]がチェーン内の個々の要素です。
「連鎖演算子」は、コンパイラコードの[`SyntaxContext::apply_mark`][am]です。

上記で述べた[`Span`][span]は、実際には、
コードの場所と[`SyntaxContext`][sc]のコンパクトな表現にすぎません。
同様に、[`Ident`]は、インターンされた[`Symbol`] + `Span`
（つまり、インターンされた文字列 + ハイジーンデータ）にすぎません。

[`Symbol`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/symbol/struct.Symbol.html
[scd]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.SyntaxContextData.html
[scdp]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.SyntaxContextData.html#structfield.parent
[sc]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.SyntaxContext.html
[scdoe]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.SyntaxContextData.html#structfield.outer_expn
[am]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.SyntaxContext.html#method.apply_mark

組み込みマクロの場合、コンテキスト：
[`SyntaxContext::empty().apply_mark(expn_id)`]を使用し、
そのようなマクロは階層ルートで定義されていると見なされます。
クレート間のハイジーンをまだ実装していないため、
`proc macro`に対しても同じことを行います。

[`SyntaxContext::empty().apply_mark(expn_id)`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.SyntaxContext.html#method.apply_mark

トークンがマクロによって生成される前にコンテキスト`X`を持っていた場合、
マクロによって生成された後、コンテキスト`X -> macro_id`を持ちます。
いくつかの例を次に示します。

例 0:

```rust,ignore
macro m() { ident }

m!();
```

ここで、最初は[`SyntaxContext::root`][scr]コンテキストを持つ`ident`は、
`m`によって生成された後、コンテキスト`ROOT -> id(m)`を持ちます。

[scr]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.SyntaxContext.html#method.root

例 1:

```rust,ignore
macro m() { macro n() { ident } }

m!();
n!();
```

この例では、`ident`は最初に`ROOT`コンテキストを持ち、
次に最初の展開後に`ROOT -> id(m)`、
次に`ROOT -> id(m) -> id(n)`を持ちます。

例 2:

これらのチェーンは、最後の要素によって完全に決定されるわけではないことに注意してください。
つまり、[`ExpnId`]は[`SyntaxContext`][sc]に同型ではありません。

```rust,ignore
macro m($i: ident) { macro n() { ($i, bar) } }

m!(foo);
```

すべての展開後、`foo`はコンテキスト`ROOT -> id(n)`を持ち、
`bar`はコンテキスト`ROOT -> id(m) -> id(n)`を持ちます。

現在、マクロ定義を追跡するこの階層は、
いわゆる[「コンテキスト移植ハック」][hack]の対象となっています。
モダン（つまり実験的）マクロは、レガシーの「Macros By Example」（MBE）
システムよりも強力なハイジーンを持っており、2つの間の奇妙な相互作用を引き起こす可能性があります。
ハックは、今のところ物事を「うまく機能させる」ことを目的としています。

[`ExpnId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.ExpnId.html
[hack]: https://github.com/rust-lang/rust/pull/51762#issuecomment-401400732

### 呼び出しサイト階層

3番目で最後の階層は、マクロ呼び出しの場所を追跡します。

この階層では、[`ExpnData::call_site`][callsite]が`子 -> 親`リンクです。

[callsite]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/hygiene/struct.ExpnData.html#structfield.call_site

例を示します。

```rust,ignore
macro bar($i: ident) { $i }
macro foo($i: ident) { $i }

foo!(bar!(baz));
```

最終出力の`baz` ASTノードの場合、展開順序階層は
`ROOT -> id(foo) -> id(bar) -> baz`ですが、呼び出しサイト階層は`ROOT ->
baz`です。

### マクロバックトレース

マクロバックトレースは、[`rustc_span::hygiene`][hy]のハイジーン機構を使用して
[`rustc_span`]で実装されています。

[`rustc_span`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/index.html

## マクロ出力の生成

上記では、マクロの出力がクレートのASTにどのように統合されるかを見て、
クレートのハイジーンデータがどのように生成されるかも見ました。
しかし、実際にマクロの出力をどのように生成するのでしょうか？
それはマクロのタイプによって異なります。

Rustには2種類のマクロがあります。
  1. `macro_rules!`マクロ（別名「Macros By Example」（MBE））、および、
  2. 手続きマクロ（procマクロ）。カスタム派生を含みます。

解析フェーズ中、通常のRustパーサーは、
マクロとその呼び出しの内容を保留します。後で、マクロは、
コードのこれらの部分を使用して展開されます。

ここでのいくつかの重要なデータ構造/インターフェース：
- [`SyntaxExtension`] - 低レベルのマクロ表現。展開関数が含まれており、
  [`TokenStream`]またはASTを別の[`TokenStream`]またはASTに変換します。
  さらに、安定性や、マクロ内で許可される不安定な機能のリストなどの追加データも含まれます。
- [`SyntaxExtensionKind`] - 展開関数にはいくつかの異なるシグネチャがある場合があります
  （1つのトークンストリームを取るか、2つを取るか、ASTの一部を取るかなど）。
  これは、それらをリストする`enum`です。
- [`BangProcMacro`]/[`TTMacroExpander`]/[`AttrProcMacro`]/[`MultiItemModifier`] -
  展開関数のシグネチャを表す`trait`。

[`SyntaxExtension`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/struct.SyntaxExtension.html
[`SyntaxExtensionKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/enum.SyntaxExtensionKind.html
[`BangProcMacro`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/trait.BangProcMacro.html
[`TTMacroExpander`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/trait.TTMacroExpander.html
[`AttrProcMacro`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/trait.AttrProcMacro.html
[`MultiItemModifier`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/base/trait.MultiItemModifier.html

## Macros By Example

MBEには、Rustパーサーとは別の独自のパーサーがあります。
マクロが展開されると、MBEパーサーを呼び出してマクロを解析して展開します。
MBEパーサーは、メタ変数（例：`$my_expr`）をバインドする必要がある場合、
Rustパーサーを呼び出すことができます。マクロ拡張のコードは
[`compiler/rustc_expand/src/mbe/`][code_dir]にあります。

### 例

```rust,ignore
macro_rules! printer {
    (print $mvar:ident) => {
        println!("{}", $mvar);
    };
    (print twice $mvar:ident) => {
        println!("{}", $mvar);
        println!("{}", $mvar);
    };
}
```

ここで`$mvar`は_メタ変数_と呼ばれます。通常の変数とは異なり、
_実行時_に値にバインドするのではなく、メタ変数は_コンパイル時_に
_トークン_のツリーにバインドします。_トークン_は、
識別子（例：`foo`）や句読点（例：`=>`）など、文法の単一の「単位」です。
`EOF`など、他の特別なトークンもあり、それ自体はトークンがこれ以上ないことを示します。
対になった括弧のような文字（`(`...`)`、`[`...`]`、`{`...`}`）から生成されるトークンツリーもあります。
これには、開閉と、その間のすべてのトークンが含まれます
（Rustでは、括弧のような文字がバランスしている必要があります）。
トークンストリームでマクロ展開を操作することで、多くの複雑さが抽象化されます。
マクロ展開（およびコンパイラの残りの大部分）は、
ソースファイルの生のバイトではなく、
コードで使用される構成要素を考慮します。
マクロ展開は、コード内のいくつかの構文構成の正確な行と列を考慮しません。
コードで使用される構成要素を考慮します。トークンを使用すると、
_どこ_を心配することなく_何_を気にすることができます。トークンの詳細については、
この本の[Parsing][parsing]章を参照してください。

```rust,ignore
printer!(print foo); // `foo` is a variable
```

マクロ呼び出しを構文木`println!("{}", foo)`に展開し、
次に構文木を`Display::fmt`への呼び出しに展開するプロセスは、
_マクロ展開_の一般的な例の1つです。

### MBEパーサー

マクロパーサーによって行われるMBE展開には2つの部分があります。
  1. 定義の解析、および、
  2. 呼び出しの解析。

MBEパーサーは、[Earley
parsing algorithm](https://en.wikipedia.org/wiki/Earley_parser)と精神的に類似したアルゴリズムを使用するため、
非決定性有限オートマトン（NFA）ベースの正規表現パーサーと考えています。
マクロパーサーは、
[`compiler/rustc_expand/src/mbe/macro_parser.rs`][code_mp]で定義されています。

マクロパーサーのインターフェースは次のとおりです（これは少し簡略化されています）。

```rust,ignore
fn parse_tt(
    &mut self,
    parser: &mut Cow<'_, Parser<'_>>,
    matcher: &[MatcherLoc]
) -> ParseResult
```

マクロパーサーで次のアイテムを使用します。

- `parser`変数は、通常のRustパーサーの状態への参照です。
  トークンストリームと解析セッションが含まれます。トークンストリームは、
  MBEパーサーに解析を求めようとしているものです。トークンの生ストリームを消費し、
  メタ変数と対応するトークンツリーのバインディングを出力します。
  解析セッションを使用して、パーサーエラーを報告できます。
- `matcher`変数は、トークンストリームと照合したい[`MatcherLoc`]sのシーケンスです。
  これらは、一致前にマクロの定義の元のトークンツリーから変換されます。

[`MatcherLoc`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/mbe/macro_parser/enum.MatcherLoc.html

正規表現パーサーの類推では、トークンストリームは入力であり、
matcherで定義されたパターンと照合しています。例を使用すると、
トークンストリームは、例の呼び出し`print foo`の内部を含むトークンのストリームである可能性がありますが、
matcherは、トークン（ツリー）のシーケンス`print $mvar:ident`である可能性があります。

パーサーの出力は[`ParseResult`]であり、
次の3つのケースのいずれかが発生したことを示します。

- **Success**: トークンストリームは指定されたmatcherと一致し、
  メタ変数から対応するトークンツリーへのバインディングを生成しました。
- **Failure**: トークンストリームはmatcherと一致せず、
  「予期されるトークンのルールがありません...」などのエラーメッセージが表示されます。
- **Error**: _パーサー内_で致命的なエラーが発生しました。たとえば、
  これは、パターンマッチが複数ある場合に発生します。これは、
  マクロがあいまいであることを示しているためです。

完全なインターフェースは[ここ][code_parse_int]で定義されています。

マクロパーサーは、通常の正規表現パーサーとほぼ同じことを行います。
ただし、1つの例外があります。`ident`、`block`、`expr`などのさまざまなタイプのメタ変数を解析するために、
マクロパーサーは通常のRustパーサーにコールバックする必要があります。

マクロ定義を解析するコードは、[`compiler/rustc_expand/src/mbe/macro_rules.rs`][code_mr]にあります。
マクロパーサーの実装の詳細については、
[`compiler/rustc_expand/src/mbe/macro_parser.rs`][code_mp]のコメントを参照してください。

例を使用すると、呼び出しからのトークンストリーム`print foo`を、
マクロ定義のルールから以前に抽出したmatcher`print $mvar:ident`と`print twice $mvar:ident`と
照合しようとします。マクロパーサーが現在のmatcherで_非終端_（例：`$mvar:ident`）を
照合する必要がある場所に到達すると、通常のRustパーサーにコールバックして、
その非終端の内容を取得します。この場合、Rustパーサーは`ident`トークンを探し、
それを見つけ（`foo`）、マクロパーサーに返します。次に、マクロパーサーは解析を続行します。

呼び出しと一致するのは、さまざまなルールからのmatcherの1つだけであることに注意してください。
複数の一致がある場合、解析はあいまいですが、一致がまったくない場合は構文エラーです。

正確に1つのルールが一致すると仮定すると、マクロ展開は次に、ルールの右側を*書き写し*、
左側と照合するときにキャプチャした一致の値を置換します。

## 手続きマクロ

手続きマクロも解析中に展開されます。ただし、
コンパイラにパーサーがあるのではなく、procマクロはカスタムの
サードパーティクレートとして実装されています。コンパイラは、
procマクロクレートとその中の特別に注釈が付けられた関数（つまり、procマクロ自体）をコンパイルし、
トークンのストリームを渡します。次に、procマクロは、
トークンストリームを変換して新しいトークンストリームを出力できます。
これは、ASTに合成されます。

procマクロで使用されるトークンストリームタイプは_安定_しているため、`rustc`は
内部的には使用しません。コンパイラの（不安定な）トークンストリームは、
[`rustc_ast::tokenstream::TokenStream`][rustcts]で定義されています。これは、
[`rustc_expand::proc_macro`][pm]と[`rustc_expand::proc_macro_server`][pms]で、
安定版[`proc_macro::TokenStream`][stablets]に変換されてから戻されます。
Rust ABIは現在不安定なので、この変換にはC ABIを使用します。

[tsmod]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/tokenstream/index.html
[rustcts]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/tokenstream/struct.TokenStream.html
[stablets]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
[pm]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/proc_macro/index.html
[pms]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/proc_macro_server/index.html
[`ParseResult`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/mbe/macro_parser/enum.ParseResult.html

<!-- TODO(rylev): more here. [#1160](https://github.com/rust-lang/rustc-dev-guide/issues/1160) -->

### カスタム派生

カスタム派生は、特別なタイプのprocマクロです。

### Macros By ExampleとMacros 2.0

より多くのハイジーン関連機能、より良いスコープと可視性ルールなどを提供することにより、
MBEシステムを改善するためのレガシーでほとんど文書化されていない取り組みがあります。
内部的には、今日のMBEと同じ機構を使用し、
いくつかの追加の構文糖を使用し、名前空間に配置できます。

<!-- TODO(rylev): more? [#1160](https://github.com/rust-lang/rustc-dev-guide/issues/1160) -->
