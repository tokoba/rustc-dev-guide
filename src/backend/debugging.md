# LLVMのデバッグ

> 注：コード生成に関する情報をお探しの場合は、代わりに[この章][codegen]を参照してください。

[codegen]: ./codegen.md

このセクションでは、コード生成におけるコンパイラのバグのデバッグ（例：コンパイラがある種のコードを生成した理由、またはLLVMでクラッシュした理由）について説明します。LLVMはそれ自体が大きなプロジェクトであり、おそらく独自のデバッグドキュメントが必要です（見つけられませんでしたが）。しかし、rustcのコンテキストで重要ないくつかのヒントを以下に示します：

### 例を最小化する

一般的なルールとして、コンパイラはコードを分析して大量の情報を生成します。したがって、通常、有用な最初のステップは最小限の例を見つけることです。これを行う1つの方法は次のとおりです

1. 問題を再現する新しいクレートを作成する（例：問題のあるクレートを依存関係として追加し、そこから使用する）

2. 外部依存関係を削除してクレートを最小化する。つまり、関連するすべてを新しいクレートに移動する

3. コードを短くすることで問題をさらに最小化する（`creduce` などのツールがこれに役立ちます）

上記のステップ2と3の方法論の詳細については、pnkfelixによるRustプログラムの最小化に特化した[エピックブログ投稿][mcve-blog]があります。

[mcve-blog]: https://blog.pnkfx.org/blog/2019/11/18/rust-bug-minimization-patterns/

### LLVM内部チェックを有効にする

公式コンパイラ（ナイトリーを含む）はLLVMアサーションを無効にしています。つまり、LLVMアサーション失敗はコンパイラのクラッシュ（ICEではなく「本当の」クラッシュ）やその他の奇妙な動作として現れる可能性があります。これらに遭遇している場合は、LLVMアサーションを有効にしたコンパイラを使用してみることをお勧めします。これは、「alt」ナイトリーを使用するか、bootstrap.tomlで `[llvm] assertions=true` を設定して自分でビルドしたコンパイラのいずれかです。そして、何か判明するかどうかを確認してください。

rustcビルドプロセスは、LLVMツールを `./build/<host-triple>/llvm/bin` にビルドします。これらは直接呼び出すことができます。これらのツールには以下が含まれます：
 * [`llc`]、ビットコード（`.bc` ファイル）を実行可能コードにコンパイルします。これはLLVMバックエンドのバグを再現するために使用できます。
 * [`opt`]、LLVM最適化パスを実行するビットコード変換器。
 * [`bugpoint`]、大きなテストケースを小さくて有用なものに削減します。
 * その他多数。以下のテキストで参照されているものもあります。

[`llc`]: https://llvm.org/docs/CommandGuide/llc.html
[`opt`]: https://llvm.org/docs/CommandGuide/opt.html
[`bugpoint`]: https://llvm.org/docs/Bugpoint.html

デフォルトでは、Rustビルドシステムは、LLVMソースコードまたはそのビルド構成設定の変更をチェックしません。したがって、`rustc` にリンクされているLLVMを再ビルドする必要がある場合は、まず `.llvm-stamp` ファイルを削除してください。これは `build/<host-triple>/llvm/` にあるはずです。

デフォルトのrustcコンパイルパイプラインには複数のコードジェネレーション単位があり、手動で再現するのが難しく、LLVMが並列で複数回呼び出されることを意味します。可能であれば（つまり、バグが消えない場合）、rustcに `-C codegen-units=1` を渡すとデバッグが簡単になります。

### 生のLLVM入力を入手する

rustcがLLVM IRを生成するには、`--emit=llvm-ir` フラグを渡す必要があります。cargoを介してビルドしている場合は、`RUSTFLAGS` 環境変数を使用します（例：`RUSTFLAGS='--emit=llvm-ir'`）。これにより、rustcはLLVM IRをターゲットディレクトリに吐き出します。

`cargo llvm-ir [options] path` は、`path` の特定の関数のLLVM IRを吐き出します。（`cargo install cargo-asm` は `cargo asm` と `cargo llvm-ir` をインストールします）。`--build-type=debug` はデバッグビルドのコードを出力します。他にも便利なオプションがあります。また、LLVM IRのデバッグ情報は出力を非常に乱雑にする可能性があります：`RUSTFLAGS="-C debuginfo=0"` は本当に便利です。

