# rustc-dev-guide 学習ガイド 既存SUMMARY.mdへの統合方法

## 統合の基本方針

1. **既存構造の尊重**: 現在のSUMMARY.mdの構造を破壊せず、学習ガイドを新しいセクションとして追加
2. **論理的な配置**: 学習者が自然な流れで学習ガイドにたどり着けるような配置
3. **相互参照の促進**: 既存の技術ドキュメントと学習ガイド間の円滑な移動
4. **段階的導入**: 学習ガイドの導入によって既存ユーザーの混乱を最小限に抑える

## 現在のSUMMARY.md構造の分析

### 現在の主要セクション

1. はじめに (getting-started.md)
2. このガイドについて (about-this-guide.md)
3. `rustc` のビルドとデバッグ
4. Rust への貢献
5. ブートストラップ
6. コンパイラの高レベルアーキテクチャ
7. ソースコード表現
8. サポートインフラストラクチャ
9. 解析
10. MIR からバイナリへ
11. 付録

## 提案する統合構造

### 1. 学習ガイドセクションの追加位置

学習ガイドは「はじめに」セクションの直後に配置することを提案します。

**理由**:

- 初心者が技術的な詳細に入る前に、体系的な学習パスを認識できる
- 既存の「このガイドについて」との相乗効果
- 学習ガイドから既存の技術ドキュメントへの自然な移行

### 2. 新しいSUMMARY.mdの完全な構造

