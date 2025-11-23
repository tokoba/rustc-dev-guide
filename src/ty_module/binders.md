# `Binder` と高階リージョン

型の一部として、または where 句の一部として、アイテムではなくジェネリックパラメータを定義することがあります。例えば、型 `for<'a> fn(&'a u32)` や where 句 `for<'a> T: Trait<'a>` は、どちらも `'a` という名前のジェネリックライフタイムを導入します。現在、`for<T>` や `for<const N: usize>` の安定した構文はありませんが、nightly では `feature(non_lifetime_binders)` を使用して、`for<T>`/`for<const N: usize>` を使用する where 句（ただし型ではない）を書くことができます。

`for` は「バインダー」と呼ばれます。なぜなら、新しい名前をスコープに導入するからです。rustc では、`Binder` 型を使用して、これらのパラメータがどこで導入され、パラメータが何であるか（つまり、数と、パラメータが型/定数/リージョンであるかどうか）を追跡します。`for<'a> fn(&'a u32)` のような型は、rustc では次のように表現されます：

```
Binder(
    fn(&RegionKind::Bound(DebruijnIndex(0), BoundVar(0)) u32) -> (),
    &[BoundVariableKind::Region(...)],
)
```

これらのパラメータの使用は、`RegionKind::Bound`（または `TyKind::Bound`/`ConstKind::Bound` バリアント）によって表されます。これらのバウンドリージョン/型/定数は、2 つの主要なデータで構成されています：

- どのバインダーを参照しているかを指定する [DebruijnIndex](../appendix/background.md#what-is-a-de-bruijn-index)。
- `Binder` が導入するパラメータのうちどれを参照しているかを指定する [`BoundVar`]。

また、診断の理由で [`BoundTyKind`]/[`BoundRegionKind`] を介していくつかの追加情報を保存することもありますが、これは型の等価性や `Ty` の意味論にとって重要ではありません。（上記の例では省略されています）

デバッグ出力（および互いに話すときに非公式に）では、これらのバウンド変数を `^DebruijnIndex_BoundVar` の形式で書く傾向があります。上記の例は、代わりに `Binder(fn(&'^0_0), &[BoundVariableKind::Region])` と書かれます。`DebruijnIndex` が `0` の場合、それを省略して `^0` と書くこともあります。

もう 1 つの具体的な例として、今回は where 句と型の `for<'a>` の混合です：

```
where
    for<'a> Foo<for<'b> fn(&'a &'b T)>: Trait,
```

これは次のように表現されます

```
Binder(
    Foo<Binder(
        fn(&'^1_0 &'^0 T/#0),
        [BoundVariableKind::Region(...)]
    )>: Trait,
    [BoundVariableKind::Region(...)]
)
```

`'^1_0` が `'a` パラメータを参照していることに注意してください。最も内側のバインダーから 1 レベル上のバインダーを参照するために `DebruijnIndex` として `1` を使用し、最初にバインドされたパラメータを参照するために var として `0` を使用します。これは `'a` です。また、`'b` パラメータを参照するために `'^0` を使用します。`DebruijnIndex` は `0`（最も内側のバインダーを参照）なので省略し、最初にバインドされたパラメータを参照する boundvar の `0` のみを残します。これは `'b` です。

各 `Binder` によって導入されるバウンド変数のセットを明示的に追跡していなかったことは過去にありました。これは多くのバグ（読む：ICE [#81193](https://github.com/rust-lang/rust/issues/81193)、[#79949](https://github.com/rust-lang/rust/issues/79949)、[#83017](https://github.com/rust-lang/rust/issues/83017)）を引き起こしました。これらを明示的に追跡することで、高階 where 句/型を構築するときに、エスケープするバウンド変数や異なるバインダーからの変数がないことをアサートできます。バインダー内の無効な型の次の例を参照してください：

```
Binder(
    fn(&'^1_0 &'^1 T/#0),
    &[BoundVariableKind::Region(...)],
)
```

これは、リージョン `'^1_0` が最も外側のバインダーよりも高いレベルのバインダーを参照しているため、つまり、エスケープするバウンド変数であるため、あらゆる種類の問題を引き起こします。`'^1` リージョン（`'^0_1` とも書けます）も、それが参照するバインダーが 2 番目のパラメータを導入しないため、不正な形式です。現代の rustc は、これらの両方の理由により、このバインダーを構築するときに ICE します。過去には、これを単に許可して動作させ、コードベースの他の部分で問題に遭遇していました。

[`BoundVar`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.BoundVar.html
[`BoundRegionKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/enum.BoundRegionKind.html
[`BoundTyKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/enum.BoundTyKind.html
