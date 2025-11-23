# MIRのデバッグ

`-Z dump-mir`フラグは、MIRのテキスト表現をダンプするために使用できます。`-Z dump-mir`と組み合わせて使用される次のオプションフラグは、以下を含む追加の出力形式を有効にします：

* `-Z dump-mir-graphviz` - MIRを制御フローグラフとして表す`.dot`ファイルをダンプします
* `-Z dump-mir-dataflow` - 制御フローグラフの各時点での[データフロー状態]を示す`.dot`ファイルをダンプします

`-Z dump-mir=F`は、コンパイルの各段階で各関数のMIRを表示できる便利なコンパイラオプションです。`-Z dump-mir`は**フィルタ**`F`を取り、どの関数とどのパスに興味があるかを制御できます。例えば：

```bash
> rustc -Z dump-mir=foo ...
```

これは、名前に`foo`を含む関数のMIRをダンプします。すべてのパスの前後でMIRをダンプします。これらのファイルは`mir_dump`ディレクトリに作成されます。おそらくかなりの数になるでしょう！

```bash
> cat > foo.rs
fn main() {
    println!("Hello, world!");
}
^D
> rustc -Z dump-mir=main foo.rs
> ls mir_dump/* | wc -l
     161
```

ファイルには`rustc.main.000-000.CleanEndRegions.after.mir`のような名前があります。これらの名前にはいくつかの部分があります：

```text
rustc.main.000-000.CleanEndRegions.after.mir
      ---- --- --- --------------- ----- before または after
      |    |   |   パスの名前
      |    |   パス内のダンプのインデックス（通常は0ですが、一部のパスは中間状態をダンプします）
      |    パスのインデックス
      ダンプされる関数などへのdef-path
```

より選択的なフィルタを作成することもできます。例えば、`main & CleanEndRegions`は、`main`*と*パス`CleanEndRegions`の*両方*を参照するものを選択します：

```bash
> rustc -Z dump-mir='main & CleanEndRegions' foo.rs
> ls mir_dump
rustc.main.000-000.CleanEndRegions.after.mir	rustc.main.000-000.CleanEndRegions.before.mir
```
<!--- TODO: Change NoLandingPads. [#1232](https://github.com/rust-lang/rustc-dev-guide/issues/1232) -->
フィルタには、`&`フィルタの複数のセットを組み合わせる`|`パーツを含めることもできます。例えば、`main & CleanEndRegions | main & NoLandingPads`は、`main`*と*`CleanEndRegions`*または*`main`*と*`NoLandingPads`の*いずれか*を選択します：

```bash
> rustc -Z dump-mir='main & CleanEndRegions | main & NoLandingPads' foo.rs
> ls mir_dump
rustc.main-promoted[0].002-000.NoLandingPads.after.mir
rustc.main-promoted[0].002-000.NoLandingPads.before.mir
rustc.main-promoted[0].002-006.NoLandingPads.after.mir
rustc.main-promoted[0].002-006.NoLandingPads.before.mir
rustc.main-promoted[1].002-000.NoLandingPads.after.mir
rustc.main-promoted[1].002-000.NoLandingPads.before.mir
rustc.main-promoted[1].002-006.NoLandingPads.after.mir
rustc.main-promoted[1].002-006.NoLandingPads.before.mir
rustc.main.000-000.CleanEndRegions.after.mir
rustc.main.000-000.CleanEndRegions.before.mir
rustc.main.002-000.NoLandingPads.after.mir
rustc.main.002-000.NoLandingPads.before.mir
rustc.main.002-006.NoLandingPads.after.mir
rustc.main.002-006.NoLandingPads.before.mir
```

（ここで、`main-promoted[0]`ファイルは、`main`関数内に現れた「昇格された定数」のMIRを参照しています。）

`-Z unpretty=mir-cfg`フラグは、クレート全体のgraphviz MIR制御フロー図を作成するために使用できます：

![A control-flow diagram](mir_cfg.svg)

TODO: 他に何かありますか？

[データフロー状態]: ./dataflow.html#graphviz-diagrams
