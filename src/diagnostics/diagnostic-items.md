# 診断アイテム

リントを書く際、特定の型、トレイト、関数をチェックすることは一般的です。
これにより、これらをチェックする方法の問題が生じます。型は完全な型パスで
チェックできます。しかし、これにはパスをハードコードする必要があり、
いくつかのエッジケースで誤分類につながる可能性があります。これに対抗するため、
rustcは診断アイテムを導入しました。これは[`Symbol`]sを介して型を識別するために使用されます。

## 診断アイテムの検索

診断アイテムは、`rustc_diagnostic_item`属性を使用して、`rustc`/`std`/`core`/`alloc`内の
アイテムに追加されます。特定の型のアイテムは、ドキュメント内のソースコードを開いて
この属性を探すことで見つけることができます。テスト中のコンパイルエラーを避けるために、
`cfg_attr`属性と一緒に追加されることが多いことに注意してください。定義は多くの場合、
次のようになります：

```rs
// これはこの型の診断アイテムです   vvvvvvv
#[cfg_attr(not(test), rustc_diagnostic_item = "Penguin")]
struct Penguin;
```

診断アイテムは通常、トレイト、型、およびスタンドアロン関数にのみ追加されます。
関連型やメソッドをチェックすることが目標の場合は、
アイテムの診断アイテムを使用し、
[*診断アイテムの使用*](#using-diagnostic-items)を参照してください。

## 診断アイテムの追加

新しい診断アイテムは、次の2つのステップで追加できます：

1. Rustリポジトリ内でターゲットアイテムを見つけます。次に、`rustc_diagnostic_item`属性を介して
   文字列として診断アイテムを追加します。これにより、テストの実行中にコンパイルエラーが
   発生する場合があります。これらのエラーは、`not(test)`条件で`cfg_attr`属性を使用することで
   回避できます（予防的な方法として、すべての`rustc_diagnostic_item`属性に対して追加しても
   問題ありません）。最終的には、次のようになります：

    ```rs
    // これが新しい診断アイテムになります        vvv
    #[cfg_attr(not(test), rustc_diagnostic_item = "Cat")]
    struct Cat;
    ```

    診断アイテムの命名規則については、
    [*命名規則*](#naming-conventions)を参照してください。

2. <!-- date-check: Feb 2023 -->
   コード内の診断アイテムは、[`rustc_span::symbol::sym`]のシンボルを介して
   アクセスされます。新しく作成した診断アイテムを追加するには、
   モジュールファイルを開いて、
   リストの正しい位置に名前（この場合は`Cat`）を追加するだけです。

これで、変更を含むプルリクエストを作成できます。:tada:

> 注意：
> Clippyのような他のプロジェクトで診断アイテムを使用する場合、
> リポジトリが同期されるまでに時間がかかる場合があります。

## 命名規則

診断アイテムにはまだ命名規則がありません。
以下は、将来使用すべきいくつかのガイドラインですが、
既存の名前とは異なる場合があります：

* 型、トレイト、列挙型は、UpperCamelCaseを使用して命名されます
  （例：`Iterator`と`HashMap`）
* `Writer`のように複数回使用される型名の場合、
  より正確な名前を選択することをお勧めします。
  おそらくモジュールを追加することで
  （例：`IoWriter`）
* 関連アイテムには独自の診断アイテムを取得すべきではなく、
  代わりに、それらが起源とする型の診断アイテムによって
  間接的にアクセスされるべきです。
* `std::mem::swap()`のようなフリースタンディング関数は、
  1つの重要な（エクスポート）モジュールをプレフィックスとして使用して
  `snake_case`を使用して命名されるべきです
  （例：`mem_swap`と`cmp_max`）
* モジュールには通常、診断アイテムを添付すべきではありません。
  診断アイテムは、パスの使用を避けるために追加されたため、
  モジュールでそれらを使用することは、おそらく逆効果です。

## 診断アイテムの使用

rustc内では、診断アイテムは[`rustc_span::symbol::sym`]モジュール内の
[`Symbol`]sを介して検索されます。これらは、[`TyCtxt::get_diagnostic_item()`]を使用して
[`DefId`]sにマッピングするか、[`TyCtxt::is_diagnostic_item()`]を使用して
[`DefId`]に一致するかどうかをチェックできます。診断アイテムから[`DefId`]にマッピングする場合、
メソッドは`Option<DefId>`を返します。これは、シンボルが診断アイテムでない場合、
または例えば`#[no_std]`でコンパイルする場合に型が登録されていない場合、`None`になる可能性があります。
以下のすべての例は、[`DefId`]sとその使用に基づいています。

### 例：型のチェック

```rust
use rustc_span::symbol::sym;

/// この例は、与えられた型（`ty`）が`TyCtxt::is_diagnostic_item()`を使用して
/// `HashMap`型を持っているかどうかをチェックします
fn example_1(cx: &LateContext<'_>, ty: Ty<'_>) -> bool {
    match ty.kind() {
        ty::Adt(adt, _) => cx.tcx.is_diagnostic_item(sym::HashMap, adt.did()),
        _ => false,
    }
}
```

### 例：トレイト実装のチェック

```rust
/// この例は、与えられたメソッドの[`DefId`]が診断アイテムによって定義された
/// トレイト実装の一部であるかどうかをチェックします。
fn is_diag_trait_item(
    cx: &LateContext<'_>,
    def_id: DefId,
    diag_item: Symbol
) -> bool {
    if let Some(trait_did) = cx.tcx.trait_of_item(def_id) {
        return cx.tcx.is_diagnostic_item(diag_item, trait_did);
    }
    false
}
```

### 関連型

診断アイテムの関連型は、最初にトレイトの[`DefId`]を取得し、
次に[`TyCtxt::associated_items()`]を呼び出すことで間接的にアクセスできます。
これは、さらなるチェックに使用できる[`AssocItems`]オブジェクトを返します。
この使用例については、[`clippy_utils::ty::get_iterator_item_ty()`]を参照してください。

### Clippyでの使用

Clippyは、可能な限り診断アイテムを使用しようとし、いくつかの
ラッパーおよびユーティリティ関数を開発しました。Clippyで診断アイテムを使用する際は、
そのドキュメントも参照してください。（[*リント作成のための共通ツール*][clippy-Common-tools-for-writing-lints]を参照。）

## 関連するissue

これらは、トピックに本当に深く潜りたい人にとって
おそらく興味深いものです :)

* [rust#60966]: 診断アイテムを導入したRust PR
* [rust-clippy#5393]: ハードコードされたパスから診断アイテムへの移行のための
  Clippyの追跡issue

<!-- Links -->

[`rustc_span::symbol::sym`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/symbol/sym/index.html
[`Symbol`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/symbol/struct.Symbol.html
[`DefId`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/def_id/struct.DefId.html
[`TyCtxt::get_diagnostic_item()`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.get_diagnostic_item
[`TyCtxt::is_diagnostic_item()`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.is_diagnostic_item
[`TyCtxt::associated_items()`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html#method.associated_items
[`AssocItems`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/assoc/struct.AssocItems.html
[`clippy_utils::ty::get_iterator_item_ty()`]: https://github.com/rust-lang/rust-clippy/blob/305177342fbc622c0b3cb148467bab4b9524c934/clippy_utils/src/ty.rs#L55-L72
[clippy-Common-tools-for-writing-lints]: https://doc.rust-lang.org/nightly/clippy/development/common_tools_writing_lints.html
[rust#60966]: https://github.com/rust-lang/rust/pull/60966
[rust-clippy#5393]: https://github.com/rust-lang/rust-clippy/issues/5393