```markdown
# 目次

[はじめに](./getting-started.md)

[このガイドについて](./about-this-guide.md)

---

# 学習ガイド

- [学習ガイドの概要](./learning-guide/README.md)
  - [学習パスの選択ガイド](./learning-guide/README.md#学習パスの選択)
  - [学習の進め方](./learning-guide/README.md#学習の進め方)
  
- [初級者向け学習パス](./learning-guide/getting-started/README.md)
  - [学習目標と計画](./learning-guide/getting-started/README.md#学習目標と計画)
  - [コンパイラの基本概念](./learning-guide/getting-started/compiler-basics.md)
  - [最初の貢献](./learning-guide/getting-started/first-contribution.md)
  - [ビルドとデバッグの基礎](./learning-guide/getting-started/build-and-debug.md)
  
- [中級者向け学習パス](./learning-guide/intermediate/README.md)
  - [学習目標と計画](./learning-guide/intermediate/README.md#学習目標と計画)
  - [コンパイラアーキテクチャ](./learning-guide/intermediate/compiler-architecture.md)
  - [ソースコード表現](./learning-guide/intermediate/source-representation.md)
  - [型システム](./learning-guide/intermediate/type-system.md)
  
- [上級者向け学習パス](./learning-guide/advanced/README.md)
  - [学習目標と計画](./learning-guide/advanced/README.md#学習目標と計画)
  - [クエリシステム](./learning-guide/advanced/query-system.md)
  - [最適化](./learning-guide/advanced/optimization.md)
  - [コード生成](./learning-guide/advanced/code-generation.md)
  
- [特定目的別学習パス](./learning-guide/specialization/README.md)
  - [学習目的別の分類](./learning-guide/specialization/README.md#学習目的別の分類)
  - [診断システム](./learning-guide/specialization/diagnostics.md)
  - [テストインフラ](./learning-guide/specialization/testing.md)
  - [パフォーマンス](./learning-guide/specialization/performance.md)
  
- [学習リソース](./learning-guide/resources/README.md)
  - [演習問題](./learning-guide/resources/exercises.md)
  - [実践プロジェクト](./learning-guide/resources/projects.md)
  - [参考資料](./learning-guide/resources/references.md)

---

# `rustc` のビルドとデバッグ

- [コンパイラのビルドと実行方法](./building/how-to-build-and-run.md)
  - [クイックスタート](./building/quickstart.md)
  - [前提条件](./building/prerequisites.md)
  - [推奨ワークフロー](./building/suggested.md)
  - [配布アーティファクト](./building/build-install-distribution-artifacts.md)
  - [ドキュメントのビルド](./building/compiler-documenting.md)
  - [Rustdoc 概要](./rustdoc.md)
  - [新しいターゲットの追加](./building/new-target.md)
  - [最適化ビルド](./building/optimized-build.md)
- [コンパイラのテスト](./tests/intro.md)
  - [テストの実行](./tests/running.md)
    - [Docker でのテスト](./tests/docker.md)
    - [CI でのテスト](./tests/ci.md)
  - [新しいテストの追加](./tests/adding.md)
  - [ベストプラクティス](./tests/best-practices.md)
  - [Compiletest](./tests/compiletest.md)
    - [UI テスト](./tests/ui.md)
    - [テストディレクティブ](./tests/directives.md)
    - [Minicore](./tests/minicore.md)
  - [エコシステムテスト](./tests/ecosystem.md)
    - [Crater](./tests/crater.md)
    - [Fuchsia](./tests/ecosystem-test-jobs/fuchsia.md)
    - [Rust for Linux](./tests/ecosystem-test-jobs/rust-for-linux.md)
  - [コード生成バックエンドのテスト](./tests/codegen-backend-tests/intro.md)
    - [Cranelift コード生成バックエンド](./tests/codegen-backend-tests/cg_clif.md)
    - [GCC コード生成バックエンド](./tests/codegen-backend-tests/cg_gcc.md)
  - [パフォーマンステスト](./tests/perf.md)
  - [その他の情報](./tests/misc.md)
- [コンパイラのデバッグ](./compiler-debugging.md)
  - [トレーシング/ログ計装の使用](./tracing.md)
- [コンパイラのプロファイリング](./profiling.md)
  - [Linux perf ツールを使用](./profiling/with_perf.md)
  - [Windows Performance Analyzer を使用](./profiling/wpa_profiling.md)
  - [Rust ベンチマークスイートを使用](./profiling/with_rustc_perf.md)
- [crates.io の依存関係](./crates-io.md)

# Rust への貢献

- [貢献手順](./contributing.md)
- [コンパイラチームについて](./compiler-team.md)
- [Git の使用](./git.md)
- [@rustbot の使い方](./rustbot.md)
- [ウォークスルー：典型的な貢献](./walkthrough.md)
- [新しい言語機能の実装](./implementing_new_features.md)
- [安定性属性](./stability.md)
- [言語機能の安定化](./stabilization_guide.md)
  - [安定化レポートテンプレート](./stabilization_report_template.md)
- [フィーチャーゲート](./feature-gates.md)
- [コーディング規約](./conventions.md)
- [破壊的変更の手順](./bug-fix-procedure.md)
- [外部リポジトリの使用](./external-repos.md)
- [ファジング](./fuzzing.md)
- [通知グループ](notification-groups/about.md)
  - [Apple](notification-groups/apple.md)
  - [ARM](notification-groups/arm.md)
  - [Emscripten](notification-groups/emscripten.md)
  - [Fuchsia](notification-groups/fuchsia.md)
  - [RISC-V](notification-groups/risc-v.md)
  - [Rust for Linux](notification-groups/rust-for-linux.md)
  - [WASI](notification-groups/wasi.md)
  - [WebAssembly](notification-groups/wasm.md)
  - [Windows](notification-groups/windows.md)
- [ライセンス](./licenses.md)
- [エディション](guides/editions.md)

# ブートストラップ

- [序文](./building/bootstrapping/intro.md)
- [ブートストラップが行うこと](./building/bootstrapping/what-bootstrapping-does.md)
- [Bootstrap の仕組み](./building/bootstrapping/how-bootstrap-does-it.md)
- [Bootstrap でのツール作成](./building/bootstrapping/writing-tools-in-bootstrap.md)
- [bootstrap のデバッグ](./building/bootstrapping/debugging-bootstrap.md)
- [依存関係における cfg(bootstrap)](./building/bootstrapping/bootstrap-in-dependencies.md)

# コンパイラの高レベルアーキテクチャ

- [序文](./part-2-intro.md)
- [コンパイラの概要](./overview.md)
- [コンパイラのソースコード](./compiler-src.md)
- [クエリ：デマンド駆動コンパイル](./query.md)
  - [クエリ評価モデルの詳細](./queries/query-evaluation-model-in-detail.md)
  - [インクリメンタルコンパイル](./queries/incremental-compilation.md)
  - [インクリメンタルコンパイルの詳細](./queries/incremental-compilation-in-detail.md)
  - [デバッグとテスト](./incrcomp-debugging.md)
  - [Salsa](./queries/salsa.md)
- [rustc のメモリ管理](./memory.md)
- [rustc のシリアライゼーション](./serialization.md)
- [並列コンパイル](./parallel-rustc.md)
- [Rustdoc の内部](./rustdoc-internals.md)
  - [検索](./rustdoc-internals/search.md)
  - [`rustdoc` テストスイート](./rustdoc-internals/rustdoc-test-suite.md)
  - [`rustdoc-gui` テストスイート](./rustdoc-internals/rustdoc-gui-test-suite.md)
  - [`rustdoc-json` テストスイート](./rustdoc-internals/rustdoc-json-test-suite.md)
- [GPU オフロードの内部](./offload/internals.md)
  - [インストール](./offload/installation.md)
  - [使用方法](./offload/usage.md)
- [Autodiff の内部](./autodiff/internals.md)
  - [インストール](./autodiff/installation.md)
  - [デバッグ方法](./autodiff/debugging.md)
  - [Autodiff フラグ](./autodiff/flags.md)
  - [型ツリー](./autodiff/type-trees.md)

# ソースコード表現

- [序文](./part-3-intro.md)
- [構文と AST](./syntax-intro.md)
  - [字句解析と構文解析](./the-parser.md)
  - [マクロ展開](./macro-expansion.md)
  - [名前解決](./name-resolution.md)
  - [属性](./attributes.md)
  - [`#[test]` の実装](./test-implementation.md)
  - [panic の実装](./panic-implementation.md)
  - [AST 検証](./ast-validation.md)
  - [フィーチャーゲートチェック](./feature-gate-ck.md)
  - [言語アイテム](./lang-items.md)
