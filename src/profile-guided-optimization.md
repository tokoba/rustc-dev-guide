# プロファイルガイド最適化

`rustc` はプロファイルガイド最適化（PGO）をサポートしています。この章では、PGOとは何か、および `rustc` におけるそのサポートがどのように実装されているかについて説明します。

## プロファイルガイド最適化とは？

PGOの基本的な概念は、プログラムの典型的な実行に関するデータ（例えば、どのブランチを取る可能性が高いか）を収集し、そのデータを使用してインライン化、マシンコードレイアウト、レジスタ割り当てなどの最適化に情報を提供することです。

プログラムの実行に関するデータを収集する方法はいくつかあります。1つは、プログラムをプロファイラ（`perf` など）内で実行することで、もう1つは、データ収集が組み込まれた計装されたバイナリを作成し、それを実行することです。後者は通常、より正確なデータを提供します。

## PGOは `rustc` でどのように実装されていますか？

`rustc` の現在のPGO実装は完全にLLVMに依存しています。LLVMは実際に[複数の形式][clang-pgo]のPGOをサポートしています：

[clang-pgo]: https://clang.llvm.org/docs/UsersManual.html#profile-guided-optimization

- サンプリングベースのPGO。ここでは、`perf` のような外部プロファイリングツールを使用してプログラムの実行に関するデータを収集します。
- GCOVベースのプロファイリング。ここでは、コードカバレッジインフラストラクチャを使用してプロファイリング情報を収集します。
- フロントエンドベースの計装。ここでは、コンパイラフロントエンド（例：Clang）が生成するLLVM IRに計装イントリンシックを挿入します（ただし、[^note-instrument-coverage]「注」を参照してください）。
- IRレベルの計装。ここでは、LLVMが最適化パス中に計装イントリンシック自体を挿入します。

`rustc` は最後のアプローチ、IRレベル計装のみをサポートしています。主に、ほぼ完全にLLVMで実装されており、Rust側でのメンテナンスがほとんど必要ないためです。幸いなことに、これは最も現代的なアプローチでもあり、最良の結果をもたらします。

つまり、計装ベースのアプローチを扱っており、プロファイリングデータは最適化されるプログラムの特別に計装されたバージョンによって生成されます。計装ベースのPGOには、コンパイル時のコンポーネントと実行時のコンポーネントがあり、全体的なワークフローを理解して、それらがどのように相互作用するかを確認する必要があります。

[^note-instrument-coverage]: 注：`rustc` は現在、実験的オプション
[`-C instrument-coverage`](./llvm-coverage-instrumentation.md)を介してフロントエンドベースのカバレッジ計装をサポートしていますが、
これらのカバレッジ結果をPGOに使用することは現時点では試みられていません。

### 全体的なワークフロー

PGO最適化されたプログラムを生成するには、次の4つのステップが必要です：

1. 計装を有効にしてプログラムをコンパイルします（例：`rustc -C profile-generate main.rs`）
2. 計装されたプログラムを実行します（例：`./main`）。これにより `default-<id>.profraw` ファイルが生成されます
3. LLVMの `llvm-profdata` ツールを使用して `.profraw` ファイルを `.profdata` ファイルに変換します
4. プロファイリングデータを利用してプログラムを再度コンパイルします
   （例：`rustc -C profile-use=merged.profdata main.rs`）

### コンパイル時の側面

上記のワークフローのどのステップにいるかに応じて、コンパイル時に2つの異なることが起こります：

#### 計装を有効にしたバイナリの作成

上述のように、プロファイリング計装はLLVMによって追加されます。`rustc` は、LLVM `PassManager` を作成するときに[適切なフラグを設定する][pgo-gen-passmanager]ことでLLVMに指示します：

```C
 // `PMBR` is an `LLVMPassManagerBuilderRef`
    unwrap(PMBR)->EnablePGOInstrGen = true;
    // Instrumented binaries have a default output path for the `.profraw` file
    // hard-coded into them:
    unwrap(PMBR)->PGOInstrGen = PGOGenPath;
```

`rustc` はまた、LLVMのプロファイリングランタイムの一部のシンボルが削除されないようにする必要があります。これは[適切なエクスポートレベルでマークする][pgo-gen-symbols]ことで行います。

[pgo-gen-passmanager]: https://github.com/rust-lang/rust/blob/1.34.1/src/rustllvm/PassWrapper.cpp#L412-L416
[pgo-gen-symbols]:https://github.com/rust-lang/rust/blob/1.34.1/src/librustc_codegen_ssa/back/symbol_export.rs#L212-L225

#### 最適化がプロファイリングデータを利用するバイナリのコンパイル

上で説明したワークフローの最後のステップでは、プログラムが再度コンパイルされます。今回はコンパイラが収集されたプロファイリングデータを使用して最適化の決定を推進します。`rustc` はここでも作業のほとんどをLLVMに任せます。基本的に、LLVM `PassManagerBuilder` にプロファイリングデータがどこにあるかを[伝えるだけ][pgo-use-passmanager]です：

```C
 unwrap(PMBR)->PGOInstrUse = PGOUsePath;
```

[pgo-use-passmanager]: https://github.com/rust-lang/rust/blob/1.34.1/src/rustllvm/PassWrapper.cpp#L417-L420

LLVMが残りを行います（例：ブランチウェイトの設定、関数への `cold` または `inlinehint` のマークなど）。

### 実行時の側面

計装ベースのアプローチには常に実行時コンポーネントもあります。つまり、計装されたプログラムを取得したら、そのプログラムを実行してプロファイリングデータを生成する必要があり、このプロファイリングデータを収集して永続化するためのインフラストラクチャが必要です。

LLVMの場合、これらの実行時コンポーネントは[compiler-rt][compiler-rt-profile]に実装されており、計装されたすべてのバイナリに静的にリンクされます。`rustc` バージョンは `library/profiler_builtins` にあり、基本的に `compiler-rt` のCコードをRustクレートにパッケージ化したものです。

`profiler_builtins` がビルドされるためには、`rustc` の `bootstrap.toml` で `profiler = true` を設定する必要があります。

[compiler-rt-profile]: https://github.com/llvm/llvm-project/tree/main/compiler-rt/lib/profile

## PGOのテスト

PGOワークフローは複数のコンパイラ呼び出しにまたがるため、ほとんどのテストは[run-makeテスト][rmake-tests]で行われます（関連するテストには名前に `pgo` が含まれています）。また、予想される計装アーティファクトがLLVM IRに表示されることをチェックする[codegenテスト][codegen-test]もあります。

[rmake-tests]: https://github.com/rust-lang/rust/tree/HEAD/tests/run-make
[codegen-test]: https://github.com/rust-lang/rust/blob/HEAD/tests/codegen-llvm/pgo-instrumentation.rs

## 追加情報

Clangのドキュメントには、[LLVMでのPGO][llvm-pgo]に関する良い概要が含まれています。

[llvm-pgo]: https://clang.llvm.org/docs/UsersManual.html#profile-guided-optimization
