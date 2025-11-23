# エコシステムテスト

Rustは、回帰を検出し、言語の進化について情報に基づいた決定を下すために、エコシステムの実際のコードとの統合をテストします。

## テスト方法

### Crater

Craterは、何千もの公開プロジェクトでテストを実行するツールです。このツールには、実行用の独自の独立したインフラストラクチャがあり、CIの一部として実行されません。詳細については、[Crater章](crater.md)を参照してください。

### `cargotest`

`cargotest`は、いくつかのサンプルプロジェクト（`servo`、`ripgrep`、`tokei`など）で`cargo test`を実行する小さなツールです。これはCIの一部として実行され、大きな回帰がないことを確認します：

```console
./x test src/tools/cargotest
```

### 大規模OSSプロジェクトビルダー

CIで回帰テストとして使用される大規模なオープンソースRustプロジェクトをビルドするCIジョブがあります。統合ジョブは次のプロジェクトをビルドします：

- [Fuchsia](./ecosystem-test-jobs/fuchsia.md)
- [Rust for Linux](./ecosystem-test-jobs/rust-for-linux.md)
