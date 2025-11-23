# THIR

THIR（「Typed High-Level Intermediate Representation」）は、以前は「High-Level Abstract IR」の意味でHAIRと呼ばれていましたが、[型チェック]の後に生成される別のIRです。<!-- date-check --> 2024年1月現在、[MIR構築]、[網羅性チェック]、[unsafe性チェック]に使用されています。

[型チェック]: ./type-checking.md
[MIR構築]: ./mir/construction.md
[網羅性チェック]: ./pat-exhaustive-checking.md
[unsafe性チェック]: ./unsafety-checking.md

名前が示すように、THIRは、型チェックが完了した後にすべての型が入力された[HIR]の低レベルバージョンです。しかし、HIRと区別するいくつかの興味深い機能があります：

- MIRと同様に、THIRは「実行可能なコード」である本体のみを表します。これには、関数本体だけでなく、たとえば`const`初期化子も含まれます。具体的には、すべての[本体所有者]がTHIRを作成します。その結果、THIRには`struct`や`trait`などのアイテムの表現がありません。

- 各THIRの本体は一時的にのみ保存され、不要になるとすぐに削除されます。コンパイルプロセスの終わりまで保存される（HIRで行われることです）のとは対照的です。

- すべてのノードの型を利用可能にすることに加えて、THIRにはHIRと比較して追加の脱糖が含まれています。たとえば、自動参照と逆参照が明示的になり、メソッド呼び出しとオーバーロードされた演算子がプレーンな関数呼び出しに変換されます。破棄スコープも明示的になります。

- 文、式、マッチアームは別々に保存されます。たとえば、`stmts`配列内の文は、[`ExprId`]として表される`exprs`配列内の式のインデックスで式を参照します。