- [HIR（高レベル中間表現）](./hir.md)
  - [AST から HIR への低レベル化](./hir/lowering.md)
  - [曖昧/非曖昧な型と定数](./hir/ambig-unambig-ty-and-consts.md)
  - [デバッグ](./hir/debugging.md)
- [THIR（型付き高レベル中間表現）](./thir.md)
- [MIR（中レベル中間表現）](./mir/index.md)
  - [MIR の構築](./mir/construction.md)
  - [MIR のビジターと走査](./mir/visitor.md)
  - [MIR のクエリとパス：MIR の取得](./mir/passes.md)
- [インラインアセンブリ](./asm.md)

# サポートインフラストラクチャ

- [コマンドライン引数](./cli.md)
- [rustc_driver と rustc_interface](./rustc-driver/intro.md)
  - [永続的に不安定な機能に関する注意](./rustc-driver/remarks-on-perma-unstable-features.md)
  - [例：型チェック](./rustc-driver/interacting-with-the-ast.md)
  - [例：診断の取得](./rustc-driver/getting-diagnostics.md)
- [エラーと lint](diagnostics.md)
  - [診断とサブ診断の構造体](./diagnostics/diagnostic-structs.md)
  - [翻訳](./diagnostics/translation.md)
  - [`LintStore`](./diagnostics/lintstore.md)
  - [エラーコード](./diagnostics/error-codes.md)
  - [診断アイテム](./diagnostics/diagnostic-items.md)
  - [`ErrorGuaranteed`](./diagnostics/error-guaranteed.md)

# 解析

- [序文](./part-4-intro.md)
- [ジェネリックパラメータの定義](./generic_parameters_summary.md)
  - [`EarlyBinder` とパラメータのインスタンス化](./ty_module/early_binder.md)
- [バインダーと高ランク領域](./ty_module/binders.md)
  - [バインダーのインスタンス化](./ty_module/instantiating_binders.md)
- [早期 vs 後期バウンドパラメータ](./early_late_parameters.md)
- [`ty` モジュール：型の表現](./ty.md)
  - [ADT とジェネリック引数](./ty_module/generic_arguments.md)
  - [パラメータ型/定数/領域](./ty_module/param_ty_const_regions.md)
- [`TypeFolder` と `TypeFoldable`](./ty-fold.md)
- [エイリアスと正規化](./normalization.md)
- [型付け/パラメータ環境](./typing_parameter_envs.md)
- [型推論](./type-inference.md)
- [トレイト解決](./traits/resolution.md)
  - [高ランクトレイト境界](./traits/hrtb.md)
  - [キャッシュの微妙な点](./traits/caching.md)
  - [暗黙の境界](./traits/implied-bounds.md)
  - [特殊化](./traits/specialization.md)
  - [Chalk ベースのトレイト解決](./traits/chalk.md)
    - [論理への低レベル化](./traits/lowering-to-logic.md)
    - [ゴールと節](./traits/goals-and-clauses.md)
    - [正準クエリ](./traits/canonical-queries.md)
    - [正準化](./traits/canonicalization.md)
  - [次世代トレイト解決](./solve/trait-solving.md)
    - [型システムの不変条件](./solve/invariants.md)
    - [ソルバー](./solve/the-solver.md)
    - [候補の優先順位](./solve/candidate-preference.md)
    - [正準化](./solve/canonicalization.md)
    - [コインダクション](./solve/coinduction.md)
    - [キャッシング](./solve/caching.md)
    - [証明木](./solve/proof-trees.md)
    - [不透明型](./solve/opaque-types.md)
    - [重要な変更と癖](./solve/significant-changes.md)
  - [`Unsize` と `CoerceUnsized` トレイト](./traits/unsize.md)
