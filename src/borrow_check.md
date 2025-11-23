# MIR 借用チェック

借用チェックは Rust の「秘密のソース」です。次のような多くのプロパティを強制する役割があります：

- すべての変数は使用される前に初期化される。
- 同じ値を 2 回移動できない。
- 借用されている間は値を移動できない。
- 可変に借用されている間は場所にアクセスできない（参照を通じて除く）。
- 不変に借用されている間は場所を変更できない。
- など

借用チェッカーは MIR 上で動作します。古い実装は HIR 上で動作していました。MIR 上で借用チェックを行うことにはいくつかの利点があります：

- MIR は HIR よりも*はるかに*単純です。徹底的な脱糖により、
  借用チェッカーのバグを防ぐのに役立ちます。（興味があれば、
  MIR ベースの借用チェッカーが修正するバグのリストを[ここ][47366]で見ることができます。）
- さらに重要なことに、MIR を使用することで[「非字句的ライフタイム」][nll]が可能になります。
  これは制御フローグラフから導出される領域です。

[47366]: https://github.com/rust-lang/rust/issues/47366
[nll]: https://rust-lang.github.io/rfcs/2094-nll.html

### 借用チェッカーの主要なフェーズ

借用チェッカーのソースは
[`rustc_borrowck` クレート][b_c]にあります。主なエントリポイントは
[`mir_borrowck`] クエリです。

[b_c]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/index.html
[`mir_borrowck`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/fn.mir_borrowck.html

- まず、MIR の**ローカルコピー**を作成します。次のステップでは、
  計算している新しい領域への参照を含むように型などを変更するため、
  このコピーをその場で変更します。
- 次に、[`replace_regions_in_mir`] を呼び出してローカル MIR を変更します。
  とりわけ、この関数は MIR 内のすべての[領域](./appendix/glossary.md#region)を
  新しい[推論変数](./appendix/glossary.md#inf-var)に置き換えます。
- 次に、データが移動されるタイミングを計算する多数の
  [データフロー解析](./appendix/background.md#dataflow)を実行します。
- 次に、MIR 全体で[2 回目の型チェック](borrow_check/type_check.md)を行います：
  この型チェックの目的は、異なる領域間のすべての制約を決定することです。
- 次に、[領域推論](borrow_check/region_inference.md)を行います。これは各領域の値、
  つまり、収集した制約に従って各ライフタイムが有効でなければならない制御フローグラフ内のポイントを計算します。
- この時点で、各ポイントで「スコープ内の借用」を計算できます。
- 最後に、MIR 上で 2 回目のウォークを行い、それが行うアクションを見てエラーを報告します。たとえば、
  `*a + 1` のようなステートメントがある場合、変数 `a` が初期化されていて、
  可変に借用されていないことをチェックします。これらのいずれかがある場合は
  エラーを報告する必要があるためです。このチェックを行うには、以前のすべての解析の結果が必要です。

[`replace_regions_in_mir`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/nll/fn.replace_regions_in_mir.html
