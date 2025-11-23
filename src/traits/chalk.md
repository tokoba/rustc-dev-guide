# Chalk ベースのトレイト解決

[Chalk][chalk] は Rust の実験的なトレイトソルバーであり、
（<!-- date-check --> 2022年5月現在）[Types team] によって開発中です。
その目標は、実装が困難な多くのトレイトシステム機能とバグ修正を
可能にすることです（例：GAT や特殊化）。新しいソルバーのハッキングを
手伝いたい場合は、rust-lang Zulip の [`#t-types`] チャンネルに立ち寄って
挨拶してください！

[Types team]: https://github.com/rust-lang/types-team
[`#t-types`]: https://rust-lang.zulipchat.com/#narrow/stream/144729-t-types

新しいスタイルのトレイトソルバーは、[chalk] で行われた作業に基づいています。
Chalk は、Rust のトレイトシステムを論理プログラミングの観点から
明示的に再構成します。これは、Rust コードを一種の論理プログラムに
「降下」させ、そのプログラムに対してクエリを実行できるようにすることで
行われます。

ここでの重要な観察は、Rust のトレイトシステムは基本的に一種の論理であり、
標準的な論理推論規則にマッピングできるということです。次に、例えば
[Prolog] ソルバーがどのように動作するかと非常に似た方法で、
これらの推論規則の解決策を探すことができます。*完全に* Prolog 規則
（ホーン節とも呼ばれる）を使用することはできませんが、やや表現力の高い
バリアントが必要であることがわかります。

[Prolog]: https://en.wikipedia.org/wiki/Prolog

chalk 自体について詳しくは、
[Chalk book](https://rust-lang.github.io/chalk/book/) セクションをご覧ください。

## 進行中の作業
新しいスタイルのトレイト解決の設計は、2つの場所で行われています：

**chalk**。[chalk] リポジトリは、トレイトシステムの新しいアイデアと
設計を実験する場所です。

**rustc**。論理規則に満足したら、rustc でそれらを実装することに進みます。
rustc の降下モジュールで、構造体、トレイト、impl 宣言を論理推論規則に
マッピングします。

[chalk]: https://github.com/rust-lang/chalk
[rustc_traits]: https://github.com/rust-lang/rust/tree/HEAD/compiler/rustc_traits
