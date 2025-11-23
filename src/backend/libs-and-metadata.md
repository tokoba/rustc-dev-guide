# ライブラリとメタデータ

コンパイラが外部クレートへの参照を見つけると、そのクレートに関する情報をロードする必要があります。この章では、そのプロセスの概要と、クレートライブラリでサポートされているファイル形式について説明します。

## ライブラリ

クレートの依存関係は、`rlib`、`dylib`、または `rmeta` ファイルからロードできます。これらのファイル形式の重要な点は、`rustc` 固有の[*メタデータ*](#metadata)を含んでいることです。このメタデータにより、コンパイラは外部クレートについて十分な情報を発見し、含まれるアイテム、エクスポートするマクロ、*その他多く*のことを理解できます。

### rlib

`rlib` は[アーカイブファイル]であり、tarファイルに似ています。このファイル形式は `rustc` 固有であり、時間の経過とともに変わる可能性があります。このファイルには以下が含まれます：

* オブジェクトコード。これはコード生成の結果です。これは通常のリンク時に使用されます。各[コードジェネレーション単位][codegen unit]ごとに個別の `.o` ファイルがあります。コードジェネレーションステップは [`-C linker-plugin-lto`][linker-plugin-lto] CLIオプションでスキップでき、各 `.o` ファイルにはLLVMビットコードのみが含まれることになります。
* [LLVMビットコード]。これはLLVMの中間表現のバイナリ表現で、`.o` ファイルにセクションとして埋め込まれています。これは[リンク時最適化][Link Time Optimization]（LTO）に使用できます。LTOが不要な場合は、[`-C embed-bitcode=no`][embed-bitcode] CLIオプションを使用してこれを削除し、コンパイル時間を改善しディスクスペースを節約できます。
* `rustc` [メタデータ]。`lib.rmeta` という名前のファイルにあります。
* シンボルテーブル。これは基本的に、そのシンボルを含むオブジェクトファイルへのオフセットを持つシンボルのリストです。これはアーカイブファイルではかなり標準的です。

[アーカイブファイル]: https://en.wikipedia.org/wiki/Ar_(Unix)
[LLVMビットコード]: https://llvm.org/docs/BitCodeFormat.html
[Link Time Optimization]: https://llvm.org/docs/LinkTimeOptimization.html
[codegen unit]: ../backend/codegen.md
[embed-bitcode]: https://doc.rust-lang.org/rustc/codegen-options/index.html#embed-bitcode
[linker-plugin-lto]: https://doc.rust-lang.org/rustc/codegen-options/index.html#linker-plugin-lto

### dylib

`dylib` はプラットフォーム固有の共有ライブラリです。これには、`.rustc` と呼ばれる特殊なリンクセクションに `rustc` [メタデータ]が含まれています。

### rmeta

`rmeta` ファイルは、クレートの[メタデータ]を含むカスタムバイナリ形式です。このファイルは、すべてのコード生成をスキップしてプロジェクトの高速な「チェック」（`cargo check` で行われるように）、ドキュメント用の十分な情報を収集すること（`cargo doc` で行われるように）、または[パイプライン化](#pipelining)に使用できます。このファイルは、[`--emit=metadata`][emit] CLIオプションを使用すると作成されます。

`rmeta` ファイルは、コンパイルされたオブジェクトファイルを含まないため、リンクをサポートしていません。

[emit]: https://doc.rust-lang.org/rustc/command-line-arguments.html#option-emit

## メタデータ

メタデータには、さまざまな要素が幅広く含まれています。このガイドでは、含まれるすべてのフィールドの詳細には触れません。[`CrateRoot`] 定義を参照して、含まれるさまざまな要素の感覚をつかむことをお勧めします。メタデータのエンコードとデコードに関するすべては、[`rustc_metadata`] パッケージにあります。

以下は、含まれるいくつかのハイライトです：

* `rustc` コンパイラのバージョン。コンパイラは他のバージョンからのファイルのロードを拒否します。
* [厳密バージョンハッシュ](#strict-version-hash)（SVH）。これは、正しい依存関係がロードされることを保証するのに役立ちます。
* [安定クレートID](#stable-crate-id)。これは、クレートを識別するために使用されるハッシュです。
* ライブラリ内のすべてのソースファイルに関する情報。これは、依存関係のソースを指す診断など、さまざまなことに使用できます。
* エクスポートされたマクロ、トレイト、型、アイテムに関する情報。一般的に、パスがクレート依存関係内の何かを参照するときに知る必要があるものすべてです。
* エンコードされた[MIR]。これはオプションで、コード生成に必要な場合にのみエンコードされます。`cargo check` はパフォーマンス上の理由でこれをスキップします。

[`CrateRoot`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/rmeta/struct.CrateRoot.html
[`rustc_metadata`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/index.html
[MIR]: ../mir/index.md

### 厳密バージョンハッシュ

厳密バージョンハッシュ（[SVH]、「クレートハッシュ」としても知られる）は、正しいクレート依存関係がロードされることを保証するために使用される64ビットハッシュです。ディレクトリに、異なる設定でビルドされた、または異なるソースからビルドされた同じ依存関係の複数のコピーが含まれている可能性があります。クレートローダーは、間違ったSVHを持つクレートをスキップします。

SVHは[インクリメンタルコンパイル][incremental compilation]セッションのファイル名にも使用されますが、その使用法はほとんど歴史的なものです。

ハッシュには以下のさまざまな要素が含まれます：

* HIRノードのハッシュ。
* すべての上流クレートハッシュ。
* すべてのソースファイル名。
* 特定のコマンドラインフラグのハッシュ（[安定クレートID](#stable-crate-id)を介した `-C metadata` や、`[TRACKED]` でマークされたすべてのCLIオプション）。

ハッシュが実際に計算される場所については、[`compute_hir_hash`] を参照してください。

[SVH]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_data_structures/svh/struct.Svh.html
[incremental compilation]: ../queries/incremental-compilation.md
[`compute_hir_hash`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast_lowering/fn.compute_hir_hash.html

### 安定クレートID

[`StableCrateId`] は、同じ名前の可能性のある異なるクレートを識別するために使用される64ビットハッシュです。これは、クレート名とすべての[`-C metadata`] CLIオプションのハッシュで、[`StableCrateId::new`] で計算されます。これは、シンボル名のマングリング、クレートのロード、その他多くの場所など、さまざまな場所で使用されます。

デフォルトでは、すべてのRustシンボルはマングリングされ、安定クレートIDを組み込みます。これにより、同じクレートの複数のバージョンを一緒に含めることができます。Cargoは、パッケージバージョン、ソース、ターゲットの種類などのさまざまな要因に基づいて、自動的に `-C metadata` ハッシュを生成します（libとtestは同じクレート名を持つことができるため、曖昧さを解消する必要があります）。

[`StableCrateId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/def_id/struct.StableCrateId.html
[`StableCrateId::new`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/def_id/struct.StableCrateId.html#method.new
[`-C metadata`]: https://doc.rust-lang.org/rustc/codegen-options/index.html#metadata

## クレートのロード

クレートのロードには、かなり多くの微妙な複雑さがあります。[名前解決][name resolution]中に、外部クレートが参照されると（`extern crate` またはパスを介して）、リゾルバは [`CrateLoader`] を使用します。これは、クレートライブラリを見つけてそれらの[メタデータ]をロードする責任があります。依存関係がロードされた後、`CrateLoader` はリゾルバがその仕事を実行するために必要な情報を提供します（マクロの展開、パスの解決など）。

各外部クレートをロードするために、`CrateLoader` は [`CrateLocator`] を使用して、1つの特定のクレートの正しいファイルを実際に見つけます。[`locator`] モジュールには、ロードがどのように機能するかについて詳細に説明する素晴らしいドキュメントがあります。全体像を把握するために、それを読むことを強くお勧めします。

依存関係の場所は、いくつかの異なる場所から来る可能性があります。直接依存関係は通常 `--extern` フラグで渡され、ローダーはそれらを直接見ることができます。直接依存関係には、独自の依存関係への参照があることが多く、それらもロードする必要があります。これらは通常、`-L` フラグで渡されたディレクトリをスキャンして、メタデータに一致するクレート名と[SVH](#strict-version-hash)を含む任意のファイルを見つけることで見つかります。ローダーは、依存関係を見つけるために[sysroot]も調べます。

クレートがロードされると、それらは[`CStore`]に保持され、クレートメタデータは[`CrateMetadata`]構造体でラップされます。解決と展開の後、`CStore` は残りのコンパイルのために [`GlobalCtxt`] に入ります。

[name resolution]: ../name-resolution.md
[`CrateLoader`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/creader/struct.CrateLoader.html
[`CrateLocator`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/locator/struct.CrateLocator.html
[`locator`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/locator/index.html
[`CStore`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/creader/struct.CStore.html
[`CrateMetadata`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/rmeta/decoder/struct.CrateMetadata.html
[`GlobalCtxt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.GlobalCtxt.html
[sysroot]: ../building/bootstrapping/what-bootstrapping-does.md#what-is-a-sysroot

## パイプライン化

コンパイル時間を改善するトリックの1つは、依存関係のメタデータが利用可能になり次第、クレートのビルドを開始することです。ライブラリの場合、依存関係のコード生成が終了するのを待つ必要はありません。Cargoは、各依存関係について[`rmeta`](#rmeta)ファイルと[`rlib`](#rlib)を出力するように `rustc` に指示することで、この手法を実装しています。できるだけ早く、`rustc` はコード生成フェーズに進む前に `rmeta` ファイルをディスクに保存します。コンパイラは、可能であれば次のクレートのビルドを開始できることをビルドツールに知らせるためにJSONメッセージを送信します。

[クレートロード](#crate-loading)システムは、`rlib` が存在しない場合（または部分的にしか書き込まれていない場合）に `rmeta` ファイルを見たときにそれを使用することを知るのに十分賢いです。

このパイプライン化はバイナリには不可能です。リンクフェーズはすべての依存関係のコード生成を必要とするためです。将来的には、リンクを別のコマンドに分割することで、このシナリオをさらに改善できる可能性があります（[#64191]を参照）。

[#64191]: https://github.com/rust-lang/rust/issues/64191

[metadata]: #metadata
