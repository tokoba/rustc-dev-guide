# バックエンドクラッシュの報告

コンパイル失敗後に大量のllvm-irコードが表示される場合、enzymeバックエンドがコードのコンパイルに失敗した可能性があります。このようなケースはデバッグが困難なため、ご協力いただけると大変ありがたいです。また、現時点ではリリースビルドの方がうまく動作する可能性が高いことにご注意ください。

ここでの最終目標は、enzyme [compiler explorer](https://enzyme.mit.edu/explorer/)でバグを再現し、[Enzyme](https://github.com/enzymead/enzyme/issues)リポジトリにバグレポートを作成することです。

`rustflags`に渡すことができる`autodiff`フラグがあり、これを使うことでこの作業を支援できます。これにより、`__enzyme_fwddiff`や`__enzyme_autodiff`の呼び出しとともに、llvm-irモジュール全体が出力されます。Linuxでの潜在的なワークフローは次のようになります：

## llvm-ir生成の制御

llvm-irを生成する前に、デバッグのために関連するrustコードを表示するのに役立つ2つのテクニックを覚えておいてください：

- **`std::hint::black_box`**：rustの変数や式を`std::hint::black_box()`でラップすることで、rustとllvmが最適化で削除するのを防ぎます。llvm-irで特定の値を検査したり手動で操作したりする必要がある場合に便利です。
- **`extern "rust"`または`extern "c"`**：特定の関数宣言がllvm-irにどのように変換されるかを確認したい場合は、`extern "rust"`または`extern "c"`として宣言できます。また、生成されたモジュール内の既存の`__enzyme_autodiff`や類似の宣言を例として探すこともできます。

## 1) llvm-irリプロデューサーを生成する

```sh
RUSTFLAGS="-Z autodiff=Enable,PrintModbefore" cargo +enzyme build --release &> out.ll
```

これにより、モジュールの前後にいくつかの警告と情報メッセージも取り込まれます。out.llを開き、`; moduleid = <somehash>`より上の行をすべて削除してください。次に、ファイルの末尾を確認し、llvm-irの一部ではないものをすべて削除します。つまり、エラーと警告を削除します。llvm-irの最後の行は、`!<somenumber> = `で始まるはずです。例えば、`!40831 = !{i32 0, i32 1037508, i32 1037538, i32 1037559}`や`!43760 = !dilocation(line: 297, column: 5, scope: !43746)`などです。

実際の数値はコードによって異なります。

## 2) llvm-irリプロデューサーを確認する

前のステップが機能したことを確認するために、llvmの`opt`ツールを使用します。optバイナリへのパスを見つけてください。`<some_dir>/rust/build/<x86/arm/...-target-triple>/ci-llvm/bin/opt`のようなパスになります。LLVMをソースからビルドした場合は、`ci-llvm`を`build`に置き換える必要があるでしょう。また、`llvmenzyme-21.<so/dll/dylib>`のパスも見つけてください。`/rust/build/target-triple/enzyme/build/enzyme/llvmenzyme-21`のようなパスです。llvmは頻繁にllvmバックエンドを更新するため、バージョン番号が高くなる可能性があることに留意してください（20、21、...）。両方が揃ったら、次のコマンドを実行します：

```sh
<path/to/opt> out.ll -load-pass-plugin=/path/to/build/<target-triple>/stage1/lib/libEnzyme-21.so -passes="enzyme" -enzyme-strict-aliasing=0  -s
```
このコマンドは将来のバージョンやシステムで失敗する可能性があります。その場合は、libEnzyme-21.soをLLVMEnzyme-21.soに置き換えてください。ビルド方法についてはEnzymeのドキュメントを参照してください。LLVMバージョンのビルド方法も調整する必要があるかもしれません。

前のステップが成功した場合、cargoでrustコードをコンパイルした時と同じエラーが表示されます。

同じエラーを再現できない場合は、rustリポジトリにissueを開いてください。成功した場合は、おめでとうございます！ただし、ファイルはまだ巨大なので、自動的に最小化しましょう。

## 3) llvm-irリプロデューサーを最小化する

まず、`llvm-extract`バイナリを見つけてください。これはoptバイナリと同じフォルダにあります。次に、以下を実行します：

```sh
<path/to/llvm-extract> -s --func=<name> --recursive --rfunc="enzyme_autodiff*" --rfunc="enzyme_fwddiff*" --rfunc=<fnc_called_by_enzyme> out.ll -o mwe.ll
```

このコマンドは、最小動作例である`mwe.ll`を作成します。

最後の`--func`フラグで渡す名前を調整してください。微分する関数に`#[no_mangle]`属性を適用すると、rustの名前で置き換えることができます。そうでない場合は、マングルされた関数名を検索する必要があります。そのためには、`out.ll`を開いて`__enzyme_fwddiff`または`__enzyme_autodiff`を検索してください。その関数呼び出しの最初の文字列が関数の名前です。例：

```llvm-ir
define double @enzyme_opt_helper_0(ptr %0, i64 %1, double %2) {
  %4 = call double (...) @__enzyme_fwddiff(ptr @_zn2ad3_f217h3b3b1800bd39fde3e, metadata !"enzyme_const", ptr %0, metadata !"enzyme_const", i64 %1, metadata !"enzyme_dup", double %2, double %2)
  ret double %4
}
```

ここで、`_zn2ad3_f217h3b3b1800bd39fde3e`が正しい名前です。先頭の`@`をコピーしないように注意してください。ステップ2)を再度実行しますが、今回は`out.ll`の代わりに`mwe.ll`を入力ファイルとして渡して`opt`コマンドを実行してください。この最小化された例でも引き続きクラッシュが再現されるか確認してください。

