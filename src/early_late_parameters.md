
# Early vs Late bound パラメータ

> **注意**: この章は主に、early/late bound が関数アイテム型/関数定義に関連する場合についてのみ説明しています。これは完全に正確ではない可能性があり、async ブロックやクロージャについてもこの章で多少議論する必要があるかもしれません。

## "early" bound または "late" bound とはどういう意味か

すべての関数定義には、`Fn*` トレイトを実装する対応するZSTがあり、これは[関数アイテム型][function_item_type]として知られています。この章のこの部分では、関数アイテム型の「デシュガリング」について少し説明します。これは、early bound と late bound のジェネリックパラメータの違いを説明する上で有用なコンテキストとなります。

まず、ジェネリックパラメータを含まない非常に簡単な例から始めましょう：

```rust
fn foo(a: String) -> u8 {
    # 1
    /* snip */
}
```

`foo` に対応する関数アイテム型とその関連する `Fn` 実装の定義を明示的に書き出すと、次のようになります：

```rust,ignore
struct FooFnItem;

impl Fn<(String,)> for FooFnItem {
    type Output = u8;
    /* fn call(&self, ...) -> ... { ... } */
}
```

`FnMut`/`FnOnce` トレイトのビルトイン実装、および `Copy` と `Clone` の実装は簡潔のため省略しています（ただし、これらのトレイトは関数アイテム型に対して実装*されています*）。

少し複雑な例として、関数にジェネリックパラメータを導入する場合があります：

```rust
fn foo<T: Sized>(a: T) -> T {
    # a
    /* snip */
}
```

定義を書き出すと次のようになります：

```rust,ignore
struct FooFnItem<T: Sized>(PhantomData<fn(T) -> T>);

impl<T: Sized> Fn<(T,)> for FooFnItem<T> {
    type Output = T;
    /* fn call(&self, ...) -> ... { ... } */
}
```

関数アイテム型 `FooFnItem` は、関数 `foo` で定義されているように、型パラメータ `T` に対してジェネリックであることに注意してください。しかし、関数で定義されたすべてのジェネリックパラメータが関数アイテム型でも定義されるわけではありません。以下に示すとおりです：

```rust
fn foo<'a, T: Sized>(a: &'a T) -> &'a T {
    # a
    /* snip */
}
```

その「デシュガリング」形式は次のようになります：

```rust,ignore
struct FooFnItem<T: Sized>(PhantomData<for<'a> fn(&'a T) -> &'a T>);

impl<'a, T: Sized> Fn<(&'a T,)> for FooFnItem<T> {
    type Output = &'a T;
    /* fn call(&self, ...) -> ... { ... } */
}
```

関数 `foo` のライフタイムパラメータ `'a` は、関数アイテム型 `FooFnItem` には存在せず、代わりに引数型を表現するためだけにビルトイン実装で導入されています。

ジェネリックパラメータがすべて関数アイテム型で定義されているわけではないということは、関数を呼び出すときにジェネリック引数が提供される段階が2つあることを意味します。

1. 関数に名前を付けるとき（例：`let a = foo;`）、`FooFnItem` の引数が提供されます。
2. 関数を呼び出すとき（例：`a(&10);`）、ビルトイン実装で定義されたパラメータが提供されます。

この2段階システムが、early vs late という命名スキームの由来です。early bound パラメータは*最も早い*段階（関数に名前を付けるとき）で提供され、late bound パラメータは*最も遅い*段階（関数を呼び出すとき）で提供されます。

前の例のデシュガリングを見ると、`T` は early bound 型パラメータであり、`'a` は late bound ライフタイムパラメータであることがわかります。`T` は関数アイテム型に存在しますが、`'a` は存在しないためです。各ジェネリックパラメータに引数が提供される場所を注釈した `foo` の呼び出し例をご覧ください：

```rust
fn foo<'a, T: Sized>(a: &'a T) -> &'a T {
    # a
    /* snip */
}

// ここで、関数アイテム型の型パラメータ `T` に
// 型引数 `String` を提供します
let my_func = foo::<String>;

// ここで（暗黙的に）ビルトイン実装の
// ライフタイムパラメータ `'a` にライフタイム引数が提供されます
my_func(&String::new());
```

[function_item_type]: https://doc.rust-lang.org/reference/types/function-item.html

## early bound と late bound パラメータの違い

### 高階関数ポインタとトレイト境界

ジェネリックパラメータが late bound であることで、関数アイテムのより柔軟な使用が可能になります。例えば、early bound ライフタイムパラメータを持つ関数 `foo` と、late bound ライフタイムパラメータ `'a` を持つ関数 `bar` がある場合、次のようなビルトイン `Fn` 実装が得られます：