- [型チェック](./type-checking.md)
  - [メソッド検索](./method-lookup.md)
  - [変性](./variance.md)
  - [コヒーレンスチェック](./coherence.md)
  - [不透明型](./opaque-types-type-alias-impl-trait.md)
    - [推論の詳細](./opaque-types-impl-trait-inference.md)
    - [トレイトにおける戻り位置 Impl Trait](./return-position-impl-trait-in-trait.md)
    - [領域推論の制限][opaque-infer]
- [定数条件チェック](./effects.md)
- [パターンと網羅性チェック](./pat-exhaustive-checking.md)
- [安全性チェック](./unsafety-checking.md)
- [MIR データフロー](./mir/dataflow.md)
- [ドロップの詳細化](./mir/drop-elaboration.md)
- [借用チェッカー](./borrow_check.md)
  - [移動と初期化の追跡](./borrow_check/moves_and_initialization.md)
    - [移動パス](./borrow_check/moves_and_initialization/move_paths.md)
  - [MIR 型チェッカー](./borrow_check/type_check.md)
  - [ドロップチェック](./borrow_check/drop_check.md)
  - [領域推論](./borrow_check/region_inference.md)
    - [制約伝播](./borrow_check/region_inference/constraint_propagation.md)
    - [ライフタイムパラメータ](./borrow_check/region_inference/lifetime_parameters.md)
    - [メンバー制約](./borrow_check/region_inference/member_constraints.md)
    - [プレースホルダーと宇宙][pau]
    - [クロージャ制約](./borrow_check/region_inference/closure_constraints.md)
    - [エラー報告](./borrow_check/region_inference/error_reporting.md)
  - [2 フェーズ借用](./borrow_check/two_phase_borrows.md)
- [クロージャキャプチャ推論](./closure.md)
- [非同期クロージャー/「コルーチンクロージャー」](coroutine-closures.md)

# MIR からバイナリへ

- [序文](./part-5-intro.md)
- [MIR 最適化](./mir/optimizations.md)
- [MIR のデバッグ](./mir/debugging.md)
- [定数評価](./const-eval.md)
  - [インタープリタ](./const-eval/interpret.md)
- [単相化](./backend/monomorph.md)
- [MIR の低レベル化](./backend/lowering-mir.md)
- [コード生成](./backend/codegen.md)
  - [LLVM の更新](./backend/updating-llvm.md)
  - [LLVM のデバッグ](./backend/debugging.md)
  - [バックエンド非依存コード生成](./backend/backend-agnostic.md)
  - [暗黙の呼び出し元位置](./backend/implicit-caller-location.md)
- [ライブラリとメタデータ](./backend/libs-and-metadata.md)
- [プロファイルガイド最適化](./profile-guided-optimization.md)
- [LLVM ソースベースコードカバレッジ](./llvm-coverage-instrumentation.md)
- [サニタイザーのサポート](./sanitizers.md)
- [Rust コンパイラにおけるデバッグサポート](./debugging-support-in-rustc.md)

---

[付録 A：背景トピック](./appendix/background.md)

[付録 B：用語集](./appendix/glossary.md)

[付録 C：コードインデックス](./appendix/code-index.md)

[付録 D：コンパイラレクチャーシリーズ](./appendix/compiler-lecture.md)

[付録 E：参考文献](./appendix/bibliography.md)

[付録 Z：HumorRust](./appendix/humorust.md)

---

