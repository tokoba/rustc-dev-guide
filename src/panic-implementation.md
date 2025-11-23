# Rustにおけるパニック

## ステップ1: `panic!` マクロの呼び出し

実際には2つのパニックマクロがあります - 1つは `core` で定義され、もう1つは `std` で定義されています。これは、`core` のコードがパニックする可能性があるためです。`core` は `std` の前にビルドされますが、パニックが `core` で発生したか `std` で発生したかにかかわらず、実行時に同じ仕組みを使用することを望んでいます。

### core でのパニックの定義

`core` の `panic!` マクロは最終的に以下の呼び出しを行います（`library/core/src/panicking.rs` 内）：

```rust
// NOTE この関数はFFI境界を越えません；Rust-to-Rustの呼び出しです
extern "Rust" {
    #[lang = "panic_impl"]
    fn panic_impl(pi: &PanicInfo<'_>) -> !;
}

let pi = PanicInfo::internal_constructor(Some(&fmt), location);
unsafe { panic_impl(&pi) }
```

実際にこれを解決するには、いくつかの間接層を経ます：

1. `compiler/rustc_middle/src/middle/weak_lang_items.rs` で、`panic_impl` はシンボル `rust_begin_unwind` を持つ 'weak lang item' として宣言されています。これは `rustc_hir_analysis/src/collect.rs` で実際のシンボル名を `rust_begin_unwind` に設定するために使用されます。

   `panic_impl` は `extern "Rust"` ブロックで宣言されていることに注意してください。これは、core が `rust_begin_unwind` という外部シンボルを呼び出そうとすることを意味します（リンク時に解決されます）。

2. `library/std/src/panicking.rs` には、次の定義があります：

```rust
/// coreクレートからのパニックのエントリーポイント。
#[cfg(not(test))]
#[panic_handler]
#[unwind(allowed)]
pub fn begin_panic_handler(info: &PanicInfo<'_>) -> ! {
    ...
}
```

特別な `panic_handler` 属性は `compiler/rustc_middle/src/middle/lang_items` を介して解決されます。`extract` 関数は `panic_handler` 属性を `panic_impl` lang item に変換します。

これで、`std` に一致する `panic_handler` lang item が得られました。この関数は `core` の `extern { fn panic_impl }` 定義と同じプロセスを経て、最終的に `rust_begin_unwind` というシンボル名になります。リンク時に、`core` のシンボル参照は `std` の定義（Rustソースでは `begin_panic_handler` という関数）に解決されます。

したがって、制御フローは実行時に core から std に渡されます。これにより、`core` からのパニックが他のパニックが使用するのと同じインフラストラクチャ（パニックフック、アンワインディングなど）を使用できるようになります。

### std でのパニックの実装

ここから実際のパニック関連のロジックが始まります。`library/std/src/panicking.rs` で、制御は `rust_panic_with_hook` に渡されます。このメソッドはグローバルパニックフックの呼び出しと二重パニックのチェックを担当します。最後に、パニックランタイムによって提供される `__rust_start_panic` を呼び出します。

`__rust_start_panic` への呼び出しは非常に奇妙です - `*mut &mut dyn PanicPayload` が `usize` に変換されて渡されます。この型を分解してみましょう：

1. `PanicPayload` は内部トレイトです。これは `PanicPayload`（ユーザー提供のペイロード型のラッパー）に実装されており、メソッド `fn take_box(&mut self) -> *mut (dyn Any + Send)` を持っています。このメソッドは、ユーザーが提供したペイロード（`T: Any + Send`）を受け取り、ボックス化して、そのボックスを生のポインタに変換します。

2. `__rust_start_panic` を呼び出すとき、`&mut dyn PanicPayload` があります。しかし、これはファットポインタ（`usize` の2倍のサイズ）です。これをFFI境界を越えてパニックランタイムに渡すために、*この可変参照への*可変参照（`&mut &mut dyn PanicPayload`）を取り、それを生のポインタ（`*mut &mut dyn PanicPayload`）に変換します。外側の生のポインタは、`Sized` 型（可変参照）を指すため、シンポインタです。したがって、このシンポインタを `usize` に変換でき、FFI境界を越えて渡すのに適しています。

最後に、この `usize` で `__rust_start_panic` を呼び出します。これでパニックランタイムに入りました。

## ステップ2: パニックランタイム

Rustは2つのパニックランタイムを提供します：`panic_abort` と `panic_unwind`。ユーザーは `Cargo.toml` でビルド時にこれらを選択します。

`panic_abort` は非常にシンプルです：`__rust_start_panic` の実装は、予想通りに中止するだけです。

`panic_unwind` はより興味深いケースです。

`__rust_start_panic` の実装では、`usize` を取り、それを `*mut &mut dyn PanicPayload` に戻し、それを参照解除し、`&mut dyn PanicPayload` で `take_box` を呼び出します。この時点で、ペイロード自体への生のポインタ（`*mut (dyn Send + Any)`）があります：つまり、`panic!` を呼び出したユーザーが提供した実際の値への生のポインタです。

この時点で、プラットフォーム非依存のコードは終了します。次に、プラットフォーム固有のアンワインディングロジック（例えば `unwind`）を呼び出します。このコードは、スタックをアンワインドし、各フレームに関連付けられた「ランディングパッド」（現在、デストラクタ）を実行し、制御を `catch_unwind` フレームに転送する責任があります。

すべてのパニックは、プロセスを中止するか、`catch_unwind` の呼び出しによってキャッチされることに注意してください。特に、stdの[ランタイムサービス][runtime service]では、ユーザー提供の `main` 関数への呼び出しは `catch_unwind` でラップされています。


[runtime service]: https://github.com/rust-lang/rust/blob/HEAD/library/std/src/rt.rs
