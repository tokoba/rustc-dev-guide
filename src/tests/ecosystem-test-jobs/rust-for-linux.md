# Rust for Linux 統合テスト

[Rust for Linux](https://rust-for-linux.com/)（RfL）は、Linux カーネルに Rust プログラミング言語のサポートを追加する取り組みです。

## Rust for Linux ジョブが壊れた場合にどうすべきか？

PR が Rust for Linux CI ジョブを壊した場合は：

- 破損が意図的でなく、一時的なものであるように見える場合は、[RfL][rfl-ping] に知らせて再試行してください。
  - PR が緊急で、再試行で修正されない場合は、CI ジョブを一時的に無効にしてください
      （`src/ci/github-actions/jobs.yml` の `image: x86_64-rust-for-linux` ジョブをコメントアウト）。
- 破損が意図的でない場合は、PR を変更して破損を解決してください。
- 破損が意図的である場合は、[RfL][rfl-ping] に知らせて、カーネルで何を変更する必要があるかを議論してください。
  - PR が緊急の場合は、CI ジョブを一時的に無効にしてください（`src/ci/github-actions/jobs.yml` の `image: x86_64-rust-for-linux` ジョブをコメントアウト）。
  - PR が数日待てる場合は、RfL メンテナが必要な変更を行った新しい Linux カーネルコミットハッシュを提供するのを待ち、それを PR に適用します。これにより、変更が機能することを確認できます（`src/ci/docker/scripts/rfl-build.sh` の `LINUX_VERSION` 環境変数を更新）。

RfL 開発者に連絡する必要がある場合は、[Rust for Linux][rfl-ping] ピンググループにピングして助けを求めることができます：

```text
@rustbot ping rfl
```

## CI での Rust for Linux のビルド

Rust for Linux は、プルリクエストがマージされる前に実行される bors テストのスイートの一部としてビルドされます。

ワークフローは Rust コンパイラの stage1 sysroot をビルドし、Linux カーネルをダウンロードして、この sysroot を使用していくつかの Rust for Linux ドライバと例をコンパイルしようとします。RfL はいくつかの不安定なコンパイラ/言語機能を使用しているため、このワークフローは特定のコンパイラ変更がそれを壊すかどうかを通知します。

プルリクエストが Rust for Linux ビルダーを壊す可能性があり、bors キューに送信する前にテストしたい場合は、単に bors に Rust for Linux 統合をビルドする try ジョブを実行するように依頼してください：
`@bors try jobs=x86_64-rust-for-linux`。

[rfl-ping]: ../../notification-groups/rust-for-linux.md