`RUSTFLAGS="-C save-temps"` は、コンパイル中の異なる段階でLLVMビットコードを出力します。これは時々便利です。出力されるLLVMビットコードは、`rustc` の `--out-dir DIR` 引数で設定されたコンパイラの出力ディレクトリの `.bc` ファイルになります。

 * `rustc` 自体を呼び出すときにLLVMバックエンドからアサーション失敗またはセグメンテーション違反が発生している場合は、これらの `.bc` ファイルのそれぞれを `llc` コマンドに渡して、同じ失敗が得られるかどうかを試すことをお勧めします。（LLVM開発者は、最小化された再現にRustクレートを使用するものよりも `.bc` ファイルに削減されたバグを好むことがよくあります。）

 * LLVMビットコードを人間が読める形式で取得するには、`.bc` ファイルを `llvm-dis` を使用して `.ll` ファイルに変換するだけです。これはrustcのターゲットローカルコンパイルにあるはずです。


rustcは `-O` が有効かどうかによって異なるIRを出力することに注意してください。これはLLVMの最適化がなくても同様です。したがって、rustcが出力するIRで遊びたい場合は、次のようにする必要があります：

```bash
$ rustc +local my-file.rs --emit=llvm-ir -O -C no-prepopulate-passes \
    -C codegen-units=1
$ OPT=./build/$TRIPLE/llvm/bin/opt
$ $OPT -S -O2 < my-file.ll > my
```

LLVMパイプライン中にLLVM IRを取得したいだけの場合、例えば最適化時のアサーションを失敗させるIRを確認したり、LLVMが特定の最適化を実行するタイミングを確認したりするには、rustcフラグ `-C llvm-args=-print-after-all` を渡し、おそらく `-C llvm-args='-filter-print-funcs=EXACT_FUNCTION_NAME` を追加します（例：`-C llvm-args='-filter-print-funcs=_ZN11collections3str21_$LT$impl$u20$str$GT$\
7replace17hbe10ea2e7c809b0bE'`）。

これは標準エラーに大量の出力を生成するため、それをファイルにパイプしたいでしょう。また、`-filter-print-funcs` も `-C codegen-units=1` も使用していない場合、複数のコードジェネレーション単位が並列で実行されるため、出力が混ざり合って何も読めなくなります。

 * 前述の方法論の1つの注意点：LLVMへの `-print` ファミリーのオプションは、パスが実行されるIR単位（例：関数のみ）のみを出力し、参照される宣言、グローバル、メタデータなどは含まれません。これは、一般的に `-print` の出力を `llc` にフィードして特定の問題を再現できないことを意味します。

 * LLVM自体内で、`SafeStackLegacyPass::runOnFunction` の開始時に `F.getParent()->dump()` を呼び出すと、モジュール全体がダンプされ、再現のためのより良い基礎を提供する可能性があります。（ただし、`-C save-temps` によってダンプされた `.bc` ファイルから同じダンプを取得できるはずです。）

特定の関数のIRだけが欲しい場合（例：アサーションを引き起こす理由や正しく最適化されない理由を確認したい場合）、`llvm-extract` を使用できます。例：

```bash
$ ./build/$TRIPLE/llvm/bin/llvm-extract \
    -func='_ZN11collections3str21_$LT$impl$u20$str$GT$7replace17hbe10ea2e7c809b0bE' \
    -S \
    < unextracted.ll \
    > extracted.ll
```

### LLVM最適化パスを調査する

最適化パスによる不正な動作が見られる場合、非常に便利なLLVMオプションは `-opt-bisect-limit` です。これは、実行する最高パスのインデックス値を示す整数を取ります。取得されたパスのインデックス値は、実行ごとに安定しています。これを、結果のプログラムに基づいて検索空間を二分する自動化ソフトウェアと組み合わせることで、誤ったパスを迅速に特定できます。`-opt-bisect-limit` が指定されている場合、すべての実行が標準エラーに表示され、そのインデックスとパスが実行されたかスキップされたかを示す出力が表示されます。制限を -1 のインデックスに設定すると（例：`RUSTFLAGS="-C llvm-args=-opt-bisect-limit=-1"`）、すべてのパスとそれに対応するインデックス値が表示されます。

最適化パイプラインで遊びたい場合は、rustcによって出力されたLLVM IRで `./build/<host-triple>/llvm/bin/` の [`opt`] ツールを使用できます。

LLVM自体の実装を調査する際には、その[内部デバッグインフラストラクチャ][llvm-debug]に注意する必要があります。これはLLVMデバッグビルドで提供され、bootstrap.tomlでこの設定を変更することでrustcのLLVMビルドで有効にできます：
```
[llvm]
# LLVMアサーションが有効かどうかを示します
assertions = true

# LLVMビルドがリリースビルドかデバッグビルドかを示します
optimize = false
```
簡単な要約：
 * `assertions=true` を設定すると、粗粒度のデバッグメッセージが有効になります。
   * それを超えて、`optimize=false` を設定すると、細粒度のデバッグメッセージが有効になります。
 * LLVMの `LLVM_DEBUG(dbgs() << msg)` は、`rustc` の `debug!(msg)` のようなものです。
 * `-debug` オプションはすべてのメッセージを有効にします。これは `rustc` で環境変数 `RUSTC_LOG=debug` を設定するようなものです。
 * `-debug-only=<pass1>,<pass2>` バリアントはより選択的です。これは `rustc` で環境変数 `RUSTC_LOG=path1,path2` を設定するようなものです。

