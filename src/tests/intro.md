# コンパイラのテスト

Rustプロジェクトは、ビルドシステム（`./x test`）によって調整されたさまざまな種類のテストを実行します。このセクションでは、さまざまなテストツールの簡単な概要を説明します。後続の章では、[テストの実行](running.md)と[新しいテストの追加](adding.md)について詳しく説明します。

## テストの種類

Rustディストリビューションの物事を実行するためのいくつかの種類のテストがあります。ほとんどすべてが`./x test`によって駆動されますが、以下に記載されているいくつかの例外があります。

### Compiletest

コンパイラ自体をテストするためのメインテストハーネスは、[compiletest]と呼ばれるツールです。

[compiletest]は、*テストスイート*に整理されたさまざまなスタイルのテストの実行をサポートしています。*テストモード*は、一連の*テストスイート*に共通のプリセット/動作を提供する場合があります。[compiletest]サポートされたテストは、[`tests`]ディレクトリにあります。

[Compiletest章][compiletest]では、このツールの使用方法について詳しく説明しています。

> 例：`./x test tests/ui`

[compiletest]: compiletest.md
[`tests`]: https://github.com/rust-lang/rust/tree/HEAD/tests

### パッケージテスト

標準ライブラリと多くのコンパイラパッケージには、典型的なRustの`#[test]`ユニットテスト、統合テスト、およびドキュメンテーションテストが含まれています。`library/`または`compiler/`ディレクトリのほとんどすべてのパッケージへのパスを`./x test`に渡すことができ、`x`は基本的にそのパッケージで`cargo test`を実行します。

例：

| コマンド                                  | 説明                              |
|-------------------------------------------|-----------------------------------|
| `./x test library/std`                    | `std`のみのテストを実行           |
| `./x test library/core`                   | `core`のみのテストを実行          |
| `./x test compiler/rustc_data_structures` | `rustc_data_structures`のテストを実行 |

標準ライブラリは、その機能をカバーするためにドキュメンテーションテストに大きく依存しています。ただし、必要に応じてユニットテストと統合テストも使用できます。ほとんどすべてのコンパイラパッケージでは、doctestが無効になっています。

すべての標準ライブラリとコンパイラのユニットテストは、別の`tests`ファイルに配置されます（これは[tidy][tidy-unit-tests]で強制されます）。これにより、テストファイルが変更されたときにクレートを再コンパイルする必要がなくなります。例：

```rust,ignore
#[cfg(test)]
mod tests;
```

このようにしなかった場合、`core`のような何かに取り組んでいた場合、標準ライブラリ全体、および`rustc`全体を再コンパイルする必要があります。

`./x test`には、これらのパッケージテストの動作を制御するためのいくつかのCLIオプションが含まれています：

* `--doc` — パッケージ内のドキュメンテーションテストのみを実行します。
* `--no-doc` — ドキュメンテーションテスト*以外*のすべてのテストを実行します。

[tidy-unit-tests]: https://github.com/rust-lang/rust/blob/HEAD/src/tools/tidy/src/unit_tests.rs

### Tidy

Tidyは、ソースコードのスタイルとフォーマット規則を検証するために使用されるカスタムツールです。たとえば、長い行を拒否します。詳細については、[コーディング規則のセクション](../conventions.md#formatting)または[Tidy Readme]を参照してください。

> 例：`./x test tidy`

[Tidy Readme]: https://github.com/rust-lang/rust/blob/HEAD/src/tools/tidy/Readme.md


### フォーマット

Rustfmtは、コンパイラ全体で統一されたスタイルを強制するためにビルドシステムと統合されています。フォーマットチェックは、上記のTidyツールによって自動的に実行されます。

例：

| コマンド                | 説明                                                               |
|-------------------------|--------------------------------------------------------------------|
| `./x fmt --check`       | フォーマットをチェックし、フォーマットが必要な場合はエラーで終了します。 |
| `./x fmt`               | コードベース全体でrustfmtを実行します。                            |
| `./x test tidy --bless` | 最初にrustfmtを実行してコードベースをフォーマットし、その後tidyチェックを実行します。 |

### ブックドキュメンテーションテスト

公開されているすべてのブックには独自のテストがあり、主にRustコードの例が合格することを検証するためのものです。内部的には、これらは基本的にmarkdownファイルで`rustdoc --test`を使用しています。テストは、ブックへのパスを`./x test`に渡すことで実行できます。

> 例：`./x test src/doc/book`

### ドキュメンテーションリンクチェッカー

すべてのドキュメンテーション間のリンクは、リンクチェッカーツールで検証され、次のように呼び出すことができます：

```console
./x test linkchecker
```

これには、すべてのドキュメンテーションをビルドする必要があり、しばらく時間がかかる場合があります。

### `distcheck`

`distcheck`は、ビルドシステムによって作成されたソース配布tarballが解凍、ビルドされ、すべてのテストが実行されることを検証します。

```console
./x test distcheck
```

### ツールテスト

Rustに含まれるパッケージは、すべてのテストも実行されます。これには、cargo、clippy、rustfmt、miri、bootstrap（Rustビルドシステム自体のテスト）などが含まれます。

ほとんどのツールは[`src/tools`]ディレクトリにあります。ツールのテストを実行するには、そのパスを`./x test`に渡すだけです。

> 例：`./x test src/tools/cargo`

通常、これらのツールは、ツールのディレクトリ内で`cargo test`を実行することを含みます。

指定されたテストセットのみを実行したい場合は、コマンドに`--test-args FILTER_NAME`を追加してください。

> 例：`./x test src/tools/miri --test-args padding`

CIでは、一部のツールは失敗が許可されています。失敗は対応するチームに通知を送信し、[toolstate website]で追跡されます。詳細については、[toolstate documentation]を参照してください。

[`src/tools`]: https://github.com/rust-lang/rust/tree/HEAD/src/tools/
[toolstate documentation]: https://forge.rust-lang.org/infra/toolstate.html
[toolstate website]: https://rust-lang-nursery.github.io/rust-toolstate/

### エコシステムテスト

Rustは、回帰を検出し、言語の進化について情報に基づいた決定を下すために、実際のコードとの統合をテストします。いくつかの種類のエコシステムテストがあり、Craterを含みます。詳細については、[エコシステムテスト章](ecosystem.md)を参照してください。

### パフォーマンステスト

コンパイラのパフォーマンスをテストおよび追跡するために、別のインフラストラクチャが使用されます。詳細については、[パフォーマンステスト章](perf.md)を参照してください。

### コードジェネレーションバックエンドテスト

[コードジェネレーションバックエンドテスト](./codegen-backend-tests/intro.md)を参照してください。

## その他の情報

[Misc info](misc.md)にいくつかの他の有用なテスト関連情報があります。

## 参考文献

以下のブログ投稿も興味深いかもしれません：

- brsonの古典的な["How Rust is tested"][howtest]

[howtest]: https://brson.github.io/2017/07/10/how-rust-is-tested