[HIR]: ./hir.md
[`ExprId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/thir/struct.ExprId.html
[本体所有者]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/enum.BodyOwnerKind.html

THIRは[`rustc_mir_build::thir`][thir-docs]に存在します。[`thir::Expr`]を構築するには、THIRが割り当てられるメモリアリーナを渡して[`thir_body`]関数を使用できます。このアリーナをドロップすると、THIRが破棄され、ピークメモリを抑えるのに役立ちます。クレートのすべての本体のTHIR表現をメモリに同時に持つことは非常に重いでしょう。

`-Zunpretty=thir-tree`フラグを`rustc`に渡すことで、THIRのデバッグ表現を取得できます。

デモンストレーションのために、次の例を使用しましょう：

```rust
fn main() {
    let x = 1 + 2;
}
```

<!-- date-check --> 2022年8月現在、THIRでの表現方法は次のとおりです：

```rust,no_run
Thir {
    // マッチアームなし
    arms: [],
    exprs: [
        // 式0、値1のリテラル
        Expr {
            ty: i32,
            temp_lifetime: Some(
                Node(1),
            ),
            span: oneplustwo.rs:2:13: 2:14 (#0),
            kind: Literal {
                lit: Spanned {
                    node: Int(
                        1,
                        Unsuffixed,
                    ),
                    span: oneplustwo.rs:2:13: 2:14 (#0),
                },
                neg: false,
            },
        },
        // 式1、リテラル1を囲むスコープ
        Expr {
            ty: i32,
            temp_lifetime: Some(
                Node(1),
            ),
            span: oneplustwo.rs:2:13: 2:14 (#0),
            kind: Scope {
                // 上記の式0への参照
                region_scope: Node(3),
                lint_level: Explicit(
                    HirId {
                        owner: DefId(0:3 ~ oneplustwo[6932]::main),
                        local_id: 3,
                    },
                ),
                value: e0,
            },
        },
        // 式2、リテラル2
        Expr {
            ty: i32,
            temp_lifetime: Some(
                Node(1),
            ),
            span: oneplustwo.rs:2:17: 2:18 (#0),
            kind: Literal {
                lit: Spanned {
                    node: Int(
                        2,
                        Unsuffixed,
                    ),
                    span: oneplustwo.rs:2:17: 2:18 (#0),
                },
                neg: false,
            },
        },
        // 式3、リテラル2を囲むスコープ
        Expr {
            ty: i32,
            temp_lifetime: Some(
                Node(1),
            ),
            span: oneplustwo.rs:2:17: 2:18 (#0),
            kind: Scope {
                region_scope: Node(4),
                lint_level: Explicit(
                    HirId {
                        owner: DefId(0:3 ~ oneplustwo[6932]::main),
                        local_id: 4,
                    },
                ),
                // 上記の式2への参照
                value: e2,
            },
        },
        // 式4、1 + 2を表す
        Expr {
            ty: i32,
            temp_lifetime: Some(
                Node(1),
            ),
            span: oneplustwo.rs:2:13: 2:18 (#0),
            kind: Binary {
                op: Add,
                // 上記のリテラルを囲むスコープへの参照
                lhs: e1,
                rhs: e3,
            },
        },
        // 式5、式4を囲むスコープ
        Expr {
            ty: i32,
            temp_lifetime: Some(
                Node(1),
            ),
            span: oneplustwo.rs:2:13: 2:18 (#0),
            kind: Scope {
                region_scope: Node(5),
                lint_level: Explicit(
                    HirId {
                        owner: DefId(0:3 ~ oneplustwo[6932]::main),
                        local_id: 5,
                    },
                ),
                value: e4,
            },
        },
        // 式6、文の周りのブロック
        Expr {
            ty: (),
            temp_lifetime: Some(
                Node(9),
            ),
            span: oneplustwo.rs:1:11: 3:2 (#0),
            kind: Block {
                body: Block {
                    targeted_by_break: false,
                    region_scope: Node(8),
                    opt_destruction_scope: None,
                    span: oneplustwo.rs:1:11: 3:2 (#0),
                    // 以下の文0への参照
                    stmts: [
                        s0,
                    ],
                    expr: None,
                    safety_mode: Safe,
                },
            },
        },
        // 式7、式6のブロック周りのスコープ
        Expr {
            ty: (),
            temp_lifetime: Some(
                Node(9),
            ),
            span: oneplustwo.rs:1:11: 3:2 (#0),
            kind: Scope {
                region_scope: Node(9),
                lint_level: Explicit(
                    HirId {
                        owner: DefId(0:3 ~ oneplustwo[6932]::main),
                        local_id: 9,
                    },
                ),
                value: e6,
            },
        },
        // 式7周りの破棄スコープ
        Expr {
            ty: (),
            temp_lifetime: Some(
                Node(9),
            ),
            span: oneplustwo.rs:1:11: 3:2 (#0),
            kind: Scope {
                region_scope: Destruction(9),
                lint_level: Inherited,
                value: e7,
            },
        },
    ],
    stmts: [
        // let文
        Stmt {
            kind: Let {
                remainder_scope: Remainder { block: 8, first_statement_index: 0},
                init_scope: Node(1),
                pattern: Pat {
                    ty: i32,
                    span: oneplustwo.rs:2:9: 2:10 (#0),
                    kind: Binding {
                        mutability: Not,
                        name: "x",
                        mode: ByValue,
                        var: LocalVarId(
                            HirId {
                                owner: DefId(0:3 ~ oneplustwo[6932]::main),
                                local_id: 7,
                            },
                        ),
                        ty: i32,
                        subpattern: None,
                        is_primary: true,
                    },
                },
                initializer: Some(
                    e5,
                ),
                else_block: None,
                lint_level: Explicit(
                    HirId {
                        owner: DefId(0:3 ~ oneplustwo[6932]::main),
                        local_id: 6,
                    },
                ),
            },
            opt_destruction_scope: Some(
                Destruction(1),
            ),
        },
    ],
}
```

[thir-docs]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_build/thir/index.html
[`thir::Expr`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/thir/struct.Expr.html
[`thir_body`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.thir_body
