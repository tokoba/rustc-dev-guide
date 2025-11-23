# その他のテスト関連情報

## `RUSTC_BOOTSTRAP` と安定性

<!-- date-check: Nov 2024 -->

これはブートストラップ/コンパイラ実装の詳細ですが、テストにも役立ちます：

- `RUSTC_BOOTSTRAP=1` は通常の安定性チェックを「迂回」し、安定版 `rustc` で不安定な機能や CLI フラグを使用できるようにします。
- `RUSTC_BOOTSTRAP=-1` は、実際には nightly `rustc` であっても、指定された `rustc` に安定版コンパイラであるかのように振る舞わせます。これは、コンパイラの一部の振る舞い（例：診断）がコンパイラが nightly かどうかによって異なる場合があるため有用です。

`//@ rustc-env` をサポートする `ui` テストやその他のテストスイートでは、次のように指定できます：

```rust,ignore
// 安定版 rustc で不安定な機能を使用可能にする
//@ rustc-env:RUSTC_BOOTSTRAP=1

// または nightly rustc に安定版 rustc のふりをさせる
//@ rustc-env:RUSTC_BOOTSTRAP=-1
```

`run-make`/`run-make-cargo` テストでは、`//@ rustc-env` はサポートされていません。個々の `rustc` 呼び出しに対して次のようにすることができます。

```rust,ignore
use run_make_support::rustc;

fn main() {
    rustc()
        // 私はとても安定しているふりをする
        .env("RUSTC_BOOTSTRAP", "-1")
        //...
        .run();
}
```