[llvm-debug]: https://llvm.org/docs/ProgrammersManual.html#the-llvm-debug-macro-and-debug-option

### ヘルプと質問

質問がある場合は、[rust-lang Zulip] に向かい、特に `#t-compiler/wg-llvm` チャンネルに向かってください。

[rust-lang Zulip]: https://rust-lang.zulipchat.com/

### 知っておくべきコンパイラオプション

`-C help` と `-Z help` コンパイラスイッチは、便利と思われるさまざまな興味深いオプションをリストアウトします。LLVM開発に関連する最も一般的なものをいくつか示します（上記のチュートリアルでいくつか使用されています）：

- `--emit llvm-ir` オプションは、テキスト形式のLLVM IRを含む `<filename>.ll` ファイルを出力します
    - `--emit llvm-bc` オプションは、バイトコード形式（`<filename>.bc`）で出力します
- `-C llvm-args=<foo>` を渡すと、llcやoptが受け入れるほとんどすべてのオプションを渡すことができます。例：`-C llvm-args=-print-before-all` を渡すと、すべてのLLVMパスの前にIRを出力します。
- `-C no-prepopulate-passes` は、LLVMパスマネージャにパスのリストを事前入力するのを避けます。これにより、rustcが生成するLLVM IRを表示できます。最適化後のLLVM IRではありません。
- `-C passes=val` オプションを使用すると、実行する追加のLLVMパスのスペース区切りリストを提供できます
- `-C save-temps` オプションは、コンパイル中にすべての一時出力ファイルを保存します
- `-Z print-llvm-passes` オプションは、実行されているLLVM最適化パスを出力します
- `-Z time-llvm-passes` オプションは、各LLVMパスの時間を測定します
- `-Z verify-llvm-ir` オプションは、LLVM IRの正しさを検証します
- `-Z no-parallel-backend` オプションは、個別のコンパイル単位の並列コンパイルを無効にします
- `-Z llvm-time-trace` オプションは、LLVMパスの詳細とタイミングを含むChrome profiler互換のJSONファイルを出力します。
- `-C llvm-args=-opt-bisect-limit=<index>` オプションを使用すると、LLVM最適化を二分探索できます。

### LLVMバグレポートの提出

LLVMバグレポートを提出する際は、おそらく問題を示す何らかの最小限の動作例が必要になるでしょう。Godboltコンパイラエクスプローラーはこれに本当に役立ちます。

1. 問題のあるコードのLLVM IRを入手したら（上記を参照）、Godboltで最小限の動作例を作成できます。[llvm.godbolt.org](https://llvm.godbolt.org)にアクセスしてください。

2. プログラミング言語として `LLVM-IR` を選択します。

3. `llc` を使用して、IRをそのまま特定のターゲットにコンパイルします：
    - いくつかの便利なフラグがあります：`-mattr` はターゲット機能を有効にし、`-march=` はターゲットを選択し、`-mcpu=` はCPUを選択します。
    - `llc -march=help` のようなコマンドは、利用可能なすべてのアーキテクチャを出力します。これは、Rustのアーキテクチャ名とLLVMの名前が一致しない場合があるため便利です。
    - どこかでrustcをコンパイルした場合、ターゲットディレクトリに `llc`、`opt` などのバイナリがあります。

4. LLVM-IRを最適化したい場合は、`opt` を使用してLLVM最適化がどのように変換するかを確認できます。

5. 問題を示すgodboltリンクができたら、LLVMバグを埋めるのはかなり簡単です。[githubの問題ページ][llvm-issues]にアクセスするだけです。

[llvm-issues]: https://github.com/llvm/llvm-project/issues

### LLVMからのバグ修正の移植

バグをLLVMバグとして特定すると、LLVMですでに報告され修正されているが、まだ修正を取得していない（または自分でLLVMに精通していて修正できる）ことがわかる場合があります。

その場合、バグの修正を直接自分のLLVMフォークに移植して、rustcがより簡単に使用できるようにすることができる場合があります。私たちのLLVMフォークは[rust-lang/llvm-project]で管理されています。そこに修正をランディングしたら、サブモジュールのコミットを変更するPRをランディングする必要もあります。Zulipでヘルプを求めてください。

[rust-lang/llvm-project]: https://github.com/rust-lang/llvm-project/
