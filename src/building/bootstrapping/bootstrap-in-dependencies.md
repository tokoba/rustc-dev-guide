# コンパイラの依存関係における `cfg(bootstrap)`

Rustコンパイラは、コンパイラ自体と循環的な依存関係を持つ可能性のある外部クレートを使用しています。コンパイラはビルドするために更新されたクレートを必要としますが、そのクレートは更新されたコンパイラを必要とします。このページでは、`#[cfg(bootstrap)]` をどのように使用してこの循環を断ち切ることができるかについて説明します。

## `#[cfg(bootstrap)]` の有効化

通常、外部クレートで `#[cfg(bootstrap)]` を使用すると、警告が発生します：

```
warning: unexpected `cfg` condition name: `bootstrap`
 --> src/main.rs:1:7
  |
1 | #[cfg(bootstrap)]
  |       ^^^^^^^^^
  |
  = help: expected names are: `docsrs`, `feature`, and `test` and 31 more
  = help: consider using a Cargo feature instead
  = help: or consider adding in `Cargo.toml` the `check-cfg` lint config for the lint:
           [lints.rust]
           unexpected_cfgs = { level = "warn", check-cfg = ['cfg(bootstrap)'] }
  = help: or consider adding `println!("cargo::rustc-check-cfg=cfg(bootstrap)");` to the top of the `build.rs`
  = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
  = note: `#[warn(unexpected_cfgs)]` on by default
```

この警告は、プロジェクトの `Cargo.toml` に以下の行を追加することで消すことができます：

```toml
[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(bootstrap)'] }
```

これで、`#[cfg(bootstrap)]` をコンパイラで使用できるのと同じように、クレート内で使用できるようになります。ブートストラップコンパイラが使用されている場合、`#[cfg(bootstrap)]` でアノテーションされたコードがコンパイルされ、それ以外の場合は `#[cfg(not(bootstrap))]` でアノテーションされたコードがコンパイルされます。

## 更新の手順

具体例として、`#[naked]` 属性を安全でない属性にした変更を使用します。これにより、`compiler-builtins` クレートとの循環依存が発生しました。

### ステップ1：コンパイラで新しい動作を受け入れる（[#139797](https://github.com/rust-lang/rust/pull/139797)）

この例では、エラーを無効化することで、古い動作と新しい動作の両方を同時に受け入れることが可能です。

### ステップ2：クレートを更新する（[#821](https://github.com/rust-lang/compiler-builtins/pull/821)）

次にクレート内で、`#[cfg(bootstrap)]` を使用して古い動作を使用するか、`#[cfg(not(bootstrap))]` を使用して新しい動作を使用します。

### ステップ3：コンパイラが使用するクレートバージョンを更新する（[#139934](https://github.com/rust-lang/rust/pull/139934)）

`compiler-builtins` の場合、これはバージョンのバンプを意味し、他のケースでは git サブモジュールの更新になる可能性があります。

### ステップ4：コンパイラから古い動作を削除する（[#139753](https://github.com/rust-lang/rust/pull/139753)）

更新されたクレートが使用できるようになりました。この例では、古い動作を削除できることを意味しました。
