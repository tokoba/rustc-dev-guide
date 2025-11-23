# 定数評価

定数評価は、コンパイル時に値を計算するプロセスです。特定のアイテム(定数/静的/配列長)については、これはアイテムのMIRが借用チェックされ最適化された後に行われます。多くの場合、アイテムの定数評価を試みると、初めてそのMIRの計算がトリガーされます。

主な例は次のとおりです:

* `static`の初期化子
* 配列長
    * スタックまたはヒープ領域を予約するために知る必要がある
* 列挙型のバリアントの判別値
    * 2つのバリアントが同じ判別値を持つことを防ぐために知る必要がある
* パターン
    * 重複するパターンをチェックするために知る必要がある

さらに、定数評価は、コンパイル時に複雑な操作を事前計算し、結果のみを保存することで、実行時の作業量またはバイナリサイズを削減するために使用できます。

定数評価のすべての使用は、「型システムに影響を与える」(配列長、列挙型バリアントの判別値、定数ジェネリックパラメータ)か、実行時に使用される式を事前計算するためだけに行われるかのいずれかに分類できます。

定数評価は、`TyCtxt`の`const_eval_*`関数を呼び出すことで行うことができます。
これらは`const_eval`クエリのラッパーです。

* `const_eval_global_id_for_typeck`は定数をvaltreeに評価するため、結果の値をコンパイラでさらに検査できます。
* `const_eval_global_id`は定数を最終値を含む「不透明なブロブ」に評価します。
  これはコードジェネレーションバックエンドとCTFE評価エンジン自体にのみ役立ちます。
* `eval_static_initializer`は静的の初期値を具体的に計算します。
  静的は特別です。他のすべての関数は静的を正しく表現せず、静的での使用を防ぐアサーションがあります。

`const_eval_*`関数は、定数が評価される環境の[`ParamEnv`](./typing_parameter_envs.html)と[`GlobalId`]を使用します。`GlobalId`は、定数または静的を参照する`Instance`、または関数の`Instance`と関数の`Promoted`テーブルへのインデックスで構成されます。

定数評価は、型システム定数の[`EvalToValTreeResult`]または[MIR定数値](mir/index.md#mir-constant-values)を表す[`EvalToConstValueResult`]のいずれかでエラーまたは評価された定数の表現を返します。それぞれ[valtree](mir/index.md#valtrees)または[MIR constant value](mir/index.md#mir-constant-values)を返します。

[`GlobalId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/interpret/struct.GlobalId.html
[`EvalToConstValueResult`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/interpret/error/type.EvalToConstValueResult.html
[`EvalToValTreeResult`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/interpret/error/type.EvalToValTreeResult.html
