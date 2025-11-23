# 型付け/パラメータ環境

## 型付け環境

型システムと相互作用する際、結果に影響を与える可能性のあるいくつかの変数を考慮する必要があります。スコープ内のwhere句のセットと、どのフェーズのコンパイラ型システム操作が実行されているか（それぞれ[`ParamEnv`][penv]および[`TypingMode`][tmode]構造体）です。

型システム操作を実行する環境がまだ作成されていない場合、[`TypingEnv`][tenv]を使用して、必要なすべての外部コンテキストを単一の型にバンドルできます。

型システム操作を実行するコンテキストが作成されると（例：[`ObligationCtxt`][ocx]または[`FnCtxt`][fnctxt]）、通常`TypingEnv`はどこにも保存されません。`TypingMode`のみが環境全体のプロパティであり、異なる`ParamEnv`をゴールごとに使用できるためです。

[ocx]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/traits/struct.ObligationCtxt.html
[fnctxt]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/fn_ctxt/struct.FnCtxt.html

## パラメータ環境

### `ParamEnv`とは何か

[`ParamEnv`][penv]は、スコープ内のwhere句のリストです。通常、特定のアイテムのwhere句に対応します。一部の句は明示的に書かれているわけではなく、[`predicates_of`][predicates_of]クエリで暗黙的に追加されます。例えば、`ConstArgHasType`や（一部の）implied boundsなどです。

ほとんどの場合、`ParamEnv`は、提供されたアイテムのwhere句から派生した`ParamEnv`を返す[`param_env`クエリ][query]を介して最初に作成されます。`ParamEnv`は、特定のアイテムから派生しない任意の句のセットで作成することもできます。例えば、[`compare_method_predicate_entailment`][method_pred_entailment]では、impl のwhere句とトレイト定義の関数のwhere句で構成されるハイブリッド`ParamEnv`を作成します。

---

次のような関数がある場合：

```rust
// `foo`は次の`ParamEnv`を持ちます：
// `[T: Sized, T: Trait, <T as Trait>::Assoc: Clone]`
fn foo<T: Trait>()
where
    <T as Trait>::Assoc: Clone,
{}
```

概念的に`foo`の内部にいる場合（たとえば、型チェックやリントの際）、型システムと相互作用するすべての場所でこの`ParamEnv`を使用します。これにより、[正規化]、ジェネリック定数の評価、where句/ゴールの証明などが、`T`がサイズ指定され、`Trait`を実装していることなどに依存できるようになります。

より具体的な例：

```rust
// `foo`は次の`ParamEnv`を持ちます：
// `[T: Sized, T: Clone]`
fn foo<T: Clone>(a: T) {
    // `foo`を型チェックする際、`requires_clone`上のすべてのwhere句を
    // 保持するために呼び出すことが合法であることを確認する必要があります。
    // つまり、`T: Clone`を証明する必要があります。`foo`を型チェックしているので、
    // `T: Clone`が保持しているかどうかをチェックしようとする際に`foo`の
    // 環境を使用します。
    //
    // `ParamEnv` `[T: Sized, T: Clone]`で`T: Clone`を証明しようとすると、
    // 証明したい境界が環境にあるため、些細に成功します。
    requires_clone(a);
}
```

または、コンパイルされない例：

```rust
// `foo2`は次の`ParamEnv`を持ちます：
// `[T: Sized]`
fn foo2<T>(a: T) {
    // foo2を型チェックする際、`T: Clone`を証明しようとします。
    // foo2を型チェックしているので、`T: Clone`を証明しようとする際に
    // foo2の環境を使用します。
    //
    // `ParamEnv` `[T: Sized]`で`T: Clone`を証明しようとすると、
    // 環境にトレイトソルバーに`T`が`Clone`を実装していることを伝える
    // ものがなく、適用できるユーザーが書いたimplも存在しないため、失敗します。
    requires_clone(a);
}
```

[predicates_of]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_analysis/collect/predicates_of/fn.predicates_of.html
[method_pred_entailment]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_analysis/check/compare_impl_item/fn.compare_method_predicate_entailment.html
[query]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.param_env
[正規化]: normalization.md

### `ParamEnv`の取得

