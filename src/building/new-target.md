# 新しいターゲットの追加

新しいターゲットのサポートを追加するための一連の手順です。望ましいゴールに到達するための多数の最終状態とパスがあるため、すべてのセクションが関連するわけではありません。

関連するドキュメントについては、[target tier policy] もご覧ください。

[target tier policy]: https://doc.rust-lang.org/rustc/target-tier-policy.html#adding-a-new-target

## 新しい LLVM の指定

非常に新しいターゲットの場合、Rust で現在出荷されているものとは異なる LLVM のフォークを使用する必要がある場合があります。その場合は、`src/llvm-project` git サブモジュール（サブモジュールが更新されるように少なくとも 1 回 `./x check` を実行する必要がある場合があります）に移動し、フォークの適切なコミットをチェックアウトし、メインの Rust リポジトリでその新しいサブモジュール参照をコミットします。

例：

```
cd src/llvm-project
git remote add my-target-llvm some-llvm-repository
git checkout my-target-llvm/my-branch
cd ..
git add llvm-project
git commit -m 'Use my custom LLVM'
```

### 事前ビルドされた LLVM の使用

すでにビルドされたローカルの LLVM チェックアウトがある場合、Rust がビルドをシステム LLVM として扱うように設定することで、冗長なビルドを避けることができる可能性があります。

`bootstrap.toml` の `target` セクションを使用して、事前ビルドされたバージョンの LLVM を使用するように Rust に指示できます：

```toml
[target.x86_64-unknown-linux-gnu]
llvm-config = "/path/to/llvm/llvm-7.0.1/bin/llvm-config"
```

システム LLVM を使用しようとしている場合、以前に次のパスを観察しましたが、システムによって異なる場合があります：

- `/usr/bin/llvm-config-8`
- `/usr/lib/llvm-8/bin/llvm-config`

codegen テストに使用される LLVM `FileCheck` ツールがインストールされている必要があることに注意してください。このツールは通常 LLVM でビルドされますが、独自の事前インストールされた LLVM を使用する場合は、別の方法で `FileCheck` を提供する必要があります。Debian ベースのシステムでは、`llvm-N-tools` パッケージ（`N` は LLVM のバージョン番号、例：`llvm-8-tools`）をインストールできます。または、`bootstrap.toml` の `llvm-filecheck` 設定項目で `FileCheck` へのパスを指定するか、`bootstrap.toml` の `codegen-tests` 項目で codegen テストを無効にできます。

## ターゲット仕様の作成

ターゲット JSON ファイルから始める必要があります。`--print target-spec-json` を使用して、既存のターゲットの仕様を確認できます：

```
rustc -Z unstable-options --target=wasm32-unknown-unknown --print target-spec-json
```

その JSON をファイルに保存し、ターゲットに合わせて適切に変更します。

### ターゲット仕様の追加

JSON 仕様を記入し、ある程度成功裏にコンパイルできるようになったら、コンパイラ自体に仕様をコピーできます。

`rustc_target::spec` モジュールの `supported_targets` マクロ内の大きなテーブルに行を追加する必要があります。次に、`target` 関数を含む新しいターゲット用の対応するファイルを追加します。

既存のターゲットを例として使用してください。

`rustc_target` クレートにターゲットを追加した後、新しいターゲットをサポートする `core`、`std` などを追加したい場合があります。その場合、おそらくいくつかの `target_*` cfg へのアクセスが必要になります。残念ながら、stage0（事前コンパイルされたコンパイラ）でビルドする場合、stage0 は新しいターゲット仕様を知らないため、ターゲット cfg が予期しないというエラーが発生し、チェックするために `--check-cfg` を渡します。

エラーを修正するには、`library/{std,alloc,core}/Cargo.toml` の異なる `Cargo.toml` に予期しない値を手動で追加する必要があります。以下は、`target_arch` として `NEW_TARGET_ARCH` を追加する例です：

