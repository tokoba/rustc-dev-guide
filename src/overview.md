# コンパイラの概要

この章は、プログラムのコンパイルの全体的なプロセス、つまり
すべてがどのように組み合わさるかについてです。

Rustコンパイラは2つの点で特別です：他のコンパイラがしないことをコードに対して行い
（例：借用チェック）、多くの型破りな実装の選択をしています
（例：クエリ）。この章では、これらについて順番に説明します。
そして、ガイドの残りの部分では、個々の部分をより詳細に見ていきます。

## コンパイラがコードに対して行うこと

まず、コンパイラがコードに対して何を行うかを見てみましょう。今のところ、
これらのステップをコンパイラがどのように実装しているかについては、
必要な場合を除いて言及しません。

### 呼び出し

コンパイルは、ユーザーがRustソースプログラムをテキストで書き、
`rustc`コンパイラを呼び出すときに始まります。コンパイラが実行する必要がある作業は
コマンドラインオプションによって定義されます。例えば、nightly機能を有効にする
（`-Z`フラグ）、`check`のみのビルドを実行する、またはLLVM
中間表現（`LLVM-IR`）を実行可能な機械語の代わりに出力することが可能です。
`rustc`実行可能ファイルの呼び出しは、`cargo`の使用を通じて間接的に行われる場合があります。

コマンドライン引数の解析は[`rustc_driver`]で行われます。このクレートは、
ユーザーが要求するコンパイル設定を定義し、それを
[`rustc_interface::Config`]としてコンパイルプロセスの残りの部分に渡します。

### 字句解析と構文解析

生のRustソーステキストは、[`rustc_lexer`]にある低レベルの*字句解析器*によって分析されます。
この段階では、ソーステキストは*トークン*として知られる原子的なソースコード単位のストリームに変換されます。
`字句解析器`はUnicode文字エンコーディングをサポートしています。

トークンストリームは、コンパイルプロセスの次の段階の準備のために
[`rustc_parse`]にある高レベルの字句解析器を通過します。
この段階で[`Lexer`]構造体が使用され、一連の検証を実行し、
文字列をインターン化されたシンボルに変換します（*インターン化*については後で説明します）。
[文字列インターン化]は、各異なる文字列値の不変コピーを1つだけ格納する方法です。

字句解析器は小さなインターフェースを持ち、`rustc`の診断インフラストラクチャに
直接依存していません。代わりに、診断をプレーンデータとして提供し、
[`rustc_parse::lexer`]で実際の診断として発行されます。`字句解析器`は、
IDE や手続きマクロ（「proc-macros」と呼ばれることもあります）の両方のために
完全な忠実度情報を保持します。

*パーサー*は、[`字句解析器`からのトークンストリームを抽象構文木（AST）に変換します][parser]。
再帰降下（トップダウン）アプローチを使用して構文解析を行います。
`パーサー`のクレートエントリーポイントは、
[`rustc_parse::parser::Parser`]にある
[`Parser::parse_crate_mod()`][parse_crate_mod]と[`Parser::parse_mod()`][parse_mod]
メソッドです。外部モジュール解析のエントリーポイントは
[`rustc_expand::module::parse_external_mod`][parse_external_mod]です。
そして、マクロ`パーサー`のエントリーポイントは[`Parser::parse_nonterminal()`][parse_nonterminal]です。

構文解析は、[`bump`]、[`check`]、[`eat`]、[`expect`]、[`look_ahead`]を含む
[`parser`]ユーティリティメソッドのセットで実行されます。

構文解析は意味的な構成要素によって整理されています。
[`rustc_parse`][rustc_parse_parser_dir]ディレクトリには、
個別の`parse_*`メソッドがあります。ソースファイル名は構成要素名に従います。
例えば、`parser`には次のファイルがあります：

