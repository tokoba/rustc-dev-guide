# 並列コンパイル

<div class="warning">
<!-- date-check --> 2024年11月時点で、
並列フロントエンドは大幅な変更が行われているため、
このページにはかなり古い情報が含まれています。

トラッキングissue: <https://github.com/rust-lang/rust/issues/113349>
</div>

<!-- date-check --> 2024年11月時点で、rustコンパイラのほとんどが並列化されています。

- コード生成部分はデフォルトで並行実行されます。`-C codegen-units=n` オプションを使用して並行タスク数を制御できます。
- HIRローワリングからコード生成までの型チェック、借用チェック、MIR最適化などの部分は、nightlyバージョンで並列化されています。現在、デフォルトでは逐次実行されており、ユーザーが `-Z threads = n` オプションを使用して手動で並列化を有効にします。
- その他の部分（字句解析、HIRローワリング、マクロ展開など）は引き続き逐次モードで実行されます。

<div class="warning">
以下のセクションは当面残していますが、かなり古くなっています。
</div>

---


## コード生成

単相化中に、コンパイラは生成されるすべてのコードを_コード生成ユニット_と呼ばれる小さなチャンクに分割します。これらは並列に実行される独立したLLVMインスタンスによって生成されます。最後に、すべてのコード生成ユニットを1つのバイナリに結合するためにリンカーが実行されます。このプロセスは [`rustc_codegen_ssa::base`] モジュールで行われます。

