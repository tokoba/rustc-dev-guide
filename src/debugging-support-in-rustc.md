# Rustコンパイラのデバッグサポート

本ドキュメントは、Rustコンパイラ(rustc)におけるデバッグツールサポートの状態を説明します。
GDB、LLDB、WinDbg/CDB、および
RustコードをデバッグするためのRustコンパイラ周辺のインフラストラクチャの概要を示します。
Rustコンパイラ自体をデバッグする方法を学びたい場合は、
[Debugging the Compiler]を参照してください。

資料は、ビデオ
[Tom Tromey discusses debugging support in rustc]から収集されました。

## 予備知識

### デバッガー

Wikipediaによると

> [debugger or debugging tool]は、他のプログラム(「ターゲット」プログラム)をテストおよびデバッグするために使用されるコンピュータプログラムです。

言語のためにデバッガーをゼロから書くには多くの作業が必要です。特に、
デバッガーがさまざまなプラットフォームでサポートされる必要がある場合はそうです。ただし、GDBおよびLLDBは、
言語のデバッグをサポートするように拡張できます。これがRustが選択した道です。
このドキュメントの主な目標は、Rustコンパイラでの上記のデバッガーサポートを文書化することです。

### DWARF

[DWARF]標準のウェブサイトによると

> DWARFは、多くのコンパイラとデバッガーがソースレベルのデバッグをサポートするために使用するデバッグファイル形式です。C、C++、Fortranなどの多くの手続き型言語の要件に対応しており、
> 他の言語にも拡張可能に設計されています。
> DWARFはアーキテクチャに依存せず、任意のプロセッサまたはオペレーティングシステムに適用できます。
> Unix、Linux、および他のオペレーティングシステムで広く使用されており、
> スタンドアロン環境でも使用されています。

DWARFリーダーは、DWARF形式を消費し、デバッガー互換の出力を作成するプログラムです。
このプログラムはコンパイラ自体に存在する場合があります。DWARFは、
Debugging Information Entry(DIE)と呼ばれるデータ構造を使用し、情報を「タグ」として保存して、関数、
変数などを示します。例えば、`DW_TAG_variable`、`DW_TAG_pointer_type`、`DW_TAG_subprogram`などです。
独自のタグと属性を発明することもできます。

### CodeView/PDB

[PDB](Program Database)は、デバッグ情報を含むMicrosoftが作成したファイル形式です。
PDBは、WinDbg/CDBなどのデバッガーや他のツールによって消費され、デバッグ情報を表示できます。
PDBには、特定のバイナリに関するデバッグ情報を記述する複数のストリームが含まれています。例えば、
型、シンボル、および指定されたバイナリのコンパイルに使用されたソースファイルなどです。CodeViewは、
PDBストリーム内に現れる[symbol records]と[type records]の構造を定義する別の形式です。

## サポートされているデバッガー

### GDB

#### Rust式パーサー

デバッグ出力を表示できるようにするには、式パーサーが必要です。
この(GDB)式パーサーは[Bison]で書かれており、
Rust式のサブセットのみを解析できます。
GDBパーサーはゼロから書かれており、rustcのパーサーを含む他のパーサーとは関係ありません。

GDBはRustのような値と型の出力を持っています。出力でRust構文のように見える方法で値と型を印刷できます。または、GDBで[ptype]として型を印刷すると、
Rustソースコードのように見えます。[manual for GDB/Rust]のドキュメントをチェックしてください。

#### パーサー拡張

式パーサーには、Rustでは実行できない機能を促進するためのいくつかの拡張があります。いくつかの制限は[manual for GDB/Rust]にリストされています。GDBのDWARFリーダーには、拡張をサポートするための特別なコードがあります。

DWARFリーダーサポートが必要ないくつかの例は次のとおりです:

1. Enum: enum型のサポートに必要です。
   Rustコンパイラは、enumに関する情報をDWARFに書き込み、
   GDBはDWARFを読み取って、タグフィールドがどこにあるか、
   タグフィールドがあるか、
   またはタグスロットが非ゼロ最適化と共有されているかなどを理解します。

2. トレイトオブジェクトの分解: トレイトオブジェクトのDWARF内の記述が、対応するvtableのスタブ記述を指し、それが今度はこのトレイトオブジェクトが存在する具体的な型を指すDWARF拡張。これは、そのトレイトオブジェクトに対して`print *object`を実行でき、GDBがトレイトオブジェクト内のペイロードの正しい型を見つける方法を理解することを意味します。

**TODO**: 以下のコメントに関して、GDB-Rustドキュメントに記載すべきかどうかを判断してください。このガイドページに重複がないようにするためです:

