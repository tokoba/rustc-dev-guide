# 永続的に不安定な機能に関する注意事項

## `rustc_private`

### 概要

`rustc_private`機能により、外部クレートがコンパイラの内部を使用できるようになります。

### 公式ツールチェーンでの`rustc-private`の使用

rustupを介して配布される公式のRustツールチェーンで`rustc_private`機能を使用する場合、2つの追加コンポーネントをインストールする必要があります：

1. **`rustc-dev`**: コンパイラライブラリを提供します
2. **`llvm-tools`**: リンクに必要なLLVMライブラリを提供します

#### インストール手順

rustupを使用して両方のコンポーネントをインストールします：

```text
rustup component add rustc-dev llvm-tools
```

#### よくあるエラー

`llvm-tools`コンポーネントがないと、次のようなリンクエラーが発生します：

```text
error: linking with `cc` failed: exit status: 1
  |
  = note: rust-lld: error: unable to find library -lLLVM-{version}
```

### カスタムツールチェーンでの`rustc-private`の使用

カスタムビルドされたツールチェーンやrustupを使用していない環境では、通常、追加の設定が必要です：

#### 要件

- LLVMライブラリがシステムのライブラリ検索パスで利用可能である必要があります
- LLVMバージョンは、Rustツールチェーンのビルドに使用されたものと一致する必要があります

#### トラブルシューティング手順

1. **LLVMのインストールを確認**: LLVMがインストールされており、アクセス可能であることを確認します
2. **ライブラリパスの設定**: 環境変数を設定する必要がある場合があります：

   ```text
   export LD_LIBRARY_PATH=/path/to/llvm/lib:$LD_LIBRARY_PATH
   ```

3. **バージョンの互換性を確認**: LLVMバージョンがRustツールチェーンと互換性があることを確認します

### 追加リソース

- [GitHub Issue #137421](https://github.com/rust-lang/rust/issues/137421): `rustc_private`のリンカー障害は、`llvm-tools`がインストールされていないために発生することが多いことを説明しています
