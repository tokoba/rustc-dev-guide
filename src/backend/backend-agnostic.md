# バックエンド非依存のコードジェネレーション

[`rustc_codegen_ssa`]
は、すべてのバックエンド（LLVM、[Cranelift]、[GCC]）が実装すべき抽象的なインターフェースを提供します。

[Cranelift]: https://github.com/rust-lang/rustc_codegen_cranelift
[GCC]: https://github.com/rust-lang/rustc_codegen_gcc
[`rustc_codegen_ssa`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/index.html

以下は、この抽象的なインターフェースを作成したリファクタリングに関する背景情報です。

## `rustc_codegen_llvm` のリファクタリング

Denis Merigoux 著、2018年10月23日

### リファクタリング前のコードの状態

MIRからLLVM IRへのコンパイルに関連するすべてのコードは、`rustc_codegen_llvm` クレート内に含まれていました。最も重要な要素の内訳は以下の通りです：

* `back` フォルダ（7,800行）は、LLVMを通じて異なるオブジェクトファイルとアーカイブを作成するメカニズムを実装していますが、並列コード生成のための通信メカニズムも実装しています。
* `debuginfo` フォルダ（3,200行）には、デバッグ情報をLLVMに渡すためのすべてのコードが含まれています。
* `llvm` フォルダ（2,200行）は、C++ APIを使用してLLVMと通信するために必要なFFIを定義しています。
* `mir` フォルダ（4,300行）は、MIRからLLVM IRへの実際の低レベル化を実装しています。
* `base.rs` ファイル（1,300行）には、いくつかのヘルパー関数が含まれていますが、コード生成を起動して作業を分散する高レベルのコードも含まれています。
* `builder.rs` ファイル（1,200行）には、基本ブロック内で個々のLLVM IR命令を生成するすべての関数が含まれています。
* `common.rs`（450行）には、さまざまなヘルパー関数と、LLVM静的値を生成するすべての関数が含まれています。
* `type_.rs`（300行）は、LLVM IRへのほとんどの型変換を定義しています。

このリファクタリングの目標は、このクレート内でLLVMに固有のコードと、他のrustcバックエンドで再利用できるコードを分離することです。例えば、`mir` フォルダはほぼ完全にバックエンド固有ですが、クレートの他の部分に大きく依存しています。コードの分離は、コードのロジックやパフォーマンスに影響を与えてはいけません。

これらの理由から、分離プロセスには、結果のコードがコンパイルされるために同時に行う必要がある2つの変換が含まれます：

1. 関数シグネチャと構造体定義内のすべてのLLVM固有の型をジェネリックに置き換える。
2. LLVM FFIを呼び出すすべての関数を、バックエンド非依存のコードとバックエンドの間のインターフェースを定義する一連のトレイト内にカプセル化する。

LLVM固有のコードは `rustc_codegen_llvm` に残されますが、すべての新しいトレイトとバックエンド非依存のコードは `rustc_codegen_ssa` に移動されます（@eddybによる名前の提案）。

### ジェネリック型と構造体

@irinagpopaは、`rustc_codegen_llvm` の型をジェネリック `Value` 型でパラメータ化し始めました。これはLLVMでは参照 `&'ll Value` として実装されています。この作業は、`mir` フォルダ内およびその他の場所のすべての構造体、ならびにLLVMの `BasicBlock` と `Type` 型に拡張されました。

LLVMコードジェネレーションの2つの最も重要な構造体は、`CodegenCx` と `Builder` です。これらは、複数のライフタイムパラメータと `Value` の型によってパラメータ化されています。

```rust,ignore
struct CodegenCx<'ll, 'tcx> {
  /* ... */
}

struct Builder<'a, 'll, 'tcx> {
  cx: &'a CodegenCx<'ll, 'tcx>,
  /* ... */
}
```

`CodegenCx` は、複数の関数を含む1つのコードジェネレーション単位をコンパイルするために使用されますが、`Builder` は1つの基本ブロックをコンパイルするために作成されます。

`rustc_codegen_llvm` のコードは、以下に対応する複数の明示的なライフタイムパラメータを処理する必要があります：

* `'tcx` は最も長いライフタイムで、プログラムの情報を含む元の `TyCtxt` に対応します。
* `'a` は、構造体内の `CodegenCx` または別のオブジェクトの短命な参照です。
* `'ll` は、`Value` や `Type` などのLLVMオブジェクトへの参照のライフタイムです。

コードにはすでに多くのライフタイムパラメータがありますが、ジェネリックにすることで、操作されるLLVMオブジェクトの特殊な性質（それらは外部ポインタです）のためだけに借用チェッカーが通過していた状況が明らかになりました。例えば、`analyse.rs` の `LocalAnalyser` に追加のライフタイムパラメータを追加する必要があり、次の定義になりました：

```rust,ignore
struct LocalAnalyzer<'mir, 'a, 'tcx> {
  /* ... */
}
```

しかし、最も重要な2つの構造体 `CodegenCx` と `Builder` は、バックエンド非依存のコードでは定義されていません。実際、それらの内容はバックエンドに非常に固有であり、バックエンドのコンテキスト用のジェネリックフィールドを介して狭いスポットだけを許可するよりも、その定義をバックエンド実装者に任せる方が理にかなっています。

### トレイトとインターフェース

バックエンドによって定義される必要があるため、`CodegenCx` と `Builder` は、バックエンドのインターフェースを定義するすべてのトレイトを実装する構造体になります。これらのトレイトは `rustc_codegen_ssa/traits` フォルダで定義され、すべてのバックエンド非依存のコードはそれらによってパラメータ化されます。例えば、`base.rs` の関数がどのようにパラメータ化されているかを説明しましょう：

```rust,ignore
pub fn codegen_instance<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>>(
    cx: &'a Bx::CodegenCx,
    instance: Instance<'tcx>
) {
    /* ... */
}
```

このシグネチャでは、前述の2つのライフタイムパラメータと、`Builder` 構造体が満たすインターフェースに対応する `BuilderMethods` トレイトを満たすマスター型 `Bx` があります。`BuilderMethods` は、構造体 `CodegenCx` によって実装される `CodegenMethods` トレイトを満たす関連型 `Bx::CodegenCx` を定義します。

トレイト側では、`traits/builder.rs` の `BuilderMethods` の定義の一部を示す例を示します：

```rust,ignore
pub trait BuilderMethods<'a, 'tcx>:
    HasCodegen<'tcx>
    + DebugInfoBuilderMethods<'tcx>
    + ArgTypeMethods<'tcx>
    + AbiBuilderMethods<'tcx>
    + IntrinsicCallMethods<'tcx>
    + AsmBuilderMethods<'tcx>
{
    fn new_block<'b>(
        cx: &'a Self::CodegenCx,
        llfn: Self::Function,
        name: &'b str
    ) -> Self;
    /* ... */
    fn cond_br(
        &mut self,
        cond: Self::Value,
        then_llbb: Self::BasicBlock,
        else_llbb: Self::BasicBlock,
    );
    /* ... */
}
```

最後に、`ExtraBackendMethods` トレイトを実装するマスター構造体は、`base.rs` の `codegen_crate` のような高レベルのコードジェネレーション駆動関数に使用されます。LLVMの場合、それは空の `LlvmCodegenBackend` です。`ExtraBackendMethods` は、`rustc_codegen_utils/codegen_backend.rs` で定義されている `CodegenBackend` を実装する同じ構造体によって実装される必要があります。

トレイト化プロセス中に、特定の関数がローカル構造体のメソッドから `CodegenCx` または `Builder` のメソッドに変換され、対応する `self` パラメータが追加されました。実際、LLVMは内部的に情報を保存しており、そのAPIを通じて呼び出されたときにアクセスできます。この情報は、これらのメソッドが呼び出されたときに持ち運ばれるRustのデータ構造には現れません。しかし、`rustc` のRustバックエンドを実装するとき、これらのメソッドには `CodegenCx` からの情報が必要になるため、追加のパラメータ（トレイトのLLVM実装では未使用）が必要です。

### リファクタリング後のコードの状態

トレイトは、LLVMのAPIに非常に似たAPIを提供します。これは最良のソリューションではありません。LLVMには非常に特殊なやり方があるためです：別のバックエンドを追加する際、より柔軟性を提供するためにトレイトの定義が変更される可能性があります。

しかし、バックエンド非依存のコードとLLVM固有のコードの現在の分離により、古い `rustc_codegen_llvm` のかなりの部分を再利用できるようになりました。最も重要な要素について、バックエンド非依存（BA）とLLVMの間の新しいLOC内訳は次のとおりです：

* `back` フォルダ：3,800（BA）対 4,100（LLVM）
* `mir` フォルダ：4,400（BA）対 0（LLVM）
* `base.rs`：1,100（BA）対 250（LLVM）
* `builder.rs`：1,400（BA）対 0（LLVM）
* `common.rs`：350（BA）対 350（LLVM）

`debuginfo` フォルダは、分割によってほとんど手つかずのままで、LLVMに固有です。その高レベルの機能のみがトレイト化されています。

新しい `traits` フォルダは、トレイト定義だけで1500行です。全体として、27,000行の古い `rustc_codegen_llvm` コードは、新しい18,500行の `rustc_codegen_llvm` と12,000行の `rustc_codegen_ssa` に分割されました。このリファクタリングにより、複数の `rustc` バックエンド間で重複する必要があった約10,000行を再利用できるようになったと言えます。

リファクタリングされたバージョンの `rustc` バックエンドは、テストスイートやパフォーマンスベンチマークで回帰を引き起こしませんでした。これは、コンパイル時のパラメトリシティのみを使用した（トレイトオブジェクトなし）リファクタリングの性質と一致しています。
