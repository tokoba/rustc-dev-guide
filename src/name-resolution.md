# 名前解決

前の章では、すべてのマクロが展開された[*抽象構文木*（`AST`）][ast]がどのように構築されるかを見ました。
それを行うには、インポートとマクロ名を解決するためにいくつかの名前解決を行う必要があることを見ました。
この章では、これが実際にどのように行われ、さらに多くのことを示します。

[ast]: ./ast-validation.md

実際には、マクロ展開中に完全な名前解決は行いません。その時点では、
インポートとマクロのみを解決します。これは、何を展開するかを知るために必要です。
後で、AST全体ができた後、クレート内のすべての名前を解決するために完全な名前解決を行います。
これは、[`rustc_resolve::late`][late]で行われます。マクロ展開中とは異なり、
この遅延展開では、新しい名前を追加できないため、名前を解決しようとするのは1回だけで済みます。
名前の解決に失敗した場合、それはコンパイラエラーです。

名前解決は複雑です。異なる名前空間（例：マクロ、値、型、ライフタイム）があり、
名前はさまざまな（ネストされた）スコープで有効な場合があります。また、
異なるタイプの名前は異なる方法で解決に失敗する可能性があり、
失敗は異なるスコープで異なる方法で発生する可能性があります。たとえば、
モジュールスコープでは、失敗は、そのモジュールに展開されていないマクロがなく、
未解決のglobインポートがないことを意味します。一方、関数本体スコープでは、
失敗には、私たちがいるブロック、すべての外部スコープ、およびグローバルスコープから
名前が存在しないことが必要です。

[late]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/late/index.html

## 基本

プログラムでは、変数、型、関数などを名前を付けて参照します。これらの名前は常に
一意であるとは限りません。たとえば、次の有効なRustプログラムを見てください。

```rust
type x = u32;
let x: x = 1;
let y: x = 2;
```

3行目の`x`が型（`u32`）か値（1）かをどのように知るのでしょうか？これらの
競合は名前解決中に解決されます。この特定のケースでは、名前解決は、
型名と変数名が別々の名前空間に住んでいるため、共存できると定義します。

Rustの名前解決は2段階のプロセスです。最初のフェーズは、
`macro`展開中に実行され、モジュールのツリーを構築し、インポートを解決します。
マクロ展開と名前解決は、[`ResolverAstLoweringExt`]トレイトを介して互いに通信します。

2番目のフェーズへの入力は、入力ファイルを解析してマクロを展開することによって生成された
構文木です。このフェーズは、ソース内のすべての名前から、
名前が導入された関連する場所へのリンクを生成します。また、
typoの提案、インポートするトレイト、または未使用のアイテムに関するlintなど、
役立つエラーメッセージも生成します。

2番目のフェーズ（[`Resolver::resolve_crate`]）の実行が成功すると、
コンパイルの残りが存在する名前（`hir::lowering::Resolver`インターフェース経由）について
尋ねるために使用できるようなインデックスが作成されます。

名前解決は[`rustc_resolve`]クレートに存在し、主要部分は
`lib.rs`にあり、いくつかのヘルパーまたはシンボルタイプ固有のロジックが他のモジュールにあります。

