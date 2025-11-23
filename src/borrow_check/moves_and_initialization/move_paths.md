# Move paths

実際には、ローカル変数の粒度で初期化を追跡するだけでは十分ではありません。Rust では、フィールドの粒度での移動と初期化も可能です:

```rust,ignore
fn foo() {
    let a: (Vec<u32>, Vec<u32>) = (vec![22], vec![44]);

    // a.0 と a.1 は両方とも初期化されています

    let b = a.0; // a.0 を移動します

    // a.0 は初期化されていませんが、a.1 はまだ初期化されています

    let c = a.0; // ERROR
    let d = a.1; // OK
}
```

これを処理するために、**move path** の粒度で初期化を追跡します。[`MovePath`] は、ユーザーが初期化、移動などできる場所を表します。したがって、例えば、ローカル変数 `a` を表す move-path があり、`a.0` を表す move-path があります。move path は、MIR の [`Place`] の概念とほぼ一致しますが、より効率的に移動分析を実行できるようにインデックス化されています。

[`MovePath`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/struct.MovePath.html
[`Place`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Place.html

## Move path インデックス

[`MovePath`] データ構造はありますが、直接参照されることはありません。代わりに、すべてのコードは [`MovePathIndex`] 型の*インデックス*を渡します。move path に関する情報を取得する必要がある場合は、このインデックスを [`MoveData` の `move_paths` フィールド][move_paths] とともに使用します。例えば、[`MovePathIndex`] `mpi` を MIR [`Place`] に変換するには、次のように [`MovePath::place`] フィールドにアクセスします:

```rust,ignore
move_data.move_paths[mpi].place
```

[move_paths]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/struct.MoveData.html#structfield.move_paths
[`MovePath::place`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/struct.MovePath.html#structfield.place
[`MovePathIndex`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/struct.MovePathIndex.html

## Move paths の構築

MIR borrow check で最初に行うことの 1 つは、move paths のセットを構築することです。これは [`MoveData::gather_moves`] 関数の一部として行われます。この関数は、MIR を歩いて、各 [`Place`] がどのようにアクセスされるかを見るために、[`MoveDataBuilder`] と呼ばれる MIR ビジターを使用します。各 [`Place`] について、対応する [`MovePathIndex`] を構築します。また、その特定の move path がいつ/どこで移動/初期化されるかを記録しますが、それについては後のセクションで説明します。

[`MoveDataBuilder`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/builder/struct.MoveDataBuilder.html
[`MoveData::gather_moves`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/struct.MoveData.html#method.gather_moves

### 不正な move paths

実際には、使用されるすべての [`Place`] に対して move-path を作成するわけではありません。特に、[`Place`] から移動することが違法である場合、[`MovePathIndex`] は必要ありません。いくつかの例:

- 配列の個々の要素を移動することはできないため、例えば `foo: [String; 3]` がある場合、`foo[1]` の move-path はありません。
- 借用された参照の内部から移動することはできないため、例えば `foo: &String` がある場合、`*foo` の move-path はありません。

これらの規則は [`move_path_for`] 関数によって強制されます。この関数は [`Place`] を [`MovePathIndex`] に変換します -- 上記のようなエラーケースでは、関数は `Err` を返します。これは、それらの場所が初期化されているかどうかを追跡する必要がないことを意味します（オーバーヘッドが削減されます）。

[`move_path_for`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/builder/struct.MoveDataBuilder.html#method.move_path_for

## 射影

[`PlaceElem`] を使用する代わりに、move paths の射影は [`MoveSubPath`] として格納されます。
移動できない射影と、スキップできる射影は表現されません。

配列のサブスライス射影（スライスパターンによって生成される）は特別です; これらは、サブスライス内の各要素に対して 1 つの [`ConstantIndex`] サブパスに変換されます。

[`PlaceElem`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/type.PlaceElem.html
[`MoveSubPath`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/enum.MoveSubPath.html
[`ConstantIndex`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/enum.MoveSubPath.html#variant.ConstantIndex

## move-path の検索

[`Place`] があり、それを [`MovePathIndex`] に変換したい場合、[`MoveData`] の [`rev_lookup`] フィールドにある [`MovePathLookup`] 構造体を使用してそれを行うことができます。2 つの異なるメソッドがあります:

[`MoveData`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/struct.MoveData.html
[`MovePathLookup`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/struct.MovePathLookup.html
[`rev_lookup`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/struct.MoveData.html#structfield.rev_lookup

- [`find_local`]。これは、ローカル変数を表す [`mir::Local`] を受け取ります。これは簡単なメソッドです。なぜなら、すべてのローカル変数に対して**常に** [`MovePathIndex`] を作成するからです。
- [`find`]。これは、任意の [`Place`] を受け取ります。このメソッドは、すべての [`Place`] に対して [`MovePathIndex`] を持っているわけではないため（「不正な move paths」のセクションで説明したように）、少し厄介です。したがって、[`find`] は、存在する最も近いパスを示す [`LookupResult`] を返します（例えば、`foo[1]` の場合、`foo` のパスだけを返すかもしれません）。

[`find`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/struct.MovePathLookup.html#method.find
[`find_local`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/struct.MovePathLookup.html#method.find_local
[`mir::Local`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Local.html
[`LookupResult`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/enum.LookupResult.html

## 相互参照

上記で述べたように、move-paths は大きなベクターに格納され、[`MovePathIndex`] を介して参照されます。しかし、このベクター内では、それらもツリー構造になっています。したがって、例えば `a.b.c` の [`MovePathIndex`] がある場合、その親の move-path `a.b` に移動できます。すべての子パスを反復することもできます: したがって、`a.b` から、パス `a.b.c` を見つけるために反復できます（ここでは、ソースで**実際に参照されている**パス上でのみ反復しており、参照されている**可能性のある**すべてのパス上で反復しているわけではありません）。これらの参照は、例えば [`find_in_move_path_or_its_descendants`] 関数で使用されます。この関数は、move-path（例えば、`a.b`）またはその move-path の子（例えば、`a.b.c`）が与えられた述語に一致するかどうかを判断します。

[`find_in_move_path_or_its_descendants`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/move_paths/struct.MoveData.html#method.find_in_move_path_or_its_descendants
