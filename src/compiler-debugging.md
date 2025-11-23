# コンパイラのデバッグ

本章では、コンパイラをデバッグするためのいくつかのヒントを紹介します。これらのヒントは、作業内容に関わらず役立つことを目指しています。他の章の中には、コンパイラの特定の部分についてのアドバイスがあるものもあります(例えば、[Queries Debugging and Testing chapter](./incrcomp-debugging.html)や[LLVM Debugging chapter](./backend/debugging.md))。

## コンパイラの設定

デフォルトでは、rustcはほとんどのデバッグ情報なしでビルドされます。デバッグ情報を有効にするには、bootstrap.tomlで`debug = true`を設定してください。

`debug = true`を設定すると、多くの異なるデバッグオプション(例:`debug-assertions`、`debug-logging`など)が有効になります。これらは必要に応じて個別に調整することもできますが、多くの人は単に`debug = true`を設定します。

GDBを使用してrustcをデバッグしたい場合は、以下のオプションで`bootstrap.toml`を設定してください:

```toml
[rust]
debug = true
debuginfo-level = 2
```

> NOTE:
> これは大量のディスク容量を使用します
> (<!-- date-check Aug 2022 -->35GB以上)、
> そしてコンパイル時間も大幅に長くなります。
> `debuginfo-level = 1`(`debug = true`の場合のデフォルト)では、
> 実行パスを追跡できますが、
> デバッグ用のシンボル情報は失われます。

デフォルト設定では、`symbol-mangling-version` v0が有効になります。
これには少なくともGDB v10.2が必要です。
それ以外の場合は、`bootstrap.toml`で新しいsymbol-mangling-versionを無効にする必要があります。

```toml
[rust]
new-symbol-mangling = false
```

> 詳細については、`bootstrap.example.toml`のコメントを参照してください。

設定オプションを変更した後は、コンパイラを再ビルドする必要があります。

## ICEファイルの抑制

デフォルトでは、rustcが内部コンパイラエラー(ICE)に遭遇すると、現在の作業ディレクトリ内に`rustc-ice-<timestamp>-<pid>.txt`という名前のICEファイルにICEの内容をダンプします。これが望ましくない場合は、`RUSTC_ICE=0`を設定することでICEファイルの作成を防ぐことができます。

## バックトレースの取得
[getting-a-backtrace]: #getting-a-backtrace

ICE(コンパイラ内のパニック)が発生した場合、`RUST_BACKTRACE=1`を設定することで、通常のRustプログラムと同様に`panic!`のスタックトレースを取得できます。MinGWではバックトレースが**動作しません**。バックトレースが`unknown`で埋め尽くされている場合や問題がある場合は、Linux、Mac、またはWindows上のMSVCを使用する方法を見つけることをお勧めします。

デフォルト設定(`debug`を`true`に設定していない場合)では、行番号が有効になっていないため、バックトレースは次のようになります:

```text
stack backtrace:
   0: std::sys::imp::backtrace::tracing::imp::unwind_backtrace
   1: std::sys_common::backtrace::_print
   2: std::panicking::default_hook::{{closure}}
   3: std::panicking::default_hook
   4: std::panicking::rust_panic_with_hook
   5: std::panicking::begin_panic
   (~~~~ LINES REMOVED BY ME FOR BREVITY ~~~~)
  32: rustc_typeck::check_crate
  33: <std::thread::local::LocalKey<T>>::with
  34: <std::thread::local::LocalKey<T>>::with
  35: rustc::ty::context::TyCtxt::create_and_enter
  36: rustc_driver::driver::compile_input
  37: rustc_driver::run_compiler
```

`debug = true`を設定すると、スタックトレースに行番号が表示されます。
その場合、バックトレースは次のようになります:

```text
stack backtrace:
   (~~~~ LINES REMOVED BY ME FOR BREVITY ~~~~)
             at /home/user/rust/compiler/rustc_typeck/src/check/cast.rs:110
   7: rustc_typeck::check::cast::CastCheck::check
             at /home/user/rust/compiler/rustc_typeck/src/check/cast.rs:572
             at /home/user/rust/compiler/rustc_typeck/src/check/cast.rs:460
             at /home/user/rust/compiler/rustc_typeck/src/check/cast.rs:370
   (~~~~ LINES REMOVED BY ME FOR BREVITY ~~~~)
  33: rustc_driver::driver::compile_input
             at /home/user/rust/compiler/rustc_driver/src/driver.rs:1010
             at /home/user/rust/compiler/rustc_driver/src/driver.rs:212
  34: rustc_driver::run_compiler
             at /home/user/rust/compiler/rustc_driver/src/lib.rs:253
```