```rust,ignore
impl<'a> Fn<(&'a String,)> for FooFnItem<'a> { /* ... */ }
impl<'a> Fn<(&'a String,)> for BarFnItem { /* ... */ }
```

`bar` 関数は厳密により柔軟なシグネチャを持っています。関数アイテム型は*任意の*ライフタイムを持つ借用で呼び出すことができますが、`foo` 関数アイテム型は関数アイテム型と同じライフタイムを持つ借用でのみ呼び出すことができます。これは、`foo` の関数アイテム型を異なるライフタイムで複数回呼び出そうとすることで示すことができます：

```rust
// `'a: 'a` 境界により、このライフタイムは early bound になります。
fn foo<'a: 'a>(b: &'a String) -> &'a String { b }
fn bar<'a>(b: &'a String) -> &'a String { b }

// early bound ジェネリックパラメータは、関数 `foo` に
// 名前を付けるときにここでインスタンス化されます。
// `'a` は early bound であるため、引数が提供されます。
let f = foo::<'_>;

// 両方の関数引数は、ライフタイムパラメータが early bound であるため
// 同じライフタイムを持つ必要があります。つまり `f` は
// 1つの特定のライフタイムに対してのみ呼び出し可能です。
//
// 異なるライフタイムの借用でこれを呼び出すため、
// borrow checker はここでエラーを出します。
f(&String::new());
f(&String::new());
```

この例では、`foo` の関数アイテム型を2回呼び出していますが、それぞれ一時変数の借用を使用しています。これら2つの借用は、一時変数が関数呼び出し中にのみ生存し、その後は生存しないため、重複するライフタイムを持つことはできません。`foo` のライフタイムパラメータが early bound であるため、`f` のすべての呼び出し元は同じライフタイムを持つ借用を提供する必要があり、これは不可能であるため borrow checker がエラーを出します。

`foo` のライフタイムパラメータが late bound であれば、各呼び出し元が借用に対して異なるライフタイム引数を提供できるため、これはコンパイルできます。上記で定義された `bar` 関数を使用してこれを示す次の例をご覧ください：

```rust
# fn foo<'a: 'a>(b: &'a String) -> &'a String { b }
# fn bar<'a>(b: &'a String) -> &'a String { b }
#
// early bound パラメータはここでインスタンス化されますが、
// `'a` は late bound であるため、ここでは提供されません。
let b = bar;

// late bound パラメータは各呼び出しサイトで個別にインスタンス化されるため、
// 各呼び出し元は異なるライフタイムを使用できます。
b(&String::new());
b(&String::new());
```

これは、関数アイテム型を高階関数ポインタに型強制したり、高階 `Fn` トレイト境界を証明したりする能力に反映されています。次の例でこれを示すことができます：

```rust
// `'a: 'a` 境界により、このライフタイムは early bound になります。
fn foo<'a: 'a>(b: &'a String) -> &'a String { b }
fn bar<'a>(b: &'a String) -> &'a String { b }

fn accepts_hr_fn(_: impl for<'a> Fn(&'a String) -> &'a String) {}

fn higher_ranked_trait_bound() {
    let bar_fn_item = bar;
    accepts_hr_fn(bar_fn_item);

    let foo_fn_item = foo::<'_>;
    // エラー
    accepts_hr_fn(foo_fn_item);
}

