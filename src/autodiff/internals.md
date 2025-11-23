Rustの`std::autodiff`モジュールは、微分可能プログラミングを可能にします：

```rust
#![feature(autodiff)]
use std::autodiff::*;

// f(x) = x * x, f'(x) = 2.0 * x
// したがって、barは (x * x, 2.0 * x) を返す
#[autodiff_reverse(bar, Active, Active)]
fn foo(x: f32) -> f32 { x * x }

fn main() {
    assert_eq!(bar(3.0, 1.0), (9.0, 6.0));
    assert_eq!(bar(4.0, 1.0), (16.0, 8.0));
}
```

`std::autodiff`モジュールの詳細なドキュメントは、[std::autodiff](https://doc.rust-lang.org/std/autodiff/index.html)で利用できます。

微分可能プログラミングは、数値計算、[固体力学][ratel]、[計算化学][molpipx]、[流体力学][waterlily]、または誤差逆伝播によるニューラルネットワークのトレーニング、[ODEソルバー][diffsol]、[微分可能レンダリング][libigl]、[量子コンピューティング][catalyst]、気候シミュレーションなど、さまざまな分野で使用されています。

[ratel]: https://gitlab.com/micromorph/ratel
[molpipx]: https://arxiv.org/abs/2411.17011
[waterlily]: https://github.com/WaterLily-jl/WaterLily.jl
[diffsol]: https://github.com/martinjrobins/diffsol
[libigl]: https://github.com/alecjacobson/libigl-enzyme-example?tab=readme-ov-file#run
[catalyst]: https://github.com/PennyLaneAI/catalyst
