# LLVMソースベースのコードカバレッジ

`rustc`は、コンパイル時にRustライブラリとバイナリに追加の命令とデータを使用して、
詳細なソースベースのコードとテストカバレッジ分析をサポートします。
これには、コマンドラインオプション（`-C instrument-coverage`）を使用します。

カバレッジ計装は、LLVMの組み込み命令
[`llvm.instrprof.increment`][llvm-instrprof-increment]への呼び出しを
コードブランチ（MIRベースの制御フロー分析に基づく）に挿入し、
LLVMはこれらを実行時に静的カウンターをインクリメントする命令に変換します。
LLVM カバレッジ計装には、ソースメタデータをエンコードする[Coverage Map]も必要です。
これは、カウンターIDを直接および間接的にファイルの場所（開始行と終了行および列を含む）にマッピングします。

カバレッジ計装の有無にかかわらず、Rustライブラリは
計装されたバイナリにリンクできます。プログラムが実行されてクリーンに終了すると、
LLVMライブラリは最終的なカウンター値をファイル（`default.profraw`または
環境変数`LLVM_PROFILE_FILE`を介して設定されたカスタムファイル）に書き込みます。

開発者は、既存のLLVMカバレッジ分析ツールを使用して、
`.profraw`ファイルを、それらを生成した対応するCoverage Map（一致するバイナリから）と共にデコードし、
分析のためのさまざまなレポートを生成します。例えば：

<img alt="Screenshot of sample `llvm-cov show` result, for function add_quoted_string"
 src="img/llvm-cov-show-01.png" class="center"/>
<br/>

詳細な手順と例は、
[rustcブック][rustc-book-instrument-coverage]に文書化されています。

[llvm-instrprof-increment]: https://llvm.org/docs/LangRef.html#llvm-instrprof-increment-intrinsic
[coverage map]: https://llvm.org/docs/CoverageMappingFormat.html
[rustc-book-instrument-coverage]: https://doc.rust-lang.org/nightly/rustc/instrument-coverage.html

## 推奨される`bootstrap.toml`設定

カバレッジ計装コードで作業する場合、通常は
`[build]`で`profiler = true`を設定して**プロファイラランタイムを有効にする**必要があります。
これにより、コンパイラは計装されたバイナリを生成でき、
完全なカバレッジテストスイートを実行できるようになります。

コンパイラとLLVMでデバッグアサーションを有効にすることをお勧めしますが、
必須ではありません。

```toml
# 「compiler」プロファイルに似ていますが、LLVMでデバッグアサーションも有効にします。
# これらのアサーションは、場合によっては不正なカバレッジマッピングを検出できます。
profile = "codegen"

[build]
# 重要：これにより、ビルドシステムにLLVMプロファイラランタイムをビルドするように指示します。
# これがないと、コンパイラはカバレッジ計装されたバイナリを生成できず、
# 多くのカバレッジテストがスキップされます。
profiler = true

[rust]
# コンパイラでデバッグアサーションを有効にします。
debug-assertions = true
```

## Rustシンボルマングリング

`-C instrument-coverage`は、
Rustシンボルマングリング`v0`を自動的に有効にします
（ユーザーが`rustc`を呼び出すときに`-C symbol-mangling-version=v0`オプションを指定したかのように）。
これにより、一貫性のある可逆的な名前マングリングが保証されます。これには2つの重要な利点があります。

1. LLVMカバレッジツールは、ソースコードへの一部の変更を含む、複数の実行にわたって
   カバレッジを分析できます。したがって、マングルされた名前はコンパイル間で一貫している必要があります。
2. LLVMカバレッジレポートは、関数ごとにカバレッジを報告でき、
   複数の型置換バリエーションで呼び出された場合でも、
   ジェネリック関数の各一意のインスタンス化のカバレッジカウントを分離します。

## LLVMプロファイラランタイム

