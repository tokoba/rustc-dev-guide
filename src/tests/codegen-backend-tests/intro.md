# コード生成バックエンドのテスト

[コード生成](../../backend/codegen.md) の章も参照してください。

主要な LLVM コード生成バックエンドに加えて、rust-lang/rust CI は特定のテストジョブで [cranelift][cg_clif] および [GCC][cg_gcc] コード生成バックエンドのテストも実行します。

関連するテストの詳細については、以下を参照してください：

- [Cranelift コード生成バックエンドのテスト](./cg_clif.md)
- [GCC コード生成バックエンドのテスト](./cg_gcc.md)

[cg_clif]: https://github.com/rust-lang/rustc_codegen_cranelift
[cg_gcc]: https://github.com/rust-lang/rustc_codegen_gcc
