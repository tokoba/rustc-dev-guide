# コンパイラのプロファイリング

このセクションでは、コンパイラのプロファイリング方法と、どこで時間を費やしているかを見つける方法について説明します。

測定したい内容に応じて、いくつかの異なるアプローチがあります:

- PRがコンパイラのパフォーマンスを改善または低下させるかどうかを確認したい場合は、
  ベンチマーク実行をリクエストするための[rustc-perfの章](tests/perf.md)を参照してください。

- `rustc`が時間を費やしている場所の中〜高レベルの概要が必要な場合:
  - `-Z self-profile`フラグと[measureme](https://github.com/rust-lang/measureme)ツールは、クエリベースのプロファイリング手法を提供します。
    詳細については[ドキュメント](https://github.com/rust-lang/measureme/blob/master/summarize/README.md)を参照してください。

- 関数レベルのパフォーマンスデータ、または上記のアプローチよりも詳細な情報が必要な場合:
  - [perf](profiling/with_perf.md)などのネイティブコードプロファイラの使用を検討してください
  - または、ナノ秒精度の完全機能を備えたグラフィカルインターフェースを持つ[tracy](https://github.com/nagisa/rust_tracy_client)を使用してください。

- クレートグラフのコンパイル時間の視覚的な表現が必要な場合は、
  [cargoの`--timings`フラグ](https://doc.rust-lang.org/nightly/cargo/reference/timings.html)を使用できます。
  例: `cargo build --timings`
  コンパイラ自体でこのフラグを使用するには、`CARGOFLAGS="--timings" ./x build`を使用してください。

- メモリ使用量をプロファイリングしたい場合は、使用しているオペレーティングシステムに応じて
  様々なツールを使用できます。
  - Windowsの場合は、[WPAガイド](profiling/wpa_profiling.md)をお読みください。

## `cargo-llvm-lines`を使ったrustcのブートストラップ時間の最適化

[cargo-llvm-lines](https://github.com/dtolnay/cargo-llvm-lines)を使用すると、
ジェネリック関数のすべてのインスタンス化にわたるLLVM IRの行数をカウントできます。
rustcのコンパイルにおける時間の大半はLLVMで費やされるため、
LLVMに渡されるコードの量を減らすことで、rustcのコンパイルが速くなるという考えです。

やや特殊なrustcのビルドプロセスで`cargo-llvm-lines`を使用するには、
必要なLLVM IRを取得するために`-C save-temps`を使用できます。このオプションは、
コンパイル中に作成される一時的な作業生成物を保持します。その中には、
最適化パイプラインへの入力を表すLLVM IRが含まれており、これは私たちの目的に理想的です。
これは、LLVMビットコード形式で`*.no-opt.bc`拡張子を持つファイルに保存されます。

使用例:
```
cargo install cargo-llvm-lines
# 通常のクレートでは`cargo llvm-lines`を実行できますが、`x`は通常ではありません :P

# 毎回実行前にクリーンを行い、以前の実行結果が混ざらないようにします。
./x clean
env RUSTFLAGS=-Csave-temps ./x build --stage 0 compiler/rustc

# 単一のクレート、例えばrustc_middle。(シェルのグロブサポートに依存します。)
# 最適化されていないLLVMビットコードを、cargo-llvm-linesが受け入れる人間が読めるLLVMアセンブリに変換します。
for f in build/x86_64-unknown-linux-gnu/stage0-rustc/x86_64-unknown-linux-gnu/release/deps/rustc_middle-*.no-opt.bc; do
  ./build/x86_64-unknown-linux-gnu/llvm/bin/llvm-dis "$f"
done
cargo llvm-lines --files ./build/x86_64-unknown-linux-gnu/stage0-rustc/x86_64-unknown-linux-gnu/release/deps/rustc_middle-*.ll > llvm-lines-middle.txt

# コンパイラのすべてのクレートを指定します。
for f in build/x86_64-unknown-linux-gnu/stage0-rustc/x86_64-unknown-linux-gnu/release/deps/*.no-opt.bc; do
  ./build/x86_64-unknown-linux-gnu/llvm/bin/llvm-dis "$f"
done
cargo llvm-lines --files ./build/x86_64-unknown-linux-gnu/stage0-rustc/x86_64-unknown-linux-gnu/release/deps/*.ll > llvm-lines.txt
```

コンパイラの出力例:
```
  Lines            Copies          Function name
  -----            ------          -------------
  45207720 (100%)  1583774 (100%)  (TOTAL)
   2102350 (4.7%)   146650 (9.3%)  core::ptr::drop_in_place
    615080 (1.4%)     8392 (0.5%)  std::thread::local::LocalKey<T>::try_with
    594296 (1.3%)     1780 (0.1%)  hashbrown::raw::RawTable<T>::rehash_in_place
    592071 (1.3%)     9691 (0.6%)  core::option::Option<T>::map
    528172 (1.2%)     5741 (0.4%)  core::alloc::layout::Layout::array
    466854 (1.0%)     8863 (0.6%)  core::ptr::swap_nonoverlapping_one
    412736 (0.9%)     1780 (0.1%)  hashbrown::raw::RawTable<T>::resize
    367776 (0.8%)     2554 (0.2%)  alloc::raw_vec::RawVec<T,A>::grow_amortized
    367507 (0.8%)      643 (0.0%)  rustc_query_system::dep_graph::graph::DepGraph<K>::with_task_impl
    355882 (0.8%)     6332 (0.4%)  alloc::alloc::box_free
    354556 (0.8%)    14213 (0.9%)  core::ptr::write
    354361 (0.8%)     3590 (0.2%)  core::iter::traits::iterator::Iterator::fold
    347761 (0.8%)     3873 (0.2%)  rustc_middle::ty::context::tls::set_tlv
    337534 (0.7%)     2377 (0.2%)  alloc::raw_vec::RawVec<T,A>::allocate_in
    331690 (0.7%)     3192 (0.2%)  hashbrown::raw::RawTable<T>::find
    328756 (0.7%)     3978 (0.3%)  rustc_middle::ty::context::tls::with_context_opt
    326903 (0.7%)      642 (0.0%)  rustc_query_system::query::plumbing::try_execute_query
```

これはインクリメンタルコンパイルや`./x check`では機能しないようなので、
rustcを何度もコンパイルすることになります。
耐えられるように、`bootstrap.toml`でいくつかの設定を変更することをお勧めします:
```
[rust]
# デバッグビルドは私のマシンでは3分の1の時間で済みますが、
# stage0以上のrustcのコンパイルは耐え難いほど遅くなります。
optimize = false

# いずれにしてもインクリメンタルは使用できないので、少し速度を上げるために無効にします。
incremental = false
# 実行しないので、デバッグチェックをコンパイルする意味はありません。
debug = false

# 単一のコード生成ユニットを使用すると出力は少なくなりますが、コンパイルは遅くなります。
codegen-units = 0  # num_cpus
```

llvm-linesの出力はいくつかのオプションの影響を受けます。
`optimize = false`で2.1GBから3.5GBに増加し、`codegen-units = 0`で4.1GBになります。

MIR最適化の影響はほとんどありません。デフォルトの`RUSTFLAGS="-Z
mir-opt-level=1"`と比較して、レベル0では0.3GB追加され、レベル2では0.2GB削減されます。
<!-- date-check --> 2022年7月時点では、
インライン化はLLVMおよびGCCコード生成バックエンドで行われ、
Craneliftのみで欠落しています。
