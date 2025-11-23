# テストの実行

`x` を使用してテストコレクション全体を実行できます。
ただし、*全体*のテストコレクションを実行することは、ローカル開発中にはほとんど望ましくありません。非常に時間がかかるためです。
ローカル開発では、テストのサブセットを実行する方法について次のサブセクションを参照してください。

<div class="warning">

単純な `./x test` を実行すると、stage 1 コンパイラをビルドしてからテストスイート全体を実行します。
これには `tests/` だけでなく、`library/`、`compiler/`、
`src/tools/` のパッケージテストなども含まれます。

通常は、変更が適用されることが期待されるテストスイートのサブセット（またはそれよりもさらに小さなテストセット）のみを実行したいはずです。
PR CI はテストコレクションのサブセットを実行し、マージキュー CI はすべてのテストコレクションを実行します。

</div>

```text
./x test
```

テスト結果はキャッシュされ、以前に成功したテストはテスト時に `ignored` となります。
すべてのテストの stdout/stderr の内容とタイムスタンプファイルは、
指定された `<target-tuple>` の `build/<target-tuple>/test/` の下にあります。テストを強制的に再実行するには（例えば、テストランナーが変更に気付かない場合）、`--force-rerun` CLI オプションを使用できます。

> **外部依存性の要件に関する注意**
>
> 一部のテストスイートは外部依存性を必要とする場合があります。これは特に
> debuginfo テストに当てはまります。一部の debuginfo テストは Python 対応の gdb を必要とします。
> gdb のインストールが Python をサポートしているかどうかは、gdb 内から `python` コマンドを使用してテストできます。
> 起動したら、いくつかの Python コード（例：`print("hi")`）を入力し、
> return を押してから `CTRL+D` を押して実行します。gdb をソースからビルドしている場合は、
> `--with-python=<path-to-python-binary>` で構成する必要があります。

## テストスイートのサブセットを実行

特定の PR で作業しているときは、通常、より小さなテストセットを実行したいでしょう。
例えば、rustc を変更した後に使用できる良い「スモークテスト」として、
一般的に正しく動作しているかを確認するために `ui` テストスイート（[`tests/ui`]）を実行するのが良いでしょう：

```text
./x test tests/ui
```

もちろん、テストスイートの選択は多少恣意的であり、行っているタスクに適さない場合があります。
例えば、debuginfo をハッキングしている場合は、debuginfo テストスイートを使用する方が良いでしょう：

```text
./x test tests/debuginfo
```

任意のテストスイートの特定のサブディレクトリのテストのみをテストする必要がある場合は、
そのディレクトリをフィルタとして `./x test` に渡すことができます：

```text
./x test tests/ui/const-generics
```

> **MSYS2 に関する注意**
>
> MSYS2 ではパスが奇妙に見え、`./x test` は
> `tests/ui/const-generics` も `tests\ui\const-generics` も認識しません。その場合は、
> `./x test ui
> --test-args="tests/ui/const-generics"` などを使用して回避できます。

同様に、パスを渡すことで単一のファイルをテストできます：

```text
./x test tests/ui/const-generics/const-test.rs
```

