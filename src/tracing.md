# tracingを使用したコンパイラのデバッグ

コンパイラには多くの[`debug!`]（または`trace!`）呼び出しがあり、多くのポイントでロギング情報を出力します。これらは、バグの場所を絞り込むだけでなく、完全に見つけるためにも非常に便利です。また、コンパイラが特定のことを行っている理由を理解するのにも役立ちます。

[`debug!`]: https://docs.rs/tracing/0.1/tracing/macro.debug.html

ログを表示するには、`RUSTC_LOG`環境変数をログフィルターに設定する必要があります。ログフィルターの完全な構文は、[`tracing-subscriber`のrustdoc](https://docs.rs/tracing-subscriber/0.2.24/tracing_subscriber/filter/struct.EnvFilter.html#directives)で見つけることができます。

## 関数レベルフィルター

rustcの多くの関数には、次のような注釈が付けられています

```
#[instrument(level = "debug", skip(self))]
fn foo(&self, bar: Type) {}
```

これにより、次のことができます

```
RUSTC_LOG=[foo]
```

これにより、次のことがすべて一度に行われます

* `foo`へのすべての関数呼び出しをログに記録する
* 引数をログに記録する（`skip`リストにあるものを除く）
* 関数が戻るまで（コンパイラのどこからでも）すべてをログに記録する

### すべてはいらない

関数のスコープによっては、その本体のすべてをログに記録したくない場合があります。例として、`do_mir_borrowck`関数は、些細なコードの借用チェックでも数百行をダンプします。

すべてのフィルターを組み合わせることができるので、クレート/モジュールパスを追加できます。例：

```
RUSTC_LOG=rustc_borrowck[do_mir_borrowck]
```

### すべての呼び出しはいらない

libcoreをコンパイルしている場合、*すべての*borrowckダンプが欲しいわけではなく、特定の関数に対してのみ欲しい場合があります。関数呼び出しをその引数で正規表現フィルタリングできます。

```
RUSTC_LOG=[do_mir_borrowck{id=\.\*from_utf8_unchecked\.\*}]
```

これにより、`from_utf8_unchecked`の借用チェックのログのみが表示されます。無視された`do_mir_borrowck`ごとに短いメッセージが表示されますが、それらの呼び出し内のものは表示されません。これは、発生している呼び出しを確認し、タイプミスがあった場合に正規表現を調整するのに役立ちます。

## クエリレベルフィルター

すべての[クエリ](query.md)は、クエリの実行中にすべてのログメッセージを表示できるように、ロギングスパンで自動的にタグ付けされます。たとえば、型チェック中にすべてをログに記録したい場合：

```
RUSTC_LOG=[typeck]
```

クエリ引数はトレーシングフィールドとして含まれているため、引数のデバッグ表示でフィルタリングできます。たとえば、`typeck`クエリには、チェックされているものの`key: LocalDefId`引数があります。正規表現を使用して、その`LocalDefId`と一致させて、特定の関数の型チェックをログに記録できます：

```
RUSTC_LOG=[typeck{key=.*name_of_item.*}]
```

異なるクエリには異なる引数があります。クエリとその引数のリストは、[`rustc_middle/src/query/mod.rs`](https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_middle/src/query/mod.rs#L18)で見つけることができます。

## 広範なモジュールレベルフィルター

`log`クレートのフィルターに似たフィルターを使用することもできます。これにより、特定のモジュール内のすべてが有効になります。これはしばしば冗長すぎて構造化されていないため、関数レベルフィルターを使用することをお勧めします。

ログフィルターは単に`debug`にしてすべての`debug!`出力とそれ以上（たとえば、`info!`も含まれます）を取得するか、`path::to::module`にして特定のモジュールから*すべて*の出力（`trace!`を含む）を取得するか、`path::to::module=debug`にして特定のモジュールから`debug!`出力とそれ以上を取得できます。

たとえば、特定のモジュールの`debug!`出力とそれ以上を取得するには、`RUSTC_LOG=path::to::module=debug rustc my-file.rs`でコンパイラを実行できます。すべての`debug!`出力が標準エラーに表示されます。

部分的なパスを使用しても、フィルターは機能することに注意してください。たとえば、`rustdoc::passes::collect_intra_doc_links`からの`info!`出力のみを表示したい場合、`RUSTDOC_LOG=rustdoc::passes::collect_intra_doc_links=info`を使用*できます*、または`RUSTDOC_LOG=rustdoc::passes::collect_intra=info`を使用できます。

rustdocを開発している場合は、代わりに`RUSTDOC_LOG`を使用してください。Miriを開発している場合は、代わりに`MIRI_LOG`を使用してください。わかるでしょう :)

使用できる完全な構文については、[`tracing`]クレートのドキュメント、特に[`debug!`]のドキュメントを参照してください。（注意：[`tracing`]クレートとその例とは異なり、`RUSTC_LOG`環境変数を使用します。rustc、rustdoc、およびその他のツールはカスタム環境変数を設定します。）

**非常に厳格なフィルターを使用しない限り、ロガーは大量の出力を出力するため、可能な限り具体的なモジュールを使用してください（複数の場合はカンマ区切り）**。標準エラーをファイルにパイプし、テキストエディターでログ出力を確認することが一般的には良いアイデアです。

それで、まとめると：

```bash
# これは、`rustc_middle/src/traits`内のすべてのデバッグ呼び出しの出力を
# 標準エラーに出力し、コンソールのバックスクロールがいっぱいになる可能性があります。
$ RUSTC_LOG=rustc_middle::traits=debug rustc +stage1 my-file.rs

# これは、`rustc_middle/src/traits`内のすべてのデバッグ呼び出しの出力を
# `traits-log`に出力するので、テキストエディターで確認できます。
$ RUSTC_LOG=rustc_middle::traits=debug rustc +stage1 my-file.rs 2>traits-log

# 推奨されません！これは、Rustコンパイラ内のすべての`debug!`呼び出しの出力を表示し、
# *非常に多く*あるため、何かを見つけるのが難しくなります。
$ RUSTC_LOG=debug rustc +stage1 my-file.rs 2>all-log

# これは、`rustc_codegen_ssa`内のすべての`info!`呼び出しの出力を表示します。
#
# `codegen_instance`には、コードジェン化されるすべての関数を出力する`info!`文があります。
# これは、どの関数がLLVMアサーションをトリガーするかを見つけるのに便利で、
# これは`debug!`ログではなく`info!`ログなので、公式のコンパイラで動作します。
$ RUSTC_LOG=rustc_codegen_ssa=info rustc +stage1 my-file.rs

# これは、`rustc_codegen_ssa`と`rustc_resolve`のすべてのログを表示します。
$ RUSTC_LOG=rustc_codegen_ssa,rustc_resolve rustc +stage1 my-file.rs

# これは、rustdocまたはそれが呼び出すrustcライブラリによって行われたすべての`info!`呼び出しの出力を表示します。
$ RUSTDOC_LOG=info rustdoc +stage1 my-file.rs

# これは、rustdocが直接行った`debug!`呼び出しのみを表示し、`rustc*`クレートは表示しません。
$ RUSTDOC_LOG=rustdoc=debug rustdoc +stage1 my-file.rs
```

## ログの色

デフォルトでは、rustc（およびrustdocやMiriなどの他のツール）は、ログ出力でANSI色を使用するタイミングについて賢くなります。ターミナルに出力している場合は色を使用し、ファイルに出力したり他の場所にパイプしたりしている場合は使用しません。ただし、非常に厳格なフィルターを使用しない限り、ターミナルでログ出力を読むのは難しいため、出力を`less`のようなページャーにパイプしたい場合があります。しかし、その場合、色がなくなり、探しているものを選び出すのが難しくなります！

`RUSTC_LOG_COLOR`環境変数（またはrustdocの場合は`RUSTDOC_LOG_COLOR`、Miriの場合は`MIRI_LOG_COLOR`など）を使用して、ログ出力に色を含めるかどうかをオーバーライドできます。3つのオプションがあります：`auto`（デフォルト）、`always`、`never`。したがって、`less`にパイプするときに色を有効にしたい場合は、次のようなコマンドを使用します：

```bash
# `-R`スイッチは、lessにANSI色をエスケープせずに出力するように指示します。
$ RUSTC_LOG=debug RUSTC_LOG_COLOR=always rustc +stage1 ... | less -R
```

`MIRI_LOG_COLOR`は、Miriからのログのみを色付けし、MiriがLLVM が呼び出すrustc関数からのログは色付けしないことに注意してください。rustcからのログを色付けするには、`RUSTC_LOG_COLOR`を使用してください。

## 結果のバイナリから`debug!`と`trace!`呼び出しを保持または削除する方法

`error!`、`warn!`、`info!`への呼び出しはコンパイラのすべてのビルドに含まれますが、`debug!`と`trace!`への呼び出しは、bootstrap.tomlで`debug-logging=true`がオンになっている場合にのみプログラムに含まれます（デフォルトではオフになっています）。したがって、`DEBUG`ログが表示されない場合、特にコンパイラを`RUSTC_LOG=rustc rustc some.rs`で実行して`INFO`ログしか表示されない場合は、bootstrap.tomlで`debug-logging=true`がオンになっていることを確認してください。

## ロギングのエチケットと規約

`debug!`への呼び出しはデフォルトで削除されるため、ほとんどの場合、「不要な」`debug!`呼び出しを追加してコミットするコードに残しておくことのパフォーマンスを心配する必要はありません。それらは出荷するもののパフォーマンスを低下させません。

とはいえ、過度のトレーシング呼び出しもあり得ます。特に、近くの他の呼び出しやここから呼び出される関数内の呼び出しと冗長である場合です。ここで完璧なバランスを取ることはできず、レビュアーの裁量に委ねられます。マージ前に`debug!`文を残すか削除するかを決定します。

非常にノイズの多いログの場合は、`debug!`よりも`trace!`を使用することが望ましい場合があります。

ゆるく従われている規約は、関数`foo`の先頭で`debug!("foo(...)")`よりも`#[instrument(level = "debug")]`を使用することです（[属性のドキュメントも参照してください](https://docs.rs/tracing-attributes/0.1.17/tracing_attributes/attr.instrument.html)）。関数内では、`debug!("xyz = {:?}", variable.field)`よりも`debug!(?variable.field)`を、`debug!("bar = {:?}", var.method(arg))`よりも`debug!(bar = ?var.method(arg))`を優先してください。この構文のドキュメントは[ここ](https://docs.rs/tracing/0.1.28/tracing/#recording-fields)で見つけることができます。

注意すべき点の1つは、ログ内の**高価な**操作です。

モジュール`rustc::foo`内に次のような文がある場合

```Rust
debug!(x = ?random_operation(tcx));
```

誰かがデバッグ`rustc`を`RUSTC_LOG=rustc::foo`で実行すると、`random_operation()`が実行されます。このデバッグ文を有効にしない`RUSTC_LOG`フィルターは、`random_operation`を実行しません。

これは、そこにあまりにも高価なものやクラッシュしやすいものを配置すべきではないことを意味します。そのモジュールのロギングを使用したい人を悩ませることになります。誰かが*別の*バグを見つけるためにロギングを使用しようとするまで、誰もそれを知りません。

[`tracing`]: https://docs.rs/tracing
