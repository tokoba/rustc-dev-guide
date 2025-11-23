# ブートストラップが何をするか

[*ブートストラップ*][boot] とは、コンパイラを使用してそれ自身をコンパイルするプロセスです。より正確には、古いコンパイラを使用して同じコンパイラの新しいバージョンをコンパイルすることを意味します。

これは鶏と卵のパラドックスを引き起こします：最初のコンパイラはどこから来たのでしょうか？それは異なる言語で書かれたに違いありません。Rust の場合、それは [OCaml で書かれました][ocaml-compiler]。しかし、それは遥か昔に放棄され、現代のバージョンの `rustc` をビルドする唯一の方法は、少し古いバージョンを使用することです。

これはまさに [`./x.py`] が行うことです：現在のベータリリースの `rustc` をダウンロードし、それを使用して新しいコンパイラをコンパイルします。

[`./x.py`]: https://github.com/rust-lang/rust/blob/HEAD/x.py

このドキュメントは主にユーザー向けの情報をカバーしていることに注意してください。ブートストラップの内部について読むには、[bootstrap/README.md][bootstrap-internals] をご覧ください。

[bootstrap-internals]: https://github.com/rust-lang/rust/blob/HEAD/src/bootstrap/README.md

## ブートストラップのステージ

### 概要

- ステージ 0：事前コンパイルされたコンパイラと標準ライブラリ
- ステージ 1：現在のコードから、以前のコンパイラによって
- ステージ 2：真に現在のコンパイラ
- ステージ 3：同一結果のテスト

`rustc` のコンパイルはステージで行われます。以下は、RustConf 2022 での Jynn Nelson の [ブートストラップに関するトーク][rustconf22-talk] から適応した図で、以下に詳細な説明があります。

`A`、`B`、`C`、`D` はブートストラップのステージの順序を示しています。
<span style="background-color: lightblue; color: black">青</span> のノードはダウンロードされたもの、<span style="background-color: yellow; color: black">黄色</span> のノードは `stage0` コンパイラでビルドされたもの、<span style="background-color: lightgreen; color: black">緑</span> のノードは `stage1` コンパイラでビルドされたものです。

[rustconf22-talk]: https://www.youtube.com/watch?v=oUIjG-y4zaA

```mermaid
graph TD
    s0c["stage0 compiler (1.86.0-beta.1)"]:::downloaded -->|A| s0l("stage0 std (1.86.0-beta.1)"):::downloaded;
    s0c & s0l --- stepb[ ]:::empty;
    stepb -->|B| s0ca["stage0 compiler artifacts (1.87.0-dev)"]:::with-s0c;
    s0ca -->|copy| s1c["stage1 compiler (1.87.0-dev)"]:::with-s0c;
    s1c -->|C| s1l("stage1 std (1.87.0-dev)"):::with-s1c;
    s1c & s1l --- stepd[ ]:::empty;
    stepd -->|D| s1ca["stage1 compiler artifacts (1.87.0-dev)"]:::with-s1c;
    s1ca -->|copy| s2c["stage2 compiler"]:::with-s1c;

    classDef empty width:0px,height:0px;
    classDef downloaded fill: lightblue;
    classDef with-s0c fill: yellow;
    classDef with-s1c fill: lightgreen;
```

### ステージ 0：事前コンパイルされたコンパイラ

stage0 コンパイラは、デフォルトでは非常に最近の _ベータ_ `rustc` コンパイラとそれに関連する動的ライブラリであり、`./x.py` がダウンロードします。（`./x.py` を設定して stage0 を別のものに変更することもできます。）

事前コンパイルされた stage0 コンパイラは、事前コンパイルされた stage0 std で [`src/bootstrap`] と [`compiler/rustc`] をコンパイルするためにのみ使用されます。

stage1 コンパイラをビルドするために、事前コンパイルされた stage0 コンパイラと std を使用することに注意してください。したがって、ツリーから新しくビルドされた std を持つコンパイラを使用するには、stage2 コンパイラをビルドする必要があります。

ここには2つの概念があります：コンパイラ（その依存関係のセットとともに）と、その「ターゲット」または「オブジェクト」ライブラリ（`std` と `rustc`）です。両方ともステージングされますが、交互に行われます。