[`rustc_codegen_ssa::base`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_codegen_ssa/base/index.html

## データ構造

並列コンパイラで使用される基礎的なスレッドセーフなデータ構造は [`rustc_data_structures::sync`] モジュールにあります。これらのデータ構造は、`parallel-compiler` が true かどうかによって異なる実装になっています。

| データ構造                   | parallel                                            | non-parallel |
| -------------------------------- | --------------------------------------------------- | ------------ |
| Lock\<T> | (parking_lot::Mutex\<T>) | (std::cell::RefCell) |
| RwLock\<T> | (parking_lot::RwLock\<T>) | (std::cell::RefCell) |
| MTLock\<T> | (Lock\<T>) | (T) |
| ReadGuard | parking_lot::RwLockReadGuard | std::cell::Ref |
| MappedReadGuard | parking_lot::MappedRwLockReadGuard | std::cell::Ref |
| WriteGuard | parking_lot::RwLockWriteGuard | std::cell::RefMut |
| MappedWriteGuard | parking_lot::MappedRwLockWriteGuard | std::cell::RefMut |
| LockGuard | parking_lot::MutexGuard | std::cell::RefMut |

- これらのスレッドセーフなデータ構造はコンパイル中に散在しており、ロック競合を引き起こし、スレッド数が4を超えるとパフォーマンスが低下する可能性があります。そのため、これらのデータ構造の使用を監査し、共有状態の使用を減らすようにリファクタリングするか、不変性、原子性、ロック順序の詳細をカバーする永続的なドキュメントを作成します。

- 一方で、並列コンパイル中に保持されない可能性のあるコンパイル中の他の不変性をまだ把握する必要があります。

[`rustc_data_structures::sync`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_data_structures/sync/index.html

### WorkerLocal

[`WorkerLocal`] は並列コンパイラ用に実装された特別なデータ構造です。スレッドプール内の各スレッドのワーカーローカル値を保持します。構築されたスレッドプールでの `Deref` 実装を介してのみワーカーローカル値にアクセスできます。それ以外の場合はパニックします。

`WorkerLocal` は並列環境での `Arena` アロケータの実装に使用されており、並列クエリにとって重要です。その実装は [`rustc_data_structures::sync::worker_local`] モジュールにあります。ただし、非並列コンパイラでは `(OneThread<T>)` として実装されており、その `T` は `Deref::deref` を介して直接アクセスできます。

[`rustc_data_structures::sync::worker_local`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_data_structures/sync/worker_local/index.html
[`WorkerLocal`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_data_structures/sync/worker_local/struct.WorkerLocal.html

## 並列イテレータ

[`rayon`] クレートが提供する並列イテレータは、並列性を実装する簡単な方法です。現在の並列コンパイラの実装では、カスタムの `rayon` [フォーク][rustc-rayon]を使用して並列でタスクを実行します。

一部のイテレータ関数は、`parallel-compiler` が true の場合にループを並列で実行するように実装されています。

| 関数（`Send` と `Sync` は省略）                             | 説明                                                 | 所有モジュール              |
| ------------------------------------------------------------ | ------------------------------------------------------------ | -------------------------- |
| **par_iter**<T: IntoParallelIterator>(t: T) -> T::Iter       | 並列イテレータを生成                                 | rustc_data_structure::sync |
| **par_for_each_in**<T: IntoParallelIterator>(t: T, for_each: impl Fn(T::Item)) | 並列イテレータを生成し、各要素に `for_each` を実行 | rustc_data_structure::sync |
| **Map::par_body_owners**(self, f: impl Fn(LocalDefId))       | クレート内のすべてのhir所有者に `f` を実行       | rustc_middle::hir::map     |
| **Map::par_for_each_module**(self, f: impl Fn(LocalDefId))   | クレート内のすべてのモジュールとサブモジュールに `f` を実行          | rustc_middle::hir::map     |
| **ModuleItems::par_items**(&self, f: impl Fn(ItemId))        | モジュール内のすべてのアイテムに `f` を実行           | rustc_middle::hir          |
| **ModuleItems::par_trait_items**(&self, f: impl Fn(TraitItemId)) | モジュール内のすべてのtraitアイテムに `f` を実行     | rustc_middle::hir          |
| **ModuleItems::par_impl_items**(&self, f: impl Fn(ImplItemId)) | モジュール内のすべてのimplアイテムに `f` を実行      | rustc_middle::hir          |
| **ModuleItems::par_foreign_items**(&self, f: impl Fn(ForeignItemId)) | モジュール内のすべてのforeign itemに `f` を実行   | rustc_middle::hir          |

コンパイラには、これらの関数を使用して並列化できる可能性のあるループが多数あります。<!-- date-check--> 2022年8月時点で、並列イテレータ関数が使用されているシナリオは以下の通りです：

| 呼び出し元                                                  | シナリオ                                                     | 呼び出し先                   |
| ------------------------------------------------------- | ------------------------------------------------------------ | ------------------------ |
| rustc_metadata::rmeta::encoder::prefetch_mir            | 後でメタデータエンコーディングに必要なクエリをプリフェッチ | par_iter                 |
| rustc_monomorphize::collector::collect_crate_mono_items | 非ジェネリックアイテムから到達可能な単相化されたアイテムを収集 | par_for_each_in          |
| rustc_interface::passes::analysis                       | matchステートメントの妥当性チェック                   | Map::par_body_owners     |
| rustc_interface::passes::analysis                       | MIR借用チェック                                             | Map::par_body_owners     |
| rustc_typeck::check::typeck_item_bodies                 | 型チェック                                                   | Map::par_body_owners     |
| rustc_interface::passes::hir_id_validator::check_crate  | hirの妥当性チェック                    | Map::par_for_each_module |
| rustc_interface::passes::analysis                       | ループ本体、属性、naked関数、不安定なabi、const本体の妥当性チェック | Map::par_for_each_module |
| rustc_interface::passes::analysis                       | MIRの生存性とintrinsicチェック                       | Map::par_for_each_module |
| rustc_interface::passes::analysis                       | デッドネスチェック                                           | Map::par_for_each_module |
| rustc_interface::passes::analysis                       | プライバシーチェック                                             | Map::par_for_each_module |
| rustc_lint::late::check_crate                           | モジュールごとのlintを実行                         | Map::par_for_each_module |
| rustc_typeck::check_crate                               | 整形式チェック                                         | Map::par_for_each_module |

並列イテレータを使用する可能性のあるループはまだ多数あります。

## クエリシステム

クエリモデルには、過度の労力なしに複数のクエリを並列に評価することを実際に実現可能にするいくつかのプロパティがあります：

- クエリプロバイダがアクセスできるすべてのデータはクエリコンテキストを介して行われるため、クエリコンテキストがアクセスの同期を処理できます。
- クエリ結果は不変である必要があるため、複数のスレッドで同時に安全に使用できます。

クエリ `foo` が評価されると、`foo` のキャッシュテーブルがロックされます。

- すでに結果がある場合は、それをクローンしてロックを解放し、完了です。
- キャッシュエントリがなく、同じ結果を計算している他のアクティブなクエリ呼び出しがない場合は、キーを「進行中」としてマークし、ロックを解放して評価を開始します。
- 同じキーに対して進行中の*別の*クエリ呼び出しがある場合は、ロックを解放し、待っている結果を他の呼び出しが計算するまでスレッドをブロックします。並列コンパイラでの**サイクルエラー検出**は、シングルスレッドモードよりも複雑なロジックを必要とします。並列クエリ内のワーカースレッドが相互依存のために進行しなくなった場合、コンパイラは追加のスレッド*（デッドロックハンドラという名前）*を使用してサイクルエラーを検出、削除、報告します。

並列クエリ機能にはまだ実装が必要な部分があり、そのほとんどは以前の `データ構造` と `並列イテレータ` に関連しています。この[オープンな機能トラッキングissue][tracking]を参照してください。

## Rustdoc

<!-- date-check--> 2022年11月時点で、`rustdoc` レンダリングを並列化する前に完了する必要があるステップがまだいくつかあります（[並列 `rustdoc`][parallel-rustdoc]のオープンな議論を参照してください）。

## リソース

ここに、さらに学ぶために使用できるいくつかのリソースがあります：

- [alexchrichtonによるパフォーマンスに関するこのIRLOスレッド][irlo1]
- [Zoxc（この取り組みの先駆者の1人）によるこのIRLOスレッド][irlo0]
- [nikomatsakisによるコンパイラ内の内部可変性のこのリスト][imlist]

[`rayon`]: https://crates.io/crates/rayon
[imlist]: https://github.com/nikomatsakis/rustc-parallelization/blob/master/interior-mutability-list.md
[irlo0]: https://internals.rust-lang.org/t/parallelizing-rustc-using-rayon/6606
[irlo1]: https://internals.rust-lang.org/t/help-test-parallel-rustc/11503
[rustc-rayon]: https://github.com/rust-lang/rustc-rayon
[tracking]: https://github.com/rust-lang/rust/issues/48685
