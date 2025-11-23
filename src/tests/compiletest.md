# Compiletest

## 序論

`compiletest`は、Rustテストスイートのメインテストハーネスです。テスト作成者が大量のテストを整理し（Rustコンパイラには何千ものテストがあります）、効率的なテスト実行（並列実行がサポートされています）を可能にし、テスト作成者が個々のテストやテストグループの動作と期待される結果を設定できるようにします。

> **macOSユーザーへの注意**
>
> macOSユーザーの場合、`SIP`（System Integrity Protection）が[Appleにネットワークリクエストを送信してコンパイル済みバイナリを一貫してチェックする][zulip]可能性があるため、テスト実行時に大幅なパフォーマンス低下が発生する可能性があります。
>
> 以下の設定を調整することで解決できます：`Privacy & Security -> Developer Tools -> Add Terminal (Or VsCode, etc.)`。

[zulip]: https://rust-lang.zulipchat.com/#narrow/stream/182449-t-compiler.2Fhelp/topic/.E2.9C.94.20Is.20there.20any.20performance.20issue.20for.20MacOS.3F

`compiletest`は、コンパイル時または実行時の成功/失敗をテストコードでチェックできます。

テストは通常、テストコードの前や内部にコメントで注釈を付けたRustソースファイルとして整理されます。これらのコメントは、`compiletest`にテストを実行するかどうか、どのように実行するか、どのような動作を期待するかなどを指示する役割を果たします。これらの注釈の詳細については、[directives](directives.md)と以下のテストスイートのドキュメントを参照してください。

新しいテストの作成に関するチュートリアルと良いテストを書くためのアドバイスについては、[Adding new tests](adding.md)と[Best practices](best-practices.md)の章を、テストスイートの実行方法については[Running tests](running.md)の章を参照してください。

引数は`--test-args`を使用するか、`--`の後に配置することでcompiletestに渡すことができます。例：
- `x test --test-args --force-rerun`
- `x test -- --force-rerun`

さらに、bootstrapはいくつかの一般的な引数を直接受け入れます。例：

`x test --no-capture --force-rerun --run --pass`。

Compiletest自体は、関連するアーティファクト（主にコンパイラ）が変更されていない場合、テストの実行を避けようとします。入力が変更されていない場合でもテストを再実行するには、`x test --test-args --force-rerun`を使用できます。

## テストスイート

すべてのテストは[`tests`]ディレクトリにあります。テストは「スイート」に整理されており、各スイートは別々のサブディレクトリにあります。各テストスイートは少し異なる動作をし、異なるコンパイラの動作と正しさのための異なるチェックを行います。例えば、[`tests/incremental`]ディレクトリにはインクリメンタルコンパイルのテストが含まれています。さまざまなスイートは[`src/tools/compiletest/src/common.rs`]の`pub enum Mode`宣言で定義されています。

以下のテストスイートが利用可能で、詳細情報へのリンクがあります：

