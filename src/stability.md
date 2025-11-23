# 安定性属性

このセクションは、rustc標準ライブラリで安定したAPIが内部で不安定なAPIを使用できるようにする安定性属性とスキームに関するものです。

**注意**：このセクションは*ライブラリ*機能についてであり、*言語*機能についてではありません。言語機能の安定化の手順については、[機能の安定化](./stabilization_guide.md)を参照してください。

## unstable

`#[unstable(feature = "foo", issue = "1234", reason = "lorem ipsum")]`属性は、アイテムを明示的に不安定としてマークします。「不安定」としてマークされたアイテムは、nightlyコンパイラであっても、対応する`#![feature]`属性をクレートに付けずに使用することはできません。この制限はクレート境界を越える場合にのみ適用され、不安定なアイテムはそれらを定義するクレート内で使用できます。

`issue`フィールドは、関連するGitHubの[issue番号]を指定します。このフィールドは必須であり、すべての不安定な機能には関連する追跡issueがあるべきです。適切な値がない稀なケースでは、`issue = "none"`が使用されます。

`unstable`属性はすべてのサブアイテムに感染し、属性を再適用する必要はありません。したがって、これをモジュールに適用すると、モジュール内のすべてのアイテムが不安定になります。

`#[stable]`属性を使用することで、特定のサブアイテムを安定にすることができます。安定性スキームは`pub`の動作と同様に機能します。非公開モジュールの公開関数を持つことができ、不安定なモジュールの安定した関数を持つことができます（逆も同様）。

以前は、[rustcのバグ][rustc bug]により、不安定なモジュール内の安定したアイテムがその場所で安定したコードから利用可能でした。<!-- date-check --> 2024年9月の時点で、[偶発的に安定化されたパス][accidentally stabilized paths]を持つアイテムは、それらのパスに依存するコードを壊さないように`#[rustc_allowed_through_unstable_modules]`属性でマークされています。破壊的変更を避けるために必要な場合を除き、この属性をこれ以上のアイテムに追加*しないで*ください。

`unstable`属性は`soft`値も持つことができ、これはハードエラーではなく、将来的な非互換性を示すデフォルトで拒否されるlintになります。これは、誤って過去に受け入れられた`bench`属性で使用されています。これにより、Cargoのlintキャップを活用して依存関係を壊さないようにします。

[issue番号]: https://github.com/rust-lang/rust/issues
[rustc bug]: https://github.com/rust-lang/rust/issues/15702
[accidentally stabilized paths]: https://github.com/rust-lang/rust/issues/113387

## stable
`#[stable(feature = "foo", since = "1.420.69")]`属性は、アイテムを明示的に安定化としてマークします。安定した関数は本体で不安定なものを使用できることに注意してください。

## rustc_const_unstable

`#[rustc_const_unstable(feature = "foo", issue = "1234", reason = "lorem ipsum")]`は`unstable`属性と同じインターフェースを持っています。これは、`const fn`の定数性が不安定であることをマークするために使用されます。これは稀なケースでのみ必要です：
- `const fn`が不安定な言語機能やintrinsicを使用する場合。（コンパイラはこれに遭遇した場合、属性を追加するように指示します。）
- `const fn`が`#[stable]`であるが、まだconst-stableにする意図がない場合。
- const-unstableなintrinsicを呼び出すために必要な機能ゲートを変更する場合。

Const-stabilityは通常の安定性とは異なり、*再帰的*です：`#[rustc_const_unstable(...)]`関数は、安定したコードから間接的にも呼び出すことができません。これは、不安定なコンパイラの実装の詳細が誤って安定したコードに漏れることや、不完全な実装の偶発的な癖に私たちを閉じ込めることを避けるためです。このチェックを微調整する方法については、下記のrustc_const_stable_indirectおよびrustc_allow_const_fn_unstable属性を参照してください。

## rustc_const_stable

`#[rustc_const_stable(feature = "foo", since = "1.420.69")]`属性は、`const fn`の定数性が`stable`であることを明示的にマークします。

## rustc_const_stable_indirect

`#[rustc_const_stable_indirect]`属性は、`#[rustc_const_unstable(...)]`関数に追加して、`#[rustc_const_stable(...)]`関数から呼び出し可能にすることができます。これは、関数がその実装の観点から安定化の準備ができていることを示します（つまり、不安定なコンパイラ機能を使用していない）。まだconst-stableでない唯一の理由はAPIの懸念です。

これは、const-callsがコンパイラで合成される言語アイテムにも追加する必要があります。これにより、これらの呼び出しが再帰的const安定性ルールをバイパスしないことを保証します。

## rustc_intrinsic_const_stable_indirect

intrinsicでは、この属性はintrinsicを「公開された安定した関数で使用する準備ができている」とマークします。intrinsicに`rustc_const_unstable`属性がある場合は、削除する必要があります。**intrinsicにこの属性を追加するには、t-langとwg-const-evalの承認が必要です！**

## rustc_default_body_unstable

`#[rustc_default_body_unstable(feature = "foo", issue = "1234", reason = "lorem ipsum")]`属性は`unstable`属性と同じインターフェースを持っています。これは、トレイト内のアイテムのデフォルト実装を不安定としてマークするために使用されます。default-body-unstableなアイテムを持つトレイトは、そのようなアイテムに対して明示的な本体を提供することで安定的に実装できます。または、対応する`#![feature]`を有効にすることでデフォルトの本体を使用できます。

