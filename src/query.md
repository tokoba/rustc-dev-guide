# クエリ: デマンド駆動型コンパイル

[コンパイラの概要]で説明されているように、Rustコンパイラは
(<!-- date-check --> 2021年7月時点でも)従来の「パスベース」のセットアップから
「デマンド駆動型」システムへの移行を続けています。コンパイラクエリ
システムは、rustcのデマンド駆動型組織の鍵です。
アイデアは非常にシンプルです。完全に独立したパス
(パース、型チェックなど)の代わりに、関数のような*クエリ*のセットが
入力ソースに関する情報を計算します。たとえば、
何らかのアイテムの[`DefId`]が与えられると、そのアイテムの型を計算して
返す`type_of`というクエリがあります。

[`DefId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/def_id/struct.DefId.html
[コンパイラの概要]: overview.md#queries

クエリの実行は*メモ化*されます。クエリを初めて呼び出すと、
計算を実行しますが、次回はハッシュテーブルから結果が返されます。さらに、クエリの実行は
*インクリメンタルコンピューテーション*にうまく適合します。アイデアは大まかに、クエリを呼び出すと、
結果がディスクから保存されたデータをロードすることによって返される*可能性がある*ということです。[^incr-comp-detail]

最終的には、コンパイラ全体の
制御フローをクエリ駆動型にしたいと考えています。実質的に、クレートでコンパイルを実行する
1つのトップレベルクエリ(`compile`)があり、これが順番にそのクレートに関する情報を要求し、
*最後*から開始します。例えば:

- `compile`クエリは、コード生成ユニットのリストを取得するように要求する可能性があります
  (つまり、LLVMでコンパイルする必要があるモジュール)。
- ただし、コード生成ユニットのリストを計算すると、Rustソースで定義されている
  すべてのモジュールのリストを返すサブクエリが呼び出されます。
- そのクエリは順番にHIRを要求する何かを呼び出します。
- これは、実際にパースを実行するまでさらに遡り続けます。

このビジョンは完全には実現されていませんが、コンパイラの大部分
(たとえば、[MIR]の生成)は現在、まさにこのように機能しています。

[^incr-comp-detail]: [インクリメンタルコンパイルの詳細]の章では、クエリとは何か、
どのように機能するかについて、より詳細な説明を提供しています。
独自のクエリを作成する場合は、これを読むことをお勧めします。

[インクリメンタルコンパイルの詳細]: queries/incremental-compilation-in-detail.md
[MIR]: mir/index.md

## クエリの呼び出し

クエリの呼び出しは簡単です。[`TyCtxt`](「型コンテキスト」)構造体は、定義された各クエリのメソッドを提供します。
たとえば、`type_of`クエリを呼び出すには、次のようにします:

```rust,ignore
let ty = tcx.type_of(some_def_id);
```

## コンパイラがクエリを実行する方法

では、クエリメソッドを呼び出すと何が起こるのか疑問に思うかもしれません。
答えは、各クエリについて、コンパイラがキャッシュを維持するということです -- クエリが
すでに実行されている場合、答えは簡単です: キャッシュから戻り値をクローンして返します
(したがって、クエリの戻り値の型が安価にクローン可能であることを確認する必要があります。
必要に応じて`Rc`を挿入してください)。

### プロバイダー

ただし、クエリがキャッシュに*ない*場合、コンパイラは対応する**プロバイダー**関数を呼び出します。
プロバイダーは特定のモジュールで実装され、コンパイラの初期化中に
[`Providers`][providers_struct]構造体に**手動で登録**される関数です。
マクロシステムは[`Providers`][providers_struct]構造体を生成します。
これは、すべてのクエリ実装の関数テーブルとして機能し、各フィールドは実際のプロバイダーへの関数ポインターです。

**注意:** `Providers`構造体はマクロによって生成され、すべてのクエリ実装の関数テーブルとして機能します。
これはRustトレイトでは**なく**、関数ポインターフィールドを持つプレーンな構造体です。

**プロバイダーはクレートごとに定義されます。** コンパイラは内部的に、
少なくとも概念的には、すべてのクレートのプロバイダーのテーブルを維持します。
現在、実際には2つのセットがあります: **ローカルクレート**(つまり、コンパイル中のもの)に関する
クエリのプロバイダーと、**外部クレート**(つまり、ローカルクレートの依存関係)に関する
クエリのプロバイダーです。クエリがターゲットとするクレートを決定するのは、
クエリの*種類*ではなく、*キー*であることに注意してください。
たとえば、`tcx.type_of(def_id)`を呼び出すと、`def_id`が参照している
クレートに応じて、ローカルクエリまたは外部クエリになる可能性があります
(これがどのように機能するかの詳細については、[`self::keys::Key`][Key]トレートを参照してください)。

プロバイダーは常に同じシグネチャを持ちます:

```rust,ignore
fn provider<'tcx>(
    tcx: TyCtxt<'tcx>,
    key: QUERY_KEY,
) -> QUERY_RESULT {
    ...
}
```

プロバイダーは2つの引数を取ります: `tcx`とクエリキー。
クエリの結果を返します。

注意: ほとんどの`rustc_*`クレートは**ローカル
プロバイダー**のみを提供します。ほとんどすべての**外部プロバイダー**は
[`rustc_metadata`クレート][rustc_metadata]を経由し、クレートメタデータから情報をロードします。
ただし、*ローカルと外部の両方*のクレートのクエリを提供するクレートもあります。
その場合、`provide`と`provide_extern`関数の両方を定義し、
[`wasm_import_module_map`][wasm_import_module_map]を通じて`rustc_driver`が呼び出すことができます。

[rustc_metadata]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_metadata/index.html
[wasm_import_module_map]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/back/symbol_export/fn.wasm_import_module_map.html

### プロバイダーのセットアップ方法

tcxが作成されると、作成者は[`Providers`][providers_struct]構造体を使用してプロバイダーを提供します。
この構造体はここのマクロによって生成されますが、基本的には関数ポインターの大きなリストです:

[providers_struct]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/query/struct.Providers.html

```rust,ignore
struct Providers {
    type_of: for<'tcx> fn(TyCtxt<'tcx>, DefId) -> Ty<'tcx>,
    // ... 各クエリに1つのフィールド
}
```

#### プロバイダーはどのように登録されますか?

`Providers`構造体は、主に`rustc_driver`クレートによって、コンパイラの初期化中に入力されます。
ただし、実際のプロバイダー関数は、様々な`rustc_*`クレート(`rustc_middle`、`rustc_hir_analysis`など)に実装されています。

プロバイダーを登録するために、各クレートは次のような[`provide`][provide_fn]関数を公開します:

[provide_fn]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/hir/fn.provide.html

```rust,ignore
pub fn provide(providers: &mut Providers) {
    *providers = Providers {
        type_of,
        // ... ここにさらにプロバイダーを追加
        ..*providers
    };
}
```

- この関数は`Providers`構造体への可変参照を受け取り、正しいプロバイダー関数を指すようにフィールドを設定します。
- `providers.type_of = type_of;`のように、フィールドを個別に割り当てることもできます。

#### 新しいプロバイダーの追加

`fubar`という新しいクエリを追加したいとします。次のようにします:

1. プロバイダー関数を実装します:

    ```rust,ignore
    fn fubar<'tcx>(tcx: TyCtxt<'tcx>, key: DefId) -> Fubar<'tcx> { ... }
    ```

2. `provide`関数に登録します:

    ```rust,ignore
    pub fn provide(providers: &mut Providers) {
        *providers = Providers {
            fubar,
            ..*providers
        };
    }
    ```

---

## 新しいクエリの追加

新しいクエリをどのように追加しますか?
クエリの定義は2つのステップで行われます:

1. クエリ名、その引数、説明を宣言します。
2. 必要に応じてクエリプロバイダーを提供します。

クエリ名と引数を宣言するには、[`compiler/rustc_middle/src/query/mod.rs`][query-mod]の
大きなマクロ呼び出しにエントリを追加するだけです。次に、いくつかの_内部_説明を含む
ドキュメントコメントを追加する必要があります。次に、クエリの_ユーザー向け_説明を含む
`desc`属性を提供します。`desc`属性はクエリサイクルでユーザーに表示されます。

これは次のようになります:

[query-mod]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/query/index.html

```rust,ignore
rustc_queries! {
    /// すべてのアイテムの型を記録します。
    query type_of(key: DefId) -> Ty<'tcx> {
        cache_on_disk_if { key.is_local() }
        desc { |tcx| "computing the type of `{}`", tcx.def_path_str(key) }
    }
    ...
}
```

クエリ定義は次のような形式です:

```rust,ignore
query type_of(key: DefId) -> Ty<'tcx> { ... }
^^^^^ ^^^^^^^      ^^^^^     ^^^^^^^^   ^^^
|     |            |         |          |
|     |            |         |          クエリ修飾子
|     |            |         結果の型
|     |            クエリキーの型
|     クエリの名前
queryキーワード
```

これらの要素を1つずつ見ていきましょう:

- **Queryキーワード:** クエリ定義の開始を示します。
- **クエリの名前:** クエリメソッドの名前
  (`tcx.type_of(..)`)。また、このクエリを表すために生成される構造体
  (`ty::queries::type_of`)の名前としても使用されます。
- **クエリキーの型:** このクエリの引数の型。
  この型は[`ty::query::keys::Key`][Key]トレートを実装する必要があります。
  これは(たとえば)それをクレートにマッピングする方法などを定義します。
- **クエリの結果の型:** このクエリによって生成される型。この型は
  (a)`RefCell`または他の内部可変性を使用せず、(b)安価にクローン可能である必要があります。
  重要でないデータ型には、インターンまたは`Rc`や`Arc`の使用が推奨されます。[^steal]
- **クエリ修飾子:** クエリの処理方法をカスタマイズする様々なフラグとオプション
  (主に[インクリメンタルコンパイル][incrcomp]に関して)。

[Key]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/query/keys/trait.Key.html
[incrcomp]: queries/incremental-compilation-in-detail.html#query-modifiers

したがって、クエリを追加するには:

- 上記の形式を使用して`rustc_queries!`にエントリを追加します。
- 適切な`provide`メソッドを変更してプロバイダーをリンクします。
  必要に応じて新しいものを追加し、`rustc_driver`が呼び出していることを確認します。

[^steal]: これらのルールの唯一の例外は`ty::steal::Steal`型で、
MIRをその場で安価に変更するために使用されます。詳細については`Steal`の定義を参照してください。
`Steal`の新しい使用は、`@rust-lang/compiler`に警告することなく追加**しないでください**。

## 外部リンク

関連する設計アイデアと追跡の問題:

- 設計ドキュメント: [On-demand Rustc incremental design doc]
- 追跡の問題: [コンパイラにおける「Red/Green」依存関係追跡]

さらなる議論と問題:

- [GitHub issue #42633]
- [Incremental Compilation Beta]
- [Incremental Compilation Announcement]

[On-demand Rustc incremental design doc]: https://github.com/nikomatsakis/rustc-on-demand-incremental-design-doc/blob/master/0000-rustc-on-demand-and-incremental.md
[コンパイラにおける「Red/Green」依存関係追跡]: https://github.com/rust-lang/rust/issues/42293
[GitHub issue #42633]: https://github.com/rust-lang/rust/issues/42633
[Incremental Compilation Beta]: https://internals.rust-lang.org/t/incremental-compilation-beta/4721
[Incremental Compilation Announcement]: https://blog.rust-lang.org/2016/09/08/incremental.html
