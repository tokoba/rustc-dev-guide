# MIRからコードジェネレーションIRへの低レベル化

コレクターから生成するシンボルのリストが得られたので、ある種のコードジェネレーションIRを生成する必要があります。この章では、rustcが通常使用するLLVM IRを想定します。実際のモノモーフィゼーションは、変換を行いながら実行されます。

バックエンドは [`rustc_codegen_ssa::base::codegen_crate`][codegen1] によって開始されることを思い出してください。最終的に、これは [`rustc_codegen_ssa::mir::codegen_mir`][codegen2] に到達し、MIRからLLVM IRへの低レベル化を行います。

[codegen1]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/base/fn.codegen_crate.html
[codegen2]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/mir/fn.codegen_mir.html

コードは、特定のMIRプリミティブを処理するモジュールに分割されています：

- [`rustc_codegen_ssa::mir::block`][mirblk] は、ブロックとそのターミネーターの変換を処理します。このモジュールが行う最も複雑で、また最も興味深いことは、必要なアンワインド処理IRを含む関数呼び出しのコードを生成することです。
- [`rustc_codegen_ssa::mir::statement`][mirst] はMIRステートメントを変換します。
- [`rustc_codegen_ssa::mir::operand`][mirop] はMIRオペランドを変換します。
- [`rustc_codegen_ssa::mir::place`][mirpl] はMIRプレース参照を変換します。
- [`rustc_codegen_ssa::mir::rvalue`][mirrv] はMIR右辺値を変換します。

[mirblk]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/mir/block/index.html
[mirst]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/mir/statement/index.html
[mirop]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/mir/operand/index.html
[mirpl]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/mir/place/index.html
[mirrv]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/mir/rvalue/index.html

関数が変換される前に、より簡単で効率的なLLVM IRを生成するのに役立ついくつかの単純で基本的な分析パスが実行されます。そのような分析パスの例は、どの変数がSSAのようなものかを把握して、それらの変数のためにLLVMの `mem2reg` に依存するのではなく、直接SSAに変換できるようにすることです。この分析は [`rustc_codegen_ssa::mir::analyze`][mirana] にあります。

[mirana]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/mir/analyze/index.html

通常、単一のMIR基本ブロックはLLVM基本ブロックにマップされますが、非常に少数の例外があります：組み込み関数または関数呼び出しや、`assert` のようなあまり基本的でないMIRステートメントは、複数の基本ブロックになる可能性があります。これは、コード生成のポータブルでないLLVM固有の部分への完璧な導入です。組み込み関数の生成は、その間に非常に少ない抽象化レベルが関与し、[`rustc_codegen_llvm::intrinsic`][llvmint] で見つけることができるため、かなり理解しやすいです。

[llvmint]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_llvm/intrinsic/index.html

他のすべては[ビルダーインターフェース][builder]を使用します。これは、上記で説明した [`rustc_codegen_ssa::mir::*`][ssamir] モジュールで呼び出されるコードです。

[builder]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_llvm/builder/index.html
[ssamir]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/mir/index.html

> TODO: 定数がどのように生成されるかを議論する