*`library/std/Cargo.toml`*:

```diff
  [lints.rust.unexpected_cfgs]
  level = "warn"
  check-cfg = [
      'cfg(bootstrap)',
-      'cfg(target_arch, values("xtensa"))',
+      # #[cfg(bootstrap)] NEW_TARGET_ARCH
+      'cfg(target_arch, values("xtensa", "NEW_TARGET_ARCH"))',
```

ブートストラップでこのターゲットを使用するには、`src/bootstrap/src/core/sanity.rs` の `STAGE0_MISSING_TARGETS` リストにターゲットトリプルを明示的に追加する必要があります。これは、ブートストラップが使用するデフォルトのコンパイラが、追加したばかりの新しいターゲットを認識しないために必要です。したがって、このターゲットが stage0 コンパイラによってまだサポートされていないことをブートストラップが認識できるように、`STAGE0_MISSING_TARGETS` に追加する必要があります。

```diff
const STAGE0_MISSING_TARGETS: &[&str] = &[
+   "NEW_TARGET_TRIPLE"
];
```

## クレートへのパッチ適用

コンパイラが依存するクレート（[`libc`][] や [`cc`][] など）に変更を加える必要がある場合があります。その場合、Cargo の [`[patch]`][patch] 機能を使用できます。たとえば、未リリースバージョンの `libc` を使用したい場合は、トップレベルの `Cargo.toml` ファイルに追加できます：

```diff
diff --git a/Cargo.toml b/Cargo.toml
index 1e83f05e0ca..4d0172071c1 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -113,6 +113,8 @@ cargo-util = { path = "src/tools/cargo/crates/cargo-util" }
 [patch.crates-io]
+libc = { git = "https://github.com/rust-lang/libc", rev = "0bf7ce340699dcbacabdf5f16a242d2219a49ee0" }

 # See comments in `src/tools/rustc-workspace-hack/README.md` for what's going on
 # here
 rustc-workspace-hack = { path = 'src/tools/rustc-workspace-hack' }
```

この後、`cargo update -p libc` を実行してロックファイルを更新します。

ローカルの `path` 依存関係にパッチを適用すると、その依存関係の警告が有効になることに注意してください。一部の依存関係は警告がなく、`bootstrap.toml` の `deny-warnings` 設定により、ビルドが突然失敗し始める可能性があります。警告を回避するには、次のようにします：

- 依存関係を変更して警告を削除する
- またはローカル開発のために、bootstrap.toml で deny-warnings = false を設定して警告を抑制します。

```toml
# bootstrap.toml
[rust]
deny-warnings = false
```

[`libc`]: https://crates.io/crates/libc
[`cc`]: https://crates.io/crates/cc
[patch]: https://doc.rust-lang.org/stable/cargo/reference/overriding-dependencies.html#the-patch-section

## クロスコンパイル

JSON でターゲット仕様を持ち、コードに含めたら、`rustc` をクロスコンパイルできます：

```
DESTDIR=/path/to/install/in \
./x install -i --stage 1 --host aarch64-apple-darwin.json --target aarch64-apple-darwin \
compiler/rustc library/std
```

ターゲット仕様がブートストラップコンパイラですでに利用可能な場合は、両方の引数に JSON ファイルの代わりにそれを使用できます。

## tier 2（ターゲット）から tier 2（ホスト）へのターゲットの昇格

tier 2 ターゲットには 2 つのレベルがあります：

- クロスコンパイルのみのターゲット（`rustup target add`）
- [ネイティブツールチェーンを持つ][tier2-native] ターゲット（`rustup toolchain install`）

[tier2-native]: https://doc.rust-lang.org/nightly/rustc/target-tier-policy.html#tier-2-with-host-tools

ターゲットをクロスコンパイルからネイティブに昇格する例については、[#75914](https://github.com/rust-lang/rust/pull/75914) をご覧ください。