fn higher_ranked_fn_ptr() {
    let bar_fn_item = bar;
    let fn_ptr: for<'a> fn(&'a String) -> &'a String = bar_fn_item;

    let foo_fn_item = foo::<'_>;
    // エラー
    let fn_ptr: for<'a> fn(&'a String) -> &'a String = foo_fn_item;
}
```

両方のケースで、borrow checker は `foo_fn_item` が任意のライフタイムを持つ借用で呼び出し可能であるとは見なさないため、エラーを出します。これは、`foo` のライフタイムパラメータが early bound であるため、`foo_fn_item` が `FooFnItem<'_>` 型を持ち、（デシュガリングされた `Fn` 実装で示されているように）同じライフタイム `'_` を持つ借用でのみ呼び出し可能だからです。

### late bound パラメータの存在下でのタービンフィッシング

前述のように、early bound と late bound パラメータの区別は、ジェネリックパラメータがインスタンス化される場所が2つあることを意味します：

- 関数に名前を付けるとき（early）
- 関数を呼び出すとき（late）

現在、呼び出しステップ中に late bound パラメータのジェネリック引数を明示的に指定する構文はありません。ジェネリック引数は、関数に名前を付けるときに early bound パラメータに対してのみ指定できます。
構文 `foo::<'static>();` は、関数呼び出しの一部であるにもかかわらず、`(foo::<'static>)();` のように動作し、関数アイテム型の early bound ジェネリックパラメータをインスタンス化します。

次の例をご覧ください：

```rust
fn foo<'a>(b: &'a u32) -> &'a u32 { b }

let f /* : FooFnItem<????> */ = foo::<'static>;
```

上記の例は、ライフタイムパラメータ `'a` が late bound であるため、「関数に名前を付ける」ステップの一部としてインスタンス化できないため、エラーになります。ライフタイムパラメータを early bound にすると、このコードはコンパイルされるようになります：

```rust
fn foo<'a: 'a>(b: &'a u32) -> &'a u32 { b }

let f /* : FooFnItem<'static> */ = foo::<'static>;
```

