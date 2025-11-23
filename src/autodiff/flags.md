# サポートされている`RUSTFLAGS`

デバッグやプロファイリングをサポートするために、実験的な`-Z autodiff` rustcフラグ（`RUSTFLAGS`経由でcargoに渡すことができます）のサポートを追加しました。これにより、rustcを再コンパイルせずにEnzymeの動作を変更できます。現在、`autodiff`には次の値をサポートしています。

### デバッグフラグ

```text
PrintTA // TypeAnalysis情報を出力
PrintTAFn // 特定の関数のTypeAnalysis情報を出力
PrintAA // ActivityAnalysis情報を出力
Print // 微分された関数の生成と最適化中に出力
PrintPerf // AD関連のパフォーマンス警告を出力
PrintModBefore // AD実行の直前にLLVM-IRモジュール全体を出力
PrintModAfter // AD実行後、最適化前にLLVM-IRモジュール全体を出力
PrintModFinal // 最適化とAD実行後にLLVM-IRモジュール全体を出力
LooseTypes // 型情報が不足している場合、中止する代わりに不正確な導関数のリスクを負う
```

<div class="warning">

`LooseTypes`は、`Can not deduce type of <X>`というEnzymeエラーを解消し、一部のコードを実行できるようにするのに役立つことがよくあります。しかし、このフラグは絶対に不正確な勾配を引き起こす可能性があることに留意してください。さらに悪いことに、勾配は特定の入力値に対しては正しいかもしれませんが、他の値に対しては正しくないかもしれません。そのため、このようなバグに関するissueを作成し、バグが修正されるまでの間だけこのフラグを一時的に使用してください。

</div>

### ベンチマークフラグ

パフォーマンス実験とベンチマークのために、以下もサポートしています：

```text
NoPostopt // AD後にLLVM-IRモジュールを最適化しない
RuntimeActivity // Enzymeのランタイムアクティビティ機能を有効にする
Inline // LLVMのデフォルトを超えて、可能な限りインライン化を最大化するようEnzymeに指示
```

複数の`autodiff`値をカンマで区切って組み合わせることができます：

```bash
RUSTFLAGS="-Z autodiff=Enable,LooseTypes,PrintPerf" cargo +enzyme build
```

`-Zautodiff=Enable`を使用すると、autodiffを使用でき、通常のrustcコンパイルパイプラインが更新されます：

1. 選択したコンパイルパイプラインを実行します。リリースビルドを選択した場合、ベクトル化とループ展開を無効にします。
2. 関数を微分します。
3. モジュール全体で選択したコンパイルパイプラインを再度実行します。今回はベクトル化とループ展開を無効にしません。
