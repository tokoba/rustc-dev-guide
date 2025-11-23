# コードインデックス

rustcには多くの重要なデータ構造があります。これは、コンパイラの主要なデータ構造のいくつかについて、どこで詳しく学べるかのガイダンスを提供する試みです。

項目            |  種類    | 短い説明           | 章            | 宣言
----------------|----------|-----------------------------|--------------------|-------------------
`BodyId` | struct | HIRノード識別子の4つのタイプのうちの1つ | [Identifiers in the HIR] | [compiler/rustc_hir/src/hir.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.BodyId.html)
`Compiler` | struct | コンパイラセッションを表し、コンパイルを駆動するために使用できます。 | [The Rustc Driver and Interface] | [compiler/rustc_interface/src/interface.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/interface/struct.Compiler.html)
`ast::Crate` | struct | 解析されたクレートの構文レベル表現 | [The parser] | [compiler/rustc_ast/src/ast.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/ast/struct.Crate.html)
`rustc_hir::Crate` | struct | クレートのASTのより抽象的でコンパイラに優しい形式 | [The Hir] | [compiler/rustc_hir/src/hir.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Crate.html)
`DefId` | struct | HIRノード識別子の4つのタイプのうちの1つ | [Identifiers in the HIR] | [compiler/rustc_hir/src/def_id.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/def_id/struct.DefId.html)
`Diag` | struct | エラーやlintなどのコンパイラ診断のための構造体 | [Emitting Diagnostics] | [compiler/rustc_errors/src/diagnostic.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.Diag.html)
`DocContext` | struct | rustdocがクレートをクロールしてドキュメントを収集するときに使用される状態コンテナ | [Rustdoc] | [src/librustdoc/core.rs](https://github.com/rust-lang/rust/blob/HEAD/src/librustdoc/core.rs)
`HirId` | struct | HIRノード識別子の4つのタイプのうちの1つ | [Identifiers in the HIR] | [compiler/rustc_hir_id/src/lib.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/struct.HirId.html)
`Lexer` | struct | 構文解析中に使用される字句解析器。コンパイルされる生のソースコードから文字を消費し、パーサーの残りの部分で使用するための一連のトークンを生成します | [The parser] |  [compiler/rustc_parse/src/lexer/mod.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/lexer/struct.Lexer.html)
`NodeId` | struct | HIRノード識別子の4つのタイプのうちの1つ。段階的に廃止中 | [Identifiers in the HIR] | [compiler/rustc_ast/src/ast.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/node_id/struct.NodeId.html)
`P` | struct | 所有された不変スマートポインタ。対照的に、`&T`は所有されておらず、`Box<T>`は不変ではありません。 | None | [compiler/rustc_ast/src/ptr.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/ptr/struct.P.html)
`ParamEnv` | struct | ジェネリックパラメータまたは`Self`に関する情報で、関連アイテムやジェネリックアイテムを扱う際に有用です | [Parameter Environment] | [compiler/rustc_middle/src/ty/mod.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.ParamEnv.html)
`ParseSess` | struct | この構造体にはパースセッションに関する情報が含まれています | [The parser] | [compiler/rustc_session/src/parse/parse.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/parse/struct.ParseSess.html)
`Rib` | struct | 名前の単一スコープを表します | [Name resolution] | [compiler/rustc_resolve/src/lib.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_resolve/late/struct.Rib.html)
`Session` | struct | コンパイルセッションに関連するデータ | [The parser], [The Rustc Driver and Interface] | [compiler/rustc_session/src/session.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/struct.Session.html)
`SourceFile` | struct | `SourceMap`の一部。単一のソースファイルに対してASTノードをそのソースコードにマップします。以前は FileMapと呼ばれていました | [The parser] | [compiler/rustc_span/src/lib.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/struct.SourceFile.html)
`SourceMap` | struct | ASTノードをそのソースコードにマップします。`SourceFile`で構成されています。以前はCodeMapと呼ばれていました | [The parser] | [compiler/rustc_span/src/source_map.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/source_map/struct.SourceMap.html)
`Span` | struct  | ユーザーのソースコード内の位置で、主にエラー報告に使用されます | [Emitting Diagnostics] | [compiler/rustc_span/src/span_encoding.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/struct.Span.html)
`rustc_ast::token_stream::TokenStream` | struct | `TokenTree`に編成されたトークンの抽象シーケンス | [The parser], [Macro expansion] | [compiler/rustc_ast/src/tokenstream.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/tokenstream/struct.TokenStream.html)
`TraitDef` | struct | この構造体には型情報を含むトレイトの定義が含まれています | [The `ty` modules] |  [compiler/rustc_middle/src/ty/trait_def.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/trait_def/struct.TraitDef.html)
`TraitRef` | struct | トレイトとその入力型の組み合わせ（例：`P0: Trait<P1...Pn>`） | [Trait Solving: Goals and Clauses]  |  [compiler/rustc_middle/src/ty/sty.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.TraitRef.html)
`Ty<'tcx>` | struct | 型チェックに使用される型の内部表現 | [Type checking] | [compiler/rustc_middle/src/ty/mod.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html)
`TyCtxt<'tcx>` | struct | 「型付けコンテキスト」。これはコンパイラの中心的なデータ構造です。あらゆる種類のクエリを実行するために使用するコンテキストです | [The `ty` modules] | [compiler/rustc_middle/src/ty/context.rs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html)

[The HIR]: ../hir.html
[Identifiers in the HIR]: ../hir.html#hir-id
[The parser]: ../the-parser.html
[The Rustc Driver and Interface]: ../rustc-driver/intro.html
[Type checking]: ../type-checking.html
[The `ty` modules]: ../ty.html
[Rustdoc]: ../rustdoc.html
[Emitting Diagnostics]: ../diagnostics.html
[Macro expansion]: ../macro-expansion.html
[Name resolution]: ../name-resolution.html
[Parameter Environment]: ../typing_parameter_envs.html
[Trait Solving: Goals and Clauses]: ../traits/goals-and-clauses.html#domain-goals