## ライブラリ機能の安定化

機能を安定化するには、次の手順に従ってください：

1. **@T-libs-api**メンバーに追跡issueでFCPを開始してもらい、FCPが完了するまで待ちます（`disposition-merge`で）。
2. `#[unstable(...)]`を`#[stable(since = "CURRENT_RUSTC_VERSION")]`に変更します。
3. このAPIのテストやdoc-testから`#![feature(...)]`を削除します。機能がコンパイラやツールで使用されている場合は、そこからも削除します。
4. これが`const fn`の場合、`#[rustc_const_stable(since = "CURRENT_RUSTC_VERSION")]`を追加します。または、これがまだconst-stabilizedされない場合は、新しい機能ゲート（新しい追跡issueで）のために`#[rustc_const_unstable(...)]`を追加します。
5. `rust-lang/rust`に対してPRを開きます。
   - 適切なラベルを追加します：`@rustbot modify labels: +T-libs-api`。
   - 追跡issueにリンクして「Closes #XXXXX」と記述します。

機能を安定化する例として、[FCPを伴う追跡issue #81656](https://github.com/rust-lang/rust/issues/81656)と関連する[実装PR #84642](https://github.com/rust-lang/rust/pull/84642)を参照できます。

## allow_internal_unstable

マクロとコンパイラのdesugarは、その本体を呼び出し側に公開します。標準ライブラリのマクロで不安定なものを使用できないことを回避するために、`#[allow_internal_unstable(feature1, feature2)]`属性があり、安定したマクロで指定された機能を使用できるようにします。

マクロがconst contextで使用され、`#[rustc_const_unstable(...)]`関数への呼び出しを生成する場合、`allow_internal_unstable`があっても*依然として*拒否されることに注意してください。マクロが誤って再帰的const安定性チェックをバイパスできないようにするために、関数に`#[rustc_const_stable_indirect]`を追加してください。

## rustc_allow_const_fn_unstable

上記で説明したように、安定した`const fn`内では不安定なconst機能は許可されません。間接的にも許可されません。

ただし、時々、機能が安定化されることはわかっているが、いつかはわからない場合や、安定した（しかし実行時に遅い）回避策があるため、不安定な機能を廃止した場合には常に安定したバージョンにフォールバックできる場合があります。そのような場合、`[rustc_allow_const_fn_unstable(feature1, feature2)]`属性を使用して、安定した（または間接的に安定した）`const fn`の本体で一部の不安定な機能を許可できます。

また、ランタイムとコンパイル時に呼び出すことが同じ動作をする必要があるという`const fn`の不変条件を守る注意も必要です（[このブログ記事][blog]も参照）。これは、たとえばメモリアドレスを整数に変換する`const fn`を作成してはならないことを意味します。なぜなら、アドレスは非決定的であり、コンパイル時には不明であることが多いからです。

**より多くの`rustc_allow_const_fn_unstable`属性を任意の`const fn`に追加する場合は、常に@rust-lang/wg-const-evalにpingしてください。**

## staged_api

`stable`または`unstable`属性を使用するすべてのクレートには、クレートに`#![feature(staged_api)]`属性を含める必要があります。

## deprecated

標準ライブラリの非推奨は、ユーザーコードの非推奨とほぼ同じです。`#[deprecated]`がアイテムに使用される場合、`stable`または`unstable`属性も必要です。

`deprecated`の形式は次のとおりです：

```rust,ignore
#[deprecated(
    since = "1.38.0",
    note = "非推奨の理由の説明",
    suggestion = "other_function"
)]
```

`suggestion`フィールドはオプションです。指定された場合、警告を修正するためのマシン適用可能な提案として使用できる文字列である必要があります。これは通常、識別子が名前変更されたが、他の重要な変更が必要ない場合に使用されます。`suggestion`フィールドが使用される場合、クレートルートに`#![feature(deprecated_suggestion)]`が必要です。

ユーザーコードとのもう1つの違いは、`since`フィールドが実際に現在のバージョンの`rustc`に対してチェックされることです。`since`が将来のバージョンにある場合、`deprecated_in_future` lintがトリガーされます。これはデフォルトで`allow`ですが、標準ライブラリのほとんどは`#![warn(deprecated_in_future)]`で警告に引き上げます。

## unstable_feature_bound
`#[unstable_feature_bound(foo)]`属性は、`#[unstable]`属性と一緒に使用して、安定した型と安定したトレイトの`impl`を不安定としてマークできます。std/coreでは、`#[unstable_feature_bound(foo)]`で注釈されたアイテムは、同じく`#[unstable_feature_bound(foo)]`で注釈された別のアイテムによってのみ使用できます。std/core外では、`#[unstable_feature_bound(foo)]`を持つアイテムを使用するには、クレートに`#![feature(foo)]`属性で機能を有効にする必要があります。

現在、`#[unstable_feature_bound]`で注釈できるアイテムは次のとおりです：
- `impl`
- 自由関数
- トレイト

[blog]: https://www.ralfj.de/blog/2018/07/19/const.html
