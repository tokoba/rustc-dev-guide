# 用語集

用語                                                  | 意味
------------------------------------------------------|--------
<span id="arena">arena、アリーナアロケーション</span> |  _アリーナ_は、他のメモリ割り当てが行われる大きなメモリバッファです。このスタイルの割り当ては_アリーナアロケーション_と呼ばれます。詳細は[この章](../memory.md)を参照してください。
<span id="ast">AST</span>                      |  `rustc_ast`クレートによって生成される_抽象構文木_。ユーザー構文を非常に忠実に反映します。
<span id="apit">APIT</span>                    |  引数位置の`impl Trait`。匿名型パラメータとしても知られています。（[リファレンスを参照](https://doc.rust-lang.org/reference/types/impl-trait.html#anonymous-type-parameters)）。
<span id="binder">binder</span>                |  _バインダー_は、変数や型が宣言される場所です。例えば、`<T>`は`fn foo<T>(..)`のジェネリック型パラメータ`T`のバインダーであり、\|`a`\|` ...`はパラメータ`a`のバインダーです。詳細は[背景の章を参照](./background.md#free-vs-bound)してください。
<span id="body-id">`BodyId`</span>             |  クレート内の特定の本体（関数や定数の定義）を参照する識別子。詳細は[HIRの章を参照](../hir.md#identifiers-in-the-hir)してください。
<span id="bound-var">束縛変数</span>     |  _束縛変数_は式/項内で宣言されるものです。例えば、変数`a`はクロージャ式\|`a`\|` a * 2`内で束縛されています。詳細は[背景の章を参照](./background.md#free-vs-bound)してください。
<span id="codegen">codegen</span>              |  _コード生成_の略。MIRをLLVM IRに変換するコード。
<span id="codegen-unit">codegen unit</span>    |  LLVM IRを生成する際、Rustコードを複数のコードジェネレーションユニット（CGUと略されることもある）にグループ化します。これらの各ユニットはLLVMによって互いに独立して処理され、並列処理が可能になります。これらはインクリメンタル再利用の単位でもあります。（[詳細を見る](../backend/codegen.md)）
<span id="completeness">完全性</span>    |  型理論の技術用語で、すべての型安全なプログラムが型チェックも通過することを意味します。健全性と完全性の両方を持つことは非常に困難で、通常は健全性がより重要です。（「健全性」を参照）。
<span id="cfg">制御フローグラフ</span>       |  プログラムの制御フローの表現。詳細は[背景の章を参照](./background.md#cfg)してください。
<span id="ctfe">CTFE</span>                    |  _コンパイル時関数評価_の略で、コンパイラがコンパイル時に`const fn`を評価する能力です。これはコンパイラの定数評価システムの一部です。（[詳細を見る](../const-eval.md)）
<span id="cx">`cx`</span>                      |  _コンテキスト_の略語として_cx_を使用する傾向があります。`tcx`、`infcx`なども参照してください。
<span id="ctxt">`ctxt`</span>                  |  _コンテキスト_の略語として_ctxt_も使用します。例：[`TyCtxt`](#TyCtxt)。[cx](#cx)または[tcx](#tcx)も参照してください。
<span id="dag">DAG</span>                      |  _有向非巡回グラフ_は、コンパイル中にクエリ間の依存関係を追跡するために使用されます。（[詳細を見る](../queries/incremental-compilation.md)）
<span id="data-flow">データフロー解析</span> |  プログラムの制御フローの各ポイントでどのプロパティが真であるかを把握する静的解析。詳細は[背景の章を参照](./background.md#dataflow)してください。
<span id="debruijn">de Bruijnインデックス</span>     |  整数のみを使用して、どのバインダーに変数がバインドされているかを記述する技法。変数の名前変更に対して不変であるという利点があります。（[詳細を見る](./background.md#what-is-a-debruijn-index)）
<span id="def-id">`DefId`</span>               |  定義を識別するインデックス（`rustc_middle/src/hir/def_id.rs`を参照）。`DefPath`を一意に識別します。詳細は[HIRの章を参照](../hir.md#identifiers-in-the-hir)してください。
<span id="discriminant">判別子</span>    |  列挙型のバリアントまたはジェネレータの状態と関連付けられた基礎値で、それを「アクティブ」として示すためのもの（ただし["バリアントインデックス"](#variant-idx)と混同しないでください）。実行時に、アクティブなバリアントの判別子は[タグ](#tag)にエンコードされます。
<span id="double-ptr">二重ポインタ</span>    |  追加のメタデータを持つポインタ。詳細は[ファットポインタ](#fat-ptr)を参照してください。
<span id="drop-glue">drop glue</span>          |  データ型のデストラクタ（`Drop`）の呼び出しを処理する（内部）コンパイラ生成命令。
<span id="dst">DST</span>                      |  *動的サイズ型*の略で、コンパイラがメモリ内のサイズを静的に知ることができない型です（例：`str`や`[u8]`）。このような型は`Sized`を実装せず、スタックに割り当てることができません。構造体の最後のフィールドとしてのみ発生できます。ポインタの背後でのみ使用できます（例：`&str`や`&[u8]`）。
<span id="ebl">早期束縛ライフタイム</span>     |  定義サイトで置換されるライフタイム領域。アイテムの`Generics`にバインドされ、`GenericArgs`を使用して置換/インスタンス化されます。**遅延束縛ライフタイム**と対比してください。（[詳細を見る](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/region_kind/enum.RegionKind.html#bound-regions)）
<span id="effect">effects</span>               |  現在のところ、constトレイトと`~const`境界のみを意味します。（[詳細を見る](../effects.md)）
<span id="empty-type">空の型</span>        |  [居住不可能型](#ut)を参照してください。
<span id="fat-ptr">ファットポインタ</span>          |  ある値のアドレスと、その値を使用するために必要なさらなる情報を運ぶ2ワード値。Rustには2種類の_ファットポインタ_があります：スライスへの参照とトレイトオブジェクトです。スライスへの参照はスライスの開始アドレスとその長さを運びます。トレイトオブジェクトは値のアドレスとその値に適したトレイトの実装へのポインタを運びます。「ファットポインタ」は「ワイドポインタ」、「二重ポインタ」としても知られています。
<span id="free-var">自由変数</span>       |  _自由変数_は、式や項内でバインドされていないものです。詳細は[背景の章を参照](./background.md#free-vs-bound)してください。
<span id="generics">generics</span>            |  アイテムに定義されたジェネリックパラメータのリスト。ジェネリックパラメータには3種類あります：型、ライフタイム、const パラメータです。
<span id="hir">HIR</span>                      |  ASTを低レベル化および脱糖して作成された_高レベル[IR](#ir)_。（[詳細を見る](../hir.md)）
<span id="hir-id">`HirId`</span>               |  def-idと「定義内オフセット」を組み合わせてHIR内の特定のノードを識別します。詳細は[HIRの章を参照](../hir.md#identifiers-in-the-hir)してください。
<span id="ice">ICE</span>                      |  _内部コンパイラエラー_の略で、コンパイラがクラッシュするときのことです。
<span id="ich">ICH</span>                      |  _インクリメンタルコンパイルハッシュ_の略で、HIRやクレートメタデータなどのフィンガープリントとして使用され、変更が行われたかどうかを確認します。これは、クレートの一部が変更され、再コンパイルすべきかどうかを確認するインクリメンタルコンパイルで有用です。
<span id="infcx">`infcx`</span>                |  型推論コンテキスト（`InferCtxt`）。（`rustc_middle::infer`を参照）
<span id="inf-var">推論変数、infer var </span> |  型、領域、const推論を行う際、_推論変数_は推論しようとしているものを表す特別な種類の型/領域です。代数のXのようなものと考えてください。例えば、プログラム内の変数の型を推論しようとする場合、その未知の型を表す推論変数を作成します。
<span id="intern">intern</span>                |  インターニングとは、文字列などの頻繁に使用される定数データを格納し、メモリ使用量と割り当て回数を減らすために、データ自体ではなく識別子（例：`Symbol`）によってデータを参照することを指します。詳細は[この章](../memory.md)を参照してください。
<span id="interpreter">インタープリタ</span>      |  const評価の中核で、コンパイル時にMIRコードを実行します。（[詳細を見る](../const-eval/interpret.md)）
<span id="intrinsic">組み込み関数</span>          |  組み込み関数は、コンパイラ自体に実装されているが、ユーザーに公開されている（多くの場合不安定に）特別な関数です。魔法的で危険なことを行います。（[`std::intrinsics`](https://doc.rust-lang.org/std/intrinsics/index.html)を参照）
<span id="ir">IR</span>                        |  _中間表現_の略で、コンパイラにおける一般的な用語です。コンパイル中、コードは生のソース（ASCIIテキスト）からさまざまなIRに変換されます。Rustでは、これらは主にHIR、MIR、LLVM IRです。各IRは、いくつかの計算セットに適しています。例えば、MIRは借用チェッカーに適しており、LLVM IRはコードジェネレーションに適しています（LLVMがそれを受け入れるため）。
<span id="irlo">IRLO、irlo</span>              |  [internals.rust-lang.org](https://internals.rust-lang.org)の略語として使用されることがあります。
<span id="item">item</span>                    |  static、const、use文、モジュール、構造体など、言語における「定義」の一種。具体的には、これは`Item`型に対応します。
<span id="lang-item">lang item</span>          |  `Sync`や`Send`などの特別な組み込みトレイトや、`Add`などの操作を表すトレイト、またはコンパイラによって呼び出される関数など、言語自体に固有の概念を表すアイテム。（[詳細を見る](https://doc.rust-lang.org/1.9.0/book/lang-items.html)）
<span id="lbl">遅延束縛ライフタイム</span>      |  呼び出しサイトで置換されるライフタイム領域。HRTBにバインドされ、`liberate_late_bound_regions`などのコンパイラの特定の関数によって置換されます。**早期束縛ライフタイム**と対比してください。（[詳細を見る](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_type_ir/region_kind/enum.RegionKind.html#bound-regions)）
<span id="local-crate">ローカルクレート</span>      |  現在コンパイル中のクレート。これは、ローカルクレートの依存関係を指す「アップストリームクレート」とは対照的です。
<span id="lto">LTO</span>                      |  *リンク時最適化*の略で、これは最終バイナリがリンクされる直前にLLVMによって提供される一連の最適化です。例えば、最終プログラムで使用されない関数を削除するなどの最適化が含まれます。_ThinLTO_は、少し拡張性と効率性を目指したLTOの変種ですが、いくつかの最適化を犠牲にする可能性があります。Rustリポジトリの問題で「FatLTO」についても読むことができます。これは非Thin LTOに与えられた愛称です。LLVMドキュメント：[こちら][lto]と[こちら][thinlto]。
<span id="llvm">[LLVM]</span>                  |  （実際には頭字語ではありません :P）オープンソースのコンパイラバックエンド。LLVM IRを受け入れ、ネイティブバイナリを出力します。その後、さまざまな言語（例：Rust）がLLVM IRを出力するコンパイラフロントエンドを実装し、LLVMを使用してLLVMがサポートするすべてのプラットフォームにコンパイルできます。
<span id="memoization">メモ化</span>      |  （純粋な）計算（純粋な関数呼び出しなど）の結果を保存して、将来それらを繰り返す必要を避けるプロセス。これは通常、実行速度とメモリ使用量のトレードオフです。
<span id="mir">MIR</span>                      |  借用チェッカーとコードジェネレーションで使用するために型チェック後に作成される_中レベル[IR](#ir)_。（[詳細を見る](../mir/index.md)）
<span id="miri">Miri</span>                    |  （アンセーフな）Rustコード内の未定義動作を検出するツール。（[詳細を見る](https://github.com/rust-lang/miri)）
<span id="mono">単相化</span>        |  型と関数のジェネリック実装を取得し、具体的な型でインスタンス化するプロセス。例えば、コードには`Vec<T>`があるかもしれませんが、最終実行可能ファイルでは、プログラムで使用されるすべての具体的な型に対して`Vec`コードのコピーを持ちます（例：`Vec<usize>`のコピー、`Vec<MyStruct>`のコピーなど）。
<span id="normalize">正規化</span>          |  より正規形式に変換する一般的な用語ですが、rustcの場合、通常は[関連型の正規化](../traits/goals-and-clauses.md#normalizeprojection---type)を指します。
<span id="newtype">newtype</span>              |  他の型のラッパー（例：`struct Foo(T)`は`T`の「newtype」です）。これはRustで、インデックスにより強い型を与えるために一般的に使用されます。
<span id="niche">ニッチ</span>                  |  _レイアウト最適化に使用できる_型の無効なビットパターン。一部の型は特定のビットパターンを持つことができません。例えば、`NonZero*`整数または参照`&T`は0のビット文字列で表現できません。これは、コンパイラが無効な「ニッチ値」を利用してレイアウト最適化を実行できることを意味します。この応用例は、[*`Option`風の列挙型での判別子の省略*](https://rust-lang.github.io/unsafe-code-guidelines/layout/enums.html#discriminant-elision-on-option-like-enums)で、型のニッチを`enum`の["タグ"](#tag)として使用し、別のフィールドを必要としません。
<span id="nll">NLL</span>                      |  [非字句的ライフタイム](../borrow_check/region_inference.md)の略で、これはRustの借用システムの拡張で、制御フローグラフに基づくようにします。
<span id="node-id">node-idまたは`NodeId`</span>  |  ASTまたはHIR内の特定のノードを識別するインデックス。徐々に段階的に廃止され、`HirId`に置き換えられています。詳細は[HIRの章を参照](../hir.md#identifiers-in-the-hir)してください。
<span id="obligation">obligation</span>        |  トレイトシステムによって証明されなければならないもの。（[詳細を見る](../traits/resolution.md)）
<span id="placeholder">placeholder</span>      |  **注：skolemizationはplaceholderによって廃止されました** 「for-all」型（例：`for<'a> fn(&'a u32)`）周辺のサブタイピングを処理し、高階トレイト境界（例：`for<'a> T: Trait<'a>`）を解決する方法。詳細は[placeholderと universeに関する章](../borrow_check/region_inference/placeholders_and_universes.md)を参照してください。
<span id="point">point</span>                  |  NLL解析でMIR内の特定の場所を指すために使用されます。通常、制御フローグラフ内のノードを指すために使用されます。
<span id="projection">射影</span>        |  「相対パス」の一般用語、例：`x.f`は「フィールド射影」、`T::Item`は["関連型射影"](../traits/goals-and-clauses.md#trait-ref)です。
<span id="pc">昇格定数</span>        |  関数から抽出され、静的スコープに昇格した定数。詳細は[このセクション](../mir/index.md#promoted)を参照してください。
<span id="provider">provider</span>            |  クエリを実行する関数。（[詳細を見る](../query.md)）
<span id="quantified">量化</span>        |  数学や論理学では、存在量化と全称量化を使用して「この条件が真であるような型Tは存在するか？」や「これはすべての型Tに対して真か？」などの質問をします。詳細は[背景の章を参照](./background.md#quantified)してください。
<span id="query">query</span>                  |  コンパイル中のサブ計算。クエリ結果は、現在のセッションまたはインクリメンタルコンパイルのためにディスクにキャッシュできます。（[詳細を見る](../query.md)）
<span id="recovery">recovery</span>            |  リカバリーとは、構文解析中に無効な構文（例：欠落したコンマ）を処理し、ASTの解析を続けることを指します。これにより、ユーザーに偽のエラーを表示することを避けます（例：構造体定義にエラーが含まれている場合に「フィールドが欠落している」エラーを表示すること）。
<span id="region">region</span>                |  文献や借用チェッカーで頻繁に使用される「ライフタイム」の別の用語。
<span id="rib">rib</span>                      |  名前リゾルバ内のデータ構造で、名前の単一スコープを追跡します。（[詳細を見る](../name-resolution.md)）
<span id="rpit">RPIT</span>                    |  戻り位置の`impl Trait`。（[リファレンスを参照](https://doc.rust-lang.org/reference/types/impl-trait.html#abstract-return-types)）。
<span id="rpitit">RPITIT</span>                |  トレイト内の戻り位置`impl Trait`。RPITとは異なり、これはジェネリック関連型（GAT）に脱糖されます。[RFC 3425](https://rust-lang.github.io/rfcs/3425-return-position-impl-trait-in-traits.html)で導入されました。（[詳細を見る](../return-position-impl-trait-in-trait.md)）
<span id="rustbuild">rustbuild</span>          |  Rustで書かれたbootstrapの部分の非推奨用語
<span id="scrutinee">スクルティニー</span>          |  スクルティニーは、`match`式および類似のパターンマッチング構造でマッチされる式です。例えば、`match x { A => 1, B => 2 }`では、式`x`がスクルティニーです。
<span id="sess">`sess`</span>                  |  コンパイラ_セッション_で、コンパイル全体で使用されるグローバルデータを格納します
<span id="side-tables">サイドテーブル</span>      |  [AST](#ast)とHIRは作成されると不変であるため、特定のノードのIDでインデックス付けされたハッシュテーブルの形式で、それらに関する追加情報を運ぶことがよくあります。
<span id="sigil">sigil</span>                  |  キーワードに似ていますが、完全に非英数字トークンで構成されています。例えば、`&`は参照のsigilです。
<span id="soundness">健全性</span>          |  型理論の技術用語。大まかに言えば、型システムが健全である場合、型チェックされたプログラムは型安全です。つまり、（安全なrustでは）値を間違った型の変数に強制することは決してできません。（「完全性」を参照）。
<span id="span">span</span>                    |  ユーザーのソースコード内の場所で、主にエラー報告に使用されます。これはファイル名/行番号/列タプルのステロイド版のようなものです：開始/終了ポイントを運び、マクロの展開とコンパイラの脱糖も追跡します。すべてを数バイトにパックしたまま（実際には、テーブルへのインデックスです）。詳細は[`Span`]データ型を参照してください。
<span id="subst">subst</span>                  |  型、定数式などの内部のジェネリックパラメータを、具体的なジェネリック引数を提供することによって[substs](#substs)で_置換_する行為。現在、コンパイラでは_インスタンス化_と呼ばれています。
<span id="substs">substs</span>                |  特定のジェネリックアイテムの_置換_（例：`HashMap<i32, u32>`の`i32`、`u32`）。現在、コンパイラでは_ジェネリック引数_のリストと呼ばれています（ただし、厳密にはこれら2つの概念は異なることに注意してください。文献を参照してください）。
<span id="sysroot">sysroot</span>              |  コンパイラが実行時にロードするビルドアーティファクトのディレクトリ。（[詳細を見る](../building/bootstrapping/what-bootstrapping-does.html#what-is-a-sysroot)）
<span id="tag">tag</span>                      |  列挙型/ジェネレータの「タグ」は、アクティブなバリアント/状態の[判別子](#discriminant)をエンコードします。タグは「直接」（フィールドに判別子を単に格納する）または["ニッチ"](#niche)を使用することができます。
<span id="tait">TAIT</span>                    |  型エイリアス`impl Trait`。[RFC 2515](https://rust-lang.github.io/rfcs/2515-type_alias_impl_trait.html)で導入されました。
<span id="tcx">`tcx`</span>                    |  「型付けコンテキスト」（`TyCtxt`）の標準変数名、コンパイラの主要なデータ構造。（[詳細を見る](../ty.md)）
<span id="lifetime-tcx">`'tcx`</span>          |  `TyCtxt`が使用する割り当てアリーナのライフタイム。コンパイルセッション中にインターンされたほとんどのデータは、このライフタイムを使用します。ただし、`'hir`ライフタイムを使用するHIRデータは例外です。（[詳細を見る](../ty.md)）
<span id="token">token</span>                  |  構文解析の最小単位。トークンは字句解析後に生成されます（[詳細を見る](../the-parser.md)）。
<span id="tls">[TLS]</span>                    |  *スレッドローカルストレージ*。変数は、各スレッドが独自のコピーを持つように定義できます（すべてのスレッドが変数を共有するのではなく）。これはLLVMとのいくつかの相互作用があります。すべてのプラットフォームがTLSをサポートしているわけではありません。
<span id="trait-ref">トレイト参照、trait ref </span> |  適切なジェネリック引数のリストとともにトレイトの名前。（[詳細を見る](../traits/goals-and-clauses.md#trait-ref)）
<span id="trans">trans</span>                  |  _変換_の略で、MIRをLLVM IRに変換するコード。[codegen](#codegen)に名前変更されました。
<span id="ty">`Ty`</span>                      |  型の内部表現。（[詳細を見る](../ty.md)）
<span id="tyctxt">`TyCtxt`</span>              |  コード内で[`tcx`](#tcx)として頻繁に参照されるデータ構造で、セッションデータとクエリシステムへのアクセスを提供します。
<span id="ufcs">UFCS</span>                    |  _汎用関数呼び出し構文_の略で、メソッドを呼び出すための明確な構文です。**用語はもう使用されていません！** _完全修飾パス/構文_を優先してください。（[詳細を見る](../type-checking.md)、[リファレンスを参照](https://doc.rust-lang.org/reference/expressions/call-expr.html#disambiguating-function-calls)）
<span id="ut">居住不可能型</span>          |  値を_持たない_型。これは、正確に1つの値を持つZSTとは異なります。居住不可能型の例は`enum Foo {}`で、バリアントがないため、決して作成できません。コンパイラは、居住不可能型を扱うコードをデッドコードとして扱うことができます。なぜなら、操作する値が存在しないからです。`!`（never型）は居住不可能型です。居住不可能型は_空の型_とも呼ばれます。
<span id="upvar">upvar</span>                  |  クロージャの外部からクロージャによってキャプチャされた変数。
<span id="variance">変性</span>            |  ジェネリックパラメータへの変更がサブタイピングにどのように影響するかを決定します。例えば、`T`が`U`のサブタイプである場合、`Vec<T>`は`Vec<U>`のサブタイプです。なぜなら、`Vec`はそのジェネリックパラメータに対して_共変_だからです。より一般的な説明については、[背景の章](./background.md#variance)を参照してください。型チェックが変性をどのように処理するかについては、[変性の章](../variance.md)を参照してください。
<span id="variant-idx">バリアントインデックス</span>    |  列挙型では、0から始まるインデックスを割り当てることでバリアントを識別します。これは純粋に内部的なもので、ユーザーが上書きできる["判別子"](#discriminant)と混同しないでください（例：`enum Bool { True = 42, False = 0 }`）。
<span id="wf">適格性</span>           |  意味的に：意味のある結果に評価される式。型システムでは：型システムのルールに従う型関連構造。
<span id="wide-ptr">ワイドポインタ</span>        |  追加のメタデータを持つポインタ。詳細は[ファットポインタ](#fat-ptr)を参照してください。
<span id="zst">ZST</span>                      |  *ゼロサイズ型*。値のサイズが0バイトである型。`2^0 = 1`なので、そのような型は正確に1つの値を持つことができます。例えば、`()`（ユニット）はZSTです。`struct Foo;`もZSTです。コンパイラはZST周辺でいくつかの優れた最適化を行うことができます。

[LLVM]: https://llvm.org/
[lto]: https://llvm.org/docs/LinkTimeOptimization.html
[`Span`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/struct.Span.html
[thinlto]: https://clang.llvm.org/docs/ThinLTO.html
[TLS]: https://llvm.org/docs/LangRef.html#thread-local-storage-models