カバレッジデータは、実行可能なRustプログラムを実行することによってのみ生成されます。`rustc`は、
カバレッジ計装されたバイナリを、`.profraw`ファイルにカウンター値を書き込むプログラムフック
（`exit`フックなど）を実装するLLVMランタイムコード
([compiler-rt][compiler-rt-profile])と静的にリンクします。

`rustc`ソースツリーでは、
`library/profiler_builtins`がLLVM `compiler-rt`コードをRustライブラリクレートにバンドルします。
`rustc`をビルドする際、
`profiler_builtins`は`build.profiler = true`が`bootstrap.toml`に設定されている場合にのみ含まれることに注意してください。

`-C instrument-coverage`でコンパイルする場合、
[`CrateLoader::postprocess()`][crate-loader-postprocess]は、
`inject_profiler_runtime()`を呼び出すことで`profiler_builtins`を動的にロードします。

[compiler-rt-profile]: https://github.com/llvm/llvm-project/tree/main/compiler-rt/lib/profile
[crate-loader-postprocess]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/creader/struct.CrateLoader.html#method.postprocess

## カバレッジ計装のテスト

[(コンパイルテストのドキュメントで`tests/coverage`
テストスイートも参照してください。)](./tests/compiletest.md#coverage-tests)

MIRのカバレッジ計装は、`mir-opt`テストで検証されます。
[`tests/mir-opt/coverage/instrument_coverage.rs`]。

LLVM IRのカバレッジ計装は、`coverage-map`モードの[`tests/coverage`]
テストスイートによって検証されます。
これらのテストは、テストプログラムをLLVM IRアセンブリにコンパイルし、
次に[`src/tools/coverage-dump`]ツールを使用して、
最終バイナリに埋め込まれるカバレッジマッピングを抽出して
きれいに印刷します。

カバレッジ計装とカバレッジレポートのエンドツーエンドテストは、
`coverage-run`モードの[`tests/coverage`]テストスイートと
[`tests/coverage-run-rustdoc`]テストスイートによって実行されます。
これらのテストは、カバレッジ計装を使用してテストプログラムをコンパイルして実行し、
次にLLVMツールを使用してカバレッジデータを
人間が読める形式のカバレッジレポートに変換します。

> `coverage-run`モードのテストには、暗黙の`//@ needs-profiler-runtime`
> ディレクティブがあるため、プロファイラランタイムが
> [`bootstrap.toml`で有効になっていない](#recommended-configtoml-settings)場合はスキップされます。

最後に、[`tests/codegen-llvm/instrument-coverage/testprog.rs`]テストは、
単純なRustプログラムを`-C instrument-coverage`でコンパイルし、
コンパイルされたプログラムのLLVM IRを、
カバレッジ対応プログラムの期待されるLLVM IR命令と構造化データと比較します。
これには、Coverage Map関連のメタデータとLLVM組み込み呼び出しのさまざまなチェックが含まれ、
ランタイムカウンターをインクリメントします。

`coverage`、`coverage-run-rustdoc`、
および`mir-opt`テストの期待される結果は、次を実行してリフレッシュできます。

```shell
./x test coverage --bless
./x test coverage-run-rustdoc --bless
./x test tests/mir-opt --bless
```

[`tests/mir-opt/coverage/instrument_coverage.rs`]: https://github.com/rust-lang/rust/blob/HEAD/tests/mir-opt/coverage/instrument_coverage.rs
[`tests/coverage`]: https://github.com/rust-lang/rust/tree/HEAD/tests/coverage
[`src/tools/coverage-dump`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/coverage-dump
[`tests/coverage-run-rustdoc`]: https://github.com/rust-lang/rust/tree/HEAD/tests/coverage-run-rustdoc
[`tests/codegen-llvm/instrument-coverage/testprog.rs`]: https://github.com/rust-lang/rust/blob/HEAD/tests/mir-opt/coverage/instrument_coverage.rs