[pau]: ./borrow_check/region_inference/placeholders_and_universes.md
[opaque-infer]: ./borrow_check/opaque-types-region-inference-restrictions.md
```

## 統合の実装手順

### 1. 段階的導入

#### フェーズ1: 基本的な学習ガイドの追加

- 学習ガイドの基本構造のみをSUMMARY.mdに追加
- 各学習パスのREADME.mdのみを作成
- 既存ユーザーへの影響を最小限に抑える

#### フェーズ2: 詳細なコンテンツの追加

- 各学習パスの詳細なコンテンツを追加
- ナビゲーションリンクの実装
- 既存ドキュメントとの相互参照の追加

#### フェーズ3: 高度な機能の実装

- インタラクティブなナビゲーション機能
- 学習進捗の追跡機能
- コミュニティとの連携機能

### 2. 既存コンテンツとの関連付け

#### 学習ガイドから既存ドキュメントへの参照

```markdown
## 詳細情報
- [コンパイラの概要](../../overview.md) - 全体的なコンパイラアーキテクチャ
- [クエリ：デマンド駆動コンパイル](../../query.md) - クエリシステムの詳細
- [高レベルコンパイラアーキテクチャ](../../part-2-intro.md) - アーキテクチャの理論的背景
```

#### 既存ドキュメントから学習ガイドへの参照

既存のドキュメントに学習ガイドへのリンクを追加：

```markdown
<!-- about-this-guide.mdに追加 -->
## 学習リソース
体系的な学習を希望される方は、[学習ガイド](./learning-guide/README.md)をご利用ください。

<!-- getting-started.mdに追加 -->
## 体系的な学習
より体系的な学習を希望される方は、[学習ガイド](./learning-guide/README.md)を参照してください。
```

### 3. ユーザーエクスペリエンスの配慮

#### 新規ユーザー向けの導入

- はじめにセクションから学習ガイドへ自然に誘導
- 学習パスの選択ガイドの提供
- 初心者向けの明確な開始点の提示

#### 既存ユーザー向けの配慮

- 既存のナビゲーション構造の維持
- 学習ガイドがオプションであることの明示
- 既存ドキュメントへの直接的なアクセスの維持

### 4. メンテナンスと更新

#### コンテンツの同期

- 学習ガイドと既存ドキュメントの内容の整合性維持
- 既存ドキュメントの更新に合わせた学習ガイドの更新
- 重複コンテンツの管理

#### フィードバックの収集

- 学習ガイドの利用状況の監視
- ユーザーからのフィードバックの収集
- 継続的な改善の実施

## 技術的な実装詳細

### 1. mdBookの設定

#### book.tomlへの追加設定

```toml
[build]
build-dir = "book"

[output.html]
additional-css = ["learning-guide.css"]
additional-js = ["learning-guide.js"]

[output.html.playground]
editable = true
```

### 2. スタイルシートの追加

#### learning-guide.cssの作成

```css
/* 学習ガイド固有のスタイル */
.learning-path {
  background-color: #f8f9fa;
  border-left: 4px solid #007bff;
  padding: 1rem;
  margin: 1rem 0;
}

.progress-indicator {
  background-color: #e9ecef;
  border-radius: 0.25rem;
  height: 0.5rem;
  margin: 1rem 0;
}

.progress-fill {
  background-color: #007bff;
  height: 100%;
  border-radius: 0.25rem;
  transition: width 0.3s ease;
}
```

### 3. JavaScript機能の追加

#### learning-guide.jsの作成

```javascript
// 学習進捗の管理
class LearningProgress {
  constructor() {
    this.progress = this.loadProgress();
    this.updateProgressIndicators();
  }

  loadProgress() {
    const saved = localStorage.getItem('rustc-learning-progress');
    return saved ? JSON.parse(saved) : {};
  }

  saveProgress() {
    localStorage.setItem('rustc-learning-progress', JSON.stringify(this.progress));
  }

  markCompleted(topic) {
    this.progress[topic] = true;
    this.saveProgress();
    this.updateProgressIndicators();
  }

  updateProgressIndicators() {
    // 進捗インジケーターの更新
  }
}

// 初期化
document.addEventListener('DOMContentLoaded', () => {
  new LearningProgress();
});
```

## 成功の評価基準

### 1. 利用状況の指標

- 学習ガイドの訪問数と滞在時間
- 各学習パスの完了率
- 学習ガイドから既存ドキュメントへの移行率

### 2. ユーザー満足度

- フィードバックと評価
- 学習効果の自己評価
- コミュニティでの言及

### 3. 貢献への影響

- 新規貢献者の増加
- 貢献の質の向上
- 学習ガイド経由での貢献数

この統合方法により、学習ガイドは既存のrustc-dev-guideプロジェクトにシームレスに統合され、学習者にとって効果的なナビゲーションツールとなります。
