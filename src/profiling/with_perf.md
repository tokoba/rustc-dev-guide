# perfを使ったプロファイリング

これは[perf](https://perf.wiki.kernel.org/index.php/Main_Page)を使ってrustcをプロファイリングする方法のガイドです。

## 初期手順

- rust-lang/rustのクリーンなチェックアウトを取得します
- `bootstrap.toml`で以下の設定を行います:
  - `rust.debuginfo-level = 1` - 行デバッグ情報を有効にします
  - `rust.jemalloc = false` - valgrindでメモリ使用量のプロファイリングができるようになります
  - それ以外はすべてデフォルトのままにします
- `./x build`を実行してフルビルドを取得します
- その結果を指すrustupツールチェーンを作成します
  - 手順については[「ビルドと実行」セクション][b-a-r]を参照してください

[b-a-r]: ../building/how-to-build-and-run.html#toolchain

## perfプロファイルの収集

perfは、あらゆる種類の情報を収集および分析するために使用できるLinux上の優れたツールです。
主にプログラムがどこで時間を費やしているかを把握するために使用されます。
ただし、キャッシュミスなどの他の種類のイベントにも使用できます。

### 基本

基本的な`perf`コマンドは次のとおりです:

```bash
perf record -F99 --call-graph dwarf XXX
```

`-F99`はperfに99Hzでサンプリングするように指示します。これにより、
長時間の実行で大量のデータを生成することを避けられます
(なぜ99Hzなのかって? これは他の周期的な活動とロックステップになる可能性が低いため、
よく選ばれます)。`--call-graph dwarf`はperfにデバッグ情報から
コールグラフ情報を取得するように指示します。これは正確です。`XXX`は
プロファイリングしたいコマンドです。例えば、次のようにします:

```bash
perf record -F99 --call-graph dwarf cargo +<toolchain> rustc
```

`cargo`を実行します -- ここで`<toolchain>`は最初に作成したツールチェーンの名前です。
ただし、注意すべき点がいくつかあります:

- 依存関係のビルドに費やされる時間をプロファイリングしたくないでしょう。
  そのため、`cargo build; cargo clean -p $C`のようなものが役立つかもしれません
  (`$C`はクレート名です)
    - ただし、通常は`touch src/lib.rs`を実行して代わりに再ビルドします。=)
- インクリメンタルがプロファイルに干渉しないようにしたいでしょう。
  そのため、`CARGO_INCREMENTAL=0`のようなものが役立ちます。

`cargo`から収集したデータを読み取るときに`addr2line xxx/elf: could not read first record`
の問題を回避するには、最新バージョンの`addr2line`を使用する必要がある場合があります:

```bash
cargo install addr2line --features="bin"
```

### `perf.rust-lang.org`テストからのperfプロファイルの収集

多くの場合、`perf.rust-lang.org`の特定のテストを分析したいと思います。
これを行う最も簡単な方法は、[rustc-perf][rustc-perf]
ベンチマークスイートを使用することです。このアプローチは[こちら](with_rustc_perf.md)で説明されています。

ベンチマークスイートCLIを使用する代わりに、ベンチマークを手動でプロファイリングすることもできます。まず、
[rustc-perf][rustc-perf]リポジトリをクローンする必要があります:

```bash
$ git clone https://github.com/rust-lang/rustc-perf
```

次に、プロファイリングしたいテストのソースコードを見つけます。テストのソースは
[`collector/compile-benchmarks`ディレクトリ][compile-time dir]と
[`collector/runtime-benchmarks`ディレクトリ][runtime dir]にあります。
では、特定のテストのディレクトリに移動しましょう; 例として`clap-rs`を使用します:

[rustc-perf]: https://github.com/rust-lang/rustc-perf
[compile-time dir]: https://github.com/rust-lang/rustc-perf/tree/master/collector/compile-benchmarks
[runtime dir]: https://github.com/rust-lang/rustc-perf/tree/master/collector/runtime-benchmarks

```bash
cd collector/compile-benchmarks/clap-3.1.6
```

この場合、`cargo check`のパフォーマンスをプロファイリングしたいとしましょう。
その場合、まず依存関係をビルドするための基本的なコマンドを実行します:

```bash
# セットアップ: まず古い結果をクリーンアップし、依存関係をビルドします:
cargo +<toolchain> clean
CARGO_INCREMENTAL=0 cargo +<toolchain> check
```

(ここでも、`<toolchain>`は最初の手順で作成したツールチェーンの名前に置き換える必要があります。)

次に: clap-rsクレート*のみ*の実行時間を記録し、cargo checkを実行します。
私は通常、このために`cargo rustc`を使用します。これにより、明示的なフラグを追加することもできます。
これは後で行います。

```bash
touch src/lib.rs
CARGO_INCREMENTAL=0 perf record -F99 --call-graph dwarf cargo rustc --profile check --lib
```

