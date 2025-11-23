# rustc-perfを使ったプロファイリング

[Rustベンチマークスイート][rustc-perf]は、
Rustコンパイラをプロファイリングおよびベンチマークするための包括的な方法を提供します。
スイートの使用方法については、[マニュアル][rustc-perf-readme]で説明されています。

ただし、スイートを手動で使用するのは少し面倒な場合があります。`rustc`コントリビューターが
これを簡単にできるように、コンパイラビルドシステム(`bootstrap`)もベンチマークスイートとの
組み込み統合を提供しており、スイートをダウンロードしてビルドし、
ローカルコンパイラツールチェーンをビルドして、簡略化されたコマンドラインインターフェースを
使用してプロファイリングできるようにします。

この統合を使用するには、`./x perf <command> [options]`コマンドを使用できます。

このコマンドには、`--stage 1`や`--stage 2`などの通常のブートストラップフラグを使用できます。
たとえば、作成されるsysrootのステージを変更できます。プロファイリングをより適切にサポートするために
`bootstrap.toml`を設定することも役立つ場合があります。たとえば、`rust.debuginfo-level = 1`を
設定すると、ビルドされたコンパイラにソース行情報が追加されます。

`x perf`は現在、以下のコマンドをサポートしています:
- `benchmark <id>`: コンパイラをベンチマークし、渡された`id`の下に結果を保存します。
- `compare <baseline> <modified>`: 渡された2つの`id`を持つ2つのコンパイラのベンチマーク結果を比較します。
- `eprintln`: コンパイラを実行して`stderr`出力をキャプチャするだけです。コンパイラは通常
  `stderr`に何も出力しないため、出力を得るには`eprintln!`呼び出しを追加する必要があるかもしれません。
- `samply`: [samply][samply]サンプリングプロファイラを使用してコンパイラをプロファイリングします。
- `cachegrind`: [Cachegrind][cachegrind]を使用して、コンパイラの実行の詳細なシミュレートトレースを生成します。

> プロファイラの詳細な説明は、[`rustc-perf`マニュアル][rustc-perf-readme-profilers]にあります。

`x perf`コマンドには以下のオプションを使用できます。これらは、スイートで使用できる
`profile_local`および`bench_local`コマンドの対応するオプションを反映しています:

- `--include`: プロファイリング/ベンチマークするベンチマークを選択します。
- `--profiles`: プロファイリング/ベンチマークするプロファイル(`Check`、`Debug`、`Opt`、`Doc`)を選択します。
- `--scenarios`: プロファイリング/ベンチマークするシナリオ(`Full`、`IncrFull`、`IncrPatched`、`IncrUnchanged`)を選択します。

[samply]: https://github.com/mstange/samply
[cachegrind]: https://www.cs.cmu.edu/afs/cs.cmu.edu/project/cmt-40/Nice/RuleRefinement/bin/valgrind-3.2.0/docs/html/cg-manual.html
[rustc-perf]: https://github.com/rust-lang/rustc-perf
[rustc-perf-readme]: https://github.com/rust-lang/rustc-perf/blob/master/collector/README.md
[rustc-perf-readme-profilers]: https://github.com/rust-lang/rustc-perf/blob/master/collector/README.md#profiling-local-builds
