# インラインアセンブリ

## 概要

rustc におけるインラインアセンブリは、主に `asm!` マクロの呼び出しを受け取り、
コンパイラの全レイヤーを通じて LLVM コード生成まで配管することを中心としています。様々な段階を通じて、
`InlineAsm` は一般的に 3 つのコンポーネントで構成されます：

- テンプレート文字列。`InlineAsmTemplatePiece` の配列として格納されます。各ピースは
リテラルまたはオペランドのプレースホルダー（フォーマット文字列と同様）のいずれかを表します。

  ```rust
  pub enum InlineAsmTemplatePiece {
      String(String),
      Placeholder { operand_idx: usize, modifier: Option<char>, span: Span },
  }
  ```

- `asm!` へのオペランドのリスト（`in`、`[late]out`、`in[late]out`、`sym`、`const`）。これらは
低レベル化の各段階で異なる方法で表現されますが、共通のパターンに従います：
  - `in`、`out`、`inout` はすべて、関連するレジスタクラス（`reg`）または明示的なレジスタ
（`"eax"`）を持ちます。
  - `inout` には 2 つの形式があります：読み取りと書き込みの両方が行われる単一の式を持つものと、
入力と出力の部分に対して 2 つの別々の式を持つものです。
  - `out` と `inout` には `late` フラグ（`lateout` / `inlateout`）があり、レジスタ
アロケータがこの出力に対して入力レジスタを再利用できることを示します。
  - `out` と `inout` の分割バリアントでは、出力に `_` を指定できます。これは
出力が破棄されることを意味します。これは、アセンブリコード用のスクラッチレジスタを割り当てるために使用されます。
  - `const` は匿名定数を参照し、一般的にインライン const のように機能します。
  - `sym` は少し特別で、パス式のみを受け入れます。これは `static`
または `fn` を指す必要があります。

- `asm!` マクロの最後に設定されたオプション。rustc にとって特に興味深いのは、
`asm!` が `()` の代わりに `!` を返すようにする `NORETURN` と、フォーマット文字列の解析を無効にする `RAW` です。残りのオプションは、ほとんど処理なしで LLVM に渡されます。

  ```rust
  bitflags::bitflags! {
      pub struct InlineAsmOptions: u16 {
          const PURE = 1 << 0;
          const NOMEM = 1 << 1;
          const READONLY = 1 << 2;
          const PRESERVES_FLAGS = 1 << 3;
          const NORETURN = 1 << 4;
          const NOSTACK = 1 << 5;
          const ATT_SYNTAX = 1 << 6;
          const RAW = 1 << 7;
          const MAY_UNWIND = 1 << 8;
      }
  }
  ```

## AST

`InlineAsm` は、[`ast::InlineAsm` 型][inline_asm_ast]で AST 内の式として表現されます。

`asm!` マクロは `rustc_builtin_macros` で実装され、`InlineAsm` AST ノードを出力します。
テンプレート文字列は `fmt_macros` を使用して解析され、位置引数と名前付きオペランドは
明示的なオペランドインデックスに解決されます。ターゲット情報はマクロ呼び出しでは利用できないため、
レジスタとレジスタクラスの検証は AST の低レベル化まで延期されます。

[inline_asm_ast]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/ast/struct.InlineAsm.html

## HIR

`InlineAsm` は、[`hir::InlineAsm` 型][inline_asm_hir]で HIR 内の式として表現されます。

AST の低レベル化は、`InlineAsmRegOrRegClass` が `Symbol` から実際のレジスタまたは
レジスタクラスに変換される場所です。テンプレート文字列のプレースホルダーに修飾子が指定されている場合、
これらはそのオペランド型で許可されているセットに対して検証されます。最後に、入力と
出力の明示的なレジスタは、競合（異なるオペランドに使用される同じレジスタ）がないかチェックされます。

[inline_asm_hir]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.InlineAsm.html

## 型チェック

各レジスタクラスには、それと一緒に使用できる型のホワイトリストがあります。すべての
オペランドの型が決定された後、`intrinsicck` パスはこれらの型がホワイトリストに
あることをチェックします。また、分割された `inout` オペランドが互換性のある型を持ち、`const`
オペランドが整数または浮動小数点数であることもチェックします。渡された型に基づいて
オペランドにテンプレート修飾子を使用する必要がある場合は、必要に応じて提案が出力されます。

## THIR

`InlineAsm` は、[`InlineAsmExpr` 型][inline_asm_thir]で THIR 内の式として表現されます。