最後のコマンドに注目してください: これは大変です! `cargo rustc`
コマンドを使用しています。これは(潜在的に)追加オプションを指定してrustcを実行します。
`--profile check`と`--lib`オプションは、`cargo check`の実行であり、
これがライブラリ(バイナリではない)であることを指定します。

この時点で、`perf`ツールを使用して結果を分析できます。例えば:

```bash
perf report
```

これにより、インタラクティブなTUIプログラムが開きます。単純なケースでは、
これが役立つことがあります。より詳細な調査には、[`perf-focus`ツール][pf]
が役立ちます。これについては以下で説明します。

**注意。** rustc-perfテストはそれぞれ特別な雪の結晶です。特に、
一部はライブラリではないため、`touch src/main.rs`を実行し、
`--lib`を渡さないようにします。どのテストがどれであるかを
最適に判断する方法はわかりません。

### NLLデータの収集

NLL実行をプロファイリングしたい場合は、次のように`cargo rustc`コマンドに
追加のオプションを渡すだけです:

```bash
touch src/lib.rs
CARGO_INCREMENTAL=0 perf record -F99 --call-graph dwarf cargo rustc --profile check --lib -- -Z borrowck=mir
```

[pf]: https://github.com/nikomatsakis/perf-focus

## `perf focus`でperfプロファイルを分析する

perfプロファイルを収集したら、それについての情報を取得したいと思います。
このために、私は個人的に[perf focus][pf]を使用しています。これは
シンプルですが便利なツールで、次のようなクエリに答えることができます:

- 「関数Fでどれだけの時間が費やされたか」(どこから呼び出されたかに関係なく)
- 「Gから呼び出されたときに関数Fでどれだけの時間が費やされたか」
- 「Gで費やされた時間を*除いた*関数Fでどれだけの時間が費やされたか」
- 「Fはどの関数を呼び出し、それらでどれだけの時間を費やしているか」

仕組みを理解するには、perfについて少しだけ知っておく必要があります。
基本的に、perfは定期的に(または何らかのイベントが発生したときに)
プロセスを*サンプリング*することで機能します。各サンプルについて、
perfはバックトレースを収集します。`perf focus`を使用すると、
バックトレースに表示される関数をテストする正規表現を記述でき、
その正規表現を満たすバックトレースを持つサンプルの割合を示します。
NLLパフォーマンスの分析方法を説明することで、おそらく最も簡単に説明できます。

### `perf-focus`のインストール

`cargo install`を使用してperf-focusをインストールできます:

```bash
cargo install perf-focus
```

### 例: MIR借用チェックでどれだけの時間が費やされているか?

テスト用のNLLデータを収集したとしましょう。MIR借用チェッカーで
どれだけの時間が費やされているかを知りたいとします。MIR borrowckの「メイン」
関数は`do_mir_borrowck`と呼ばれるので、次のコマンドを実行できます:

```bash
$ perf focus '{do_mir_borrowck}'
Matcher    : {do_mir_borrowck}
Matches    : 228
Not Matches: 542
Percentage : 29%
```

`'{do_mir_borrowck}'`引数は**マッチャー**と呼ばれます。これは
バックトレースに適用されるテストを指定します。この場合、
`{X}`は、バックトレースに正規表現`X`を満たす*何らかの*関数が
存在する必要があることを示します。この場合、そのregexは
必要な関数の名前だけです(実際には、名前のサブセットです;
完全な名前には、モジュールパスなど他の多くのものが含まれます)。
このモードでは、perf-focusは`do_mir_borrowck`がスタックにあった
サンプルの割合を出力します: この場合、29%です。

**c++filtに関する注意。** `perf`からデータを取得するために、`perf focus`は
現在`perf script`を実行しています(おそらくより良い方法があるでしょう...)。
`perf script`がC++のマングル名を出力することがあることがわかりました。
これは煩わしいです。`perf script | head`を自分で実行することで確認できます
-- `rustc::middle`の代わりに`5rustc6middle`のような名前が表示される場合、
同じ問題があります。これは次のようにして解決できます:

```bash
perf script | c++filt | perf focus --from-stdin ...
```

これにより、`perf script`の出力が`c++filt`を通してパイプされ、
それらの名前がより親しみやすい形式にほぼ変換されるはずです。
`perf focus`への`--from-stdin`フラグは、`perf focus`を実行する代わりに、
stdinからデータを取得するように指示します。これをより
便利にする必要があります(最悪の場合、`perf focus`に`c++filt`オプションを追加するか、
常に使用するようにするかもしれません -- これはかなり無害です)。

### 例: MIR borrowckはトレイトの解決にどれだけの時間を費やしているか?

おそらく、MIR borrowckがトレイトチェッカーでどれだけの時間を費やしているかを
知りたいと思うでしょう。より複雑な正規表現を使用してこれを尋ねることができます:

```bash
$ perf focus '{do_mir_borrowck}..{^rustc::traits}'
Matcher    : {do_mir_borrowck},..{^rustc::traits}
Matches    : 12
Not Matches: 1311
Percentage : 0%
```