[This comment by Tom](https://github.com/rust-lang/rustc-dev-guide/pull/316#discussion_r284027340)
> gdbのRust拡張と制限はgdbマニュアルに文書化されています:
<https://sourceware.org/gdb/onlinedocs/gdb/Rust.html> -- ただし、これはgdbの便利な変数とレジスタがgdbの$規約に従うことを言及しておらず、RustパーサーがgdbのA拡張を実装していることも言及していません。

[This question by Aman](https://github.com/rust-lang/rustc-dev-guide/pull/316#discussion_r285401353)
> @tromey この部分をGDB-Rustドキュメントに記載すべきと思いますか?重複を避けるためです。

### LLDB

#### Rust式パーサー

この式パーサーはC++で書かれています。これは[Recursive Descent parser]の一種です。
GDBよりもわずかに少ないRust言語を実装しています。
LLDBはRustのような値と型の出力を持っています。

#### 開発者ノート

* LLDBにはプラグインアーキテクチャがありますが、言語サポートには機能しません。
* GDBは一般的にLinuxでより良く機能します。

### WinDbg/CDB

Microsoftは、Rustで書かれたプログラムのデバッグをサポートする[Windows Debugging Tools]を提供しています。例えば、Windows Debugger(WinDbg)やConsole Debugger(CDB)などです。これらの
デバッガーは、バイナリの`PDB`からデバッグ情報を解析し(利用可能な場合)、デバッガーで提供する視覚化を構築します。

#### Natvis

WinDbgとCDBの両方は、Natvisフレームワークを使用して、デバッガー内の任意の型のカスタム視覚化を定義および表示することをサポートしています。Rustコンパイラは、標準ライブラリ内の型のサブセット(例:
`std`、`core`、および`alloc`)のカスタム視覚化を定義するNatvisファイルのセットを定義しています。これらのNatvisファイルは、`*-pc-windows-msvc`ターゲットトリプルによって生成される`PDB`に埋め込まれ、デバッグ時にこれらのカスタム視覚化を自動的に有効にします。このデフォルトは、rustc `strip`フラグを`debuginfo`
または`symbols`に設定することでオーバーライドできます。

Rustは、`#[debugger_visualizer]`属性を使用して、標準ライブラリ以外のクレートのNatvisファイルを埋め込むことをサポートしています。
デバッガービジュアライザーを埋め込む方法の詳細については、
[`debugger_visualizer` attribute]のセクションを参照してください。

## DWARFと`rustc`

[DWARF]は、コンパイラがデバッガーが読み取るデバッグ情報を生成する標準的な方法です。
これはmacOSとLinuxでの_デバッグ形式_です。
マルチ言語で拡張可能な形式であり、
Rustの目的にはほぼ十分です。
したがって、現在の実装はDWARFの概念を再利用しています。
これは、DWARFのいくつかの概念がセマンティックにRustと整合していない場合でも当てはまります。なぜなら、
一般的に、2つの間に何らかのマッピングがあり得るからです。

Rustコンパイラが出力し、デバッガーが理解するいくつかのDWARF拡張があります。これらは
DWARF標準には_含まれていません_。

* Rustコンパイラは、仮想テーブルのDWARFを出力します。この`vtable`オブジェクトには、
  実際の型を指す`DW_AT_containing_type`があります。これにより、デバッガーはトレイトオブジェクト
   ポインタを分解して、ペイロードを正しく見つけることができます。例えば、これはgdbのテストケースからのそのようなDIEです
   リポジトリ:

   ```asm
   <1><1a9>: Abbrev Number: 3 (DW_TAG_structure_type)
      <1aa>   DW_AT_containing_type: <0x1b4>
      <1ae>   DW_AT_name        : (indirect string, offset: 0x23d): vtable
      <1b2>   DW_AT_byte_size   : 0
      <1b3>   DW_AT_alignment   : 8
   ```

* もう1つの拡張は、Rustコンパイラがタグなしの判別共用体を出力できることです。
  この項目の[DWARF feature request]を参照してください。

### DWARFの現在の制限

* トレイト - DWARFでトレイトを表現する方法について、通常よりも大きな変更が必要です。
* DWARFは、構造体とタプルを区別する方法を提供していません。Rustコンパイラは
`__0`でフィールドを出力し、デバッガーはそのような名前のシーケンスを探してこの制限を克服します。
例えば、この場合、デバッガーは`x.0`の代わりに`x.__0`を介してフィールドを見ます。
これは、デバッガーのRustパーサーを介して解決されるため、`x.0`を実行できるようになりました。

DWARFは、プラットフォームABIに関するいくつかの情報をデバッガーが知っていることに依存しています。
Rustは常にそれを行うわけではありません。

## 開発者ノート

このセクションは、開発の特定の側面についての講演からのものです。

## 何が欠けているか

### macOS上のLLDBデバッグサーバーのコード署名

Wikipediaによると、[System Integrity Protection]は

> System Integrity Protection(SIP、ルートレスとも呼ばれる)は、Appleのm acOSオペレーティングシステムのセキュリティ機能で、OS X El Capitanで導入されました。これは、カーネルによって強制されるいくつかのメカニズムで構成されています。中心的なものは、特定の「資格」を持たないプロセスによるシステム所有のファイルとディレクトリの変更からの保護です。
> rootユーザーまたはroot権限(sudo)で実行されたプロセスでも。

これは、`ptrace`システムコールを使用するプロセスを防ぎます。プロセスが`ptrace`を使用したい場合は、コード署名される必要があります。署名する証明書は、マシンで信頼されている必要があります。

[Apple developer documentation for System Integrity Protection]を参照してください。

Appleに登録してこの署名を行うためのキーを取得する必要がある場合があります。Tomは、Mozillaが
署名できるキーの最大数に達しているため、これを実行できないかどうかを調べました。TomはMozillaがより多くのキーを取得できるかどうかわかりません。

あるいは、Tomは、Appleを介してキーを取得するためにRust法人が必要かもしれないと提案しています。
この問題は技術的な性質ではありません。そのようなキーがあれば、GDBにも署名して
出荷できます。

### DWARFとトレイト

RustトレイトはDWARFにまったく出力されません。これの影響は、メソッド`x.method()`の呼び出しが
そのままでは機能しないことです。理由は、そのメソッドが、
型ではなく、トレイトによって実装されているためです。その情報が存在しないため、トレイトメソッドの検索が欠落しています。

DWARFには、インターフェース型の概念があります(おそらくJavaのために追加されました)。Tomのアイデアは、この
インターフェース型をトレイトとして使用することでした。

DWARFは、参照型ではなく具体的な名前のみを扱います。したがって、型のトレイトの与えられた実装は、これらのインターフェース(`DW_tag_interface`型)の1つになります。また、それが実装される型は、この型が実装するすべてのインターフェースを記述します。これにはDWARF拡張が必要です。

Githubの問題: [https://github.com/rust-lang/rust/issues/33014]

## デバッグ情報変更の典型的なプロセス(LLVM)

LLVMにはDebug Info(DI)ビルダーがあります。これがRustが呼び出す主要なものです。
これがDWARFを直接ではなく最初に出力されるため、最初にLLVMを変更する必要がある理由です。
これは、構築してLLVMに渡すメタデータの一種です。Rustc/LLVMハンドオフのために、
型の表現を構築するためにいくつかのLLVM DIビルダーメソッドが呼び出されます。

このプロセスのステップは次のとおりです:

1. LLVMの変更が必要です。

   LLVMはインターフェース型をまったく出力しないため、これを最初にLLVMに実装する必要があります。

   これが良いアイデアであることについて、LLVMメンテナのサインオフを得ます。

2. DWARF拡張を変更します。

3. デバッガーを更新します。

   DWARFリーダー、式エバリュエータを更新します。

4. Rustコンパイラを更新します。

   この新しい情報を出力するように変更します。

### 手続きマクロのステッピング

深く考えるべき質問は、手続きマクロを実際にデバッグする方法です。
マクロ展開のためにどの場所を出力しますか?以下のいくつかのケースを考えてみてください -

* マクロの呼び出し場所を出力できます。
* マクロの定義場所を出力できます。
* マクロの内容の場所を出力できます。

RFC: [https://github.com/rust-lang/rfcs/pull/2117]

焦点は、マクロに何をすべきかを決定させることです。これは、マクロがコンパイラに
行マーカーがどこにあるべきかを伝えることができるある種の属性を持つことで実現できます。これは、ブレークポイントを設定する場所と、ステップ実行時に何が起こるかに影響します。

## デバッグ情報のソースファイルチェックサム

DWARFとCodeView(PDB)の両方は、関連するバイナリに貢献した各ソースファイルの暗号化ハッシュの埋め込みをサポートしています。

暗号化ハッシュは、デバッガーがソースファイルが実行可能ファイルと一致するかどうかを検証するために使用できます。ソースファイルが一致しない場合、デバッガーはユーザーに警告を提供できます。

ハッシュは、与えられたソースファイルが
実行可能ファイルのコンパイルに使用されて以来変更されていないことを証明するためにも使用できます。MD5とSHA1の両方に実証された脆弱性があるため、
このアプリケーションにはSHA256の使用が推奨されます。

Rustコンパイラは、`SourceMap`内の対応する`SourceFile`に各ソースファイルのハッシュを保存します。外部クレートへの入力ファイルのハッシュは`rlib`メタデータに保存されます。

デフォルトのハッシュアルゴリズムは、ターゲット仕様で設定されています。これにより、ターゲットは
すべてのターゲットがすべてのハッシュアルゴリズムをサポートしているわけではないため、利用可能な最良のハッシュを指定できます。

ターゲットのハッシュアルゴリズムは、`-Z source-file-checksum=`
コマンドラインオプションでオーバーライドすることもできます。

#### DWARF 5

DWARFバージョン5は、使用中のソースファイルバージョンを検証するためのMD5ハッシュの埋め込みをサポートしています。
DWARF 5 - Section 6.2.4.1 opcode DW_LNCT_MD5

#### LLVM

LLVM IRは、DIFileノードでMD5とSHA1(およびLLVM 11以降でSHA256)のソースファイルチェックサムをサポートしています。

[LLVM DIFile documentation](https://llvm.org/docs/LangRef.html#difile)

#### Microsoft Visual C++ Compiler /ZH option

MSVCコンパイラは、`/ZH`コンパイラオプションを使用してPDBにMD5、SHA1、またはSHA256ハッシュを埋め込むことをサポートしています。

[MSVC /ZH documentation](https://docs.microsoft.com/en-us/cpp/build/reference/zh)

#### Clang

ClangはMD5チェックサムを常に埋め込みますが、これはドキュメントには表示されません。

## 将来の作業

#### 名前マングリングの変更

* `libiberty`(gccソースツリー)の新しいデマングラー。
* LLVMまたはLLDBの新しいデマングラー。

**TODO**: デマングラーソースの場所を確認してください。[#1157](https://github.com/rust-lang/rustc-dev-guide/issues/1157)

#### 式のためのRustコンパイラの再利用

これは重要なアイデアです。なぜなら、デバッガーは大部分、型推論を実装しようとしないからです。デバッガーに入力する際は、
実際のソースコードよりもはるかに明示的である必要があります。したがって、ソースから式をコピーしてデバッガーに貼り付けて同じ答えを期待することはできませんが、これは良いことです。これはコンパイラを使用することで支援できます。

確かに実行可能ですが、大規模なプロジェクトです。確かにデバッガーへのブリッジが必要です。デバッガーだけがメモリにアクセスできるためです。GDB(gcc)とLLDB(clang)の両方に
この機能があります。LLDBはClangを使用してコードをJITにコンパイルし、GDBもGCCで同じことを実行できます。

両方のデバッガーの式評価は、Rustのスーパーセットとサブセットの両方を実装しています。
式言語のみを実装していますが、
GDBには便利な変数などのいくつかの拡張も追加しています。
したがって、このルートを取る場合は、
このブリッジを行うだけでなく、コンパイラにいくつかの拡張を理解させるためのモードを追加する必要がある場合があります。

[Tom Tromey discusses debugging support in rustc]: https://www.youtube.com/watch?v=elBxMRSNYr4
[Debugging the Compiler]: compiler-debugging.md
[debugger or debugging tool]: https://en.wikipedia.org/wiki/Debugger
[Bison]: https://www.gnu.org/software/bison/
[ptype]: https://ftp.gnu.org/old-gnu/Manuals/gdb/html_node/gdb_109.html
[DWARF]: http://dwarfstd.org
[manual for GDB/Rust]: https://sourceware.org/gdb/onlinedocs/gdb/Rust.html
[Recursive Descent parser]: https://en.wikipedia.org/wiki/Recursive_descent_parser
[System Integrity Protection]: https://en.wikipedia.org/wiki/System_Integrity_Protection
[DWARF feature request]: http://dwarfstd.org/ShowIssue.php?issue=180517.2
[https://github.com/rust-lang/rfcs/pull/2117]: https://github.com/rust-lang/rfcs/pull/2117
[https://github.com/rust-lang/rust/issues/33014]: https://github.com/rust-lang/rust/issues/33014
[Apple developer documentation for System Integrity Protection]: https://developer.apple.com/library/archive/releasenotes/MacOSX/WhatsNewInOSX/Articles/MacOSX10_11.html#//apple_ref/doc/uid/TP40016227-SW11
[PDB]: https://llvm.org/docs/PDB/index.html
[symbol records]: https://llvm.org/docs/PDB/CodeViewSymbols.html
[type records]: https://llvm.org/docs/PDB/CodeViewTypes.html
[Windows Debugging Tools]: https://docs.microsoft.com/en-us/windows-hardware/drivers/debugger/
[`debugger_visualizer` attribute]: https://doc.rust-lang.org/nightly/reference/attributes/debugger.html#the-debugger_visualizer-attribute