[`compiler/rustc`]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc
[`src/bootstrap`]: https://github.com/rust-lang/rust/tree/HEAD/src/bootstrap

### ステージ 1：現在のコードから、以前のコンパイラによって

rustc のソースコードは、`stage0` コンパイラでコンパイルされて `stage1` コンパイラを生成します。

### ステージ 2：真に現在のコンパイラ

次に、インツリーの std を持つ stage1 コンパイラを使用してコンパイラを再ビルドし、`stage2` コンパイラを生成します。

`stage1` コンパイラ自体は、事前コンパイルされた `stage0` コンパイラと std によってビルドされたため、作業ディレクトリのソースによってビルドされていません。これは、`stage0` コンパイラによって生成された ABI が、`stage1` コンパイラによって作成された ABI と一致しない可能性があることを意味し、これは動的ライブラリ、テスト、および `rustc_private` を使用するツールに問題を引き起こす可能性があります。

`proc_macro` クレートは、`proc_macro::bridge` と呼ばれる `C` FFI レイヤーを使用してこの問題を回避し、`stage1` で使用できるようにしていることに注意してください。

`stage2` コンパイラは、`rustup` およびその他のすべてのインストール方法で配布されるものです。ただし、古いコンパイラで新しいコンパイラを最初にビルドし、次にそれを使用して新しいコンパイラをそれ自体でビルドする必要があるため、ビルドには非常に長い時間がかかります。