HIR と比較して重要な変更点は、`Sym` が `SymFn`（`expr` が `fn` のリテラル ZST）
または `SymStatic`（`static` の `DefId` を指す）のいずれかに低レベル化されたことです。

[inline_asm_thir]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/thir/struct.InlineAsmExpr.html

## MIR

`InlineAsm` は、[`TerminatorKind::InlineAsm` バリアント][inline_asm_mir]で MIR の `Terminator` として表現されます。

THIR の低レベル化の一部として、`InOut` と `SplitInOut` オペランドは、別々の
`in_value` と `out_place` を持つ分割形式に低レベル化されます。

意味的には、`InlineAsm` ターミネータは `Call` ターミネータに似ていますが、
`Call` が単一の戻り値の出力場所しか持たないのに対し、複数の出力場所を持ちます。

[inline_asm_mir]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.TerminatorKind.html#variant.InlineAsm

## コード生成

オペランドは、LLVM コード生成に渡される前にもう一度低レベル化されます。これは
`rustc_codegen_ssa` の[`InlineAsmOperandRef` 型][inline_asm_codegen]によって表現されます。

オペランドは、以下のように LLVM オペランドと制約コードに低レベル化されます：
- `out` と `inout` オペランドの出力部分は、LLVM の要求に応じて最初に追加されます。遅延出力
オペランドには制約コードに `=` プレフィックスが追加され、非遅延出力オペランドには `=&`
プレフィックスが追加されます。
- `in` オペランドは通常通り追加されます。
- `inout` オペランドは、対応する出力オペランドに結び付けられます。
- `sym` オペランドは、`"s"` 制約を使用して、関数ポインタまたはポインタとして渡されます。
- `const` オペランドは文字列にフォーマットされ、テンプレート文字列に直接挿入されます。

テンプレート文字列は LLVM 形式に変換されます：
- `$` 文字は `$$` としてエスケープされます。
- `const` オペランドは文字列に変換され、直接挿入されます。
- プレースホルダーは `${X:M}` としてフォーマットされます。ここで `X` はオペランドインデックス、`M` は修飾子
文字です。修飾子は Rust 形式から LLVM 形式に変換されます。

様々なオプションは、クロバー制約または LLVM 属性に変換されます。詳細については
[RFC](https://github.com/Amanieu/rfcs/blob/inline-asm/text/0000-inline-asm.md#mapping-to-llvm-ir)
を参照してください。

LLVM は、特定の制約コードに対してどの型を受け入れるかについて時々かなり厳格であるため、
サポートされている型との間で変換を挿入する必要がある場合があることに注意してください。各レジスタクラスでどの型がサポートされているかの詳細については、
LLVM のターゲット固有の ISelLowering.cpp ファイルを参照してください。

[inline_asm_codegen]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/traits/enum.InlineAsmOperandRef.html

## 新しいアーキテクチャのサポートの追加

インラインアセンブリのサポートをアーキテクチャに追加することは、主にそのアーキテクチャの
レジスタとレジスタクラスを定義することです。レジスタクラスのすべての定義は
`compiler/rustc_target/asm/` にあります。

さらに、これらのレジスタクラスの LLVM 制約コードへの低レベル化を
`compiler/rustc_codegen_llvm/asm.rs` に実装する必要があります。

新しいアーキテクチャを追加する場合は、LLVM のソースコードと相互参照してください：
- LLVM は、特定の制約コードで使用できる型に制限があります。
`lib/Target/${ARCH}/${ARCH}ISelLowering.cpp` の `getRegForInlineAsmConstraint` 関数を参照してください。
- LLVM は、内部使用のために特定のレジスタを予約しており、インラインアセンブリブロックの周りで
適切に保存/復元されません。これらのレジスタは
`lib/Target/${ARCH}/${ARCH}RegisterInfo.cpp` の `getReservedRegs` 関数にリストされています。
フレーム/ベースポインタなどの「条件付き」予約レジスタは、
関数がフレーム/ベースポインタを必要とするかどうかを事前に知ることができないため、
常に Rust の目的で予約されているものとして扱う必要があります。

## テスト

インラインアセンブリには様々なテストがあります：

- `tests/assembly-llvm/asm`
- `tests/ui/asm`
- `tests/codegen-llvm/asm-*`

インラインアセンブリでサポートされているすべてのアーキテクチャには、
レジスタクラスと型のすべての組み合わせをテストする包括的なテストが
`tests/assembly-llvm/asm` に必要です。