[`Resolver::resolve_crate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/struct.Resolver.html#method.resolve_crate
[`ResolverAstLoweringExt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast_lowering/trait.ResolverAstLoweringExt.html
[`rustc_resolve`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/index.html

## 名前空間

さまざまな種類のシンボルは、さまざまな名前空間に住んでいます。たとえば、型は変数と
衝突しません。これは通常発生しません。変数は小文字で始まり、型は大文字で始まるためですが、
これは単なる慣例です。これは、警告付きでコンパイルされる有効なRustコードです。

```rust
type x = u32;
let x: x = 1;
let y: x = 2; // See? x is still a type here.
```

これに対処するために、リゾルバーはそれらを分離して保持し、
それらに対して別々の構造を構築します。

つまり、コードが名前空間について話すとき、それはモジュール階層を意味するのではなく、
型対値対マクロを意味します。

## スコープとリブ

名前は、ソースコードの特定の領域でのみ表示されます。これは階層構造を形成しますが、
必ずしも単純なものではありません。あるスコープが別のスコープの一部である場合、
外部スコープに表示される名前が内部スコープにも表示されるとは限らず、
同じものを指すとも限りません。

それに対処するために、コンパイラは[`Rib`]sの概念を導入します。これは、
スコープの抽象化です。表示される名前のセットが潜在的に変更されるたびに、
新しい[`Rib`]がスタックにプッシュされます。これが発生する可能性のある場所には、
たとえば次のものがあります。

[`Rib`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/late/struct.Rib.html

* ブロックを囲む中括弧、関数境界、モジュールなどの明白な場所。
* `let`バインディングの導入 – これは、同じ名前の別のバインディングをシャドウできます。
* マクロ展開境界 – マクロハイジーンに対処するため。

名前を検索するとき、[`リブ`]のスタックは最も内側から外側に向かって走査されます。
これは、名前の最も近い意味（他のものによってシャドウされていないもの）を見つけるのに役立ちます。
外側の[`Rib`]への遷移は、使用可能な名前にも影響を与える可能性があります。
ネストされた関数（クロージャではない）がある場合、
内側の関数は、通常のスコープルールでは表示されるべきであっても、
外側の関数のパラメータとローカルバインディングにアクセスできません。例：

[`ribs`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/late/struct.LateResolutionVisitor.html#structfield.ribs

```rust
fn do_something<T: Default>(val: T) { // <- New rib in both types and values (1)
    // `val` is accessible, as is the helper function
    // `T` is accessible
   let helper = || { // New rib on the block (2)
        // `val` is accessible here
    }; // End of (2), new rib on `helper` (3)
    // `val` is accessible, `helper` variable shadows `helper` function
    fn helper() { // <- New rib in both types and values (4)
        // `val` is not accessible here, (4) is not transparent for locals
        // `T` is not accessible here
    } // End of (4)
    let val = T::default(); // New rib (5)
    // `val` is the variable, not the parameter here
} // End of (5), (3) and (1)
```

ルールは名前空間によって少し異なるため、各名前空間には、
他のものと並行して構築される独自の独立した[`Rib`]スタックがあります。
さらに、ローカルラベル（ループまたはブロックの名前など）の[`Rib`]スタックもあり、
これ自体は完全な名前空間ではありません。

## 全体的な戦略

クレート全体の名前解決を実行するために、構文ツリーがトップダウンで走査され、
遭遇したすべての名前が解決されます。これは、ほとんどの種類の名前で機能します。
名前を使用する時点で、名前はすでに[`Rib`]階層に導入されているためです。

これにはいくつかの例外があります。アイテムは使用する前に遭遇する必要がないため、
少し厄介です。したがって、すべてのブロックは、
その[`Rib`]を埋めるために、最初にアイテムをスキャンする必要があります。

さらに問題のあるものとして、再帰的な固定点解決を必要とするインポートと、
コードの残りを処理する前に解決して展開する必要があるマクロがあります。

したがって、解決は複数の段階で実行されます。

## 投機的なクレートロード

便利なエラーを提供するために、rustcは、まだロードされていない場合でも、
スコープにインポートするパスを探すために、すべてのクレートのすべてのモジュールを調べて、
可能な一致を探します。これには、まだロードされていないクレートさえ含まれます！

エラーに遭遇したときに、まだロードされていないクレートをイーガーロードしてインポート提案を含めることは、
_投機的なクレートロード_と呼ばれます。エラーに遭遇したのは[`rustc_resolve`]であり、
ユーザーではないため、遭遇したエラーは報告されるべきではありません。
これを行う関数は[`lookup_import_candidates`]であり、
[`rustc_resolve::diagnostics`]に存在します。

[`rustc_resolve`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/index.html
[`lookup_import_candidates`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/struct.Resolver.html#method.lookup_import_candidates
[`rustc_resolve::diagnostics`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/diagnostics/index.html

投機的なロードとユーザーが開始したロードを区別するために、
[`rustc_resolve`]は`record_used`パラメータを渡します。
これは、ロードが投機的である場合は`false`です。

## TODO: [#16](https://github.com/rust-lang/rustc-dev-guide/issues/16)

これは、コードを学習する最初のパスの結果です。確かに
不完全で、十分に詳細ではありません。場所によっては不正確な可能性もあります。
それでも、そこで何が起こっているかについての有用な最初の道しるべを提供しているかもしれません。

* 正確に何にリンクし、それがどのように公開され、
  コンパイルの次の段階でどのように消費されるか？
* 誰がそれを呼び出し、実際にどのように使用されるか。
* それはパスであり、その後結果のみが使用されるのか、
  それとも段階的に計算できるのか？
* 全体的な戦略の説明は少し曖昧です。
* `Rib`という名前はどこから来たのか？
* これには独自のテストがあるのか、それともe2eテストの一部としてのみテストされるのか？