`x` はパスを渡すことで単一のツールテストを実行することをまだサポートしていません。
[以下](#running-an-individual-test)で説明する `--test-args` 引数を使用する必要があります。

```text
./x test src/tools/miri --test-args tests/fail/uninit/padding-enum.rs
```

### tidy スクリプトのみを実行

```text
./x test tidy
```

### 標準ライブラリでテストを実行

```text
./x test --stage 0 library/std
```

これは `std` のテストのみを実行することに注意してください。
`core` や他のクレートをテストしたい場合は、それらを明示的に指定する必要があります。

### tidy スクリプトと標準ライブラリのテストを実行

```text
./x test --stage 0 tidy library/std
```

### stage 1 コンパイラを使用して標準ライブラリでテストを実行

```text
./x test --stage 1 library/std
```

実行したいテストスイートをリストすることで、
まったく変更しなかったコンポーネントのテストを実行することを避けられます。

<div class="warning">

bors は完全な stage 2 ビルドでのみテストを実行することに注意してください。
したがって、テストは通常 stage 1 で正常に動作しますが、いくつかの制限があります。

</div>

### stage 2 コンパイラを使用してすべてのテストを実行

```text
./x test --stage 2
```

<div class="warning">
これを行う必要はほとんどありません。
CI があなたのためにこれらのテストを実行します。
</div>

## コンパイラ/ライブラリでユニットテストを実行

特定のファイルでユニットテストを実行したい場合があります：

```text
./x test compiler/rustc_data_structures/src/thin_vec/tests.rs
```

しかし残念ながら、これは不可能です。
代わりに次を呼び出す必要があります：

```text
./x test compiler/rustc_data_structures/ --test-args thin_vec
```

## 個別のテストを実行

人々がよく行いたいもう1つの一般的なことは、**個別のテスト**を実行することです。
多くの場合、修正しようとしているテストです。
前述のように、完全なファイルパスを渡してこれを実現できます。または、`--test-args` オプションを指定して `x` を呼び出すこともできます：

```text
./x test tests/ui --test-args issue-1234
```

内部的には、テストランナーは標準的な Rust テストランナー（`#[test]` で得られるものと同じ）を呼び出すため、このコマンドは名前に「issue-1234」を含むテストをフィルタリングします。
したがって、`--test-args` は関連するテストのコレクションを実行する良い方法です。

## テスト実行時に `rustc` に引数を渡す

テストを実行するときに特定のコンパイラ引数を指定すると便利な場合があります。
`RUSTFLAGS` を使用せずに（例えば、不安定な機能の開発中に `-Z` フラグを使用する場合など）。

これは `./x test` の `--compiletest-rustc-args` オプションで行うことができ、
テストをビルドするときにコンパイラに追加の引数を渡すことができます。

## リファレンスファイルの編集と更新

コンパイラの出力を意図的に変更した場合、または新しいテストを作成している場合は、
test サブコマンドに `--bless` を渡すことができます。

例として、`tests/ui` の一部のテストが失敗している場合、このコマンドを実行できます：

```text
./x test tests/ui --bless
```

これにより、すべての `test/ui` テストの `.stderr`、`.stdout`、または `.fixed` ファイルが自動的に調整されます。
もちろん、`--bless` フラグなしでテストを実行するときと同様に、`--test-args your_test_name` フラグで特定のテストのみをターゲットにすることもできます。

## テスト実行の設定

テスト実行にはいくつかのオプションがあります：

* `bootstrap.toml` には `rust.verbose-tests` オプションがあります。`false` の場合、各テストは
  単一のドットを出力します（デフォルト）。
  `true` の場合、すべてのテストの名前が出力されます。
  これは [Rust テストハーネス](https://doc.rust-lang.org/rustc/tests/) の `--quiet` オプションと同等です。
* 環境変数 `RUST_TEST_THREADS` は、
  テストに使用する同時スレッド数に設定できます。

## `--pass $mode` を渡す

Pass UI テストには現在、`check-pass`、`build-pass`、`run-pass` の3つのモードがあります。
`--pass $mode` を渡すと、これらのテストは、テストファイルに `//@ ignore-pass` ディレクティブが存在しない限り、指定された `$mode` で実行されるように強制されます。
例えば、`tests/ui` のすべてのテストを `check-pass` として実行できます：

```text
./x test tests/ui --pass check
```

`--pass $mode` を渡すことで、テスト時間を短縮できます。
各モードについては、[pass/fail 期待値の制御](ui.md#controlling-passfail-expectations) を参照してください。

## 異なる「比較モード」でテストを実行

UI テストは、コンパイラが置かれている特定の「モード」によって異なる出力を持つ場合があります。
例えば、Polonius モードを使用する場合、テスト `foo.rs` は
まず `foo.polonius.stderr` で期待される出力を探し、見つからない場合は
通常の `foo.stderr` にフォールバックします。
次のコマンドは Polonius モードで UI テストスイートを実行します：

```text
./x test tests/ui --compare-mode=polonius
```

詳細については、[比較モード](compiletest.md#compare-modes) を参照してください。

## テストを手動で実行

場合によっては、テストを手動で実行する方が簡単で高速です。
ほとんどのテストは単なる `.rs` ファイルなので、[rustup ツールチェーンを作成](../building/how-to-build-and-run.md#creating-a-rustup-toolchain)した後、次のようなことができます：

```text
rustc +stage1 tests/ui/issue-1234.rs
```

これははるかに高速ですが、常に機能するとは限りません。
例えば、一部のテストには特定のコンパイラフラグを指定するディレクティブが含まれていたり、他のクレートに依存していたりするため、
これらのオプションなしでは同じように実行されない場合があります。

## リモートマシンでテストを実行

テストはリモートマシンで実行できます（例：異なるアーキテクチャのビルドをテストするため）。
これは、ビルドマシンで `remote-test-client` を使用してテストプログラムをリモートマシンで実行されている `remote-test-server` に送信することによって行われます。
`remote-test-server` はテストプログラムを実行し、結果をビルドマシンに送り返します。
`remote-test-server` は*認証されていないリモートコード実行*を提供するため、使用場所には注意してください。

これを行うには、まずリモートマシン用の `remote-test-server` をビルドします
（RISC-V を例として使用）：

```text
./x build src/tools/remote-test-server --target riscv64gc-unknown-linux-gnu
```

バイナリは `./build/host/stage2-tools/$TARGET_ARCH/release/remote-test-server` に作成されます。
これをリモートマシンにコピーします。

リモートマシンで、`--bind
0.0.0.0:12345` フラグ（およびオプションで `--verbose` フラグ）を使用して `remote-test-server` を実行します。
出力は次のようになります：

```console
$ ./remote-test-server --verbose --bind 0.0.0.0:12345
starting test server
listening on 0.0.0.0:12345!
```

サーバーを 0.0.0.0 にバインドすると、マシンに到達できるすべてのホストがマシン上で任意のコードを実行できることに注意してください。
ポート 12345 への外部アクセスをブロックするファイアウォールを設定するか、
バインド時により制限的な IP アドレスを使用することを強くお勧めします。

`remote-test-server` が動作しているかどうかは、接続して `ping\n` を送信することでテストできます。
`pong` と応答するはずです：

```console
$ nc $REMOTE_IP 12345
ping
pong
```

リモートランナーを使用してテストを実行するには、`TEST_DEVICE_ADDR` 環境変数を設定してから、通常どおり `x` を使用します。
例えば、IP アドレス `1.2.3.4` の RISC-V マシンの `ui` テストを実行するには、次を使用します：

```text
export TEST_DEVICE_ADDR="1.2.3.4:12345"
./x test tests/ui --target riscv64gc-unknown-linux-gnu
```

`remote-test-server` が verbose フラグ付きで実行された場合、テストマシンでの出力は次のようになります：

```text
[...]
run "/tmp/work/test1007/a"
run "/tmp/work/test1008/a"
run "/tmp/work/test1009/a"
run "/tmp/work/test1010/a"
run "/tmp/work/test1011/a"
run "/tmp/work/test1012/a"
run "/tmp/work/test1013/a"
run "/tmp/work/test1014/a"
run "/tmp/work/test1015/a"
run "/tmp/work/test1016/a"
run "/tmp/work/test1017/a"
run "/tmp/work/test1018/a"
[...]
```

テストは `x` を実行しているマシンでビルドされ、リモートマシンではビルドされません。
予期せずビルドに失敗したテスト（または不正なビルド出力を生成する `ui` テスト）は、リモートマシンで実行されることなく失敗する可能性があります。

## エミュレータでのテスト

一部のプラットフォームは、容易に利用できないアーキテクチャのエミュレータを介してテストされます。
標準ライブラリが十分にサポートされており、
ホストオペレーティングシステムが TCP/IP ネットワーキングをサポートしているアーキテクチャの場合、リモートマシンでのテストに関する上記の手順を参照してください（この場合、リモートマシンはエミュレートされています）。

エミュレータ内でテストを実行するためのツールのセットもあります。
`arm-android` や `arm-unknown-linux-gnueabihf` などのプラットフォームは、
GitHub Actions でエミュレーション下でテストを自動的に実行するように設定されています。
以下では、ターゲットのテストがエミュレーション下でどのように実行されるかを見ていきます。

[armhf-gnu] 用の Docker イメージには、ARM CPU アーキテクチャをエミュレートする [QEMU] が含まれています。
Rust ツリーには、テストプログラムとライブラリをエミュレータに送信し、
エミュレータ内でテストを実行し、結果を読み取るためのプログラムである [remote-test-client] と
[remote-test-server] というツールが含まれています。
Docker イメージは `remote-test-server` を起動するように設定されており、
ビルドツールは `remote-test-client` を使用してサーバーと通信し、
テストの実行を調整します（[src/bootstrap/src/core/build_steps/test.rs] を参照）。

iOS/tvOS/watchOS/visionOS シミュレータで実行するには、同様に「リモート」マシンとして扱うことができます。
ここで興味深い詳細は、ネットワークがシミュレータインスタンスとホスト macOS の間で共有されているため、
ローカルループバックアドレス `127.0.0.1` を使用できることです。
次のような感じで動作するはずです：

```sh
# iOS シミュレータ用のテストサーバーをビルド：
./x build src/tools/remote-test-server --target aarch64-apple-ios-sim

# 既にシミュレータインスタンスが開いている場合は、次からデバイス UUID をコピー：
xcrun simctl list devices booted
UDID=01234567-89AB-CDEF-0123-456789ABCDEF

# または、新しいシミュレータインスタンスを作成して起動：
xcrun simctl list runtimes
xcrun simctl list devicetypes
UDID=$(xcrun simctl create $CHOSEN_DEVICE_TYPE $CHOSEN_RUNTIME)
xcrun simctl boot $UDID
# 詳細は https://nshipster.com/simctl/ を参照してください。

# ポート 12345 でランナーを生成：
xcrun simctl spawn $UDID ./build/host/stage2-tools/aarch64-apple-ios-sim/release/remote-test-server -v --bind 127.0.0.1:12345

# 新しいターミナルで、ランナー経由でテストを実行：
export TEST_DEVICE_ADDR="127.0.0.1:12345"
./x test --host='' --target aarch64-apple-ios-sim --skip tests/debuginfo
# FIXME(madsmtm): debuginfo テストが動作するようにする（`.dSYM` フォルダをターゲットにコピーする必要がある可能性があります）。
```

[armhf-gnu]: https://github.com/rust-lang/rust/tree/HEAD/src/ci/docker/host-x86_64/armhf-gnu/Dockerfile
[QEMU]: https://www.qemu.org/
[remote-test-client]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/remote-test-client
[remote-test-server]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/remote-test-server
[src/bootstrap/src/core/build_steps/test.rs]: https://github.com/rust-lang/rust/blob/HEAD/src/bootstrap/src/core/build_steps/test.rs

## wasi (wasm32-wasip1) でテストをテストする

一部のテストは wasm ターゲットに固有です。
これらのテストを実行するには、`x test` に `--target wasm32-wasip1` を渡す必要があります。
さらに、wasi sdk が必要です。
[wasi sdk リポジトリ] からインストール手順に従って、コンピュータに sysroot を取得してください。
[wasm32-wasip1 ターゲットサポートページ] には、sdk がビルドできる必要がある最小バージョンが指定されています。
時間がかかり、多くの非常に懸念される c++ 警告を出すいくつかの cmake コマンド...
次に、`bootstrap.toml` で次のように sysroot を指定します：

```toml
[target.wasm32-wasip1]
wasi-root = "<wasi-sdk location>/build/sysroot/install/share/wasi-sysroot"
```

私の場合、rust フォルダの隣に git clone したので、`../wasi-sdk/build/....` でした。
これで、テストは実行されるはずです。他に何も設定する必要はありません。

[wasi sdk リポジトリ]: https://github.com/WebAssembly/wasi-sdk
[wasm32-wasip1 ターゲットサポートページ]: https://github.com/rust-lang/rust/blob/HEAD/src/doc/rustc/src/platform-support/wasm32-wasip1.md#building-the-target.

[`tests/ui`]: https://github.com/rust-lang/rust/tree/HEAD/tests/ui