開発のために、通常は `--stage 1` フラグを使用してものをビルドしたいだけです。[コンパイラのビルド](../how-to-build-and-run.html#building-the-compiler) をご覧ください。

### ステージ 3：同一結果のテスト

ステージ 3 はオプションです。新しいコンパイラの健全性をチェックするために、`stage2` コンパイラでライブラリをビルドできます。結果は以前と同一であるべきです。何かが壊れていない限り。

### ステージのビルド

スクリプト [`./x`] は、各サブコマンドに対して最も意図している可能性の高いステージを選択しようとします。以下は、デフォルトのステージを持つ `x` コマンドです：

- `check`: `--stage 1`
- `clippy`: `--stage 1`
- `doc`: `--stage 1`
- `build`: `--stage 1`
- `test`: `--stage 1`
- `dist`: `--stage 2`
- `install`: `--stage 2`
- `bench`: `--stage 2`

`--stage N` を明示的に渡すことで、常にステージをオーバーライドできます。

ステージについての詳細は、[以下を参照してください](#understanding-stages-of-bootstrap)。

[`./x`]: https://github.com/rust-lang/rust/blob/HEAD/x

## ブートストラップの複雑さ

ビルドシステムは現在のベータコンパイラを使用して `stage1` ブートストラッピングコンパイラをビルドするため、コンパイラのソースコードは、ベータに到達するまで一部の機能を使用できません（そうでなければ、ベータコンパイラがそれらをサポートしないため）。一方、[コンパイライントリンシック][intrinsics] と内部機能については、機能を使用する _必要があります_。さらに、コンパイラは `nightly` 機能（`#![feature(...)]`）を多用しています。この問題をどのように解決できるでしょうか？

使用される方法は2つあります：

1. ビルドシステムは `stage0` でビルドする際に `--cfg bootstrap` を設定するため、`cfg(not(bootstrap))` を使用して `stage1` でビルドする際にのみ機能を使用できます。この方法で `--cfg bootstrap` を設定することは、安定化されたばかりの機能に使用され、`stage0` でビルドする際には `#![feature(...)]` が必要ですが、`stage1` では必要ありません。
2. ビルドシステムは `RUSTC_BOOTSTRAP=1` を設定します。この特別な変数は、Rust の安定性保証を _破る_ ことを意味します：`nightly` でないコンパイラで `#![feature(...)]` の使用を許可します。_`RUSTC_BOOTSTRAP=1` の設定は、コンパイラをブートストラップする場合を除いて決して使用すべきではありません。_

[boot]: https://en.wikipedia.org/wiki/Bootstrapping_(compilers)
[intrinsics]: ../../appendix/glossary.md#intrinsic
[ocaml-compiler]: https://github.com/rust-lang/rust/tree/ef75860a0a72f79f97216f8aaa5b388d98da6480/src/boot

## ブートストラップのステージの理解

### 概要

これはブートストラップの個別のステージについての詳細な説明です。

`./x` が使用する規約は次のとおりです：

- `--stage N` フラグは、ステージ N コンパイラ（`stageN/rustc`）を実行することを意味します。
- 「ステージ N アーティファクト」は、ステージ N コンパイラによって _生成された_ ビルドアーティファクトです。
- ステージ N+1 コンパイラは、ステージ N の *アーティファクト* から組み立てられます。このプロセスは _アップリフト_ と呼ばれます。

#### ビルドアーティファクト

`./x` でビルドできるものはすべて _ビルドアーティファクト_ です。ビルドアーティファクトには以下が含まれますが、これに限定されません：

- バイナリ、`stage0-rustc/rustc-main` など
- 共有オブジェクト、`stage0-sysroot/rustlib/libstd-6fae108520cf72fe.so` など
- [rlib] ファイル、`stage0-sysroot/rustlib/libstd-6fae108520cf72fe.rlib` など
- rustdoc によって生成された HTML ファイル、`doc/std` など

[rlib]: ../../serialization.md

#### 例

- `./x test tests/ui` は、`stage1` コンパイラをビルドし、それに対して `compiletest` を実行することを意味します。コンパイラに取り組んでいる場合、これは通常使用したいテストコマンドです。
- `./x test --stage 0 library/std` は、ソースから `rustc` をビルドせずに標準ライブラリのテストを実行することを意味します（「stage0 でビルドし、次にアーティファクトをテストする」）。標準ライブラリに取り組んでいる場合、これは通常使用したいテストコマンドです。
- `./x build --stage 0` は、stage0 `rustc` でビルドすることを意味します。
- `./x doc --stage 1` は、stage0 `rustdoc` を使用してドキュメント化することを意味します。

#### やるべきでないことの例

- `./x test --stage 0 tests/ui` は有用ではありません：_ベータ_ コンパイラでテストを実行し、ソースから `rustc` をビルドしません。代わりに `test tests/ui` を使用してください。これはソースから `stage1` をビルドします。
- `./x test --stage 0 compiler/rustc` はコンパイラをビルドしますが、テストを実行しません：`cargo test -p rustc` を実行していますが、`cargo` は Rust のテストを理解しません。これを使用する必要はありません。代わりに `test`（引数なし）を使用してください。
- `./x build --stage 0 compiler/rustc` はコンパイラをビルドしますが、`libstd` や `libcore` さえもビルドしません。ほとんどの場合、代わりに `./x build library` を使用したいでしょう。これにより、lang アイテムを定義せずにプログラムをコンパイルできます。

### ビルドと実行

簡単に言うと、_ステージ 0 は `stage0` コンパイラを使用して `stage0` アーティファクトを作成し、後で stage1 コンパイラにアップリフトされます_。

0 以外の各ステージでは、2つの主要なステップが実行されます：

1. `std` がステージ N コンパイラによってコンパイルされます。
2. その `std` は、ステージ N アーティファクト（ステージ N+1 コンパイラ）を含む、ステージ N コンパイラによってビルドされたプログラムにリンクされます。

これは、ステージ N アーティファクトをステージ N コンパイラでビルドしている「単なる」別のプログラムと考えると、やや直感的です：`build --stage N compiler/rustc` は、ステージ N アーティファクトをステージ N コンパイラでビルドした `std` にリンクしています。

### ステージと `std`

ここでは2つの `std` ライブラリが関係していることに注意してください：

1. `stageN/rustc` に _リンクされた_ ライブラリ。これはステージ N-1 によってビルドされました（ステージ N-1 `std`）
2. `stageN/rustc` でプログラムを _コンパイルするために使用される_ ライブラリ。これはステージ N によってビルドされました（ステージ N `std`）。

ステージ N `std` は、ステージ N コンパイラで有用な作業を行うためにほぼ必須です。それがなければ、`#![no_core]` のプログラムのみをコンパイルできます -- あまり有用ではありません！

これらが異なる必要がある理由は、それらが必ずしも ABI 互換ではないためです：新しいレイアウト最適化、`MIR` への変更、または `nightly` にあるがベータにはない Rust メタデータへのその他の変更がある可能性があります。

これは、`--keep-stage 1 library/std` が機能する場所でもあります。コンパイラへのほとんどの変更は実際には ABI を変更しないため、`stage1` で `std` を生成したら、別のコンパイラでそれを再利用できる可能性があります。ABI が変更されていなければ、問題なく、その `std` を再コンパイルする時間を費やす必要はありません。フラグ `--keep-stage` は、ビルドスクリプトに以前のコンパイルが問題ないと想定し、それらのアーティファクトを適切な場所にコピーするように指示し、`cargo` の呼び出しをスキップします。

### rustc のクロスコンパイル

*クロスコンパイル* は、別のアーキテクチャで実行されるコードをコンパイルするプロセスです。たとえば、x86 マシンを使用して ARM バージョンの rustc をビルドしたい場合があります。`stage2` `std` のビルドは、クロスコンパイルしている場合に異なります。

これは、`./x` が次のロジックを使用するためです：`HOST` と `TARGET` が同じ場合、`stage2` に `stage1` `std` を再利用します！これは、`stage1` `std` が `stage1` コンパイラでコンパイルされた、つまり現在チェックアウトしているソースコードを使用するコンパイラでコンパイルされたため、健全です。したがって、`stage2/rustc` がコンパイルする `std` と同一（したがって ABI 互換）であるべきです。

ただし、クロスコンパイルする場合、`stage1` `std` はホストでのみ実行されます。したがって、`stage2` コンパイラは、ターゲット用に `std` を再コンパイルする必要があります。

（テーブルで `stage2` が非ホスト `std` ターゲットのみをビルドする方法を参照してください）。

### 'sysroot' とは何ですか？

`cargo` でプロジェクトをビルドすると、依存関係のビルドアーティファクトは通常 `target/debug/deps` に保存されます。これには、`cargo` が知っている依存関係のみが含まれます。特に、標準ライブラリは含まれません。`std` や `proc_macro` はどこから来るのでしょうか？それらは **sysroot** から来ます。これは、コンパイラが実行時にビルドアーティファクトをロードする多数のディレクトリのルートです。`sysroot` は標準ライブラリだけを保存するのではありません -- 実行時にロードする必要があるものすべてが含まれます。それには以下が含まれます（ただし、これに限定されません）：

- ライブラリ `libstd`/`libtest`/`libproc_macro`。
- `rustc_private` を使用する場合のコンパイラクレート自体。インツリーでは常に存在します。ツリー外では、`rustup` で `rustc-dev` をインストールする必要があります。
- LLVM プロジェクトの共有オブジェクトファイル `libLLVM.so`。インツリーでは、これはソースからビルドされるか、CI からダウンロードされます。ツリー外では、`rustup` で `llvm-tools-preview` をインストールする必要があります。

これまでにリストされたすべてのアーティファクトは *コンパイラ* の実行時依存関係です。それらは `rustc --print sysroot` で確認できます：

```
$ ls $(rustc --print sysroot)/lib
libchalk_derive-0685d79833dc9b2b.so  libstd-25c6acf8063a3802.so
libLLVM-11-rust-1.50.0-nightly.so    libtest-57470d2aa8f7aa83.so
librustc_driver-4f0cc9f50e53f0ba.so  libtracing_attributes-e4be92c35ab2a33b.so
librustc_macros-5f0ec4a119c6ac86.so  rustlib
```

標準ライブラリの実行時依存関係もあります！これらは `lib/` ではなく `lib/rustlib/` にあります。

```
$ ls $(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/lib | head -n 5
libaddr2line-6c8e02b8fedc1e5f.rlib
libadler-9ef2480568df55af.rlib
liballoc-9c4002b5f79ba0e1.rlib
libcfg_if-512eb53291f6de7e.rlib
libcompiler_builtins-ef2408da76957905.rlib
```

ディレクトリ `lib/rustlib/` には、標準ライブラリの公開 API の一部ではないが、それを実装するために使用される `hashbrown` や `cfg_if` などのライブラリが含まれます。また、`lib/rustlib/` はリンカーの検索パスの一部ですが、`lib` は検索パスの一部になることはありません。

#### `-Z force-unstable-if-unmarked`

`lib/rustlib/` は検索パスの一部であるため、そこに含まれるクレートについて慎重である必要があります。特に、標準ライブラリを除くすべてのクレートは、`-Z force-unstable-if-unmarked` フラグでビルドされます。これは、それをロードするために `#![feature(rustc_private)]` を使用する必要があることを意味します（常に利用可能な標準ライブラリとは対照的に）。

`-Z force-unstable-if-unmarked` フラグには、正しいクレートが `unstable` としてマークされていることを強制するためのさまざまな目的があります。これは主に、rustc と標準ライブラリが、`staged_api` 自体を使用しない crates.io 上の任意のクレートにリンクできるようにするために導入されました。`rustc` はまた、このフラグに依存して、すべてのクレートを `rustc_private` 機能で `unstable` としてマークしており、各クレートを慎重に `unstable` でマークする必要がありません。

このフラグは、ブートストラップスクリプトによって `rustc` と標準ライブラリのすべてに自動的に適用されます。これは、コンパイラとそのすべての依存関係が、すべてのユーザーに `sysroot` で配布されるために必要です。

このフラグには次の効果があります：

- クレート自体が `stable` または `unstable` としてマークされていない場合、`rustc_private` 機能でクレートを「`unstable`」としてマークします。
- これらのクレートが、属性を必要とせずに他の強制的に不安定なクレートにアクセスできるようにします。通常、クレートは他の `unstable` クレートを使用するために `#![feature(rustc_private)]` 属性が必要です。ただし、それは crates.io のクレートが自身の依存関係にアクセスすることを不可能にします。なぜなら、そのクレートには `feature(rustc_private)` 属性がありませんが、*すべて* が `-Z force-unstable-if-unmarked` でコンパイルされるからです。

`-Z force-unstable-if-unmarked` を使用しないコードは、これらの強制的に不安定なクレートにアクセスするために `#![feature(rustc_private)]` クレート属性を含める必要があります。これは、`MIRI` や `clippy` などの `rustc` 自体をリンクするものに必要です。

sysroot についてのさらなる議論は以下で見つけることができます：
- `sysroot` からロードされた依存関係に `extern crate` を使用する理由を説明する [rustdoc PR]
- [Zulip での sysroot に関する議論](https://rust-lang.zulipchat.com/#narrow/stream/182449-t-compiler.2Fhelp/topic/deps.20in.20sysroot/)
- [ツリー外で rustdoc をビルドすることに関する議論](https://rust-lang.zulipchat.com/#narrow/stream/182449-t-compiler.2Fhelp/topic/How.20to.20create.20an.20executable.20accessing.20.60rustc_private.60.3F)

[rustdoc PR]: https://github.com/rust-lang/rust/pull/76728

## `bootstrap` によって呼び出されるコマンドへのフラグの渡し方

便利なことに、`./x` を使用すると、ブートストラップ時に `rustc` と `cargo` にステージ固有のフラグを渡すことができます。`RUSTFLAGS_BOOTSTRAP` 環境変数は、ブートストラップステージ（`stage0`）に `RUSTFLAGS` として渡され、`RUSTFLAGS_NOT_BOOTSTRAP` は後のステージのアーティファクトをビルドする際に渡されます。`RUSTFLAGS` は機能しますが、`bootstrap` 自体のビルドにも影響するため、使用したいことはまれです。最後に、`MAGIC_EXTRA_RUSTFLAGS` は、すべての依存関係を再コンパイルせずに rustc にフラグを渡すために `cargo` キャッシュをバイパスします。

- `RUSTDOCFLAGS`、`RUSTDOCFLAGS_BOOTSTRAP`、`RUSTDOCFLAGS_NOT_BOOTSTRAP` は `RUSTFLAGS` に類似していますが、`rustdoc` 用です。
- `CARGOFLAGS` は cargo 自体に引数を渡します（例：`--timings`）。`CARGOFLAGS_BOOTSTRAP` と `CARGOFLAGS_NOT_BOOTSTRAP` は `RUSTFLAGS_BOOTSTRAP` と同様に機能します。
- `--test-args` はテストランナーに引数を渡します。`tests/ui` の場合、これは `compiletest` です。ユニットテストと doc テストの場合、これは `libtest` ランナーです。

ほとんどのテストランナーは `--help` を受け入れ、これを使用してランナーが受け入れるオプションを見つけることができます。

## 環境変数

ブートストラップ中には、使用されるコンパイラ内部の環境変数が多数あります。中間バージョンの `rustc` を実行しようとしている場合、これらの環境変数の一部を手動で設定する必要がある場合があります。そうでないと、次のようなエラーが発生します：

```text
thread 'main' panicked at 'RUSTC_STAGE was not set: NotPresent', library/core/src/result.rs:1165:5
```

`./stageN/bin/rustc` が環境変数に関するエラーを出す場合、通常は何かが非常に間違っていることを意味します -- `rustc` や `std` または環境変数に依存する何かをコンパイルしようとしているなど。そのような状況で実際に `rustc` を呼び出す必要がある unlikely なケースでは、`x` コマンドに `-vvv` を追加することで、ブートストラップ shim にすべての `env` 変数を出力させることができます。

最後に、ブートストラップは [cc-rs crate] を使用しており、これには環境変数を介して `C` コンパイラと `C` フラグを設定する [独自の方法][env-vars] があります。

[cc-rs crate]: https://github.com/rust-lang/cc-rs
[env-vars]: https://docs.rs/cc/latest/cc/#external-configuration-via-environment-variables

## ビルドコマンドの `stdout` の説明

このパートでは、アクション内のビルドコマンドの `stdout` を調査します（上記のトピックと同様ですが、より詳細で完全なドキュメント）。`x build --dry-run` コマンドを実行すると、ビルド出力は次のようになります：

```text
Building stage0 library artifacts (x86_64-unknown-linux-gnu -> x86_64-unknown-linux-gnu)
Copying stage0 library from stage0 (x86_64-unknown-linux-gnu -> x86_64-unknown-linux-gnu / x86_64-unknown-linux-gnu)
Building stage0 compiler artifacts (x86_64-unknown-linux-gnu -> x86_64-unknown-linux-gnu)
Copying stage0 rustc from stage0 (x86_64-unknown-linux-gnu -> x86_64-unknown-linux-gnu / x86_64-unknown-linux-gnu)
Assembling stage1 compiler (x86_64-unknown-linux-gnu)
Building stage1 library artifacts (x86_64-unknown-linux-gnu -> x86_64-unknown-linux-gnu)
Copying stage1 library from stage1 (x86_64-unknown-linux-gnu -> x86_64-unknown-linux-gnu / x86_64-unknown-linux-gnu)
Building stage1 tool rust-analyzer-proc-macro-srv (x86_64-unknown-linux-gnu)
Building rustdoc for stage1 (x86_64-unknown-linux-gnu)
```

### Building stage0 {std,compiler} artifacts

これらのステップは、提供された（通常はダウンロードされた）コンパイラを使用して、ローカルの Rust ソースを使用できるライブラリにコンパイルします。

### Copying stage0 \{std,rustc\}

これは、ライブラリとコンパイラのアーティファクトを `cargo` から `stage0-sysroot/lib/rustlib/{target-triple}/lib` にコピーします

### Assembling stage1 compiler

これは、「stage0 ... artifacts のビルド」でビルドしたライブラリを `stage1` コンパイラの `lib/` ディレクトリにコピーします。これらは、コンパイラ自体が実行するために使用するホストライブラリです。これらは、新しいコンパイラが生成するアーティファクトによって実際に使用されるわけではありません。このステップでは、生成した `rustc` と `rustdoc` バイナリも `build/$HOST/stage/bin` にコピーします。

`stage1/bin/rustc` は、stage0（事前コンパイルされた）コンパイラと std でビルドされた完全に機能的なコンパイラです。インツリーのコンパイラと std で完全にソースからビルドされたコンパイラを使用するには、stage1（インツリー）コンパイラと std を使用してコンパイルされた stage2 コンパイラをビルドする必要があります。
