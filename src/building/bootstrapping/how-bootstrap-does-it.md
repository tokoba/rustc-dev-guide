# ブートストラップがどのように動作するか

ブートストラップの中心的な概念は、ビルド [`Step`] です。これらは [`Builder::ensure`] によって連鎖されます。[`Builder::ensure`] は [`Step`] を入力として受け取り、まだ実行されていない場合にのみ [`Step`] を実行します。[`Step`] についてより詳しく見てみましょう。

## [`Step`] の概要

[`Step`] は、何らかの成果物を生成するプロセスに含まれる、粒度の細かいアクションの集まりを表します。Makefile のルールのように考えることができます。[`Step`] トレイトは次のように定義されています：

```rs,no_run
pub trait Step: 'static + Clone + Debug + PartialEq + Eq + Hash {
    type Output: Clone;

    const DEFAULT: bool = false;
    const ONLY_HOSTS: bool = false;

    // Required methods
    fn run(self, builder: &Builder<'_>) -> Self::Output;
    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_>;

    // Provided method
    fn make_run(_run: RunConfig<'_>) { ... }
}
```

- `run` は実際の作業を行う関数です。[`Builder::ensure`] が `run` を呼び出します。
- `should_run` はコマンドラインインターフェースで、`x build foo` のような呼び出しが与えられた [`Step`] を実行すべきかどうかを決定します。パスが提供されない「デフォルト」コンテキストでは、`make_run` が直接呼び出されます。
- `make_run` は、他のステップの依存関係ではなく、CLI を介して直接要求されたもののためにのみ呼び出されます。

## エントリーポイント

コアブートストラップコードに到達する前に、いくつかの準備ステップがあります：

1. シェルスクリプトまたは `make`：[`./x`](https://github.com/rust-lang/rust/blob/HEAD/x) または [`./x.ps1`](https://github.com/rust-lang/rust/blob/HEAD/x.ps1) または `make`
2. 便利なラッパースクリプト：[`x.py`](https://github.com/rust-lang/rust/blob/HEAD/x.py)
3. [`src/bootstrap/bootstrap.py`](https://github.com/rust-lang/rust/blob/HEAD/src/bootstrap/bootstrap.py)
4. [`src/bootstrap/src/bin/main.rs`](https://github.com/rust-lang/rust/blob/HEAD/src/bootstrap/src/bin/main.rs)

実装の詳細については、[src/bootstrap/README.md](https://github.com/rust-lang/rust/blob/HEAD/src/bootstrap/README.md) をご覧ください。

[`Step`]: https://doc.rust-lang.org/nightly/nightly-rustc/bootstrap/core/builder/trait.Step.html
[`Builder::ensure`]: https://doc.rust-lang.org/nightly/nightly-rustc/bootstrap/core/builder/struct.Builder.html#method.ensure