[`tests`]: https://github.com/rust-lang/rust/blob/HEAD/tests
[`src/tools/compiletest/src/common.rs`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/compiletest/src/common.rs

### コンパイラ固有のテストスイート

| テストスイート                                | 目的                                                                                                             |
|-------------------------------------------|---------------------------------------------------------------------------------------------------------------------|
| [`ui`](ui.md)                             | コンパイルおよび/または結果の実行可能ファイルの実行からのstdout/stderrスナップショットをチェック                      |
| `ui-fulldeps`                             | リンク可能な`rustc`のビルドを必要とする`ui`テスト（`extern crate rustc_span;`の使用やプラグインとしての使用など） |
| [`pretty`](#pretty-printer-tests)         | プリティプリントをチェック                                                                                               |
| [`incremental`](#incremental-tests)       | インクリメンタルコンパイルの動作をチェック                                                                              |
| [`debuginfo`](#debuginfo-tests)           | デバッガを実行するデバッグ情報生成をチェック                                                                        |
| [`codegen-*`](#codegen-tests)             | コード生成をチェック                                                                                               |
| [`codegen-units`](#codegen-units-tests)   | codegenユニットのパーティショニングをチェック                                                                                     |
| [`assembly`](#assembly-tests)             | アセンブリ出力をチェック                                                                                               |
| [`mir-opt`](#mir-opt-tests)               | MIR生成と最適化をチェック                                                                              |
| [`coverage`](#coverage-tests)             | カバレッジ計装をチェック                                                                                      |
| [`coverage-run-rustdoc`](#coverage-tests) | 計装されたdoctestsも実行する`coverage`テスト                                                                |
| [`crashes`](#crash-tests)               | コンパイラが特定の入力でICE/パニック/クラッシュすることをチェックして、偶発的な修正を捕捉                             |

### 汎用テストスイート

[`run-make`](#run-make-tests)は、Rustプログラムを使用する汎用テストです。

### Rustdocテストスイート

| テストスイート                           | 目的                                                                  |
|--------------------------------------|--------------------------------------------------------------------------|
| [`rustdoc`][rustdoc-html-tests]      | `rustdoc`のHTML出力をチェック                                           |
| [`rustdoc-gui`][rustdoc-gui-tests]   | Webブラウザを使用して`rustdoc`のGUIをチェック                                |
| [`rustdoc-js`][rustdoc-js-tests]     | `rustdoc`の検索エンジンとインデックスをチェック                                |
| [`rustdoc-js-std`][rustdoc-js-tests] | 標準ライブラリドキュメントでの`rustdoc`の検索エンジンとインデックスをチェック        |
| [`rustdoc-json`][rustdoc-json-tests] | `rustdoc`のJSON出力をチェック                                           |
| `rustdoc-ui`                         | `rustdoc`の端末出力をチェック（[こちらも参照](ui.md)）                   |

一部のrustdoc固有のテストは`ui/rustdoc/`にもあります。
これらは`rustc`の一部として実行される（`rustdoc`だけでなく）rustdoc関連またはrustdoc固有のlintをチェックします。
rustdocに関連するrun-makeテストは通常`run-make/rustdoc-*/`という名前です。

[rustdoc-html-tests]: ../rustdoc-internals/rustdoc-test-suite.md
[rustdoc-gui-tests]: ../rustdoc-internals/rustdoc-gui-test-suite.md
[rustdoc-js-tests]: ../rustdoc-internals/search.md#testing-the-search-engine
[rustdoc-json-tests]: ../rustdoc-internals/rustdoc-json-test-suite.md

### プリティプリンタテスト

[`tests/pretty`]のテストは、`rustc`の「プリティプリンティング」機能を実行します。`rustc`の`-Z unpretty` CLIオプションは、入力ソースをマクロ展開後のRustソースなどのさまざまな異なる形式に変換します。

プリティプリンタテストには、以下で説明するいくつかの[directives](directives.md)があります。
これらのコマンドはテストの動作を大幅に変更できますが、コマンドなしのデフォルトの動作は次のとおりです：

1. ソースファイルで`rustc -Zunpretty=normal`を実行します。
2. 前のステップの出力で`rustc -Zunpretty=normal`を実行します。
3. 前の2つのステップの出力は同じである必要があります。
4. 出力で`rustc -Zno-codegen`を実行して、型チェックができることを確認します
   （`cargo check`に似ています）。

上記のいずれかのコマンドが失敗した場合、テストは失敗します。

プリティプリンティングテストのディレクティブは次のとおりです：

- `pretty-mode`は、プリティプリントテストが実行されるべきモード（つまり、`-Zunpretty`への引数）を指定します。指定されていない場合のデフォルトは`normal`です。
- `pretty-compare-only`は、プリティテストがプリティプリントされた出力を比較するだけにします（上記のステップ3の後に停止します）。展開された出力をコンパイルして型チェックを試みません。これは、有効なRustに展開されないプリティモード、または展開された出力をコンパイルできない他の状況で必要です。
- `pp-exact`は、プリティプリントテストが特定の出力を生成することを確認するために使用されます。値なしで指定された場合、プリティプリント出力は元のソースと一致する必要があります。`//@
  pp-exact:foo.pp`のように値を指定すると、プリティプリントされた出力が指定されたファイルの内容と一致することを確認します。それ以外の場合、`pp-exact`が指定されていない場合、プリティプリントされた出力はもう一度プリティプリントされ、2回のプリティプリントラウンドの出力が比較されて、プリティプリントされた出力が定常状態に収束することを確認します。

[`tests/pretty`]: https://github.com/rust-lang/rust/tree/HEAD/tests/pretty

### インクリメンタルテスト

[`tests/incremental`]のテストは、インクリメンタルコンパイルを実行します。これらは[`revisions` directive](#revisions)を使用して、compiletestに一連のステップでコンパイラを実行するよう指示します。

Compiletestは、`-C incremental`フラグを使用して空のディレクトリから開始し、各リビジョンに対してコンパイラを実行し、前のステップからのインクリメンタル結果を再利用します。

リビジョンは次のように始める必要があります：

* `rpass` — テストはコンパイルして正常に実行される必要があります
* `rfail` — テストは正常にコンパイルされる必要がありますが、実行可能ファイルは実行に失敗する必要があります
* `cfail` — テストはコンパイルに失敗する必要があります

リビジョンを一意にするには、`rpass1`と`rpass2`のようにサフィックスを追加する必要があります。

ソースの変更をシミュレートするために、compiletestは現在のリビジョン名で`--cfg`フラグも渡します。

例えば、これは2回実行され、関数の変更をシミュレートします：

```rust,ignore
//@ revisions: rpass1 rpass2

#[cfg(rpass1)]
fn foo() {
    println!("one");
}

#[cfg(rpass2)]
fn foo() {
    println!("two");
}

fn main() { foo(); }
```

`cfail`テストは、特定の部分文字列がコンパイラ出力のどこにも表示されてはならないことを指定する`forbid-output`ディレクティブをサポートします。これは特定のエラーが表示されないことを確認するのに役立ちますが、エラーメッセージは時間とともに変化し、テストが正しいことをチェックしなくなっても合格する可能性があるため、脆弱です。

`cfail`テストは、テストが内部コンパイラエラー（ICE）を引き起こすべきことを指定する`should-ice`ディレクティブをサポートします。これは、ICE後もインクリメンタルキャッシュが引き続き機能することをチェックするための非常に特殊なディレクティブです。

[`tests/incremental`]: https://github.com/rust-lang/rust/tree/HEAD/tests/incremental


### デバッグ情報テスト

[`tests/debuginfo`]のテストは、デバッグ情報生成をテストします。これらはプログラムをビルドし、デバッガを起動し、デバッガにコマンドを発行します。1つのテストでcdb、gdb、lldbを使用できます。

ほとんどのテストには、適切なデバッグ情報を生成するために`//@ compile-flags: -g`ディレクティブまたは類似のものが必要です。

行にブレークポイントを設定するには、その行に`// #break`コメントを追加します。

デバッグ情報テストは、一連のデバッガコマンドと、デバッガからの期待される出力を指定する「チェック」行で構成されます。

コマンドは`// $DEBUGGER-command:$COMMAND`の形式のコメントで、`$DEBUGGER`は使用されているデバッガで、`$COMMAND`は実行するデバッガコマンドです。

デバッガの値は次のとおりです：

- `cdb`
- `gdb`
- `gdbg` — RustサポートなしのGDB（7.11より古いバージョン）
- `gdbr` — Rustサポート付きのGDB
- `lldb`
- `lldbg` — RustサポートなしのLLDB
- `lldbr` — Rustサポート付きのLLDB（これはもう存在しません）

出力をチェックするコマンドは`// $DEBUGGER-check:$OUTPUT`の形式で、`$OUTPUT`は期待される出力です。

例えば、以下はテストをビルドし、デバッガを起動し、ブレークポイントを設定し、プログラムを起動し、値を検査し、デバッガが出力するものをチェックします：

```rust,ignore
//@ compile-flags: -g

//@ lldb-command: run
//@ lldb-command: print foo
//@ lldb-check: $0 = 123

fn main() {
    let foo = 123;
    b(); // #break
}

fn b() {}
```

次の[directives](directives.md)は、現在使用されているデバッガに基づいてテストを無効にするために使用できます：

- `min-cdb-version: 10.0.18317.1001` — cdbのバージョンが指定されたバージョンより低い場合、テストを無視します
- `min-gdb-version: 8.2` — gdbのバージョンが指定されたバージョンより低い場合、テストを無視します
- `ignore-gdb-version: 9.2` — gdbのバージョンが指定されたバージョンと等しい場合、テストを無視します
- `ignore-gdb-version: 7.11.90 - 8.0.9` — gdbのバージョンが範囲内（両端を含む）にある場合、テストを無視します
- `min-lldb-version: 310` — lldbのバージョンが指定されたバージョンより低い場合、テストを無視します
- `rust-lldb` — lldbがRustプラグインを含んでいない場合、テストを無視します。注：LLDBの「Rust」バージョンはもう存在しないため、これは常に無視されます。これはおそらく削除されるべきです。

`--debugger`オプションをcompiletestに渡すことで、テストを実行する単一のデバッガを指定できます。
例えば、`./x test tests/debuginfo -- --debugger gdb`はGDBコマンドのみをテストします。

> **lldbデバッグ情報テストをローカルで実行する際の注意**
>
> lldbデバッグ情報テストをローカルで実行したい場合、現在Windowsでは次のことが必要です：
>
> - Python 3.10がインストールされていること。
> - `python310.dll`が`PATH`環境変数で利用可能であること。これは`python.org`から入手する標準のPythonインストーラでは提供されていません。
>   手動で`PATH`に追加する必要があります。
>
> そうでない場合、lldbデバッグ情報テストは不可解な方法でクラッシュを引き起こす可能性があります。

[`tests/debuginfo`]: https://github.com/rust-lang/rust/tree/HEAD/tests/debuginfo

> **Windows 11で`cdb.exe`を取得する際の注意**
>
> `cdb.exe`は、Visual Studioインストーラ（Visual Studio 2022インストーラなど）の「Desktop Development with C++」ワークロードプロファイルの一部である適切な「Windows 11 SDK」と一緒に取得されます。
>
> **ただし**、これだけではデフォルトで十分ではありません。`cdb.exe`が必要な場合は、インストール済みアプリに移動し、最新の「Windows Software Development
> Kit」を見つけ（OSがWindows 11と呼ばれていても、これは`Windows 10.0.22161.3233`と表示される可能性があります）、「Modify」→「Change」をクリックしてから「Debugging Tools for Windows」を選択して`cdb.exe`を取得する必要があります。

### コード生成テスト

[`tests/codegen-llvm`]のテストは、LLVMコード生成をテストします。これらは`--emit=llvm-ir`フラグを使用してテストをコンパイルし、LLVM IRを出力します。次に、LLVM [FileCheck]ツールを実行します。テストには、生成されたコードをチェックするためのさまざまな`// CHECK`コメントが注釈として付けられています。チュートリアルと詳細については、[FileCheck]ドキュメントを参照してください。

同様のテストセットについては、[アセンブリテスト](#assembly-tests)も参照してください。

`#![no_std]`クロスコンパイルテストを使用する必要がある場合は、[`minicore`テスト補助](./minicore.md)の章を参照してください。

[`tests/codegen-llvm`]: https://github.com/rust-lang/rust/tree/HEAD/tests/codegen-llvm
[FileCheck]: https://llvm.org/docs/CommandGuide/FileCheck.html


### アセンブリテスト

[`tests/assembly-llvm`]のテストは、LLVMアセンブリ出力をテストします。これらは`--emit=asm`フラグを使用してテストをコンパイルし、アセンブリ出力を含む`.s`ファイルを出力します。次に、LLVM [FileCheck]ツールを実行します。

各テストには、アセンブリ出力のタイプを示す`emit-asm`または`ptx-linker`の値を持つ`//@ assembly-output:`ディレクティブで注釈を付ける必要があります。

次に、アセンブリ出力をチェックするためのさまざまな`// CHECK`コメントで注釈を付ける必要があります。チュートリアルと詳細については、[FileCheck]ドキュメントを参照してください。

同様のテストセットについては、[コード生成テスト](#codegen-tests)も参照してください。

`#![no_std]`クロスコンパイルテストを使用する必要がある場合は、[`minicore`テスト補助](./minicore.md)の章を参照してください。

[`tests/assembly-llvm`]: https://github.com/rust-lang/rust/tree/HEAD/tests/assembly-llvm


### コード生成ユニットテスト

[`tests/codegen-units`]のテストは、[単相化](../backend/monomorph.md)コレクタとCGUパーティショニングをテストします。

これらのテストは、単相化収集パスの結果を出力するフラグ、つまり`-Zprint-mono-items`を使用して`rustc`を実行し、ファイル内の特別な注釈を使用してそれと比較します。

次に、テストには、`name`が`fn <u32 as Trait>::foo`のようなrustcによって出力される単相化された文字列である`//~ MONO_ITEM name`の形式のコメントで注釈を付ける必要があります。

CGUパーティショニングをチェックするには、`//~ MONO_ITEM name @@ cgu`の形式のコメントを使用します。ここで、`cgu`はCGU名と括弧内のリンケージ情報のスペース区切りリストです。例：`//~ MONO_ITEM static function::FOO @@
statics[Internal]`

[`tests/codegen-units`]: https://github.com/rust-lang/rust/tree/HEAD/tests/codegen-units


### MIR最適化テスト

[`tests/mir-opt`]のテストは、生成されたMIRの一部をチェックして、正しく生成され、期待される最適化を実行していることを確認します。詳細については、[MIR Optimizations](../mir/optimizations.md)の章を参照してください。

Compiletestは、いくつかのフラグを使用してテストをビルドし、MIR出力をダンプし、最適化のベースラインを設定します：

* `-Copt-level=1`
* `-Zdump-mir=all`
* `-Zmir-opt-level=4`
* `-Zvalidate-mir`
* `-Zdump-mir-exclude-pass-number`

テストには、期待されるMIR出力を含むファイルを指定する`// EMIT_MIR`コメントで注釈を付ける必要があります。`x test --bless`を使用して、初期の期待ファイルを作成できます。

`EMIT_MIR`コメントには、いくつかの形式があります：

- `// EMIT_MIR $MIR_PATH.mir` — これは、指定されたファイル名がMIRダンプからの正確な出力と一致することをチェックします。例えば、
  `my_test.main.SimplifyCfg-elaborate-drops.after.mir`は、テストディレクトリからそのファイルをロードし、rustcからのダンプと比較します。

  「after」ファイル（最適化後）をチェックすることは、最適化後の最終状態に興味がある場合に便利です。まれに、完全性のために「before」ファイルを使用したい場合があります。

- `// EMIT_MIR $MIR_PATH.diff` — `$MIR_PATH`は、`my_test_name.my_function.EarlyOtherwiseBranch`のようなMIRダンプのファイル名です。Compiletestは、`.before.mir`と`.after.mir`ファイルを差分し、差分出力を`EMIT_MIR`コメントからの期待される`.diff`ファイルと比較します。

  これは、最適化がMIRをどのように変更するかを確認したい場合に便利です。

- `// EMIT_MIR $MIR_PATH.dot` — 追加のMIRデータをダンプする特定のフラグ（例：`.dot`ファイルを生成する`-Z dump-mir-graphviz`）を使用する場合、これは出力が指定されたファイルと一致することをチェックします。

デフォルトでは、32ビットと64ビットのターゲットは同じダンプファイルを使用しますが、定数内のポインタや他のビット幅依存のものが存在する場合に問題が生じる可能性があります。その場合、テストに`// EMIT_MIR_FOR_EACH_BIT_WIDTH`を追加すると、32ビットシステムと64ビットシステム用に別々のファイルが生成されます。

[`tests/mir-opt`]: https://github.com/rust-lang/rust/tree/HEAD/tests/mir-opt


### `run-make`テスト

[`tests/run-make`]と[`tests/run-make-cargo`]のテストは、Rust *レシピ*を使用する汎用テストです。これらは、`rustc`呼び出しなどの任意のRustコードを可能にする小さなプログラム（`rmake.rs`）で、[`run_make_support`]ライブラリによってサポートされます。Rustレシピを使用すると、究極の柔軟性が提供されます。

`run-make`テストは、他のテストスイートがニーズに適さない場合に使用する必要があります。

`run-make-cargo`テストスイートは、ツリー内の`cargo`とツリー内の`rustc`を連携してテストする必要があるユースケースをサポートするために、追加でツリー内の`cargo`をビルドします。
`run-make`テストスイートはツリー内の`cargo`にアクセスできません（そのため、反復が高速なテストスイートになります）。

#### Rustレシピの使用

各テストは、*レシピ*と呼ばれる`rmake.rs` Rustプログラムを含む別のディレクトリに配置する必要があります。レシピは、`run_make_support`ライブラリがリンクされた状態でcompiletestによってコンパイルおよび実行されます。

新しいユーティリティや機能が必要な場合は、[`run_make_support`]ライブラリを拡張および改善することを検討してください。

`//@ only-<target>`や`//@ ignore-<target>`のようなCompiletestディレクティブは、UIテストと同様に`rmake.rs`でサポートされています。ただし、リビジョンやディレクティブによる補助のビルドは現在サポートされていません。

`rmake.rs`と`run-make-support`は、nightly/不安定な機能を使用*してはいけません*。ステージ0のrustcがベータ版または安定版のrustcである可能性があるため、それらでコンパイル可能である必要があります。

#### `rmake.rs`テストがコンパイル可能かどうかを素早くチェック

ステージ1のrustcをビルドせずに`rmake.rs`テストがコンパイル可能かどうかを素早くチェックできます。ステージ0のコンパイラで`rmake.rs`を強制的にコンパイルします：

```bash
$ COMPILETEST_FORCE_STAGE0=1 x test --stage 0 tests/run-make/<test-name>
```

もちろん、一部のテストはこの方法では正常に*実行*されません。

#### `rmake.rs`でrust-analyzerを使用

他のテストプログラムと同様に、run-makeテストで使用される`rmake.rs`スクリプトは、デフォルトではrust-analyzer統合がありません。

特定のテストで作業する際にこれを回避するには、テストのディレクトリに一時的に`Cargo.toml`ファイルを作成します
（例：`tests/run-make/sysroot-crates-are-unstable/Cargo.toml`）
次の内容で：

<div class="warning">

この`Cargo.toml`やその`Cargo.lock`を実際のPRに追加しないように注意してください！

</div>

```toml
# Convince cargo that this isn't part of an enclosing workspace.
[workspace]

[package]
name = "rmake"
version = "0.1.0"
edition = "2021"

[dependencies]
run_make_support = { path = "../../../src/tools/run-make-support" }

[[bin]]
name = "rmake"
path = "rmake.rs"
```

次に、対応するエントリを`"rust-analyzer.linkedProjects"`に追加します
（例：`.vscode/settings.json`）：

```json
"rust-analyzer.linkedProjects": [
  "tests/run-make/sysroot-crates-are-unstable/Cargo.toml"
],
```

[`tests/run-make`]: https://github.com/rust-lang/rust/tree/HEAD/tests/run-make
[`tests/run-make-cargo`]: https://github.com/rust-lang/rust/tree/HEAD/tests/run-make-cargo
[`run_make_support`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/run-make-support

### カバレッジテスト

[`tests/coverage`]のテストは、異なる方法でカバレッジ計装をテストする複数のテストモードで共有されます。`coverage`テストスイートを実行すると、すべてのカバレッジモードで各テストが自動的に実行されます。

各モードには、そのモードでのみカバレッジテストを実行するためのエイリアスもあります：

```bash
./x test coverage # すべてのカバレッジモードでtests/coverageのすべてを実行
./x test tests/coverage # 上記と同じ

./x test tests/coverage/if.rs # すべてのカバレッジモードで指定されたテストを実行

./x test coverage-map # 「coverage-map」モードのみでtests/coverageのすべてを実行
./x test coverage-run # 「coverage-run」モードのみでtests/coverageのすべてを実行

./x test coverage-map -- tests/coverage/if.rs # 「coverage-map」モードのみで指定されたテストを実行
```

何らかの理由で特定のテストがカバレッジテストモードの1つで実行されるべきでない場合は、`//@ ignore-coverage-map`または`//@ ignore-coverage-run`ディレクティブを使用します。

#### `coverage-map`スイート

`coverage-map`モードでは、これらのテストはソースコード領域とLLVMによって出力されるカバレッジカウンタ間のマッピングを検証します。`--emit=llvm-ir`でテストをコンパイルし、カスタムツール（[`src/tools/coverage-dump`]）を使用してIRに埋め込まれたカバレッジマッピングを抽出してプリティプリントします。これらのテストはプロファイラランタイムを必要としないため、PR CIジョブで実行され、ローカルで実行/blessが簡単です。

これらのカバレッジマップテストは、MIR低下やMIR最適化の変更に敏感で、異なるが同一のカバレッジレポートを生成するマッピングを生成する可能性があります。

経験則として、カバレッジ固有のコードを変更しないPRは、`coverage-run`テストが引き続き合格する限り、必要に応じて`coverage-map`テストを**自由に再bless**できます。実際の変更を心配する必要はありません。

#### `coverage-run`スイート

`coverage-run`モードでは、これらのテストはカバレッジレポートのエンドツーエンドテストを実行します。カバレッジ計装でテストプログラムをコンパイルし、そのプログラムを実行して生カバレッジデータを生成し、LLVMツールを使用してそのデータを人間が読めるコードカバレッジレポートに処理します。

計装されたバイナリはLLVMプロファイラランタイムに対してリンクされる必要があるため、`coverage-run`テストは、プロファイラランタイムが`bootstrap.toml`で有効になっていない場合、**自動的にスキップ**されます：

```toml
# bootstrap.toml
[build]
profiler = true
```

これは、通常PR CIジョブでは実行されませんが、マージに使用される完全なCIジョブセットの一部として実行されることも意味します。

#### `coverage-run-rustdoc`スイート

[`tests/coverage-run-rustdoc`]のテストも、計装されたdoctestsを実行し、カバレッジレポートに含めます。これにより、メインの`coverage`スイートのみを実行する際にrustdocをビルドする必要がなくなります。

[`tests/coverage`]: https://github.com/rust-lang/rust/tree/HEAD/tests/coverage
[`src/tools/coverage-dump`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/coverage-dump
[`tests/coverage-run-rustdoc`]: https://github.com/rust-lang/rust/tree/HEAD/tests/coverage-run-rustdoc

### クラッシュテスト

[`tests/crashes`]は、コンパイラがICE、パニック、またはその他の方法でクラッシュすることが期待されるテストのコレクションとして機能し、偶発的な修正が追跡されます。以前は、これは<https://github.com/rust-lang/glacier>で行われていましたが、rust-lang/rustテストスイート内で行う方が便利です。

スイート内のテストは、rustcをICE、パニック、またはその他の方法でクラッシュさせることが不可欠です。テストは、rustcが1または0以外の終了ステータスで終了すると「合格」します。

詳細なstdout/stderrを表示したい場合は、`COMPILETEST_VERBOSE_CRASHES=1`を設定する必要があります。例：

```bash
$ COMPILETEST_VERBOSE_CRASHES=1 ./x test tests/crashes/999999.rs --stage 1
```

誰でもissue trackerから["untracked" crashes]を追加できます。複数のissueからのテストケースを1つのPRに含めることを強くお勧めします。
その際、各issue番号をファイル名（`12345.rs`で十分です）とファイル内に`//@ known-bug: #12345`ディレクティブで記載する必要があります。PRがマージされたら、関連するissueに`S-bug-has-test`で[ラベル付け][labeling]してください。

クラッシュの1つを修正した場合は、それを`tests/ui`の適切なサブディレクトリに移動し、意味のある名前を付けてください。ファイルの先頭に、なぜこのテストが存在するのかを説明するドキュメントコメントを追加してください。できれば、例がrustcを以前にクラッシュさせた方法と、rustcがICE/パニック/クラッシュするのを防ぐために何が行われたかを簡単に説明すると良いでしょう。

以下を追加すると

```text
Fixes #NNNNN
Fixes #MMMMM
```

プルリクエストの説明に追加すると、マージ時に対応するチケットが自動的にクローズされます。

修正が実際にissueの根本原因を修正し、単なるサブセットではないことを最初に確認してください。issue番号は、ファイル名またはテストファイル内の`//@
known-bug`ディレクティブで見つけることができます。

[`tests/crashes`]: https://github.com/rust-lang/rust/tree/HEAD/tests/crashes
["untracked" crashes]: https://github.com/rust-lang/rust/issues?q=is%3Aissue+state%3Aopen+label%3AI-ICE%2CI-crash+label%3AT-compiler+label%3AS-has-mcve+-label%3AS-bug-has-test
[labeling]: https://forge.rust-lang.org/release/issue-triaging.html#applying-and-removing-labels

## 補助crateのビルド

一部のテストでは、追加の補助crateをコンパイルする必要があることがよくあります。
それを支援する複数の[directives](directives.md)があります：

- `aux-build`
- `aux-crate`
- `aux-bin`
- `aux-codegen-backend`
- `proc-macro`

`aux-build`は、指定されたソースファイルから別のcrateをビルドします。ソースファイルは、テストファイルの隣の`auxiliary`というディレクトリにある必要があります。

```rust,ignore
//@ aux-build: my-helper.rs

extern crate my_helper;
// ... my_helperを使用できます。
```

auxクレートは、可能な場合dylibとしてビルドされます（プラットフォームがdylibをサポートしていない場合、またはauxファイルで`no-prefer-dynamic`ヘッダが指定されている場合を除く）。`-L`フラグは、extern crateを見つけるために使用されます。

`aux-crate`は`aux-build`に非常に似ています。ただし、`--extern`フラグを使用してextern crateにリンクし、crateをextern preludeとして使用できるようにします。
これにより、依存関係の名前変更など、`--extern`フラグの追加構文を指定できます。例えば、`//@ aux-crate:foo=bar.rs`は`auxiliary/bar.rs`をコンパイルし、テスト内で`foo`という名前で使用できるようにします。
これは、Cargoが依存関係の名前変更を行う方法に似ています。

`aux-bin`は`aux-build`に似ていますが、ライブラリではなくバイナリをビルドします。バイナリは、テストの作業ディレクトリに対して相対的な`auxiliary/bin`で利用できます。

`aux-codegen-backend`は`aux-build`に似ていますが、コンパイルされたdylibをメインファイルのビルド時に`-Zcodegen-backend`に渡します。これは、コンパイラクレートの使用を必要とするため、`tests/ui-fulldeps`のテストでのみ機能します。

### 補助proc-macro

proc-macro依存関係が必要な場合は、`proc-macro`ディレクティブを使用できます。このディレクティブは`aux-build`と同じように動作します。つまり、proc-macroテスト補助ファイルを、メインテストファイルと同じ親フォルダの下の`auxiliary`フォルダに配置する必要があります。ただし、proc-macroテスト補助用に`aux-build`と比較して4つの追加のプリセット動作があります：

1. auxテストファイルは`--crate-type=proc-macro`でビルドされます。
2. auxテストファイルは`-C prefer-dynamic`なしでビルドされます。つまり、aux crateのdylibを生成しようとしません。
3. aux crateは`--extern <aux_crate_name>`を介してextern preludeを通じてテストファイルで使用できるようになります。UIテストはデフォルトでエディション2015であるため、auxクレート名を`use`インポートで使用したい場合は、メインテストファイルがエディション2018以降を使用していない限り、`extern <aux_crate_name>`を指定する必要があることに注意してください。
4. `proc_macro` crateがextern preludeモジュールとして使用可能になります。`extern proc_macro;`についても、エディション2015と新しいエディションの区別が同じように適用されます。

例えば、テスト`tests/ui/cat/meow.rs`とproc-macro補助`tests/ui/cat/auxiliary/whiskers.rs`がある場合：

```text
tests/ui/cat/
    meow.rs                 # メインテストファイル
    auxiliary/whiskers.rs   # 補助
```

```rs
// tests/ui/cat/meow.rs

//@ proc-macro: whiskers.rs

extern crate whiskers; // uiテストはデフォルトでエディション2015であるため必要

fn main() {
  whiskers::identity!();
}
```

```rs
// tests/ui/cat/auxiliary/whiskers.rs

extern crate proc_macro;
use proc_macro::*;

#[proc_macro]
pub fn identity(ts: TokenStream) -> TokenStream {
    ts
}
```

> **注**：`proc-macro`ヘッダは現在、rustdocテストの`build-aux-doc`ヘッダと一緒に機能しません。その場合は、`aux-build`ヘッダを使用し、`#![crate_type="proc_macro"]`、および`//@
> force-host`と`//@ no-prefer-dynamic`ヘッダをproc-macroで使用する必要があります。

## リビジョン

リビジョンを使用すると、1つのテストファイルを複数のテストに使用できます。これは、ファイルの先頭に特別なディレクティブを追加することで行われます：

```rust,ignore
//@ revisions: foo bar baz
```

これにより、テストが3回コンパイル（およびテスト）されます。1回は`--cfg foo`、1回は`--cfg bar`、1回は`--cfg baz`です。したがって、テスト内で`#[cfg(foo)]`などを使用して、これらの結果をそれぞれ調整できます。

ディレクティブと期待されるエラーメッセージを特定のリビジョンにカスタマイズすることもできます。これを行うには、ディレクティブの場合は`//@`の後に、UIエラー注釈の場合は`//`の後に`[revision-name]`を追加します：

```rust,ignore
// cfg `foo`でのみ渡すフラグ：
//@[foo]compile-flags: -Z verbose-internals

#[cfg(foo)]
fn test_foo() {
    let x: usize = 32_u32; //[foo]~ ERROR mismatched types
}
```

複数のリビジョンをカンマ区切りリストで指定できます。例：`//[foo,bar,baz]~^`。

LLVM [FileCheck]ツールを使用するテストスイートでは、現在のリビジョン名がFileCheckディレクティブの追加プレフィックスとしても登録されます：

```rust,ignore
//@ revisions: NORMAL COVERAGE
//@[COVERAGE] compile-flags: -Cinstrument-coverage
//@[COVERAGE] needs-profiler-runtime

// COVERAGE:   @__llvm_coverage_mapping
// NORMAL-NOT: @__llvm_coverage_mapping

// CHECK: main
fn main() {}
```

すべてのディレクティブがリビジョンにカスタマイズされたときに意味を持つわけではないことに注意してください。例えば、`ignore-test`ディレクティブ（およびすべての「ignore」ディレクティブ）は現在、特定のリビジョンではなく、テスト全体にのみ適用されます。リビジョンにカスタマイズされたときに実際に機能することが意図されている唯一のディレクティブは、エラーパターンとコンパイラフラグです。

<!-- date-check jul 2023 -->
次のテストスイートがリビジョンをサポートしています：

- ui
- assembly
- codegen
- coverage
- debuginfo
- rustdoc UIテスト
- incremental（これらは本質的に並列実行できないため特殊です）

### 未使用のリビジョン名の無視

通常、他のディレクティブやエラー注釈で言及されているリビジョン名は、`revisions`ディレクティブで宣言された実際のリビジョンに対応している必要があります。これは`./x test tidy`チェックによって強制されます。

何らかの理由でリビジョン名をリビジョンリストから一時的に削除する必要がある場合、上記のチェックを抑制するには、代わりにリビジョン名を`//@ unused-revision-names:`ヘッダに追加します。

未使用の名前として`*`を指定する（つまり、`//@ unused-revision-names: *`）と、任意の未使用のリビジョン名を言及できるようになります。

## 比較モード

Compiletestは、_比較モード_と呼ばれる異なるモードで実行でき、異なるコンパイラフラグを有効にしてすべてのテストの動作を比較するために使用できます。
これにより、特定のフラグでどのような違いが現れるかを強調し、発生する可能性のある問題をチェックできます。

テストを別のモードで実行するには、`--compare-mode` CLIフラグを渡す必要があります：

```bash
./x test tests/ui --compare-mode=chalk
```

可能な比較モードは次のとおりです：

- `polonius` — `-Zpolonius`でPoloniusを実行します。
- `chalk` — `-Zchalk`でChalkを実行します。
- `split-dwarf` — `-Csplit-debuginfo=unpacked`で展開されたsplit-DWARFを実行します。
- `split-dwarf-single` — `-Csplit-debuginfo=packed`でパックされたsplit-DWARFを実行します。

UIテストが異なるモードに対して異なる出力をどのようにサポートするかについては、[UI compare modes](ui.md#compare-modes)を参照してください。

CIでは、比較モードは1つのLinuxビルダーでのみ使用され、次の設定でのみ使用されます：

- `tests/debuginfo`：`split-dwarf`モードを使用します。これにより、split-DWARFを有効にしてもデバッグ情報テストが影響を受けないことを確認できます。

比較モードは[リビジョン](#revisions)とは別であることに注意してください。すべてのリビジョンは`./x test tests/ui`を実行するとテストされますが、比較モードは`--compare-mode`フラグを介して個別に手動で実行する必要があります。
