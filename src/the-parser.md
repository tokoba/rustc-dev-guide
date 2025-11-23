# 字句解析と構文解析

コンパイラが最初に行うことは、プログラム（UTF-8 Unicodeテキスト）を受け取り、文字列よりも便利なデータ形式に変換することです。これは2つの段階で行われます：字句解析と構文解析。

  1. _字句解析_は、文字列を[トークン]のストリームに変換します。たとえば、`foo.bar + buz`は、トークン`foo`、`.`、`bar`、`+`、および`buz`に変換されます。これは[`rustc_lexer`][lexer]で実装されています。

[トークン]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/token/index.html
[lexer]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lexer/index.html

  2. _構文解析_は、トークンのストリームを受け取り、コンパイラが作業しやすい構造化された形式に変換します。通常、[*抽象構文木*（AST）][ast]と呼ばれます。

## AST

ASTはメモリ内でRustプログラムの構造をミラーリングし、`Span`を使用して特定のASTノードをそのソーステキストにリンクします。ASTは[`rustc_ast`][rustc_ast]で定義されており、トークンとトークンストリームの定義、ASTを変更するためのデータ構造/トレイト、およびコンパイラのAST関連の他の部分の共有定義（レキサーやマクロ展開など）が含まれています。

AST内のすべてのノードには、独自の[`NodeId`]があります。これには、構造体などのトップレベルアイテムだけでなく、個々の文や式も含まれます。[`NodeId`]は、クレート内でASTノードを一意に識別する識別番号です。

ただし、クレート内で絶対的であるため、AST内の単一のノードを追加または削除すると、すべての後続の[`NodeId`]が変更されます。これにより、[`NodeId`]はインクリメンタルコンパイルにはほとんど役に立たなくなります。インクリメンタルコンパイルでは、できるだけ少ないものを変更したいためです。

[`NodeId`]は、マクロ展開や名前解決など、ASTで直接動作する`rustc`のすべてのビットで使用されます（これらについては次のいくつかの章で詳しく説明します）。

[`NodeId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/node_id/struct.NodeId.html

## 構文解析

パーサーは、レキサーへの高レベルインターフェースとマクロ展開後に実行される検証ルーチンとともに、[`rustc_parse`][rustc_parse]で定義されています。特に、[`rustc_parse::parser`][parser]にはパーサーの実装が含まれています。

パーサーへのメインエントリーポイントは、[rustc_parse][rustc_parse]内のさまざまな`parse_*`関数などです。これらを使用すると、[`SourceFile`][sourcefile]（たとえば、単一ファイルのソース）をトークンストリームに変換したり、トークンストリームからパーサーを作成したり、パーサーを実行して[`Crate`]（ルートASTノード）を取得したりすることができます。

コピー量を最小限に抑えるために、[`Lexer`]と[`Parser`]の両方には、親の[`ParseSess`]にバインドするライフタイムがあります。これには、解析中に必要なすべての情報と、[`SourceMap`]自体が含まれています。

解析中に、マクロ定義または呼び出しに遭遇する場合があることに注意してください。これらは展開のために取り置きます（[マクロ展開](./macro-expansion.md)を参照）。展開自体にはマクロの出力を解析する必要がある場合があり、さらに多くのマクロが明らかになる可能性があります。

## 字句解析の詳細

字句解析のコードは2つのクレートに分かれています：

- [`rustc_lexer`]クレートは、トークンを構成するチャンクに`&str`を分割する責任があります。[`rustc_lexer`]内のレキサーは手書きですが、生成された有限状態機械としてレキサーを実装するのが一般的です。

- [`Lexer`]は、[`rustc_lexer`]を`rustc`固有のデータ構造と統合します。具体的には、[`rustc_lexer`]によって返されるトークンに`Span`情報を追加し、識別子をインターンします。

[`Crate`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/ast/struct.Crate.html
[`Parser`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/struct.Parser.html
[`ParseSess`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/parse/struct.ParseSess.html
[`rustc_lexer`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lexer/index.html
[`SourceMap`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/source_map/struct.SourceMap.html
[`Lexer`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/lexer/struct.Lexer.html
[ast]: ./ast-validation.md
[parser]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/parser/index.html
[rustc_ast]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/index.html
[rustc_parse]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_parse/index.html
[sourcefile]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/struct.SourceFile.html