## `-Z`フラグ

コンパイラには多数の`-Z *`フラグがあります。これらは、ナイトリーでのみ有効な不安定なフラグです。その多くはデバッグに役立ちます。`-Z`フラグの完全なリストを取得するには、`-Z help`を使用してください。

便利なフラグの1つは`-Z verbose-internals`です。これは一般的に、デバッグに役立つ可能性のある詳細情報の出力を有効にします。

以下では、選ばれたいくつかについて詳しく説明します。

### エラーのバックトレースを取得する
[getting-a-backtrace-for-errors]: #getting-a-backtrace-for-errors

コンパイラがエラーメッセージを出力する地点までのバックトレースを取得したい場合は、`-Z treat-err-as-bug=n`を渡すことができます。これにより、コンパイラは`n`番目のエラーでパニックします。`=n`を省略すると、コンパイラは`n`に`1`を仮定し、最初のエラーでパニックします。

例:

```bash
$ cat error.rs
```

```rust
fn main() {
    1 + ();
}
```

```
$ rustc +stage1 error.rs
error[E0277]: cannot add `()` to `{integer}`
 --> error.rs:2:7
  |
2 |       1 + ();
  |         ^ no implementation for `{integer} + ()`
  |
  = help: the trait `Add<()>` is not implemented for `{integer}`

error: aborting due to previous error
```

さて、上記のエラーはどこから来ているのでしょうか?

```
$ RUST_BACKTRACE=1 rustc +stage1 error.rs -Z treat-err-as-bug
error[E0277]: the trait bound `{integer}: std::ops::Add<()>` is not satisfied
 --> error.rs:2:7
  |
2 |     1 + ();
  |       ^ no implementation for `{integer} + ()`
  |
  = help: the trait `std::ops::Add<()>` is not implemented for `{integer}`

error: internal compiler error: unexpected panic

note: the compiler unexpectedly panicked. this is a bug.

note: we would appreciate a bug report: https://github.com/rust-lang/rust/blob/HEAD/CONTRIBUTING.md#bug-reports

note: rustc 1.24.0-dev running on x86_64-unknown-linux-gnu

note: run with `RUST_BACKTRACE=1` for a backtrace

thread 'rustc' panicked at 'encountered error with `-Z treat_err_as_bug',
/home/user/rust/compiler/rustc_errors/src/lib.rs:411:12
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose
backtrace.
stack backtrace:
  (~~~ IRRELEVANT PART OF BACKTRACE REMOVED BY ME ~~~)
   7: rustc::traits::error_reporting::<impl rustc::infer::InferCtxt<'a, 'tcx>>
             ::report_selection_error
             at /home/user/rust/compiler/rustc_middle/src/traits/error_reporting.rs:823
   8: rustc::traits::error_reporting::<impl rustc::infer::InferCtxt<'a, 'tcx>>
             ::report_fulfillment_errors
             at /home/user/rust/compiler/rustc_middle/src/traits/error_reporting.rs:160
             at /home/user/rust/compiler/rustc_middle/src/traits/error_reporting.rs:112
   9: rustc_typeck::check::FnCtxt::select_obligations_where_possible
             at /home/user/rust/compiler/rustc_typeck/src/check/mod.rs:2192
  (~~~ IRRELEVANT PART OF BACKTRACE REMOVED BY ME ~~~)
  36: rustc_driver::run_compiler
             at /home/user/rust/compiler/rustc_driver/src/lib.rs:253
```

素晴らしい!エラーのバックトレースが得られました!

### 遅延バグのデバッグ

`-Z eagerly-emit-delayed-bugs`オプションを使用すると、遅延バグのデバッグが簡単になります。
これは遅延バグを通常のエラーに変換し、表示されるようにします。これは`-Z treat-err-as-bug`と組み合わせて使用することで、特定の遅延バグで停止してバックトレースを取得できます。

### エラー作成場所の取得

`-Z track-diagnostics`は、エラーが出力される場所を把握するのに役立ちます。これは`#[track_caller]`を使用し、エラーと一緒にその場所を出力します:

