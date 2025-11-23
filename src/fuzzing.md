# ファジング

<!-- date-check: Mar 2023 -->

このガイドの目的において、*ファジング*とは、rustc のバグを発見する目的で、さまざまなプログラムをコンパイルするテスト手法のことです。ファジングは、内部コンパイラエラー（ICE）を見つけるためによく使用されます。ファジングは、ユーザーがバグに遭遇する前にバグを見つけ、バグを追跡しやすくする小さな自己完結型のプログラムを提供できるため、有益です。しかし、いくつかの一般的な間違いは、ファジングの有用性を低下させ、コントリビューターの生活を困難にする可能性があります。Rust プロジェクトに対する肯定的な影響を最大化するために、ファジングで生成されたバグを報告する前に、このガイドをお読みください！

## ガイドライン

### 要約

*お願い：*

- バグが最新の nightly rustc でまだ存在することを確認する
- バグレポートとともに、合理的に最小限の自己完結型の例を含める
- バグレポートテンプレートで要求されたすべての情報を含める
- 同じメッセージとクエリスタックを持つ既存のレポートを検索する
- バグが維持される場合、`rustfmt` でテストケースをフォーマットする
- バグがファジングによって見つかったことを示す

*お願いしないこと：*

- `custom_mir`、`lang_items`、`no_core`、`rustc_attrs` を含む（ただし、これらに限定されない）内部機能を使用する多数のバグを報告しないでください。
- rustc をクラッシュさせることが知られている入力でファジングをシードしないでください（詳細は以下）。

### 議論

ICE が既に報告されているものの重複であるかどうか確信が持てない場合は、関連すると思われる issue にリンクして報告してください。一般に、同じ行にあるが異なる*クエリスタック*を持つ ICE は、通常、異なるバグです。例えば、[#109020][#109020] と [#109129][#109129] は類似したエラーメッセージを持っていました：

```
error: internal compiler error: compiler/rustc_middle/src/ty/normalize_erasing_regions.rs:195:90: Failed to normalize <[closure@src/main.rs:36:25: 36:28] as std::ops::FnOnce<(Emplacable<()>,)>>::Output, maybe try to call `try_normalize_erasing_regions` instead
```
```
error: internal compiler error: compiler/rustc_middle/src/ty/normalize_erasing_regions.rs:195:90: Failed to normalize <() as Project>::Assoc, maybe try to call `try_normalize_erasing_regions` instead
```
しかし、異なるクエリスタックを持っていました：
```
query stack during panic:
#0 [fn_abi_of_instance] computing call ABI of `<[closure@src/main.rs:36:25: 36:28] as core::ops::function::FnOnce<(Emplacable<()>,)>>::call_once - shim(vtable)`
end of query stack
```
```
query stack during panic:
#0 [check_mod_attrs] checking attributes in top-level module
#1 [analysis] running analysis passes on this crate
end of query stack
```

[#109020]: https://github.com/rust-lang/rust/issues/109020
[#109129]: https://github.com/rust-lang/rust/issues/109129

## コーパスの構築

コーパスを構築するときは、rustc をクラッシュさせることが既に知られているテストの収集を避けてください。そのようなテストでシードされたファジングは、同じ根本原因を持つバグを生成する可能性が高く、全員の時間を無駄にします。これを避ける最も簡単な方法は、コーパス内の各ファイルをループし、ICE を引き起こすかどうかを確認し、そうである場合は削除することです。

コーパスを構築するには、以下を使用することをお勧めします：

- rustc/rust-analyzer/clippy テストスイート（またはソースコード）--- ただし、既に失敗することが知られているテストは避けてください。これらは多くの場合、`//@ failure-status: 101` または `//@ known-bug: #NNN` のようなコメントで始まります。
- アーカイブされた [Glacier][glacier] リポジトリの既に修正された ICE --- ただし、`ices/` にある未修正のものは避けてください！

[glacier]: https://github.com/rust-lang/glacier

## エクストラクレジット

ICE を提出した後、Rust プロジェクトを支援するためにできることがいくつかあります。

- バグを[二分探索][bisect]して、いつ導入されたかを把握します。回帰している PR/コミットを見つけた場合は、issue に `S-has-bisection` ラベルを付けることができます。見つからない場合は、代わりに `E-needs-bisection` を適用することを検討してください。
- 「気を散らすもの」を修正する：構文エラーや借用チェックエラーなど、ICE のトリガーに寄与しないテストケースの問題
- テストケースを最小化します（以下を参照）。成功した場合、issue に `S-has-mcve` ラベルを付けることができます。そうでない場合は、`E-needs-mcve` を適用できます。
- 最小限のテストケースを[クラッシュテスト][crash test]として rust-lang/rust リポジトリに追加します。その際、PR に他の「追跡されていない」クラッシュを含めることを検討してください。PR がマージされたら、関連するすべての issue に `S-bug-has-test` を付けることを忘れないでください。

[ラベルの適用と削除][labeling]も参照してください。

[bisect]: https://rust-lang.github.io/cargo-bisect-rustc/
[crash test]: tests/compiletest.html#crash-tests
[labeling]: https://forge.rust-lang.org/release/issue-triaging.html#applying-and-removing-labels

## 最小化

ファジングで生成された入力を注意深く*最小化*することは役立ちます。最小化するときは、元のエラーを保持するように注意し、構文、型チェック、借用チェックエラーなどの気を散らす問題を導入しないようにしてください。

最小化を支援するツールがいくつかあります。これらのツールを使用する際に、構文、型、借用チェックエラーを導入しないようにする方法がわからない場合は、完全なテストケースと最小化されたテストケースの両方を投稿してください。一般的に、*構文を認識する*ツールは、最小限の時間で最良の結果を提供します。[`treereduce-rust`][treereduce] と [picireny][picireny] は構文を認識します。[`halfempty`][halfempty] はそうではありませんが、一般的に高品質なツールです。

[halfempty]: https://github.com/googleprojectzero/halfempty
[picireny]: https://github.com/renatahodovan/picireny
[treereduce]: https://github.com/langston-barrett/treereduce

## 効果的なファジング

rustc をファジングするときは、機械語の生成を避けることをお勧めします。これは主に LLVM によって行われるためです。代わりに `--emit=mir` を試してください。

さまざまなコンパイラフラグは、さまざまな問題を明らかにすることができます。`-Zmir-opt-level=4` は、デフォルトでは実行されない MIR 最適化パスをオンにし、興味深いバグを明らかにする可能性があります。`-Zvalidate-mir` は、そのようなバグを明らかにするのに役立ちます。

自分でビルドしたコンパイラをファジングしている場合は、1秒あたりの実行回数を増やすために、`-C target-cpu=native` または PGO/BOLT でビルドすることをお勧めします。もちろん、複数のビルド構成を試して、実際にどれが優れたスループットをもたらすかを確認することが最善です。

追加のバグを見つけるために、デバッグアサーションを有効にして rustc をソースからビルドすることをお勧めしますが、これはトレードオフです：すべての実行に余分な作業が必要になるため、ファジングを遅くする可能性があります。デバッグアサーションを有効にするには、rustc をコンパイルするときに `bootstrap.toml` に以下を追加します：

```toml
[rust]
debug-assertions = true
```

再現にデバッグアサーションが必要な ICE は、[`requires-debug-assertions`][requires-debug-assertions] タグを付ける必要があります。

[requires-debug-assertions]: https://github.com/rust-lang/rust/labels/requires-debug-assertions

## 既存のプロジェクト

- [fuzz-rustc][fuzz-rustc] は、libfuzzer で rustc をファジングする方法を示しています
- [icemaker][icemaker] は、ICE をキャッチするために、さまざまなフラグで多数のソースファイルに対して rustc や他のツールを実行します
- [tree-splicer][tree-splicer] は、正しい構文を維持しながら既存のファイルを組み合わせることで、新しいソースファイルを生成します

[fuzz-rustc]: https://github.com/dwrensha/fuzz-rustc
[icemaker]: https://github.com/matthiaskrgr/icemaker/
[tree-splicer]: https://github.com/langston-barrett/tree-splicer/
