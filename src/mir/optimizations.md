# MIR最適化

MIR最適化は、コード生成の前により良いMIRを生成するために[MIR][mir]上で実行される最適化です。これは2つの理由で重要です：第一に、最終的に生成される実行可能コードがより良くなり、第二に、LLVMの作業が少なくなるため、コンパイルが速くなります。MIRはジェネリック（まだ[単相化][monomorph]されていない）であるため、これらの最適化は特に効果的です。ジェネリックバージョンを最適化できるため、すべての単相化がより安価になります！

[mir]: ../mir/index.md
[monomorph]: ../appendix/glossary.md#mono

MIR最適化は、借用チェックの後に実行されます。MIRを改善するために、一連の最適化パスを実行します。一部のパスはすべてのコードで実行する必要があり、一部のパスは実際には最適化を行わず、ただチェックするだけで、一部のパスは`release`モードでのみ有効になります。

[`optimized_mir`][optmir]クエリが呼び出されて、指定された[`DefId`][defid]に対する最適化されたMIRを生成します。このクエリは、借用チェッカーが実行され、何らかの検証が行われたことを確認します。次に、MIRを[盗み][steal]、最適化し、改善されたMIRを返します。

[optmir]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_transform/fn.optimized_mir.html
[defid]: ../appendix/glossary.md#def-id
[steal]: ../mir/passes.md#stealing

## 新しい最適化を追加するためのクイックスタート

1. `tests/mir-opt`にRustソースファイルを作成し、最適化したいコードを示します。これはシンプルに保つ必要があるため、最適化に必要でない場合は`println!`やその他のフォーマットコードを避けてください。その理由は、`println!`、`format!`などは、最適化がテストに何をするかを理解しにくくする可能性のある大量のMIRを生成するためです。

2. `./x test --bless tests/mir-opt/<your-test>.rs`を実行して、MIRダンプを生成します。ダンプ方法については、[このREADME][mir-opt-test-readme]を参照してください。

3. 現在の作業ディレクトリの状態をコミットします。最適化を実装する前にテスト出力をコミットする理由は、あなた（およびレビュアー）が最適化が変更したものの前後の差分を確認できるようにするためです。

4. [`compiler/rustc_mir_transform/src`]に新しい最適化を実装します。これを行う最も速く簡単な方法は、

   1. 小さな最適化（[`remove_storage_markers`]など）を選択して新しいファイルにコピーし、
   2. 最適化を[`run_optimization_passes()`]関数のリストの1つに追加し、
   3. コピーした最適化の変更を開始します。

5. `./x test --bless tests/mir-opt/<your-test>.rs`を再実行して、MIRダンプを再生成します。差分を見て、期待どおりかどうかを確認します。

6. `./x test tests/ui`を実行して、最適化が何かを壊していないかどうかを確認します。

7. 最適化に問題がある場合は、少し実験して、ステップ5と6を繰り返します。

8. コミットしてPRを開きます。まだ機能していなくても、いつでもこれを行うことができるため、PRでフィードバックを求めることができます。その場合は「WIP」PR（PRタイトルに`[WIP]`をプレフィックスするか、進行中であることを他の方法で示す）を開きます。

   必ず、祝福されたテスト出力もコミットしてください！CIが通過するために必要であり、レビュアーにとって非常に役立ちます。

途中で質問がある場合は、Zulipの`#t-compiler/wg-mir-opt`でお気軽にお尋ねください。

[mir-opt-test-readme]: https://github.com/rust-lang/rust/blob/HEAD/tests/mir-opt/README.md
[`compiler/rustc_mir_transform/src`]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_mir_transform/src
[`remove_storage_markers`]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_mir_transform/src/remove_storage_markers.rs
[`run_optimization_passes()`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_transform/fn.run_optimization_passes.html

## 最適化パスの定義

実行されるパスのリストと実行される順序は、[`run_optimization_passes`][rop]関数によって定義されます。これには、実行するパスの配列が含まれています。配列内の各パスは、[`MirPass`]トレイトを実装する構造体です。配列は`&dyn MirPass`トレイトオブジェクトの配列です。通常、パスは[`rustc_mir_transform`][trans]クレートの独自のモジュールで実装されます。

[rop]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_transform/fn.run_optimization_passes.html
[`MirPass`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_transform/pass_manager/trait.MirPass.html
[trans]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_transform/index.html

パスの例は次のとおりです：

- `CleanupPostBorrowck`：コード生成ではなく、解析にのみ必要な一部の情報を削除します。
- `ConstProp`：[定数伝播][constprop]を行います。

より多くの例については、[`MirPass` rustdocsの「実装者」セクション][impl]を参照してください。

[impl]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_transform/pass_manager/trait.MirPass.html#implementors
[constprop]: https://en.wikipedia.org/wiki/Constant_folding#Constant_propagation

## MIR最適化レベル

MIR最適化には、さまざまな準備状態のレベルがあります。実験的な最適化は、誤コンパイルを引き起こしたり、コンパイル時間を遅くしたりする可能性があります。これらのパスは、フィードバックを収集し、パスを変更しやすくするために、nightlyビルドにまだ含まれています。遅いまたはその他の実験的な最適化パスで作業できるようにするには、`-Z mir-opt-level`デバッグフラグを指定できます。レベルの定義は、[コンパイラMCP]にあります。MIRパスを開発していて、最適化パスを実行すべきかどうかをクエリする場合は、[`tcx.sess.opts.unstable_opts.mir_opt_level`][mir_opt_level]を使用して現在のレベルをチェックできます。

[コンパイラMCP]: https://github.com/rust-lang/compiler-team/issues/319
[mir_opt_level]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/config/struct.UnstableOptions.html#structfield.mir_opt_level