```
$ RUST_BACKTRACE=1 rustc +stage1 error.rs -Z track-diagnostics
error[E0277]: cannot add `()` to `{integer}`
 --> src\error.rs:2:7
  |
2 |     1 + ();
  |       ^ no implementation for `{integer} + ()`
-Ztrack-diagnostics: created at compiler/rustc_trait_selection/src/traits/error_reporting/mod.rs:638:39
  |
  = help: the trait `Add<()>` is not implemented for `{integer}`
  = help: the following other types implement trait `Add<Rhs>`:
            <&'a f32 as Add<f32>>
            <&'a f64 as Add<f64>>
            <&'a i128 as Add<i128>>
            <&'a i16 as Add<i16>>
            <&'a i32 as Add<i32>>
            <&'a i64 as Add<i64>>
            <&'a i8 as Add<i8>>
            <&'a isize as Add<isize>>
          and 48 others

For more information about this error, try `rustc --explain E0277`.
```

これは`-Z treat-err-as-bug`と似ていますが異なります:
- 出力されるすべてのエラーの場所を出力します
- デバッグシンボル付きでビルドされたコンパイラは必要ありません
- 大きなスタックトレースを読む必要がありません

## ログ出力の取得

コンパイラはログ記録のために[`tracing`]クレートを使用しています。

[`tracing`]: https://docs.rs/tracing

詳細については、[the guide section on tracing](./tracing.md)を参照してください。

## リグレッションの絞り込み(二分探索)

[cargo-bisect-rustc][bisect]ツールは、`rustc`の動作を変更した正確なPRを見つけるための迅速で簡単な方法として使用できます。提供したプロジェクトに対して、リグレッションが見つかるまで`rustc`のPRアーティファクトを自動的にダウンロードしてテストします。その後、そのPRを調べて*なぜ*変更されたかのコンテキストを得ることができます。使用方法については、[this tutorial][bisect-tutorial]を参照してください。

[bisect]: https://github.com/rust-lang/cargo-bisect-rustc
[bisect-tutorial]: https://rust-lang.github.io/cargo-bisect-rustc/tutorial.html

## RustのCIからアーティファクトをダウンロードする

kennytmによる[rustup-toolchain-install-master][rtim]ツールを使用して、特定のSHA1に対してRustのCIによって生成されたアーティファクトをダウンロードできます。これは基本的に、いくつかのPRの正常なランディングに対応します。ローカル使用のためにそれらをセットアップします。これは`@bors try`によって生成されたアーティファクトでも機能します。これは、自分でビルドせずにPRの結果ビルドを調べたい場合に役立ちます。

[rtim]: https://github.com/kennytm/rustup-toolchain-install-master

## `#[rustc_*]` TEST属性

コンパイラは、多くの内部(永続的に不安定な)属性を定義しています。その一部は、追加のコンパイラ内部情報をダンプすることによってデバッグに役立ちます。これらには`rustc_`というプレフィックスが付けられ、内部機能`rustc_attrs`の背後にゲートされています(例:`#![feature(rustc_attrs)]`で有効化)。

完全かつ最新のリストについては、[`builtin_attrs`]を参照してください。より具体的には、`TEST`とマークされたものです。
注目すべきものをいくつか紹介します:

