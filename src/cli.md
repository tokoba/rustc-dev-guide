# コマンドライン引数

コマンドラインフラグは [rustc book][cli-docs] で文書化されています。すべての*安定した*
フラグはそこで文書化されるべきです。不安定なフラグは
[unstable book] で文書化されるべきです。

新しいコマンドライン引数を追加するための*手順*の詳細については、[新しいオプションの forge ガイド][forge guide for new options]を参照してください。

## ガイドライン

- フラグは互いに直交している必要があります。たとえば、複数のアクション `foo` と `bar` の
  json 出力バリアントがある場合、`--foo-json` と `--bar-json` を追加するよりも、追加の
  `--json` フラグの方が優れています。
- `no-` プレフィックスを持つフラグは避けてください。代わりに、
  `-C embed-bitcode=no` のように、[`parse_bool`] 関数を使用してください。
- フラグが複数回渡された場合の動作を考慮してください。状況によっては、
  値を（順番に！）累積する必要があります。他の
  状況では、後続のフラグが以前のフラグを上書きする必要があります（たとえば、
  lint レベルフラグ）。そして、一部のフラグ（`-o` など）は、複数のフラグが何を意味するかが
  あまりにも曖昧な場合はエラーを生成する必要があります。
- より理解しやすいコンパイラスクリプトのためにも、常にオプションに長い説明的な名前を付けてください。
- `--verbose` フラグは、`rustc`
  出力に詳細情報を追加するためのものです。たとえば、`--version`
  フラグと一緒に使用すると、コンパイラコードのハッシュに関する情報が得られます。
- 実験的なフラグとオプションは、`-Z
  unstable-options` フラグの背後で保護する必要があります。

[cli-docs]: https://doc.rust-lang.org/rustc/command-line-arguments.html
[forge guide for new options]: https://forge.rust-lang.org/compiler/proposals-and-stabilization.html#compiler-flags
[unstable book]: https://doc.rust-lang.org/nightly/unstable-book/
[`parse_bool`]: https://github.com/rust-lang/rust/blob/e5335592e78354e33d798d20c04bcd677c1df62d/src/librustc_session/options.rs#L307-L313