## 4) (オプション) llvm-irリプロデューサーをさらに最小化する

前のステップの後、約5k行の`mwe.ll`ファイルができているはずです。これを50行まで削減してみましょう。`opt`と`llvm-extract`の隣にある`llvm-reduce`バイナリを見つけてください。エラーメッセージの最初の行をコピーします。例えば：

```sh
opt: /home/manuel/prog/rust/src/llvm-project/llvm/lib/ir/instructions.cpp:686: void llvm::callinst::init(llvm::functiontype*, llvm::value*, llvm::arrayref<llvm::value*>, llvm::arrayref<llvm::operandbundledeft<llvm::value*> >, const llvm::twine&): assertion `(args.size() == fty->getnumparams() || (fty->isvararg() && args.size() > fty->getnumparams())) && "calling a function with bad signature!"' failed.
```

`segfault`だけが表示される場合は、意味のあるエラーメッセージがなく、自動的にできることも多くないため、5)に進んでください。
それ以外の場合は、以下の内容を含む`script.sh`ファイルを作成します。

```sh
#!/bin/bash
<path/to/your/opt> $1 -load-pass-plugin=/path/to/llvmenzyme-19.so -passes="enzyme" \
    |& grep "/some/path.cpp:686: void llvm::callinst::init"
```

grepに渡すエラーメッセージを少し試してみてください。エラーが一意であることを確認できるように十分長くする必要があります。ただし、`(`または`)`を含む長いエラーの場合は、正しくエスケープする必要があり、面倒になることがあります。次を実行します：

```sh
<path/to/llvm-reduce> --test=script.sh mwe.ll
```

`input isn't interesting! verify interesting-ness test`と表示された場合は、script.shのエラーメッセージが間違っています。grepが実際のエラーに一致することを確認する必要があります。すべてうまくいけば、多くの反復処理が表示され、最後に新しい`reduced.ll`ファイルができます。`opt`で同じエラーが引き続き発生することを確認してください。

### 高度なデバッグ：手動llvm-ir調査

最小化されたリプロデューサー（`mwe.ll`または`reduced.ll`）ができたら、さらに深く掘り下げることができます：

- **手動編集：** llvm-irを手動で書き換えてみてください。間接呼び出しに関する問題など、特定の問題については、`__enzyme_virtualreverse`のようなenzyme固有のイントリンシックを調査する必要があるかもしれません。これらの使用方法を理解するには、enzymeのドキュメントやソースコードを参照する必要があるかもしれません。
- **enzymeテストケース：** 問題に関連する機能やイントリンシックの正しい使用方法を示す可能性のある関連テストケースを[enzymeリポジトリ](https://github.com/enzymead/enzyme/tree/main/enzyme/test)で探してください。

## 5) バグを報告する

その後、`mwe.ll`（または`reduced.ll`）の例を[compiler explorer](https://enzyme.mit.edu/explorer/)にコピー＆ペーストできるはずです。

- 言語として`llvm ir`を選択し、コンパイラとして`opt 20`を選択します。
- コンパイラの右側のフィールドを`-passes="enzyme"`に置き換えます（まだ設定されていない場合）。
- うまくいけば、すでに馴染みのあるエラーが再び表示されます。
- 共有ボタンを使用してリンクをコピーしてください。
- [https://github.com/enzymead/enzyme/issues](https://github.com/enzymead/enzyme/issues)にissueを作成し、`mwe.ll`と（ある場合は）`reduced.ll`、およびcompiler explorerへのリンクを共有してください。rustコードやそのリンクも自由に追加してください。

#### 調査結果の文書化

`"attempting to call an indirect active function whose runtime value is inactive"`のような一部のenzymeエラーは、歴史的に混乱を引き起こしてきました。このような問題を調査する場合、完全な解決策を見つけられなくても、調査結果を文書化することを検討してください。洞察がenzymeに一般的であり、rustでの使用に固有でない場合、メインの[enzymeドキュメント](https://github.com/enzymead/www)に貢献することが最良の第一歩であることが多いです。また、関連するenzyme githubのissueで調査結果を言及したり、必要に応じてこれらのドキュメントへの更新を提案したりすることもできます。これにより、他の人がゼロから始める必要がなくなります。

明確なリプロデューサーとドキュメントがあれば、enzyme開発者がバグを修正できることを期待できます。それが起こると、rustコンパイラ内のenzymeサブモジュールが更新され、rustコードを微分できるようになります。rust-adの改善にご協力いただきありがとうございます。

# rustコードの最小化

最小限のllvm-irリプロデューサーを用意するだけでなく、依存関係のない最小限のrustリプロデューサーを用意することも役立ちます。これにより、修正後にテストケースとしてciに追加できるため、将来のリグレッションを回避できます。

rustリプロデューサーの最小化を支援するソリューションがいくつかあります。これがおそらく最も簡単な自動化されたアプローチです：[cargo-minimize](https://github.com/nilstrieb/cargo-minimize)。

それ以外にも、[`treereduce`](https://github.com/langston-barrett/treereduce)、[`halfempty`](https://github.com/googleprojectzero/halfempty)、[`picireny`](https://github.com/renatahodovan/picireny)、場合によっては[`creduce`](https://github.com/csmith-project/creduce)など、さまざまな代替手段があります。