型システムと相互作用する際に間違った[`ParamEnv`][penv]を使用すると、ICE、不正な形式のプログラムのコンパイル、またはすべきでないときのエラーにつながる可能性があります。[#82159](https://github.com/rust-lang/rust/pull/82159)および[#82067](https://github.com/rust-lang/rust/pull/82067)を、コンパイラが正しいparam envを使用するように変更し、その過程でICEを修正したPRの例として参照してください。

大多数の場合、`ParamEnv`が必要なとき、それはすでにスコープ内のどこかに存在するか、呼び出しスタックの上位にあり、渡されるべきです。既存の`ParamEnv`を見つけられる場所の網羅的ではないリスト：

- 型チェック中、`FnCtxt`には[`param_env`フィールド][fnctxt_param_env]があります
- late lintsを書くとき、`LateContext`には[`param_env`フィールド][latectxt_param_env]があります
- well formednessチェック中、`WfCheckingCtxt`には[`param_env`フィールド][wfckctxt_param_env]があります
- MIR Typeckに使用される`TypeChecker`には[`param_env`フィールド][mirtypeck_param_env]があります
- next-gen trait solverでは、すべての`Goal`には、ゴールを証明する環境を指定する[`param_env`フィールド][goal_param_env]があります
- 既存の[`TypeRelation`][typerelation]を編集する際、[`PredicateEmittingRelation`][predicate_emitting_relation]を実装している場合、[`param_env`メソッド][typerelation_param_env]が利用可能になります。

スコープ内のどこかに使用できる`ParamEnv`があるかどうかわからない場合は、[`#t-compiler/help`][compiler_help] Zulipチャネルでスレッドを開くことをお勧めします。誰かが`ParamEnv`を取得できる場所を指摘できる可能性があります。

`ParamEnv`を手動で構築することは、通常、何らかのトップレベル分析の開始時にのみ必要です（例：hir typeckまたは借用チェック）。そのような場合、3つの方法があります：

- 特定の定義に関連付けられた環境を返す[`tcx.param_env(def_id)`クエリ][param_env_query]を呼び出す。
- [`ParamEnv::empty`][env_empty]で空の環境を作成する。
- [`ParamEnv::new`][param_env_new]を使用して任意のwhere句のセットで環境を構築する。次に、[`traits::normalize_param_env_or_error`][normalize_env_or_error]を呼び出して、環境内のすべてのwhere句を正規化およびelaborateする処理を行います。

`param_env`クエリを使用することが、`ParamEnv`を構築する最も一般的な方法です。ほとんどの場合、コンパイラは特定の定義の一部として分析を実行しているためです。

`ParamEnv::empty`で空の環境を作成することは、通常コードジェン（[`TypingEnv::fully_monomorphized`][tenv_mono]経由で間接的に）でのみ行われるか、ジェネリックパラメータに遭遇することを期待しない一部の分析の一部として行われます（例：coherence/orphan checkのさまざまな部分）。

任意のwhere句のセットから環境を作成することは通常不要であり、必要な環境がソースコードの実際のアイテムに対応しない場合にのみ行う必要があります（例：[`compare_method_predicate_entailment`][method_pred_entailment]）。

[param_env_new]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.ParamEnv.html#method.new
[normalize_env_or_error]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/traits/fn.normalize_param_env_or_error.html
[fnctxt_param_env]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/fn_ctxt/struct.FnCtxt.html#structfield.param_env
[latectxt_param_env]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/context/struct.LateContext.html#structfield.param_env
[wfckctxt_param_env]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_analysis/check/wfcheck/struct.WfCheckingCtxt.html#structfield.param_env
[goal_param_env]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_infer/infer/canonical/ir/solve/struct.Goal.html#structfield.param_env
[typerelation_param_env]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_infer/infer/trait.PredicateEmittingRelation.html#tymethod.param_env
[typerelation]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/relate/trait.TypeRelation.html
[mirtypeck_param_env]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/type_check/struct.TypeChecker.html#structfield.param_env
[env_empty]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.ParamEnv.html#method.empty
[param_env_query]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/fn_ctxt/struct.FnCtxt.html#structfield.param_env
[predicate_emitting_relation]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/relate/combine/trait.PredicateEmittingRelation.html
[tenv_mono]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TypingEnv.html#method.fully_monomorphized
[compiler_help]: https://rust-lang.zulipchat.com/#narrow/channel/182449-t-compiler.2Fhelp

### `ParamEnv`の構築方法

[`ParamEnv`][pe]の作成は、ユーザーが書いたアイテムで定義されたwhere句のリストを単に使用するよりも複雑です。envにスーパートレイトをelaborateし、すべてのエイリアスを完全に正規化する必要があります。このロジックは[`traits::normalize_param_env_or_error`][normalize_env_or_error]によって処理されます（elaborationについては何も言及していませんが）。

#### スーパートレイトのelaborate

`fn foo<T: Copy>()`のような関数がある場合、`Copy`トレイトには`Clone`スーパートレイトがあるため、関数内で`T: Clone`を証明できるようにしたいです。`ParamEnv`の構築は、env内のすべてのトレイト境界を調べ、トレイトで見つかったスーパートレイトの新しいwhere句を明示的に`ParamEnv`に追加します。

具体的な例：

```rust
trait Trait: SuperTrait {}
trait SuperTrait: SuperSuperTrait {}

// `bar`のelaborateされていない`ParamEnv`は次のようになります：
// `[T: Sized, T: Copy, T: Trait]`
fn bar<T: Copy + Trait>(a: T) {
    requires_impl(a);
}

fn requires_impl<T: Clone + SuperSuperTrait>(a: T) {}
```

envをelaborateしなかった場合、`requires_impl`呼び出しは型チェックに失敗します。`T: Clone`または`T: SuperSuperTrait`を証明できないためです。実際には、envをelaborateするため、`bar`の`ParamEnv`は実際には次のようになります：
`[T: Sized, T: Copy, T: Clone, T: Trait, T: SuperTrait, T: SuperSuperTrait]`
これにより、`bar`を型チェックする際に`T: Clone`と`T: SuperSuperTrait`を証明できます。

`Clone`トレイトには`Sized`スーパートレイトがありますが、envには2つの`T: Sized`境界（スーパートレイト用と暗黙的に追加された`T: Sized`境界用）はありません。elaborateプロセス（[`util::elaborate`][elaborate]経由で実装）がwhere句を重複排除するためです。

この副作用として、実際にスーパートレイトのelaborationが行われなくても、env内の既存のwhere句も重複排除されます。次の例を参照してください：

```rust
trait Trait {}
// elaborateされていない`ParamEnv`は次のようになります：
// `[T: Sized, T: Trait, T: Trait]`
// しかし、elaboration後は次のようになります：
// `[T: Sized, T: Trait]`
fn foo<T: Trait + Trait>() {}
```

[next-gen trait solver][next-gen-solver]もこのelaborationが行われることを要求します。

[elaborate]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_infer/traits/util/fn.elaborate.html
[next-gen-solver]: ./solve/trait-solving.md

#### すべての境界の正規化

古いトレイトソルバーでは、`ParamEnv`に保存されたwhere句は完全に正規化される必要があります。そうでないと、トレイトソルバーが正しく機能しません。`ParamEnv`を正規化する必要がある具体的な例：

```rust
trait Trait<T> {
    type Assoc;
}

trait Other {
    type Bar;
}

impl<T> Other for T {
    type Bar = u32;
}

// `foo`の正規化されていない`ParamEnv`は次のようになります：
// `[T: Sized, U: Sized, U: Trait<T::Bar>]`
fn foo<T, U>(a: U)
where
    U: Trait<<T as Other>::Bar>,
{
    requires_impl(a);
}

fn requires_impl<U: Trait<u32>>(_: U) {}
```

人間として、`<T as Other>::Bar`は`u32`と等しいことがわかるので、`U`のトレイト境界は`U: Trait<u32>`と同等です。実際には、古いソルバーでこの環境で`U: Trait<u32>`を証明しようとすると、`<T as Other>::Bar`が`u32`と等しいことを判断できないため失敗します。

これを回避するために、`ParamEnv`を構築した後に正規化します。そのため、`foo`の`ParamEnv`は実際には`[T: Sized, U: Sized, U: Trait<u32>]`となり、トレイトソルバーが`ParamEnv`内の`U: Trait<u32>`を使用してトレイト境界`U: Trait<u32>`が保持していることを判断できるようになります。

この回避策はすべてのケースで機能するわけではありません。associated typesを正規化するには`ParamEnv`が必要であり、これがブートストラップ問題を引き起こします。正規化された`ParamEnv`が必要ですが、その`ParamEnv`を得るために正規化する必要があります。現在、正規化されていないparam envを使用して`ParamEnv`を一度正規化しており、これが実際には問題のないケースが多いですが、これが壊れる例もあります（[example]）。

次世代トレイトソルバーでは、`ParamEnv`内のすべてのwhere句が完全に正規化されている必要はなく、`ParamEnv`を構築する際に正規化しません。

[example]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e6933265ea3e84eaa47019465739992c
[pe]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.ParamEnv.html

## 型付けモード

型システム操作を実行するコンテキストによって、異なる動作が必要になる場合があります。たとえば、coherenceでは、ゴールが保持しないと見なすことができる場合や、型が不等であると見なすことができる場合について、より厳しい要件があります。

型システム操作が実行されているコンパイラの「フェーズ」を追跡することは、[`TypingMode`][tmode]列挙型によって行われます。`TypingMode`列挙型のドキュメントは非常に優れているので、ここで逐語的に繰り返すのではなく、APIドキュメントを直接読むことをお勧めします。

[penv]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.ParamEnv.html
[tenv]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TypingEnv.html
[tmode]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/type.TypingMode.html
