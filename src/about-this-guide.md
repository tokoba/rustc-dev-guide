# このガイドについて

このガイドは、rustc（Rust コンパイラ）がどのように動作するかを文書化し、
rustc の開発に関わる新しい貢献者を支援することを目的としています。

このガイドにはいくつかのパートがあります：

1. [Building and debugging `rustc`][p1]：
   ビルド、デバッグ、プロファイリングなど、どのような形で貢献する場合でも役立つ情報が含まれています。
1. [Contributing to Rust][p2]：
   貢献の手順、git と Github の使用、機能の安定化など、どのような形で貢献する場合でも役立つ情報が含まれています。
1. [Bootstrapping][p3]：
   Rust コンパイラが以前のバージョンを使用して自身をビルドする方法について説明し、
   ブートストラッププロセスとデバッグ方法の紹介が含まれています。
1. [High-level Compiler Architecture][p4]：
   コンパイラの高レベルアーキテクチャとコンパイルプロセスの段階について説明します。
1. [Source Code Representation][p5]：
   ユーザーからの生のソースコードを取得し、
   コンパイラが簡単に扱える様々な形式に変換するプロセスについて説明します。
1. [Supporting Infrastructure][p6]：
   コマンドライン引数の規約、rustc_driver や rustc_interface などのコンパイラのエントリポイント、
   エラーと lint の設計と実装について説明します。
1. [Analysis][p7]：
   コンパイラがコードの様々な特性をチェックし、
   コンパイルプロセスの後続段階に情報を提供するために使用する解析について説明します（例：型チェック）。
1. [MIR to Binaries][p8]：リンクされた実行可能な機械語コードがどのように生成されるか。
1. 最後に、有用な参照情報を含む[Appendices][p9]があります。
   用語集を含むいくつかの異なる情報があります。

[p1]: ./building/how-to-build-and-run.html
[p2]: ./contributing.md
[p3]: ./building/bootstrapping/intro.md
[p4]: ./part-2-intro.md
[p5]: ./part-3-intro.md
[p6]: ./cli.md
[p7]: ./part-4-intro.md
[p8]: ./part-5-intro.md
[p9]: ./appendix/background.md

### 絶え間ない変化

`rustc` は実際の製品品質のプロダクトであり、
かなりの数の貢献者によって継続的に作業されていることを心に留めておいてください。
そのため、コードベースの変更や技術的負債はかなりあります。
さらに、このガイド全体で議論されているアイデアの多くは、
まだ完全には実現されていない理想的な設計です。
これらすべてにより、このガイドをすべてにおいて完全に最新の状態に保つことは非常に困難です！

ガイド自体ももちろんオープンソースであり、
ソースは [a GitHub repository] でホストされています。
ガイドに間違いを見つけた場合は、issue を報告してください。
さらに良いのは、修正の PR を開くことです！

ガイドに貢献する場合は、
[このガイドのドキュメント作成に関する対応するサブセクション][subsection on writing documentation in this guide]をご覧ください。

[subsection on writing documentation in this guide]: contributing.md#contributing-to-rustc-dev-guide

> 「諸行無常なり——
> これを智慧をもって見るとき、苦しみから離れる。」
> _ダンマパダ、第 277 節_

## その他の情報源

以下のサイトも役に立つかもしれません：

- このガイドには、コンパイラの様々な部分がどのように動作するか、
  またコンパイラに貢献する方法についての情報が含まれています。
- [rustc API docs] -- コンパイラ、開発ツール、内部ツールの rustdoc ドキュメント
- [Forge] -- Rust のインフラストラクチャ、チームの手順などに関するドキュメントが含まれています
- [compiler-team] -- Rust コンパイラチームのホームベースで、チームの手順、
  アクティブなワーキンググループ、チームのカレンダーについての説明があります。
- [std-dev-guide] -- 標準ライブラリの開発に関する同様のガイド。
- [The t-compiler Zulip][z]
- [Rust Internals forum][rif]、質問をしたり Rust の内部について議論する場所
- [Rust reference][rr]、Rust の内部について特に話しているわけではありませんが、
  それでも素晴らしいリソースです
- 古くなっていますが、[Tom Lee の素晴らしいブログ記事][tlgba]は非常に役に立ちます
- [Rust Compiler Testing Docs][rctd]
- [@bors] については、[このチートシート][cheatsheet]が役に立ちます
- プログラミング時には、Google は常に役に立ちます。
  [すべての Rust ドキュメント][gsearchdocs]（標準ライブラリ、
  コンパイラ、書籍、リファレンス、ガイド）を検索して、
  言語とコンパイラに関する情報をすばやく見つけることができます。
- Rustdoc の組み込み検索機能を使用して、
  見ているクレート内の型や関数に関するドキュメントを見つけることもできます。
  型シグネチャでも検索できます！たとえば、`* -> vec` を検索すると、
  `Vec<T>` を返すすべての関数が見つかるはずです。
  _ヒント：_ 任意の Rustdoc ページで `?` を入力すると、
  より多くのヒントとキーボードショートカットを見つけることができます！


[rustc dev guide]: about-this-guide.md
[gsearchdocs]: https://www.google.com/search?q=site:doc.rust-lang.org+your+query+here
[stddocs]: https://doc.rust-lang.org/std
[rif]: http://internals.rust-lang.org
[rr]: https://doc.rust-lang.org/book/
[rustforge]: https://forge.rust-lang.org/
[tlgba]: https://tomlee.co/2014/04/a-more-detailed-tour-of-the-rust-compiler/
[ro]: https://www.rustaceans.org/
[rctd]: tests/intro.md
[cheatsheet]: https://bors.rust-lang.org/
[Miri]: https://github.com/rust-lang/miri
[@bors]: https://github.com/bors
[a GitHub repository]: https://github.com/rust-lang/rustc-dev-guide/
[rustc API docs]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle
[Forge]: https://forge.rust-lang.org/
[compiler-team]: https://github.com/rust-lang/compiler-team/
[std-dev-guide]: https://std-dev-guide.rust-lang.org/
[z]: https://rust-lang.zulipchat.com/#narrow/stream/131828-t-compiler