- [`expr.rs`](https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_parse/src/parser/expr.rs)
- [`pat.rs`](https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_parse/src/parser/pat.rs)
- [`ty.rs`](https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_parse/src/parser/ty.rs)
- [`stmt.rs`](https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_parse/src/parser/stmt.rs)

この命名スキームは、コンパイラの多くの段階で使用されています。
構文解析、lowering、型チェック、[型付き高レベル中間表現（`THIR`）][thir] lowering、
および[中レベル中間表現（`MIR`）][mir]構築のソース全体で、
同じ名前のファイルまたはディレクトリが見つかります。

マクロ展開、`AST`検証、名前解決、および早期リントも、
字句解析と構文解析の段階で行われます。

[`rustc_ast::ast`]::{[`Crate`], [`Expr`], [`Pat`], ...} `AST`ノードは
パーサーから返され、エラー処理には標準の[`Diag`] APIが使用されます。
一般的に、Rustのコンパイラはエラーから回復しようとし、
Rust文法のスーパーセットを解析しながら、エラー型も発行します。

### `AST` lowering

次に、`AST`は[高レベル中間表現（`HIR`）][hir]に変換されます。
これはコンパイラにとってよりフレンドリーな`AST`の表現です。
このプロセスは「lowering」と呼ばれ、ループや`async fn`などの
短縮または省略された構文構成要素の展開と形式化である多くの脱糖を伴います。

次に、`HIR`を使用して[*型推論*]（式の型の自動検出プロセス）、
[*トレイト解決*]（`trait`への参照と各implをペアリングするプロセス）、
および[*型チェック*]を行います。型チェックは、
`HIR`（[`hir::Ty`]）で見つかった型（ユーザーが書いたものを表す）を、
コンパイラが使用する内部表現（[`Ty<'tcx>`]）に変換するプロセスです。
情報は型の安全性、正確性、およびプログラムで使用される型の整合性を
検証するために使用されるため、型チェックと呼ばれます。

### `MIR` lowering

`HIR`はさらに`MIR`にloweringされます
（[借用チェック]に使用されます）。これは、`THIR`（パターンと網羅性チェックに使用される、
さらに脱糖された`HIR`）を構築して`MIR`に変換することによって行われます。

`MIR`は汎用であるため、[MIRで多くの最適化][mir-opt]を行います。これにより、
後のコード生成とコンパイル速度が向上します。`MIR`レベルでいくつかの
最適化を行う方が、`LLVM-IR`レベルで行うよりも簡単です。
例えば、LLVMは[`simplify_try`] `MIR`-optが探すパターンを
最適化できないようです。

Rustコードも、コード生成中に[_monomorphization_]されます。
これは、すべてのジェネリックコードのコピーを作成し、
型パラメータを具体的な型に置き換えることを意味します。
これを行うには、どの具体的な型のコードを生成するかのリストを収集する必要があります。
これは_monomorphizationコレクション_と呼ばれ、`MIR`レベルで行われます。

[_monomorphized_]: https://en.wikipedia.org/wiki/Monomorphization

### コード生成

次に、単に_コード生成_または_codegen_と呼ばれるものを開始します。
[コード生成段階][codegen]は、ソースの高レベル表現が
実行可能バイナリに変換されるときです。`rustc`はコード生成にLLVMを使用するため、
最初のステップは`MIR`を`LLVM-IR`に変換することです。
ここで`MIR`が実際にmonomorphizationされます。`LLVM-IR`はLLVMに渡され、
LLVMはそれに対して多くの最適化を行い、基本的にアセンブリコードに
追加の低レベル型と注釈を追加した機械語を発行します
（例：ELFオブジェクトまたは`WASM`）。次に、異なるライブラリ/バイナリが
リンクされて最終的なバイナリが生成されます。