| 属性 | 説明 |
|----------------|-------------|
| `rustc_def_path` | アイテムの[`def_path_str`]をダンプします。 |
| `rustc_dump_def_parents` | 特定の定義の`DefId`親のチェーンをダンプします。 |
| `rustc_dump_item_bounds` | アイテムの[`item_bounds`]をダンプします。 |
| `rustc_dump_predicates` | アイテムの[`predicates_of`]をダンプします。 |
| `rustc_dump_vtable` | implまたはdyn型の型エイリアスのvtableレイアウトをダンプします。 |
| `rustc_hidden_type_of_opaques` | クレート内の各不透明型の[hidden type][opaq]をダンプします。 |
| `rustc_layout` | [このセクション](#debugging-type-layouts)を参照してください。 |
| `rustc_object_lifetime_default` | アイテムの[object lifetime defaults]をダンプします。 |
| `rustc_outlives` | アイテムの暗黙的な境界をダンプします。より正確には、アイテムの[`inferred_outlives_of`]です。 |
| `rustc_regions` | NLLクロージャのリージョン要件をダンプします。 |
| `rustc_symbol_name` | アイテムのマングルされた&デマングルされた[`symbol_name`]をダンプします。 |
| `rustc_variances` | アイテムの[variances]をダンプします。 |

以下では、選ばれたいくつかについて詳しく説明します。

[`builtin_attrs`]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_feature/src/builtin_attrs.rs
[`def_path_str`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.def_path_str
[`inferred_outlives_of`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.inferred_outlives_of
[`item_bounds`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.item_bounds
[`predicates_of`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.predicates_of
[`symbol_name`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.symbol_name
[object lifetime defaults]: https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes
[opaq]: ./opaque-types-impl-trait-inference.md
[variances]: ./variance.md

### Graphviz出力のフォーマット(.dotファイル)
[formatting-graphviz-output]: #formatting-graphviz-output

特定の機能をデバッグするためのコンパイラオプションの中には、graphvizグラフを生成するものがあります。例えば、関数に付けられた`#[rustc_mir(borrowck_graphviz_postflow="suffix.dot")]`属性は、`-Zdump-mir-dataflow`と組み合わせて、様々なボローチェッカーのデータフローグラフをダンプします。

これらはすべて`.dot`ファイルを生成します。これらのファイルを表示するには、graphvizをインストールし(例:`apt-get install graphviz`)、次のコマンドを実行してください:

```bash
$ dot -T pdf maybe_init_suffix.dot > maybe_init_suffix.pdf
$ firefox maybe_init_suffix.pdf # またはお好みのPDFビューア
```

### 型レイアウトのデバッグ

内部属性`#[rustc_layout]`を使用して、それが付けられた型の[`Layout`]をダンプできます。例:

```rust
#![feature(rustc_attrs)]

#[rustc_layout(debug)]
type T<'a> = &'a u32;
```

次のように出力されます:

```text
error: layout_of(&'a u32) = Layout {
    fields: Primitive,
    variants: Single {
        index: 0,
    },
    abi: Scalar(
        Scalar {
            value: Pointer,
            valid_range: 1..=18446744073709551615,
        },
    ),
    largest_niche: Some(
        Niche {
            offset: Size {
                raw: 0,
            },
            scalar: Scalar {
                value: Pointer,
                valid_range: 1..=18446744073709551615,
            },
        },
    ),
    align: AbiAndPrefAlign {
        abi: Align {
            pow2: 3,
        },
        pref: Align {
            pow2: 3,
        },
    },
    size: Size {
        raw: 8,
    },
}
 --> src/lib.rs:4:1
  |
4 | type T<'a> = &'a u32;
  | ^^^^^^^^^^^^^^^^^^^^^

error: aborting due to previous error
```

[`Layout`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/abi/struct.Layout.html


## `rustc`をデバッグするためのCodeLLDBの設定

VSCodeを使用していて、関心のあるコードの部分に対してデバッグレベル1または2をリクエストするように`bootstrap.toml`を編集した場合、VSCodeの[CodeLLDB]拡張機能を使用してデバッグできるはずです。

以下は、ビルドされたディレクトリから直接ステージ1コンパイラを実行するために使用される`launch.json`ファイルのサンプルです(「インストール」する必要はありません):

```javascript
// .vscode/launch.json
{
    "version": "0.2.0",
    "configurations": [
      {
        "type": "lldb",
        "request": "launch",
        "name": "Launch",
        "args": [],  // コンパイラに渡す文字列コマンドライン引数の配列
        "program": "${workspaceFolder}/build/host/stage1/bin/rustc",
        "windows": {  // windowsを使用している場合に適用
            "program": "${workspaceFolder}/build/host/stage1/bin/rustc.exe"
        },
        "cwd": "${workspaceFolder}",  // プログラム起動時の現在の作業ディレクトリ
        "stopOnEntry": false,
        "sourceLanguages": ["rust"]
      }
    ]
  }
```

[CodeLLDB]: https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb
