## 重要な変更と特異性

以下のアイテムのいくつかは既に別に言及されていますが、このページは
古いトレイトシステム実装からの主な変更を追跡します。また、ソルバーが理想化された実装から大きく乖離している方法のいくつかにも言及します。このドキュメントは単純化され、エッジケースを無視しています。各ステートメントに暗黙の
「ほとんど」を追加することをお勧めします。

### 正規化

新しいソルバーは、ネストされたゴールを評価する際に[正規化]を使用します。可能性のある複数の候補がある場合、
各候補は熱心に正規化されます。次に、
それらの正規レスポンスをマージしようとします。これは、トレイトシステム内で正規化を使用しない古い実装とは異なります。

これは、両方のソルバーの設計に大きな影響を与えます。正規化を使用して候補の制約を保存しない場合、候補選択は
各候補の制約を破棄し、選択後に候補を再評価することによってのみ制約を適用する必要があります：[ソース][evaluate_stack]。
正規化がないと、ゴールの評価から推論制約を
キャッシュすることもできません。これにより、古い実装には2つのシステムが存在します：
*evaluate*と*fulfill*。*評価*はキャッシュされ、推論制約を適用せず、候補を選択する際に使用されます。*Fulfillment*は推論と領域
制約を適用し、キャッシュされず、推論制約を適用します。

正規化を使用することで、新しい実装は*評価*と
*fulfillment*をマージでき、複雑さと動作の微妙な違いを回避できます。これは、
キャッシングを大幅に簡素化し、追跡されていない情報に誤って依存することを防ぎます。
選択後に候補を再評価することを避け、複数の候補の
レスポンスをマージできるようにします。ただし、評価中にゴールを正規化すると
、新しい実装は、トレイト解決中にサイクルに遭遇した際に不動点アルゴリズムを使用することを余儀なくされます：[ソース][cycle-fixpoint]。

[正規化]: ./canonicalization.md
[evaluate_stack]: https://github.com/rust-lang/rust/blob/47dd709bedda8127e8daec33327e0a9d0cdae845/compiler/rustc_trait_selection/src/traits/select/mod.rs#L1232-L1237
[cycle-fixpoint]: https://github.com/rust-lang/rust/blob/df8ac8f1d74cffb96a93ae702d16e224f5b9ee8c/compiler/rustc_trait_selection/src/solve/search_graph.rs#L382-L387

### 遅延エイリアス等価性

新しい実装は、エイリアスを関連付ける際に`AliasRelate`ゴールを発行しますが、
古い実装は代わりにエイリアスを構造的に関連付けます。これにより、
新しいソルバーは、関連するエイリアスを正規化できるまで等価性を停滞させることができます。

古いソルバーの動作は不完全で、熱心な正規化に依存しています。
これは、曖昧なエイリアスを推論変数で置き換えます。これは
境界変数を含むエイリアスには不可能であるため、古い実装は
バインダー内のエイリアスを正しく処理しません。例：[#102048]。詳細については、
[正規化]に関する章を参照してください。

[#102048]: https://github.com/rust-lang/rust/issues/102048

### ネストされたゴールの熱心な評価

新しい実装は、呼び出し側に返す代わりに、ネストされたゴールを熱心に処理します。古い実装は両方を行います。評価では、ネストされた
ゴールは[熱心に処理されます][eval-nested]が、fulfillmentは単に
[それらを後で処理するために返します][fulfill-nested]。

新しい実装は候補選択のためにネストされたゴールを熱心に処理できる必要があるため、常にそうすることで複雑さが軽減されます。また、将来的にはより多くの候補を
マージできるようにする可能性もあります。

[eval-nested]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_trait_selection/src/traits/select/mod.rs#L1271-L1277
[fulfill-nested]: https://github.com/rust-lang/rust/blob/df8ac8f1d74cffb96a93ae702d16e224f5b9ee8c/compiler/rustc_trait_selection/src/traits/fulfill.rs#L708-L712

### ネストされたゴールは不動点に達するまで評価される

新しい実装は、不動点に達するまで常にゴールをループで評価します。
古い実装は、*fulfillment*でのみそうしますが、*評価*ではそうしません。
常にそうすることで推論が強化され、
トレイトソルバーの順序依存性が軽減されます。[trait-system-refactor-initiative#102]を参照してください。

[trait-system-refactor-initiative#102]: https://github.com/rust-lang/trait-system-refactor-initiative/issues/102

### 証明木と診断情報の提供

新しい実装は、診断情報を直接追跡せず、
代わりに関連情報を遅延的に計算するために使用される[証明木][trees]を提供します。これはまだ完全には具体化されておらず、やや粗雑です。
目標は、ハッピーパスでこの情報を追跡することを避けてパフォーマンスを向上させ、診断データに誤って依存して動作することを避けることです。

[trees]: ./proof-trees.md

## 新しい実装の主要な特異性

### 環境候補がある場合はimplを隠す

`Trait`ゴールを証明するために少なくとも1つの`ParamEnv`または`AliasBound`候補がある場合、
`Trait`と`Projection`ゴールの両方に対してすべてのimpl候補を破棄します：[ソース][discard-from-env]。これにより、ユーザーが`where`境界によって完全にカバーされるimplを使用することを防ぎ、古い実装の動作と一致させ、いくつかの奇妙なエラーを回避します。
例：[trait-system-refactor-initiative#76]。

[discard-from-env]: https://github.com/rust-lang/rust/blob/03994e498df79aa1f97f7bbcfd52d57c8e865049/compiler/rustc_trait_selection/src/solve/assembly/mod.rs#L785-L789
[trait-system-refactor-initiative#76]: https://github.com/rust-lang/trait-system-refactor-initiative/issues/76

### `NormalizesTo`ゴールは関数である

[正規化]の章を参照してください。`NormalizesTo`ゴールを計算する前に、期待されるタームを制約されていない
推論変数で置き換えて、正規化に影響を与えないようにします。これは、`NormalizesTo`ゴールが他のすべてのゴールの種類とはやや異なる方法で処理され、追加のソルバーサポートが必要であることを意味します。最も顕著なのは、
それらの曖昧なネストされたゴールが呼び出し側に返され、呼び出し側がそれらを評価することです。
詳細については[#122687]を参照してください。

[#122687]: https://github.com/rust-lang/rust/pull/122687
