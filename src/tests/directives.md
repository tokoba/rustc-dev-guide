# Compiletestディレクティブ

<!--
FIXME(jieyouxu) この章を完全に改訂する。
-->

ディレクティブは、compiletestにテストをビルドして解釈する方法を指示する特別なコメントです。
これらは`rmake.rs` [run-makeテスト](compiletest.md#run-make-tests)にも表示される可能性があります。

これらは通常、このテストの要点を説明する短いコメントの後に配置されます。Compiletestテストスイートは、コメントがディレクティブであることを示すために`//@`を使用します。
例えば、このテストは`//@ compile-flags`コマンドを使用して、テストがコンパイルされるときにrustcに渡すカスタムフラグを指定します：

```rust,ignore
// Test the behavior of `0 - 1` when overflow checks are disabled.

//@ compile-flags: -C overflow-checks=off

fn main() {
    let x = 0 - 1;
    ...
}
```

ディレクティブはスタンドアロン（`//@ run-pass`のように）または値を取る（`//@
compile-flags: -C overflow-checks=off`のように）ことができます。

ディレクティブは1行に1つのディレクティブで記述されます：同じ行に複数のディレクティブを記述することはできません。例えば、`//@ only-x86
only-windows`と書くと、`only-windows`はコメントとして解釈され、別のディレクティブとしては解釈されません。

## Compiletestディレクティブのリスト

以下は、compiletestディレクティブのリストです。利用可能な場合は、コマンドをより詳細に説明するセクションにディレクティブがリンクされています。このリストは網羅的ではない可能性があります。ディレクティブは一般的に、compiletestソースの[`directives.rs`]にある`TestProps`構造体を参照することで見つけることができます。

[`directives.rs`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/compiletest/src/directives.rs

### アセンブリ

<!-- date-check: Oct 2024 -->

| ディレクティブ         | 説明                   | サポートされているテストスイート | 可能な値                        |
|-------------------|-------------------------------|-----------------------|----------------------------------------|
| `assembly-output` | チェックするアセンブリ出力の種類 | `assembly`            | `emit-asm`, `bpf-linker`, `ptx-linker` |

### 補助ビルド

[Building auxiliary crates](compiletest.html#building-auxiliary-crates)を参照

| ディレクティブ             | 説明                                                                                           | サポートされているテストスイート                  | 可能な値                               |
|-----------------------|-------------------------------------------------------------------------------------------------------|----------------------------------------|-----------------------------------------------|
| `aux-bin`             | 補助バイナリをビルドし、テストディレクトリ相対の`auxiliary/bin`で利用可能にする                      | `run-make`/`run-make-cargo`以外のすべて | 補助`.rs`ファイルへのパス                  |
| `aux-build`           | 指定されたソースファイルから別のcrateをビルド                                                     | `run-make`/`run-make-cargo`以外のすべて | 補助`.rs`ファイルへのパス                  |
| `aux-crate`           | `aux-build`と同様だが、extern preludeとして使用可能にする                                                | `run-make`/`run-make-cargo`以外のすべて | `<extern_prelude_name>=<path/to/aux/file.rs>` |
| `aux-codegen-backend` | `aux-build`と同様だが、コンパイル済みdylibをメインファイルのビルド時に`-Zcodegen-backend`に渡す | `ui-fulldeps`                          | codegenバックエンドファイルへのパス                  |
| `proc-macro`          | `aux-build`と同様だが、補助に対してhostを強制し、`-Cprefer-dynamic`を使用しない[^pm]。                | `run-make`/`run-make-cargo`以外のすべて | 補助proc-macro `.rs`ファイルへのパス       |
| `build-aux-docs`      | 補助のドキュメントもビルドします。注：これは`aux-build`でのみ機能し、`aux-crate`では機能しません。     | `run-make`/`run-make-cargo`以外のすべて | N/A                                           |

[^pm]: 詳細については、compiletestの章の[Auxiliary proc-macroセクション](compiletest.html#auxiliary-proc-macro)を参照してください。

### 結果の期待値の制御

[Controlling pass/fail
expectations](ui.md#controlling-passfail-expectations)を参照。

| ディレクティブ                   | 説明                                 | サポートされているテストスイート                     | 可能な値 |
|-----------------------------|---------------------------------------------|-------------------------------------------|-----------------|
| `check-pass`                | ビルド（codegenなし）は合格する必要がある           | `ui`, `crashes`, `incremental`            | N/A             |
| `check-fail`                | ビルド（codegenなし）は失敗する必要がある           | `ui`, `crashes`                           | N/A             |
| `build-pass`                | ビルドは合格する必要がある                        | `ui`, `crashes`, `codegen`, `incremental` | N/A             |
| `build-fail`                | ビルドは失敗する必要がある                        | `ui`, `crashes`                           | N/A             |
| `run-pass`                  | プログラムはコード`0`で終了する必要がある             | `ui`, `crashes`, `incremental`            | N/A             |
| `run-fail`                  | プログラムはコード`1..=127`で終了する必要がある       | `ui`, `crashes`                           | N/A             |
| `run-crash`                 | プログラムはクラッシュする必要がある                          | `ui`                                      | N/A             |
| `run-fail-or-crash`         | プログラムは`run-fail`または`run-crash`する必要がある      | `ui`                                      | N/A             |
| `ignore-pass`               | `--pass`フラグを無視する                        | `ui`, `crashes`, `codegen`, `incremental` | N/A             |
| `dont-check-failure-status` | 正確な失敗ステータス（つまり`1`）をチェックしない | `ui`, `incremental`                       | N/A             |
| `failure-status`            | チェック                                       | `ui`, `crashes`                           | 任意の`u16`       |
| `should-ice`                | 失敗ステータスが`101`であることをチェック               | `coverage`, `incremental`                 | N/A             |
| `should-fail`               | Compiletestセルフテスト                       | すべて                                       | N/A             |

### 出力スナップショットと正規化の制御

詳細については、[Normalization](ui.md#normalization)、[Output
comparison](ui.md#output-comparison)、[Rustfix tests](ui.md#rustfix-tests)を参照してください。

| ディレクティブ                         | 説明                                                                                                              | サポートされているテストスイート                        | 可能な値                                                                         |
|-----------------------------------|--------------------------------------------------------------------------------------------------------------------------|----------------------------------------------|-----------------------------------------------------------------------------------------|
| `check-run-results`               | テストバイナリ`run-{pass,fail}`出力スナップショットの実行をチェック                                                                  | `ui`, `crashes`, `incremental` （`run-pass`の場合） | N/A                                                                                     |
| `error-pattern`                   | 出力に特定の文字列が含まれていることをチェック                                                                             | `ui`, `crashes`, `incremental` （`run-pass`の場合） | 文字列                                                                                  |
| `regex-error-pattern`             | 出力に正規表現パターンが含まれていることをチェック                                                                               | `ui`, `crashes`, `incremental` （`run-pass`の場合） | 正規表現                                                                                   |
| `check-stdout`                    | テストバイナリの実行からの`stdout`を`error-pattern`に対してチェック[^check_stdout]                                          | `ui`, `crashes`, `incremental`               | N/A                                                                                     |
| `normalize-stderr-32bit`          | スナップショットと比較する前に、実際のstderr（32ビットプラットフォーム用）を`"<raw>" -> "<normalized>"`ルールで正規化 | `ui`, `incremental`                          | `"<RAW>" -> "<NORMALIZED>"`、`<RAW>`/`<NORMALIZED>`は正規表現キャプチャと置換構文 |
| `normalize-stderr-64bit`          | スナップショットと比較する前に、実際のstderr（64ビットプラットフォーム用）を`"<raw>" -> "<normalized>"`ルールで正規化 | `ui`, `incremental`                          | `"<RAW>" -> "<NORMALIZED>"`、`<RAW>`/`<NORMALIZED>`は正規表現キャプチャと置換構文 |
| `normalize-stderr`                | スナップショットと比較する前に、実際のstderrを`"<raw>" -> "<normalized>"`ルールで正規化                        | `ui`, `incremental`                          | `"<RAW>" -> "<NORMALIZED>"`、`<RAW>`/`<NORMALIZED>`は正規表現キャプチャと置換構文 |
| `normalize-stdout`                | スナップショットと比較する前に、実際のstdoutを`"<raw>" -> "<normalized>"`ルールで正規化                        | `ui`, `incremental`                          | `"<RAW>" -> "<NORMALIZED>"`、`<RAW>`/`<NORMALIZED>`は正規表現キャプチャと置換構文 |
| `dont-check-compiler-stderr`      | 実際のコンパイラstderrとstderrスナップショットをチェックしない                                                                    | `ui`                                         | N/A                                                                                     |
| `dont-check-compiler-stdout`      | 実際のコンパイラstdoutとstdoutスナップショットをチェックしない                                                                    | `ui`                                         | N/A                                                                                     |
| `dont-require-annotations`        | 指定された診断種類（`//~ KIND`）の行注釈が網羅的であることを要求しない                               | `ui`, `incremental`                          | `ERROR`, `WARN`, `NOTE`, `HELP`, `SUGGESTION`                                           |
| `run-rustfix`                     | すべての提案を`rustfix`経由で適用し、修正された出力をスナップショットし、修正された出力がビルドされることをチェック                                | `ui`                                         | N/A                                                                                     |
| `rustfix-only-machine-applicable` | `run-rustfix`だが、機械適用可能な提案のみ                                                                    | `ui`                                         | N/A                                                                                     |
| `exec-env`                        | テストを実行するときに設定する環境変数                                                                                     | `ui`, `crashes`                              | `<KEY>=<VALUE>`                                                                         |
| `unset-exec-env`                  | テストを実行するときに設定を解除する環境変数                                                                                   | `ui`, `crashes`                              | 任意の環境変数名                                                                        |
| `stderr-per-bitwidth`             | 各ビット幅のstderrスナップショットを生成                                                                             | `ui`                                         | N/A                                                                                     |
| `forbid-output`                   | stderrや`cfail`出力に表示されてはならないパターン                                                                 | `ui`, `incremental`                          | 正規表現パターン                                                                           |
| `run-flags`                       | テスト実行可能ファイルに渡されるフラグ                                                                                      | `ui`                                         | 任意のフラグ                                                                         |
| `known-bug`                       | 既知のバグのため、エラー注釈は不要                                                                              | `ui`, `crashes`, `incremental`               | issue番号`#123456`                                                                  |
| `compare-output-by-lines`         | 出力を単一の文字列としてではなく、行ごとに比較                                                              | すべて                                          | N/A                                                                                     |

[^check_stdout]: 現在<!-- date-check: Oct 2024 -->これには奇妙な癖があり、テストバイナリのstdoutとstderrが連結され、この結合された出力で`error-pattern`がマッチされます。これは少なくとも疑わしいです。

### テストの実行タイミングの制御

これらのディレクティブは、いくつかの状況でテストを無視するために使用されます。これは、テストがコンパイルまたは実行されないことを意味します。

* `ignore-X`、ここで`X`はターゲットの詳細またはテストを無視する他の基準です（以下を参照）
* `only-X`は`ignore-X`に似ていますが、そのターゲットまたはステージでテストを実行*のみ*します
* `ignore-auxiliary`は、1つ以上の他のメインテストファイルに*参加*するファイルを対象としていますが、`compiletest`がファイル自体をビルドしようとするべきではありません。実際に補助ファイルを使用しているメインテストへのバックリンクを含めてください。
* `ignore-test`は常にテストを無視します。これは、テストが現在機能していない場合に一時的にテストを無効にするために使用できますが、後で再度有効にするためにツリーに保持したい場合に使用できます。

`ignore-X`または`only-X`の`X`の例：

* 完全なターゲットトリプル：`aarch64-apple-ios`
* アーキテクチャ：`aarch64`, `arm`, `mips`, `wasm32`, `x86_64`, `x86`,
  ...
* OS：`android`, `emscripten`, `freebsd`, `ios`, `linux`, `macos`, `windows`,
  ...
* 環境（ターゲットトリプルの4番目の単語）：`gnu`, `msvc`, `musl`
* ポインタ幅：`32bit`, `64bit`
* エンディアン：`endian-big`
* ステージ：`stage1`, `stage2`
* バイナリフォーマット：`elf`
* チャンネル：`stable`, `beta`
* クロスコンパイル時：`cross-compile`
* [リモートテスト]が使用される場合：`remote`
* 特定のデバッガがテストされる場合：`cdb`, `gdb`, `lldb`
* 特定のデバッガバージョンが一致する場合：`ignore-gdb-version`
* 特定の[比較モード]：`compare-mode-polonius`, `compare-mode-chalk`,
  `compare-mode-split-dwarf`, `compare-mode-split-dwarf-single`
* カバレッジテストで使用される2つの異なるテストモード：
  `ignore-coverage-map`, `ignore-coverage-run`
* distツールチェーンをテストする場合：`dist`
  * これは`COMPILETEST_ENABLE_DIST_TESTS=1`で有効にする必要があります
* ターゲットの`rustc_abi`：例：`rustc_abi-x86_64-sse2`

次のディレクティブは、rustcビルド設定とターゲット設定をチェックします：

* `needs-asm-support` — **ホスト**アーキテクチャが`asm!`の安定サポートを持たない場合に無視します。`--target`経由で明示的なターゲットにクロスコンパイルするテストの場合は、代わりに`needs-llvm-components`を使用して、適切なバックエンドが利用可能であることを確認してください。
* `needs-profiler-runtime` — プロファイラランタイムがターゲットで有効になっていない場合、テストを無視します
  （rustcの`bootstrap.toml`の`build.profiler = true`）
* `needs-sanitizer-support` — サニタイザサポートがターゲットで有効になっていない場合に無視します（rustcの`bootstrap.toml`の`sanitizers = true`）
* `needs-sanitizer-{address,hwaddress,leak,memory,thread}` — 対応するサニタイザがターゲットで有効になっていない場合に無視します（AddressSanitizer、ハードウェア支援AddressSanitizer、LeakSanitizer、MemorySanitizer、ThreadSanitizerのいずれか）
* `needs-run-enabled` — 実行されるテストで、実行が無効になっている場合に無視します。テストの実行は、`x test --run=never`フラグで無効にするか、fuchsiaで実行することで無効にできます。
* `needs-unwind` — ターゲットがアンワインドサポートを持たない場合に無視します
* `needs-rust-lld` — rust lldサポートが有効になっていない場合に無視します（`bootstrap.toml`の`rust.lld = true`）
* `needs-threads` — ターゲットがスレッドサポートを持たない場合に無視します
* `needs-subprocess`  — ターゲットがサブプロセスサポートを持たない場合に無視します
* `needs-symlink` — ターゲットがシンボリックリンクをサポートしていない場合に無視します。これは、開発者が特権シンボリックリンク権限を有効にしていない場合、Windowsで当てはまる可能性があります。
* `ignore-std-debug-assertions` — stdがデバッグアサーション付きでビルドされている場合に無視します。
* `needs-std-debug-assertions` — stdがデバッグアサーションなしでビルドされている場合に無視します。
* `ignore-rustc-debug-assertions` — rustcがデバッグアサーション付きでビルドされている場合に無視します。
* `needs-rustc-debug-assertions` — rustcがデバッグアサーションなしでビルドされている場合に無視します。
* `needs-target-has-atomic` — ターゲットが指定されたすべてのアトミック幅のサポートを持たない場合に無視します。例えば、`//@ needs-target-has-atomic: 8,
  16, ptr`を含むテストは、カンマ区切りのアトミック幅リストをサポートしている場合にのみ実行されます。
* `needs-dynamic-linking` — ターゲットが動的リンクをサポートしていない場合に無視します
  （`dylib`および`cdylib`クレートタイプを作成できないこととは直交しています）
* `needs-crate-type` — ターゲットプラットフォームが、カンマ区切りで指定された1つ以上のクレートタイプをサポートしていない場合に無視します。例えば、
  `//@ needs-crate-type: cdylib, proc-macro`は、ターゲットが`proc-macro`クレートタイプをサポートしていないため、`wasm32-unknown-unknown`ターゲットでテストを無視します。
* `needs-target-std` — ターゲットプラットフォームがstdサポートを持たない場合に無視します。
* `ignore-backends` — 空白文字で区切られた、リストされたバックエンドを無視します。このディレクティブは`--bypass-ignore-backends=[BACKEND]`コマンドラインフラグで上書きできることに注意してください。
* `needs-backends` — 現在のcodegenバックエンドがリストされている場合にのみテストを実行します。

次のディレクティブはLLVMサポートをチェックします：

* `exact-llvm-major-version: 19` — llvmメジャーバージョンが指定されたllvmメジャーバージョンと一致しない場合に無視します。
* `min-llvm-version: 13.0` — LLVMバージョンが指定された値より低い場合に無視します
* `min-system-llvm-version: 12.0` — システムLLVMを使用していて、そのバージョンが指定された値より低い場合に無視します
* `max-llvm-major-version: 19` — LLVMメジャーバージョンが指定されたメジャーバージョンより高い場合に無視します
* `ignore-llvm-version: 9.0` — 特定のLLVMバージョンを無視します
* `ignore-llvm-version: 7.0 - 9.9.9` — 範囲内（両端を含む）のLLVMバージョンを無視します
* `needs-llvm-components: powerpc` — 特定のLLVMコンポーネントがビルドされていない場合に無視します。注：コンポーネントが存在しない場合、CI（`COMPILETEST_REQUIRE_ALL_LLVM_COMPONENTS`が設定されている場合）でテストは失敗します。
* `needs-forced-clang-based-tests` — 環境変数`RUSTBUILD_FORCE_CLANG_BASED_TESTS`が設定されていない限り、テストは無視されます。これにより、LLVMと一緒にclangをビルドできます
  * これは2つのCIジョブ（[`x86_64-gnu-debug`]と
    [`aarch64-gnu-debug`]）でのみ設定され、`run-make`テストのサブセットのみを実行します。このディレクティブを持つ他のテストはまったく実行されません。これは通常、望ましいことではありません。

デバッガを無視するためのディレクティブについては、[Debuginfo tests](compiletest.md#debuginfo-tests)も参照してください。

[`x86_64-gnu-debug`]: https://github.com/rust-lang/rust/blob/ab3dba92db355b8d97db915a2dca161a117e959c/src/ci/docker/host-x86_64/x86_64-gnu-debug/Dockerfile#L32
[`aarch64-gnu-debug`]: https://github.com/rust-lang/rust/blob/20c909ff9cdd88d33768a4ddb8952927a675b0ad/src/ci/docker/host-aarch64/aarch64-gnu-debug/Dockerfile#L32

### テストのビルド方法への影響

| ディレクティブ           | 説明                                                                                  | サポートされているテストスイート                      | 可能な値                                                                            |
|---------------------|----------------------------------------------------------------------------------------------|--------------------------------------------|---------------------------------------------------------------------------------------------|
| `compile-flags`     | テストまたは補助ファイルをビルドするときに`rustc`に渡されるフラグ                                   | `run-make`/`run-make-cargo`以外のすべて | 任意の有効な`rustc`フラグ、例：`-Awarnings -Dfoo`。`-Cincremental`または`--edition`は不可 |
| `edition`           | テストのビルドに使用されるエディション                                                           | `run-make`/`run-make-cargo`以外のすべて | 任意の有効な`--edition`値                                                                |
| `rustc-env`         | `rustc`を実行するときに設定する環境変数                                                          | `run-make`/`run-make-cargo`以外のすべて | `<KEY>=<VALUE>`                                                                            |
| `unset-rustc-env`   | `rustc`を実行するときに設定を解除する環境変数                                                        | `run-make`/`run-make-cargo`以外のすべて | 任意の環境変数名                                                                           |
| `incremental`       | インクリメンタルテストスイート外のテストに対する適切なインクリメンタルサポート                       | `ui`, `crashes`                            | N/A                                                                                        |
| `no-prefer-dynamic` | `-C prefer-dynamic`を使用せず、`--crate-type=dylib`プリセットフラグ経由でdylibとしてビルドしない | `ui`, `crashes`                            | N/A                                                                                        |

<div class="warning">

インクリメンタルテストスイートにないインクリメンタルテストを使用したい（`run-make`/`run-make-cargo`以外の）テストは、`compile-flags`経由で`-C incremental`を渡してはならず、代わりに`//@ incremental`ディレクティブを使用する必要があります。

代わりに、テストを適切なインクリメンタルテストとして書くことを検討してください。

</div>

#### editionディレクティブ

`//@ edition`ディレクティブは、正確なエディション、エディションの有界範囲、またはエディションの左有界半開範囲を取ることができます。
これは、`./x test`がテストを実行するために使用するエディションに影響します。

例：

* `//@ edition: 2018`ディレクティブを持つテストは、2018エディションの下でのみ実行されます。
* `//@ edition: 2015..2021`ディレクティブを持つテストは、2015、2018、および2021エディションの下で実行できます。
  ただし、CIは範囲内の最低エディション（この例では2015）でのみテストを実行します。
* `//@ edition: 2018..`ディレクティブを持つテストは、2018エディション以上で実行されます。
  ただし、CIは範囲内の最低エディション（この例では2018）でのみテストを実行します。

`-- --edition=`引数を渡すことで、`./x test`に特定のエディションを使用させることもできます。
ただし、`//@ edition`ディレクティブを持つテストは、引数に渡された値をクランプします。
例えば、`./x test -- --edition=2015`を実行する場合：

* `//@ edition: 2018`を持つテストは、2018エディションで実行されます。
* `//@ edition: 2015..2021`を持つテストは、2015エディションで実行されます。
* `//@ edition: 2018..`を持つテストは、2018エディションで実行されます。

### Rustdoc

| ディレクティブ   | 説明                                                  | サポートされているテストスイート                   | 可能な値           |
|-------------|--------------------------------------------------------------|---------------------------------------|---------------------------|
| `doc-flags` | テストまたは補助ファイルをビルドするときに`rustdoc`に渡されるフラグ | `rustdoc`, `rustdoc-js`, `rustdoc-json` | 任意の有効な`rustdoc`フラグ |

<!--
**FIXME(rustdoc)**: `check-test-line-numbers-match`は何をしますか？
<https://rust-lang.zulipchat.com/#narrow/stream/266220-t-rustdoc/topic/What.20is.20the.20.60check-test-line-numbers-match.60.20directive.3F>で質問しました。
-->

#### テストスイート固有のディレクティブ

テストスイート[`rustdoc`][rustdoc-html-tests]、[`rustdoc-js`/`rustdoc-js-std`][rustdoc-js-tests]、[`rustdoc-json`][rustdoc-json-tests]は、基本的な構文がcompiletestディレクティブのものに似ているが、最終的には別々のツールによって読み取られてチェックされる追加のディレクティブセットをそれぞれ備えています。詳細については、上記にリンクされているそれぞれの章を参照してください。

[rustdoc-html-tests]: ../rustdoc-internals/rustdoc-test-suite.md
[rustdoc-js-tests]: ../rustdoc-internals/search.html#testing-the-search-engine
[rustdoc-json-tests]: ../rustdoc-internals/rustdoc-json-test-suite.md

### プリティプリンティング

[Pretty-printer](compiletest.md#pretty-printer-tests)を参照。

#### その他のディレクティブ

* `no-auto-check-cfg` — 自動check-cfgを無効にする（`--check-cfg`テストのみ）
* [`revisions`](compiletest.md#revisions) — 複数回コンパイル
-[`forbid-output`](compiletest.md#incremental-tests) — インクリメンタルcfailは出力パターンを拒否
* [`should-ice`](compiletest.md#incremental-tests) — インクリメンタルcfailはICEする必要がある
* [`reference`] — リファレンスのルールへのリンク注釈
* `disable-gdb-pretty-printers` — debuginfoテスト用のgdbプリティプリンタを無効にする

[`reference`]: https://github.com/rust-lang/reference/blob/master/docs/authoring.md#test-rule-annotations

### ツール固有のディレクティブ

次のディレクティブは、これらのツールを使用するテストスイートで、特定のコマンドラインツールの呼び出し方法に影響します：

* `filecheck-flags`は、LLVMの`FileCheck`ツールを実行するときに追加のフラグを追加します。
  * [codegenテスト](compiletest.md#codegen-tests)、
  [assemblyテスト](compiletest.md#assembly-tests)、
  [MIR-optテスト](compiletest.md#mir-opt-tests)で使用されます。
* `llvm-cov-flags`は、LLVMの`llvm-cov`ツールを実行するときに追加のフラグを追加します。
  * `coverage-run`モードの[coverageテスト](compiletest.md#coverage-tests)で使用されます。

### Tidy固有のディレクティブ

次のディレクティブは、[tidyスクリプト](../conventions.md#formatting)がテストを検証する方法を制御します。

* `ignore-tidy-target-specific-tests`は、テストが特定のターゲット用にコンパイルされる場合（`compile-flag`ディレクティブの`--target`フラグ経由）に、適切なLLVMコンポーネントが必要であること（`needs-llvm-components`ディレクティブ経由）のチェックを無効にします。
* [`unused-revision-names`](compiletest.md#ignoring-unused-revision-names) -
      未知のリビジョン名の言及に対するtidyチェックを抑制します。

## 置換

ディレクティブの値は、対応する値に置き換えられるいくつかの変数の置換をサポートしています。例えば、特定のファイルへのパスを使用してコンパイラフラグを渡す必要がある場合、次のようなものが機能する可能性があります：

```rust,ignore
//@ compile-flags: --remap-path-prefix={{src-base}}=/the/src
```

ここで、センチネル`{{src-base}}`は、以下に説明する適切なパスに置き換えられます：

* `{{cwd}}`：compiletestが実行されるディレクトリ。これはチェックアウトのルートではない可能性があるため、可能な限り使用を避ける必要があります。
  * 例：`/path/to/rust`, `/path/to/build/root`
* `{{src-base}}`：テストが定義されているディレクトリ。これは[出力正規化]の`$DIR`と同等です。
  * 例：`/path/to/rust/tests/ui/error-codes`
* `{{build-base}}`：テストの出力が格納されるベースディレクトリ。これは[出力正規化]の`$TEST_BUILD_DIR`と同等です。
  * 例：`/path/to/rust/build/x86_64-unknown-linux-gnu/test/ui`
* `{{rust-src-base}}`：libstd/libcore/...が配置されているsysrootディレクトリ
* `{{sysroot-base}}`：テストのビルドに使用されるsysrootディレクトリのパス。
  * 主に、API経由でコンパイラを実行する`ui-fulldeps`テストを対象としています。
* `{{target-linker}}`：このテストのために`-Clinker`に渡されるリンカ。リンカのオーバーライドがアクティブでない場合は空白です。
  * 主に、API経由でコンパイラを実行する`ui-fulldeps`テストを対象としています。
* `{{target}}`：テストがコンパイルされるターゲット
  * 例：`x86_64-unknown-linux-gnu`

この置換を使用するテストの例については、
[`tests/ui/argfile/commandline-argfile.rs`](https://github.com/rust-lang/rust/blob/HEAD/tests/ui/argfile/commandline-argfile.rs)を参照してください。


## ディレクティブの追加

テストプロパティや動作を個々のテストごとに定義する必要がある場合、新しいディレクティブを追加します。ディレクティブプロパティは、実行時にディレクティブのバッキングストア（コマンドの現在の値を保持）として機能します。

新しいディレクティブプロパティを追加するには：

1. [`src/tools/compiletest/src/directives.rs`]の`pub struct TestProps`宣言を探し、新しいパブリックプロパティを宣言の最後に追加します。
2. 構造体宣言の直後の`impl TestProps`実装ブロックを探し、新しいプロパティをデフォルト値に初期化します。

### 新しいディレクティブパーサーの追加

`compiletest`がテストファイルに遭遇すると、ファイルを1行ずつ解析し、同じく[`src/tools/compiletest/src/directives.rs`]にある`Config`構造体の実装ブロックで定義されたすべてのパーサーを呼び出します（`Config`構造体の宣言ブロックは[`src/tools/compiletest/src/common.rs`]にあります）。
`TestProps`の`load_from()`メソッドは、現在のテキスト行を各パーサーに渡そうとします。各パーサーは、行が`//@ must-compile-successfully`や`//@ failure-status`のような特定のコメント付き（`//@`）ディレクティブで始まるかどうかをチェックします。コメントマーカーの後の空白はオプションです。

パーサーは、テストファイルでディレクティブとして指定されるか、テストファイルでパラメータ値が指定されることで、指定されたディレクティブプロパティのデフォルト値を上書きします（ディレクティブによって異なります）。

`impl Config`で定義されたパーサーは、通常`parse_<directive-name>`という名前です
（kebab-caseの`<directive-command>`がsnake_caseの`<directive_command>`に変換されることに注意してください）。`impl Config`は、単純な存在または非存在（`parse_name_directive()`）、`directive:parameter(s)`
（`parse_name_value_directive()`）、特定の`cfg`属性が定義されている場合のみのオプションの解析（`has_cfg_prefix()`）など、一般的なパターンを簡単に解析できるいくつかの「低レベル」パーサーも定義しています。低レベルのパーサーは、`impl Config`ブロックの終わり近くにあります。それらとその関連するパーサーをすぐ上で確認して、不必要に追加の解析コードを書くことを避けるために、どのように使用されているかを確認してください。

具体的な例として、[`src/tools/compiletest/src/directives.rs`]の
`parse_failure_status()`パーサーの実装を以下に示します：

```diff
@@ -232,6 +232,7 @@ pub struct TestProps {
     // customized normalization rules
     pub normalize_stdout: Vec<(String, String)>,
     pub normalize_stderr: Vec<(String, String)>,
+    pub failure_status: i32,
 }

 impl TestProps {
@@ -260,6 +261,7 @@ impl TestProps {
             run_pass: false,
             normalize_stdout: vec![],
             normalize_stderr: vec![],
+            failure_status: 101,
         }
     }

@@ -383,6 +385,10 @@ impl TestProps {
             if let Some(rule) = config.parse_custom_normalization(ln, "normalize-stderr") {
                 self.normalize_stderr.push(rule);
             }
+
+            if let Some(code) = config.parse_failure_status(ln) {
+                self.failure_status = code;
+            }
         });

         for key in &["RUST_TEST_NOCAPTURE", "RUST_TEST_THREADS"] {
@@ -488,6 +494,13 @@ impl Config {
         self.parse_name_directive(line, "pretty-compare-only")
     }

+    fn parse_failure_status(&self, line: &str) -> Option<i32> {
+        match self.parse_name_value_directive(line, "failure-status") {
+            Some(code) => code.trim().parse::<i32>().ok(),
+            _ => None,
+        }
+    }
```

### 動作変更の実装

テストが特定のディレクティブを呼び出すと、その結果として何らかの動作が変更されることが期待されます。どのような動作が変更されるかは、明らかにディレクティブの目的に依存します。`failure-status`の場合、変更される動作は、`compiletest`がデフォルト値ではなく、テストで呼び出されたディレクティブによって定義された失敗コードを期待することです。

`failure-status`に固有（すべてのディレクティブは動作変更を呼び出すために異なる実装を持つため）ですが、おそらく1つのケースの動作変更実装を見ることは、単に例として役立つかもしれません。`failure-status`を実装するために、[`src/tools/compiletest/src/runtest.rs`]にある`TestCx`実装ブロックにある`check_correct_failure_status()`関数が以下のように変更されました：

```diff
@@ -295,11 +295,14 @@ impl<'test> TestCx<'test> {
     }

     fn check_correct_failure_status(&self, proc_res: &ProcRes) {
-        // The value the Rust runtime returns on failure
-        const RUST_ERR: i32 = 101;
-        if proc_res.status.code() != Some(RUST_ERR) {
+        let expected_status = Some(self.props.failure_status);
+        let received_status = proc_res.status.code();
+
+        if expected_status != received_status {
             self.fatal_proc_rec(
-                &format!("failure produced the wrong error: {}", proc_res.status),
+                &format!("Error: expected failure status ({:?}) but received status {:?}.",
+                         expected_status,
+                         received_status),
                 proc_res,
             );
         }
@@ -320,7 +323,6 @@ impl<'test> TestCx<'test> {
         );

         let proc_res = self.exec_compiled_test();
-
         if !proc_res.status.success() {
             self.fatal_proc_rec("test run failed!", &proc_res);
         }
@@ -499,7 +501,6 @@ impl<'test> TestCx<'test> {
                 expected,
                 actual
             );
-            panic!();
         }
     }
```

`self.props.failure_status`を使用してディレクティブプロパティにアクセスすることに注意してください。失敗ステータスディレクティブを指定しないテストでは、`self.props.failure_status`は、この記事の執筆時点でのデフォルト値101に評価されます。しかし、例えば`//@ failure-status: 1`というディレクティブを指定するテストの場合、`self.props.failure_status`は1に評価されます。これは、`parse_failure_status()`がそのテスト専用に`TestProps`のデフォルト値を上書きしたためです。

[`src/tools/compiletest/src/directives.rs`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/compiletest/src/directives.rs
[`src/tools/compiletest/src/common.rs`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/compiletest/src/common.rs
[`src/tools/compiletest/src/runtest.rs`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/compiletest/src/runtest.rs
