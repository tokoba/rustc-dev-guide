# サニタイザーのサポート

rustcコンパイラには、以下のサニタイザーのサポートが含まれています：

* [AddressSanitizer][clang-asan] より高速なメモリエラー検出器。
  ヒープ、スタック、グローバルへの範囲外アクセス、解放後の使用、return後の使用、二重解放、無効な解放、メモリリークを検出できます。
* [ControlFlowIntegrity][clang-cfi] LLVM制御フロー整合性（CFI）は、前方エッジの制御フロー保護を提供します。
* [Hardware-assisted AddressSanitizer][clang-hwasan] AddressSanitizerに似たツールですが、部分的なハードウェアアシストに基づいています。
* [KernelControlFlowIntegrity][clang-kcfi] LLVMカーネル制御フロー整合性（KCFI）は、オペレーティングシステムのカーネルに前方エッジの制御フロー保護を提供します。
* [LeakSanitizer][clang-lsan] ランタイムメモリリーク検出器。
* [MemorySanitizer][clang-msan] 未初期化読み取りの検出器。
* [ThreadSanitizer][clang-tsan] 高速なデータ競合検出器。

## サニタイザーの使い方

サニタイザーを有効にするには、`-Z sanitizer=...`オプションを指定してコンパイルします。値は`address`、`cfi`、`hwaddress`、`kcfi`、`leak`、`memory`、`thread`のいずれかです。
サニタイザーの使用方法の詳細については、[The Unstable Book]のサニタイザーフラグを参照してください。

[The Unstable Book]: https://doc.rust-lang.org/unstable-book

## rustcでのサニタイザーの実装方法

サニタイザー（CFIを除く）の実装は、ほぼ完全にLLVMに依存しています。rustcは、LLVMコンパイル時のインストルメンテーションパスとランタイムライブラリの統合ポイントです。実装の最も重要な側面のハイライト：

*  サニタイザーランタイムライブラリは[compiler-rt]プロジェクトの一部であり、`bootstrap.toml`で有効にすると[サポートされているターゲット][sanitizer-targets]で[ビルドされます][sanitizer-build]：

   ```toml
   [build]
   sanitizers = true
   ```

   ランタイムは[ターゲットのlibdirに配置されます][sanitizer-copy]。

*  LLVMコード生成中に、インストルメンテーション対象の関数は適切なLLVM属性で[マークされます][sanitizer-attribute]：`SanitizeAddress`、`SanitizeHWAddress`、`SanitizeMemory`、または`SanitizeThread`。デフォルトでは、すべての関数がインストルメント化されますが、この動作は`#[sanitize(xyz = "on|off|<other>")]`で変更できます。

*  インストルメンテーションを実行するかどうかの決定は、関数の粒度でのみ可能です。これらの決定が関数間で異なる場合、インライン化を禁止する必要がある場合があります。[MIRレベル][inline-mir]と[LLVMレベル][inline-llvm]の両方で。

*  rustcによって生成されたLLVM IRは、[専用のLLVMパス][sanitizer-pass]によってインストルメント化され、サニタイザーごとに異なります。インストルメンテーションパスは最適化パスの後に呼び出されます。

*  実行可能ファイルを生成する際、サニタイザー固有のランタイムライブラリが[リンクされます][sanitizer-link]。ライブラリはターゲットのlibdirで検索されます。最初に、オーバーライドされたシステムルートに相対的に検索され、次にデフォルトのシステムルートに相対的に検索されます。デフォルトのシステムルートへのフォールバックにより、cargo `-Z build-std`やxargoによって構築されたsysrootオーバーライドを使用する場合でも、サニタイザーランタイムが利用可能であることが保証されます。