ここでは`..`演算子を使用して、「`do_mir_borrowck`がスタックにあり、
その後、名前が`rustc::traits`で始まる何らかの関数がある頻度は?」と尋ねました
(基本的に、そのモジュールのコード)。答えは「ほとんどない」ことがわかります
-- この説明に当てはまるサンプルは12個だけです(サンプルが*まったく*ない場合は、
クエリが間違っていることを示すことがよくあります)。

興味があれば、`--print-match`オプションを使用して、
どのサンプルかを正確に見つけることができます。これにより、
各サンプルの完全なバックトレースが出力されます。行の先頭の`|`は、
正規表現が一致した部分を示します。

### 例: MIR borrowckはどこで時間を費やしているか?

多くの場合、より「探索的な」クエリを実行したいと思います。例えば、
MIR borrowckが29%の時間を占めていることはわかっていますが、
その時間はどこで費やされているのでしょうか?
そのためには、`--tree-callees`オプションが最適なツールであることがよくあります。
通常、`--tree-min-percent`または`--tree-max-depth`も指定したいでしょう。
結果は次のようになります:

```bash
$ perf focus '{do_mir_borrowck}' --tree-callees --tree-min-percent 3
Matcher    : {do_mir_borrowck}
Matches    : 577
Not Matches: 746
Percentage : 43%

Tree
| matched `{do_mir_borrowck}` (43% total, 0% self)
: | rustc_borrowck::nll::compute_regions (20% total, 0% self)
: : | rustc_borrowck::nll::type_check::type_check_internal (13% total, 0% self)
: : : | core::ops::function::FnOnce::call_once (5% total, 0% self)
: : : : | rustc_borrowck::nll::type_check::liveness::generate (5% total, 3% self)
: : : | <rustc_borrowck::nll::type_check::TypeVerifier<'a, 'b, 'tcx> as rustc::mir::visit::Visitor<'tcx>>::visit_mir (3% total, 0% self)
: | rustc::mir::visit::Visitor::visit_mir (8% total, 6% self)
: | <rustc_borrowck::MirBorrowckCtxt<'cx, 'tcx> as rustc_mir_dataflow::DataflowResultsConsumer<'cx, 'tcx>>::visit_statement_entry (5% total, 0% self)
: | rustc_mir_dataflow::do_dataflow (3% total, 0% self)
```

`--tree-callees`で何が起こるかというと:

- 正規表現に一致する各サンプルを見つけます
- 正規表現マッチの*後*に発生するコードを見て、
  コールツリーを構築しようとします

`--tree-min-percent 3`オプションは、「時間の3%以上を占めるものだけを表示する」
という意味です。これがないと、ツリーは非常にノイズが多くなり、
mallocの内部のようなランダムなものが含まれることがよくあります。
`--tree-max-depth`も便利です。これは単に印刷するレベル数を制限します。

各行について、その関数での全体的な時間の割合(「total」)と、
**その関数だけで費やされ、その関数の呼び出し先では費やされていない**時間の割合(self)を
表示します。通常、「total」がより興味深い数値ですが、常にそうとは限りません。

### 相対的なパーセンテージ

デフォルトでは、perf-focusのすべては**プログラム全体の実行**に対して相対的です。
これは、視点を保つのに役立ちます -- ホットスポットを見つけるために掘り下げていくと、
プログラム全体の実行という観点から見れば、この「ホットスポット」は実際には
重要ではないという事実を見失うことがよくあります。また、
異なるクエリ間のパーセンテージを簡単に比較できるようにします。

とはいえ、相対的なパーセンテージが役立つこともあるため、`perf
focus`には`--relative`オプションがあります。この場合、パーセンテージは
一致するサンプル(すべてのサンプルに対して)についてのみリストされます。
たとえば、borrowck自体に対する相対的なパーセンテージを次のように取得できます:

```bash
$ perf focus '{do_mir_borrowck}' --tree-callees --relative --tree-max-depth 1 --tree-min-percent 5
Matcher    : {do_mir_borrowck}
Matches    : 577
Not Matches: 746
Percentage : 100%

Tree
| matched `{do_mir_borrowck}` (100% total, 0% self)
: | rustc_borrowck::nll::compute_regions (47% total, 0% self) [...]
: | rustc::mir::visit::Visitor::visit_mir (19% total, 15% self) [...]
: | <rustc_borrowck::MirBorrowckCtxt<'cx, 'tcx> as rustc_mir_dataflow::DataflowResultsConsumer<'cx, 'tcx>>::visit_statement_entry (13% total, 0% self) [...]
: | rustc_mir_dataflow::do_dataflow (8% total, 1% self) [...]
```

ここで、`compute_regions`が「47% total」として表示されていることがわかります
-- これは、`do_mir_borrowck`の47%がその関数で費やされていることを意味します。
以前は20%でした -- これは`do_mir_borrowck`自体が全体時間の43%に過ぎないためです
(そして`.47 * .43 = .20`)。