コンパイラの現在の実装が目指していることは、early *および* late bound ライフタイムパラメータの両方を持つ関数にライフタイム引数を指定するときにエラーを出すことです。実際には、過度な破壊のため、一部のケースは実際には将来の互換性警告のみです（[#42868](https://github.com/rust-lang/rust/issues/42868)）：

- ライフタイム引数の数が early bound ライフタイムパラメータの数と同じ場合、エラーの代わりに FCW が発行されます
- メソッド呼び出し構文を使用する場合、エラーは常に FCW にダウングレードされます

これを示すために、さまざまな種類の関数を書き出し、それぞれに late と early bound のライフタイムを与えることができます：

```rust,ignore
fn free_function<'a: 'a, 'b>(_: &'a (), _: &'b ()) {}

struct Foo;

trait Trait: Sized {
    fn trait_method<'a: 'a, 'b>(self, _: &'a (), _: &'b ());
    fn trait_function<'a: 'a, 'b>(_: &'a (), _: &'b ());
}

impl Trait for Foo {
    fn trait_method<'a: 'a, 'b>(self, _: &'a (), _: &'b ()) {}
    fn trait_function<'a: 'a, 'b>(_: &'a (), _: &'b ()) {}
}

impl Foo {
    fn inherent_method<'a: 'a, 'b>(self, _: &'a (), _: &'b ()) {}
    fn inherent_function<'a: 'a, 'b>(_: &'a (), _: &'b ()) {}
}
```

次に、最初のケースとして、各関数を単一のライフタイム引数（1つの early bound ライフタイムパラメータに対応）で呼び出し、ハードエラーではなく FCW のみが発生することに注意してください。

```rust
#![deny(late_bound_lifetime_arguments)]

# fn free_function<'a: 'a, 'b>(_: &'a (), _: &'b ()) {}
#
# struct Foo;
#
# trait Trait: Sized {
#     fn trait_method<'a: 'a, 'b>(self, _: &'a (), _: &'b ());
#     fn trait_function<'a: 'a, 'b>(_: &'a (), _: &'b ());
# }
#
# impl Trait for Foo {
#     fn trait_method<'a: 'a, 'b>(self, _: &'a (), _: &'b ()) {}
#     fn trait_function<'a: 'a, 'b>(_: &'a (), _: &'b ()) {}
# }
#
# impl Foo {
#     fn inherent_method<'a: 'a, 'b>(self, _: &'a (), _: &'b ()) {}
#     fn inherent_function<'a: 'a, 'b>(_: &'a (), _: &'b ()) {}
# }
#
// early bound パラメータと同じ数の引数を指定することは
// 常に将来の互換性警告になります
Foo.trait_method::<'static>(&(), &());
Foo::trait_method::<'static>(Foo, &(), &());
Foo::trait_function::<'static>(&(), &());
Foo.inherent_method::<'static>(&(), &());
Foo::inherent_function::<'static>(&(), &());
free_function::<'static>(&(), &());
```

2番目のケースでは、各関数をライフタイムパラメータ（early または late bound）の数よりも多いライフタイム引数で呼び出し、メソッド呼び出しがハードエラーとなる自由/関連関数とは対照的に FCW となることに注意してください：

```rust
# fn free_function<'a: 'a, 'b>(_: &'a (), _: &'b ()) {}
#
# struct Foo;
#
# trait Trait: Sized {
#     fn trait_method<'a: 'a, 'b>(self, _: &'a (), _: &'b ());
#     fn trait_function<'a: 'a, 'b>(_: &'a (), _: &'b ());
# }
#
# impl Trait for Foo {
#     fn trait_method<'a: 'a, 'b>(self, _: &'a (), _: &'b ()) {}
#     fn trait_function<'a: 'a, 'b>(_: &'a (), _: &'b ()) {}
# }
#
# impl Foo {
#     fn inherent_method<'a: 'a, 'b>(self, _: &'a (), _: &'b ()) {}
#     fn inherent_function<'a: 'a, 'b>(_: &'a (), _: &'b ()) {}
# }
#
// early bound パラメータよりも多くの引数を指定することは、
// メソッド呼び出し構文を使用する場合は将来の互換性警告になります。
Foo.trait_method::<'static, 'static, 'static>(&(), &());
Foo.inherent_method::<'static, 'static, 'static>(&(), &());
// しかし、メソッド呼び出し構文を使用しない場合はハードエラーになります。
Foo::trait_method::<'static, 'static, 'static>(Foo, &(), &());
Foo::trait_function::<'static, 'static, 'static>(&(), &());
Foo::inherent_function::<'static, 'static, 'static>(&(), &());
free_function::<'static, 'static, 'static>(&(), &());
```

late と early bound ライフタイムパラメータの両方に対して十分なライフタイム引数を指定した場合でも、これらの引数は late bound パラメータに提供されるライフタイムを注釈するために実際には使用されません。これは、非静的な借用を提供しながら `'static` をタービンフィッシングすることで示すことができます：

```rust
struct Foo;

impl Foo {
    fn inherent_method<'a: 'a, 'b>(self, _: &'a (), _: &'b String ) {}
}

Foo.inherent_method::<'static, 'static>(&(), &String::new());
```

これは、`&String::new()` 関数引数が `'static` ライフタイムを持っていないにもかかわらずコンパイルされます。これは、関数を実際に呼び出すときに、「余分な」ライフタイム引数が late bound パラメータに対して考慮されるのではなく、破棄されるためです。

### late bound パラメータを持つ型の生存性

関数アイテム型を含む型の生存境界をチェックするとき、early bound パラメータを考慮します。例えば：

```rust
fn foo<T>(_: T) {}

fn requires_static<T: 'static>(_: T) {}

fn bar<T>() {
    let f /* : FooFnItem<T> */ = foo::<T>;
    requires_static(f);
}
```

型パラメータ `T` は early bound であるため、`foo` の関数アイテム型のデシュガリングは `struct FooFnItem<T>` のようになります。次に、`FooFnItem<T>: 'static` が成立するためには、`T: 'static` も成立する必要があります。そうでなければ、健全性バグが発生します。

残念ながら、コンパイラのバグにより、early bound *ライフタイム*を考慮していません。これは、未解決の健全性バグ [#84366](https://github.com/rust-lang/rust/issues/84366) の原因です。これは、生存性/型生存境界について early/late bound パラメータ間の「違い」を示すことが不可能であることを意味します。late bound になることができる唯一の種類のジェネリックパラメータはライフタイムであり、これは不正確に処理されているためです。

それにもかかわらず、理論的には、[#84366](https://github.com/rust-lang/rust/issues/84366) が修正されれば、以下のコード例はそのような違いを示す*はず*です：

```rust
fn early_bound<'a: 'a>(_: &'a String) {}
fn late_bound<'a>(_: &'a String) {}

fn requires_static<T: 'static>(_: T) {}

fn bar<'b>() {
    let e = early_bound::<'b>;
    // これはエラーになる*はず*ですが、そうなっていません
    requires_static(e);

    let l = late_bound;
    // これは正しくエラーになりません
    requires_static(l);
}
```

## パラメータが late bound になるための要件

### ライフタイムパラメータである必要がある

型パラメータと const パラメータは、`dyn for<T> Fn(Box<T>)` や `for<T> fn(Box<T>)` のような型をサポートする方法がないため、late bound にすることはできません。このような型を呼び出すには、基礎となる関数を単相化できる必要がありますが、これは動的ディスパッチを介した間接参照では不可能です。

### where 句で使用してはならない

現在、ジェネリックパラメータが where 句で使用されている場合、それは early bound でなければなりません。例えば：

```rust
# trait Trait<'a> {}
fn foo<'a, T: Trait<'a>>(_: &'a String, _: T) {}
```

この例では、ライフタイムパラメータ `'a` は where 句 `T: Trait<'a>` に現れるため、early bound と見なされます。これは、`'a: 'a` のような「自明な」where 句や、関数引数の well-formedness によって暗黙的に示されるものであっても当てはまります。例えば：

```rust
fn foo<'a: 'a>(_: &'a String) {}
fn bar<'a, T: 'a>(_: &'a T) {}
```

これらの関数の両方で、ライフタイムパラメータ `'a` は、使用されている where 句が呼び出し元に実質的に制約を課さない場合でも、early bound と見なされます。

この制限の理由は、2つのことの組み合わせです：

- late bound パラメータに対する境界は、インスタンス化されるまで証明できません
- 関数ポインタとトレイトオブジェクトには、基礎となる関数からのまだ証明されていない where 句を表現する方法がありません

次の例を見てみましょう：

```rust
trait Trait<'a> {}
fn foo<'a, T: Trait<'a>>(_: &'a T) {}

let f = foo::<String>;
let f = f as for<'a> fn(&'a String);
f(&String::new());
```

型チェック中の*ある時点*で、このコードに対してエラーが発行されるべきです。`String` は任意のライフタイムに対して `Trait` を実装していないためです。

ライフタイム `'a` が late bound の場合、これはチェックが困難になります。`foo` に名前を付けるとき、まだインスタンス化されていないため、`T: Trait<'a>` トレイト境界の一部として使用するライフタイムがわかりません。関数アイテム型を関数ポインタに型強制するとき、関数を呼び出すときに証明する必要がある `String: Trait<'a>` トレイト境界を追跡する方法がありません。

ライフタイム `'a` が early bound の場合（rustc の現在の実装ではそうです）、トレイト境界は関数 `foo` に名前を付けるときにチェックできます。where 句で使用されるパラメータを early bound にすることを要求することで、関数で定義された where 句をチェックする自然な場所が得られます。

最後に、*暗黙境界*で使用される場合、ライフタイムを early bound にする必要はありません。例えば：

```rust
fn foo<'a, T>(_: &'a T) {}

let f = foo;
f(&String::new());
f(&String::new());
```

このコードはコンパイルされ、ライフタイムパラメータが late bound であることを示しています。`'a` が型 `&'a T` で使用されており、暗黙的に `T: 'a` が成立することを要求しているにもかかわらずです。暗黙境界は特別に扱うことができます。暗黙境界を導入する型は関数ポインタ型のシグネチャにあるため、関数を呼び出すときに `T: 'a` を証明する必要があることがわかります。

### 引数型によって制約される必要がある

関数アイテム型のビルトイン実装が制約されていないジェネリックパラメータを持たないようにすることが重要です。これは健全性の問題につながる可能性があるためです。これは、ユーザーが書いた実装に適用されるのと同じ種類の制限です。例えば、次のコードはエラーになります：

```rust
trait Trait {
    type Assoc;
}

impl<'a> Trait for u8 {
    type Assoc = &'a String;
}
```

関数アイテムのビルトイン実装の類似例は次のようになります：

```rust,ignore
fn foo<'a>() -> &'a String { /* ... */ }
```

ライフタイムパラメータ `'a` が late bound の場合、制約されていないライフタイムを持つビルトイン実装になります。`'a` が late bound である場合の関数アイテム型とその実装のデシュガリングを手動で書き出すことで、これを示すことができます：

```rust,ignore
// 注意：これはデモンストレーション用です。実際には `'a` は early bound です
struct FooFnItem;

impl<'a> Fn<()> for FooFnItem {
    type Output = &'a String;
    /* fn call(...) -> ... { ... } */
}
```

このような状況を避けるために、`'a` を early bound と見なします。これにより、実装のライフタイムが self 型によって制約されます：

```rust,ignore
struct FooFnItem<'a>(PhantomData<fn() -> &'a String>);

impl<'a> Fn<()> for FooFnItem<'a> {
    type Output = &'a String;
    /* fn call(...) -> ... { ... } */
}
```