[*trait solving*]: traits/resolution.md
[*type checking*]: type-checking.md
[*type inference*]: type-inference.md
[`bump`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/struct.Parser.html#method.bump
[`check`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/struct.Parser.html#method.check
[`Crate`]: https://doc.rust-lang.org/beta/nightly-rustc/rustc_ast/ast/struct.Crate.html
[`diag`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.Diag.html
[`eat`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/struct.Parser.html#method.eat
[`expect`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/struct.Parser.html#method.expect
[`Expr`]: https://doc.rust-lang.org/beta/nightly-rustc/rustc_ast/ast/struct.Expr.html
[`hir::Ty`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Ty.html
[`look_ahead`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/struct.Parser.html#method.look_ahead
[`Parser`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/struct.Parser.html
[`Pat`]: https://doc.rust-lang.org/beta/nightly-rustc/rustc_ast/ast/struct.Pat.html
[`rustc_ast::ast`]: https://doc.rust-lang.org/beta/nightly-rustc/rustc_ast/index.html
[`rustc_driver`]: rustc-driver/intro.md
[`rustc_interface::Config`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/interface/struct.Config.html
[`rustc_lexer`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lexer/index.html
[`rustc_parse::lexer`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/lexer/index.html
[`rustc_parse::parser::Parser`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/struct.Parser.html
[`rustc_parse`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/index.html
[`simplify_try`]: https://github.com/rust-lang/rust/pull/66282
[`Lexer`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/lexer/struct.Lexer.html
[`Ty<'tcx>`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html
[borrow checking]: borrow_check.md
[codegen]: backend/codegen.md
[hir]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/index.html
[lex]: the-parser.md
[mir-opt]: mir/optimizations.md
[mir]: mir/index.md
[parse_crate_mod]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/struct.Parser.html#method.parse_crate_mod
[parse_external_mod]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/module/fn.parse_external_mod.html
[parse_mod]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/struct.Parser.html#method.parse_mod
[parse_nonterminal]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/struct.Parser.html#method.parse_nonterminal
[parser]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/index.html
[rustc_parse_parser_dir]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_parse/src/parser
[String interning]: https://en.wikipedia.org/wiki/String_interning
[thir]: ./thir.md

## どのように行うか

ここまで、コンパイラがコードに対して何を行うかの高レベルビューを見てきました。
次に、それをすべてどのように行うかの高レベルビューを見てみましょう。
コンパイラが満たす/最適化する必要がある多くの制約と
競合する目標があります。例えば：

- コンパイル速度：プログラムのコンパイルにどのくらいの速さか？より多く/より良い
  コンパイル時解析は、多くの場合コンパイルが遅くなることを意味します。
  - また、インクリメンタルコンパイルをサポートしたいので、それを
    考慮に入れる必要があります。ユーザーがプログラムを変更した場合、
    やり直す必要がある作業と再利用できる作業を追跡するにはどうすればよいでしょうか？
    - また、インクリメンタルキャッシュにあまり多くのものを保存することはできません。
      ディスクからロードするのに長い時間がかかり、
      ユーザーのシステムで多くのスペースを取る可能性があるためです...
- コンパイラメモリ使用量：プログラムをコンパイルしている間、
  必要以上のメモリを使用したくありません。
- プログラム速度：コンパイルされたプログラムはどのくらい速いか？より多く/より良い
  コンパイル時解析は、多くの場合、コンパイラがより良い最適化を行えることを意味します。
- プログラムサイズ：コンパイルされたバイナリはどのくらい大きいか？前のポイントと同様です。
- コンパイラコンパイル速度：コンパイラのコンパイルにどのくらい時間がかかるか？
  これは貢献者とコンパイラのメンテナンスに影響します。
- 実装の複雑さ：コンパイラの構築は、人/グループができる最も難しいことの1つであり、
  Rustは非常に単純な言語ではないため、
  コンパイラのコードベースをどのように管理可能にするか？
- コンパイラの正確性：コンパイラが生成するバイナリは、
  入力プログラムが言うことを行うべきであり、
  絶えず進行中の膨大な変更にもかかわらず、それを続けるべきです。
- 統合：他の多くのツールがコンパイラをさまざまな方法で使用する必要があります
  （例：`cargo`、`clippy`、`MIRI`）。これはサポートする必要があります。
- コンパイラの安定性：コンパイラは、stableチャネルでクラッシュしたり、
  不適切に失敗したりしてはなりません。
- Rustの安定性：コンパイラは、実装に対して常に行われている多くの変更にもかかわらず、
  以前にコンパイルされたプログラムを壊さないことによって、
  Rustの安定性保証を尊重する必要があります。
- 他のツールの制限：`rustc`はバックエンドでLLVMを使用し、LLVMには
  活用するいくつかの強みと、回避する必要があるいくつかの側面があります。

したがって、ガイドの残りの部分を続けるときは、これらのことを念頭に置いてください。
これらは、多くの場合、私たちが行う決定に情報を与えます。

### 中間表現

ほとんどのコンパイラと同様に、`rustc`は計算を容易にするために
いくつかの中間表現（IR）を使用します。一般的に、ソースコードで直接作業することは
非常に不便でエラーが発生しやすいです。ソースコードは人間にとってフレンドリーである
ように設計されていますが、同時に曖昧性がないようにもなっていますが、
型チェックのようなことをするにはあまり便利ではありません。

代わりに、`rustc`を含むほとんどのコンパイラは、
分析しやすいソースコードからある種のIRを構築します。
`rustc`にはいくつかのIRがあり、それぞれ異なる目的に最適化されています：

- トークンストリーム：字句解析器は、ソースコードから直接トークンのストリームを生成します。
  このトークンストリームは、パーサーが生のテキストよりも扱いやすいです。
- 抽象構文木（`AST`）：抽象構文木は、字句解析器によって生成された
  トークンストリームから構築されます。これは、
  ユーザーが書いたものをほぼ正確に表します。構文の健全性チェック
  （例：ユーザーが型を書いた場所に型が期待されていることを確認する）を
  行うのに役立ちます。
- 高レベルIR（HIR）：これは、脱糖された`AST`の一種です。まだユーザーが
  構文的に書いたものに近いですが、いくつかの暗黙的なものを含みます。
  例えば、いくつかの省略されたライフタイムなど。このIRは型チェックに適しています。
- 型付き`HIR`（THIR）_以前は高レベル抽象IR（HAIR）_：これは
  `HIR`とMIRの間の中間です。`HIR`に似ていますが、完全に型付けされており、
  もう少し脱糖されています（例：メソッド呼び出しと暗黙的な逆参照は
  完全に明示的になります）。その結果、`HIR`からよりも`THIR`から`MIR`に
  loweringする方が簡単です。
- 中レベルIR（`MIR`）：このIRは基本的にコントロールフローグラフ（CFG）です。CFGは、
  プログラムの基本ブロックと、それらの間でコントロールフローがどのように
  移動できるかを示す図の一種です。同様に、`MIR`には、
  その中に単純な型付きステートメント（例：割り当て、単純な計算など）と、
  他の基本ブロックへのコントロールフローエッジ（例：呼び出し、値の削除など）を
  持つ基本ブロックの束があります。`MIR`は、借用チェックと
  他の重要なデータフローベースのチェック（例：未初期化値のチェック）に使用されます。
  また、一連の最適化と定数評価（`MIRI`を介して）にも使用されます。
  `MIR`はまだ汎用であるため、monomorphization後よりも
  効率的に多くの解析をここで行うことができます。
- `LLVM-IR`：これは、LLVMコンパイラへのすべての入力の標準形式です。`LLVM-IR`は、
  多くの注釈を持つ型付きアセンブリ言語の一種です。
  LLVMを使用するすべてのコンパイラ（例：clang Cコンパイラも`LLVM-IR`を出力します）が
  使用する標準形式です。`LLVM-IR`は、他のコンパイラが発行しやすく、
  LLVMがその上で多くの最適化を実行するのに十分なリッチさを持つように設計されています。

もう1つ注意すべきことは、コンパイラ内の多くの値が_interned_されていることです。
これは、値を_[arena]_と呼ばれる特別なアロケータに割り当てる
パフォーマンスとメモリの最適化です。次に、arenaに割り当てられた値への
参照を渡します。これにより、同一の値（例：プログラム内の型）が
一度だけ割り当てられ、ポインタを比較することで安価に比較できるようになります。
多くの中間表現がインターン化されています。

[arena]: https://en.wikipedia.org/wiki/Region-based_memory_management

### クエリ

最初の大きな実装の選択は、コンパイラでの_クエリ_システムの使用です。
Rustコンパイラは、順次実行されるコードに対する一連のパスとして
整理されて_いません_。Rustコンパイラはこれを行って、
インクリメンタルコンパイルを可能にします。つまり、ユーザーがプログラムに
変更を加えて再コンパイルする場合、新しいバイナリを出力するために
可能な限り冗長な作業を少なくしたいのです。

`rustc`では、上記のすべての主要なステップは、互いに呼び出す一連のクエリとして
整理されています。例えば、何かの型を尋ねるクエリと、
関数の最適化された`MIR`を尋ねる別のクエリがあります。
これらのクエリは互いに呼び出すことができ、すべてがクエリシステムを通じて
追跡されます。クエリの結果はディスクにキャッシュされるため、
コンパイラは最後のコンパイルからどのクエリの結果が変更されたかを判断し、
それらのみをやり直すことができます。これがインクリメンタルコンパイルの
仕組みです。

原則として、クエリ化されたステップでは、各アイテムに対して上記の各作業を行います。
例えば、関数の`HIR`を取得し、クエリを使用してその`HIR`の`LLVM-IR`を要求します。
これにより、最適化された`MIR`の生成が駆動され、それが借用チェッカーを駆動し、
それが`MIR`の生成を駆動します。

...ただし、これは非常に単純化しすぎています。実際には、一部のクエリは
ディスクにキャッシュされず、コンパイラの一部の部分は、デッドコードであっても
正確性のためにすべてのコードに対して実行する必要があります（例：借用チェッカー）。
例えば、[現在、`mir_borrowck`クエリは最初にクレートのすべての関数で実行されます][passes]。
次に、codegenバックエンドが`collect_and_partition_mono_items`クエリを呼び出します。
このクエリは、すべての到達可能な関数に対して`optimized_mir`を再帰的に要求し、
その関数に対して`mir_borrowck`を実行してからcodegenユニットを作成します。
この種の分割は、到達不可能な関数がまだエラーを発行するようにするために
残る必要があります。

[passes]: https://github.com/rust-lang/rust/blob/e69c7306e2be08939d95f14229e3f96566fb206c/compiler/rustc_interface/src/passes.rs#L791

さらに、コンパイラはもともとクエリシステムを使用するように構築されていませんでした。
クエリシステムはコンパイラに後付けされたため、
その部分の一部はまだクエリ化されていません。また、LLVMは私たちのコードではないため、
それもクエリ化されていません。計画は、最終的に
前のセクションでリストされているすべてのステップをクエリ化することですが、
<!-- date-check --> 2022年11月現在、`HIR`と`LLVM-IR`の間のステップのみが
クエリ化されています。つまり、字句解析、構文解析、名前解決、および
マクロ展開は、プログラム全体に対して一度に行われます。

ここでもう1つ言及すべきことは、非常に重要な「型付けコンテキスト」、
[`TyCtxt`]です。これは、すべてのものの中心にある巨大な構造体です。
（名前はほとんど歴史的なものです。これは型理論の`Γ`や`Δ`の意味での
「型付けコンテキスト」では_ありません_。ソースコードの構造体の名前がそうであるため、
名前は保持されています。）すべてのクエリは[`TyCtxt`]型のメソッドとして定義されており、
メモリ内クエリキャッシュもそこに格納されています。コードでは、通常、
型付けコンテキストのハンドルである`tcx`という変数があります。
また、`'tcx`という名前のライフタイムも見られます。これは、何かが
[`TyCtxt`]のライフタイムに結び付けられていることを意味します
（通常はそこに格納またはインターン化されています）。

[`TyCtxt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html

コンパイラのクエリの詳細については、[クエリの章][queries]を参照してください。

[queries]: ./query.md

### `ty::Ty`

型はRustで非常に重要であり、多くのコンパイラ解析の中核を形成しています。
（ユーザーのプログラムの）型を表す（コンパイラ内の）主要な型は
[`rustc_middle::ty::Ty`][ty]です。これは非常に重要なので、
[`ty::Ty`][ty]に関する章全体がありますが、今のところ、
それが存在し、`rustc`が型を表す方法であることだけを言及したいと思います！

また、[`rustc_middle::ty`]モジュールが前に述べた[`TyCtxt`]構造体を定義していることに
注意してください。

[ty]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html
[`rustc_middle::ty`]: https://doc.rust-lang.org/beta/nightly-rustc/rustc_middle/ty/index.html

### 並列処理

コンパイラのパフォーマンスは、改善したい問題です
（そして常に取り組んでいます）。その1つの側面は、
`rustc`自体を並列化することです。

現在、rustcでデフォルトで並列になっているのは1つの部分だけです：
[コード生成](./parallel-rustc.md#Codegen)。

ただし、コンパイラの残りの部分はまだ並列ではありません。
これには多くの努力が費やされてきましたが、一般的に難しい問題です。
現在のアプローチは、[`RefCell`]を[`Mutex`]に変える、つまり、
スレッドセーフな内部可変性に切り替えることです。ただし、
ロックの競合、並行性下でのクエリシステムの不変性の維持、
およびコードベースの複雑さには継続的な課題があります。
`bootstrap.toml`で並列コンパイルを有効にすることで、
現在の作業を試すことができます。まだ初期段階ですが、
すでにいくつかの有望なパフォーマンス改善があります。

[`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
[`Mutex`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html

### ブートストラップ

`rustc`自体はRustで書かれています。では、コンパイラをどのようにコンパイルするのでしょうか？
古いコンパイラを使用して新しいコンパイラをコンパイルします。
これは[_bootstrapping_]と呼ばれます。

ブートストラップには多くの興味深い意味があります。例えば、
Rustの主要なユーザーの1つがRustコンパイラ自体であることを意味するため、
常に自分たちのソフトウェアをテストしています（「自分たちのドッグフードを食べる」）。

ブートストラップの詳細については、
[ガイドのブートストラップセクション][rustc-bootstrap]を参照してください。

[_bootstrapping_]: https://en.wikipedia.org/wiki/Bootstrapping_(compilers)
[rustc-bootstrap]: building/bootstrapping/intro.md

<!--
# 未解決の質問

- LLVMはデバッグビルドで最適化を行うことがありますか？
- コンパイルプロセスの各フェーズを自分のソースで探索するにはどうすればよいですか
  （字句解析器、パーサー、HIRなど）？- 例えば、`cargo rustc -- -Z unpretty=hir-tree`を使用すると、
  `HIR`表現を表示できます
- `X`の主要なソースエントリーポイントは何ですか？
- フェーズは、異なるプラットフォーム間で機械語へのクロスコンパイルに対してどこで分岐しますか？
-->

# 参考文献

- コマンドライン解析
  - ガイド：[Rustcドライバーとインターフェース](rustc-driver/intro.md)
  - ドライバー定義：[`rustc_driver`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver/)
  - メインエントリーポイント：[`rustc_session::config::build_session_options`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/config/fn.build_session_options.html)
- 字句解析：ユーザープログラムをトークンのストリームにレックスする
  - ガイド：[字句解析と構文解析](the-parser.md)
  - 字句解析器定義：[`rustc_lexer`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lexer/index.html)
  - メインエントリーポイント：[`rustc_lexer::cursor::Cursor::advance_token`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lexer/cursor/struct.Cursor.html#method.advance_token)
- 構文解析：トークンのストリームを抽象構文木（AST）に解析する
  - ガイド：[字句解析と構文解析](the-parser.md)
  - ガイド：[マクロ展開](macro-expansion.md)
  - ガイド：[名前解決](name-resolution.md)
  - パーサー定義：[`rustc_parse`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/index.html)
  - メインエントリーポイント：
    - [クレート内の最初のファイルのエントリーポイント](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/passes/fn.parse.html)
    - [アウトラインモジュール解析のエントリーポイント](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_expand/module/fn.parse_external_mod.html)
    - [マクロフラグメントのエントリーポイント][parse_nonterminal]
  - `AST`定義：[`rustc_ast`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/ast/index.html)
  - 機能ゲート：**TODO**
  - 早期リント：**TODO**
- 高レベル中間表現（HIR）
  - ガイド：[HIR](hir.md)
  - ガイド：[HIRの識別子](hir.md#identifiers-in-the-hir)
  - ガイド：[`HIR`マップ](hir.md#the-hir-map)
  - ガイド：[`AST`から`HIR`へのlowering](./hir/lowering.md)
  - コードの`HIR`表現を表示する方法`cargo rustc -- -Z unpretty=hir-tree`
  - Rustc `HIR`定義：[`rustc_hir`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/index.html)
  - メインエントリーポイント：**TODO**
  - 後期リント：**TODO**
- 型推論
  - ガイド：[型推論](type-inference.md)
  - ガイド：[tyモジュール：型の表現](ty.md)（意味論）
  - メインエントリーポイント（型推論）：[`InferCtxtBuilder::enter`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_infer/infer/struct.InferCtxtBuilder.html#method.enter)
  - メインエントリーポイント（本体の型チェック）：[`typeck`クエリ](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.typeck)
    - これら2つの関数は分離できません。
- 中レベル中間表現（MIR）
  - ガイド：[`MIR`（中レベルIR）](mir/index.md)
  - 定義：[`rustc_middle/src/mir`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/index.html)
  - MIRを操作するソースの定義：[`rustc_mir_build`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_build/index.html)、[`rustc_mir_dataflow`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/index.html)、[`rustc_mir_transform`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_transform/index.html)
- 借用チェッカー
  - ガイド：[MIR借用チェック](borrow_check.md)
  - 定義：[`rustc_borrowck`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/index.html)
  - メインエントリーポイント：[`mir_borrowck`クエリ](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/fn.mir_borrowck.html)
- `MIR`最適化
  - ガイド：[MIR最適化](mir/optimizations.md)
  - 定義：[`rustc_mir_transform`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_transform/index.html)
  - メインエントリーポイント：[`optimized_mir`クエリ](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_transform/fn.optimized_mir.html)
- コード生成
  - ガイド：[コード生成](backend/codegen.md)
  - LLVMで`LLVM-IR`から機械語を生成する - **TODO: 参照？**
  - メインエントリーポイント：[`rustc_codegen_ssa::base::codegen_crate`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/base/fn.codegen_crate.html)
    - これはmonomorphizationを行い、1つのcodegenユニットの`LLVM-IR`を生成します。
      次に、LLVMを実行するバックグラウンドスレッドを開始します。
      これは後で結合する必要があります。
    - Monomorphizationは[`FunctionCx::monomorphize`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/mir/struct.FunctionCx.html#method.monomorphize)と[`rustc_codegen_ssa::base::codegen_instance`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/base/fn.codegen_instance.html)を介して遅延的に発生します