[compiler-rt]: https://github.com/llvm/llvm-project/tree/main/compiler-rt
[sanitizer-build]: https://github.com/rust-lang/rust/blob/1ead4761e9e2f056385768614c23ffa7acb6a19e/src/bootstrap/src/core/build_steps/llvm.rs#L958-L1031
[sanitizer-targets]: https://github.com/rust-lang/rust/blob/1ead4761e9e2f056385768614c23ffa7acb6a19e/src/bootstrap/src/core/build_steps/llvm.rs#L1073-L1111
[sanitizer-copy]: https://github.com/rust-lang/rust/blob/1ead4761e9e2f056385768614c23ffa7acb6a19e/src/bootstrap/src/core/build_steps/compile.rs#L637-L676
[sanitizer-attribute]: https://github.com/rust-lang/rust/blob/1.55.0/compiler/rustc_codegen_llvm/src/attributes.rs#L42-L58
[inline-mir]: https://github.com/rust-lang/rust/blob/1.55.0/compiler/rustc_mir/src/transform/inline.rs#L314-L316
[inline-llvm]: https://github.com/rust-lang/llvm-project/blob/9330ec5a4c1df5fc1fa62f993ed6a04da68cb040/llvm/include/llvm/IR/Attributes.td#L225-L241
[sanitizer-pass]: https://github.com/rust-lang/rust/blob/1.55.0/compiler/rustc_codegen_llvm/src/back/write.rs#L660-L678
[sanitizer-link]: https://github.com/rust-lang/rust/blob/1.55.0/compiler/rustc_codegen_ssa/src/back/link.rs#L1053-L1089

## サニタイザーのテスト

サニタイザーは、[`tests/codegen-llvm/sanitize*.rs`][test-cg]のコード生成テストと、[`tests/ui/sanitizer/`][test-ui]ディレクトリのエンドツーエンド機能テストで検証されています。

サニタイザー機能のテストには、サニタイザーランタイム（`bootstrap.toml`で`sanitizer = true`の場合にビルドされる）と、特定のサニタイザーをサポートするターゲットが必要です。特定のターゲットでサニタイザーがサポートされていない場合、サニタイザーテストは無視されます。この動作は、compiletestの`needs-sanitizer-*`ディレクティブによって制御されています。

[test-cg]: https://github.com/rust-lang/rust/tree/HEAD/tests/codegen-llvm
[test-ui]: https://github.com/rust-lang/rust/tree/HEAD/tests/ui/sanitizer

## 新しいターゲットでサニタイザーを有効にする

LLVMによってすでにサポートされている新しいターゲットでサニタイザーを有効にするには：

1. [ターゲット定義][target-definition]の`supported_sanitizers`のリストにサニタイザーを含めます。これで、`rustc --target .. -Zsanitizer=..`はサニタイザーがサポートされていることを認識します。
2. [ターゲット用のランタイムをビルドし、libdirに含めます。][sanitizer-targets]
3. [あなたのターゲットがサニタイザーをサポートしていることをcompiletestに教えます。][compiletest-definition]`needs-sanitizer-*`でマークされたテストがターゲットで実行されるようになります。
4. テストを実行して検証します：`./x test --force-rerun tests/ui/sanitize/`
5. リリースプロセスの一部としてサニタイザーランタイムをビルドおよび配布するために、[CIコンフィグで--enable-sanitizersを設定します][ci-configuration]。

[target-definition]: https://github.com/rust-lang/rust/blob/1.55.0/compiler/rustc_target/src/spec/x86_64_unknown_linux_gnu.rs#L10-L11
[compiletest-definition]: https://github.com/rust-lang/rust/blob/1.55.0/src/tools/compiletest/src/util.rs#L87-L116
[ci-configuration]: https://github.com/rust-lang/rust/blob/1.55.0/src/ci/docker/host-x86_64/dist-x86_64-linux/Dockerfile#L94

## 追加情報

* [サニタイザープロジェクトページ](https://github.com/google/sanitizers/wiki/)
* [ClangのAddressSanitizer][clang-asan]
* [ClangのControlFlowIntegrity][clang-cfi]
* [Hardware-assisted AddressSanitizer][clang-hwasan]
* [ClangのKernelControlFlowIntegrity][clang-kcfi]
* [ClangのLeakSanitizer][clang-lsan]
* [ClangのMemorySanitizer][clang-msan]
* [ClangのThreadSanitizer][clang-tsan]

[clang-asan]: https://clang.llvm.org/docs/AddressSanitizer.html
[clang-cfi]: https://clang.llvm.org/docs/ControlFlowIntegrity.html
[clang-hwasan]: https://clang.llvm.org/docs/HardwareAssistedAddressSanitizerDesign.html
[clang-kcfi]: https://clang.llvm.org/docs/ControlFlowIntegrity.html#fsanitize-kcfi
[clang-lsan]: https://clang.llvm.org/docs/LeakSanitizer.html
[clang-msan]: https://clang.llvm.org/docs/MemorySanitizer.html
[clang-tsan]: https://clang.llvm.org/docs/ThreadSanitizer.html
