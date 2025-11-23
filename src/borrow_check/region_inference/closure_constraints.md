# クロージャ制約の伝播

型テストと全称領域をチェックするとき、クロージャ本体内にいる場合、まだ証明できない制約に遭遇することがあります！しかし、必要な制約は実際には成立するかもしれません（ただ、まだわからないだけです）。したがって、クロージャ内にいる場合、まだ証明できないすべての制約を収集して返します。後で、クロージャを作成した MIR ノードを borrow check するときに、これらの制約が成立することもチェックできます。その時点で、成立することを証明できない場合、エラーを報告します。

## これがどのように実装されているか

`RegionInferenceContext::solve` 内でクロージャを borrow check する際、ローカルで証明できない場合、型 outlives および領域 outlives 制約を親に伝播しようと別々に試みます。

### 領域 outlive 制約

`RegionInferenceContext::check_universal_regions` が何らかの outlives 制約 `'longer_fr: 'shorter_fr` を証明できない場合、`fn try_propagate_universal_region_error` で伝播しようとします。これらの全称領域は両方とも、クロージャにローカルまたは外部領域のいずれかです。

`'longer_fr` がローカル全称領域の場合、`'longer_fr` によって生存される最大の外部領域 `'fr_minus` を検索します。つまり、`'longer_fr: 'fr_minus`。複数のそのような領域がある場合、`mutual_immediate_postdominator` を選びます: すべての GLB の GLB を繰り返し計算する不動点です。詳細については [TransitiveRelation::postdom_upper_bound](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_data_structures/transitive_relation/struct.TransitiveRelation.html#method.postdom_upper_bound) を参照してください。

`'fr_minus` が存在する場合、`'shorter_fr` のすべての非ローカル上界を生存するよう要求します。常に少なくとも 1 つの非ローカル上界 `'static` があります。

### 型 outlive 制約

型 outlives 制約は `check_type_tests` で証明されます。これは outlives グラフの計算後に行われ、グラフは今は不変です。

クロージャ内で `fn eval_verify_bound` 経由で証明できないすべての型テストについて、`try_promote_type_test` を呼び出します。`TypeTest` は、`verify_bound` とともに型 outlives 境界 `generic_kind: lower_bound` を表します。`VerifyBound` が `lower_bound` に対して成立する場合、制約は満たされます。`try_promote_type_test` は `verify_bound` を気にしません。

まず、`fn try_promote_type_test_subject` を呼び出します。この関数は `GenericKind` を受け取り、クロージャにローカルなものを参照しなくなった `ClosureOutlivesSubject` に変換しようとします。これは、その型のすべての自由領域を `'static` または、その自由領域と等しい領域パラメータのいずれかに置き換えることによって行われます。この操作は、`generic_kind` に置き換えられない領域が含まれている場合に失敗します。

次に、`lower_bound` を呼び出し元のコンテキストに昇格させます。下界がプレースホルダーと等しい場合、それを `'static` に置き換えます

次に、`lower_bound` によって生存されることが要求されるすべての全称領域 `uv`、つまり borrow checking が領域制約を追加したものを見ます。これらのそれぞれについて、`uv` を生存することが知られているすべての非ローカル全称領域に対して `ClosureOutlivesRequirement` を発行します。

この時点で、クロージャの領域グラフを既に構築し、それが一貫していることを別々にチェックしているため、ここで outlive 制約 `uv: lower_bound` を仮定することもできます。

したがって、証明できない型 outlives 境界がある場合、例えば `T: 'local_infer`、領域グラフを使用して `'a: local_infer` を持つ全称変数 `'a` に移動します。`'a` がローカルの場合、仮定された outlived 制約を使用して非ローカルのものに移動します。

次に、昇格された型テストのリストを `BorrowCheckResults` に格納します。
その親を borrow check する際に、`TypeChecker::prove_closure_bounds` でそれらを適用します。

TODO: それが正確にどのように機能するかを説明する :3
