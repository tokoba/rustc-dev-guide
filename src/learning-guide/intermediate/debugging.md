# デバッグ手法

このセクションでは、Rustコンパイラ特有のデバッグ技術と効率的な問題特定方法について学習します。コンパイラの複雑な問題を解決するための実践的なスキルを身につけましょう。

## コンパイラデバッグの基本

### デバッグの特徴

#### 一般的なデバッグとの違い

| 特徴 | 一般的なアプリケーション | コンパイラ |
|------|------------------|----------|
| 対象 | ランタイムの振る舞い | コンパイル時の処理 |
| データ | アプリケーション状態 | AST、MIR、型情報 |
| ツール | デバッガ、ロガー | コンパイラフラグ、ダンプ |
| 問題 | クラッシュ、バグ | エラー、パフォーマンス |

#### コンパイラデバッグの課題

- **複雑なデータ構造**: AST、MIR、型情報など
- **多段階の変換**: ソース→AST→HIR→MIR→LLVM IR
- **大量のデータ**: 大規模なコードベースの処理
- **抽象度の高い処理**: 最適化や変換の複雑さ

### デバッグの基本アプローチ

#### 1. 問題の分類

```bash
# エラーの種類を特定
rustc --error-format=json 2>&1 | jq '.message'
```

**エラーの分類**:

- **パースエラー**: 構文解析の問題
- **型エラー**: 型チェックの問題
- **借用エラー**: 借用チェックの問題
- **最適化エラー**: 最適化パスの問題
- **コード生成エラー**: バックエンドの問題

#### 2. 再現ケースの特定

```rust
// 最小再現ケースの作成例
fn main() {
    // 問題の核心部分のみを抽出
    let x = 5;
    let y = &x;
    let z = x; // ここで問題が発生
    println!("{}", y);
}
```

#### 3. 段階的な分離

```bash
# 各フェーズで問題を特定
rustc -Z parse-only                    # パースのみ
rustc -Z no-analysis                  # 解析なし
rustc -Z mir-opt-level=0             # MIR最適化なし
rustc -Z codegen-backend=llvm          # 特定バックエンド
```

## コンパイラ特有のデバッグツール

### 1. コンパイラフラグの活用

#### 基本的なデバッグフラグ

```bash
# 詳細な出力
rustc -Z verbose                    # 詳細なコンパイル情報
rustc -Z time-passes                 # 各パスの実行時間
rustc -Z input-stats                 # 入力の統計情報

# 中間表現の出力
rustc -Z ast-json                    # ASTのJSON出力
rustc -Z hir-tree                   # HIRのツリー出力
rustc -Z dump-mir=all               # MIRのダンプ
rustc -Z emit=llvm-ir               # LLVM IRの出力
```

#### 型チェック関連のフラグ

```bash
# 型推論の詳細
rustc -Z infer-dump                  # 型推論のダンプ
rustc -Z trace-macros               # マクロ展開のトレース
rustc -Z borrowck=mir              # 借用チェックのMIR出力

# 型チェックの詳細
rustc -Z polymorphize=debug         # 多相化のデバッグ
rustc -Z identify_regions            # 領域の特定
```

#### 最適化関連のフラグ

```bash
# MIR最適化
rustc -Z mir-opt-level=3            # 最適化レベル指定
rustc -Z dump-mir-graphviz          # MIRのGraphviz出力
rustc -Z unpretty=mir-cfg          # 制御フローグラフの出力

# LLVM最適化
rustc -Z llvm-stats                 # LLVMの統計
rustc -Z llvm-time-passes            # LLVMパスの実行時間
```

### 2. ログとトレース

#### 環境変数によるログ制御

```bash
# ログレベルの設定
export RUST_LOG=debug               # デバッグレベル
export RUST_LOG=rustc::middle::typeck=trace  # 特定モジュール

# トレースの有効化
export RUSTC_LOG=debug              # コンパイラのデバッグログ
export RUSTC_FORCE_UNSTABLE=1       #不安定な機能の有効化
```

#### ログ出力の例

```bash
# 型チェックのトレース
rustc -Z trace-macros -Z input-stats file.rs 2> trace.log

# パフォーマンスのプロファイル
rustc -Z time-passes -Z llvm-stats file.rs 2> perf.log
```

### 3. デバッガの活用

#### GDBによるデバッグ

```bash
# デバッグビルドの作成
./x.py build --stage 1 --debug

# GDBでのデバッグ
gdb --args rustc -Z verbose file.rs

# ブレークポイントの設定
(gdb) break rustc::middle::typeck::check::fn_ctxt::check_fn
(gdb) run
```

#### LLDBによるデバッグ（macOS）

```bash
# LLDBでのデバッグ
lldb -- rustc -Z verbose file.rs

# ブレークポイントの設定
(lldb) breakpoint set --name check_fn
(lldb) run
```

## 各コンポーネントのデバッグ

### 1. パーサーのデバッグ

#### パースエラーの特定

```bash
# トークンの出力
rustc -Z parse-tree file.rs

# パースの詳細
rustc -Z ast-json file.rs | jq '.'
```

#### パーサーの内部状態

```rust
// パーサーのデバッグ出力
impl<'a> Parser<'a> {
    fn debug_token(&self) {
        eprintln!("Current token: {:?}", self.token);
        eprintln!("Position: {:?}", self.span);
    }
}
```

### 2. 型チェックのデバッグ

#### 型推論の追跡

```bash
# 型推論の詳細な出力
rustc -Z infer-dump -Z verbose file.rs

# 制約の出力
rustc -Z dump-typeck-data file.rs
```

#### 型エラーのデバッグ

```rust
// 型チェックのデバッグ実装
impl<'tcx> TypeChecker<'tcx> {
    fn debug_type(&self, ty: Ty<'tcx>) {
        eprintln!("Type: {}", self.tcx.ty_to_string(ty));
        eprintln!("Kind: {:?}", ty.kind());
    }
}
```

### 3. MIRのデバッグ

#### MIRの可視化

```bash
# MIRのテキスト出力
rustc -Z dump-mir=all file.rs

# MIRのGraphviz出力
rustc -Z dump-mir=graphviz file.rs
dot -Tpng mir.dot -o mir.png
```

#### MIR最適化の追跡

```bash
# 各最適化パスの前後のMIR
rustc -Z dump-mir=all -Z mir-opt-level=3 file.rs

# 特定の最適化パス
rustc -Z dump-mir=ConstProp -Z mir-opt-level=3 file.rs
```

### 4. 借用チェックのデバッグ

#### 借用の可視化

```bash
# 借用チェックのMIR出力
rustc -Z borrowck=mir file.rs

# 領域推論の詳細
rustc -Z identify_regions file.rs
```

#### 借用エラーの分析

```rust
// 借用チェックのデバッグ実装
impl<'cx, 'tcx> BorrowChecker<'cx, 'tcx> {
    fn debug_borrow(&self, borrow: &BorrowData<'tcx>) {
        eprintln!("Borrow: {:?}", borrow);
        eprintln!("Place: {}", self.borrowckcx.place_to_string(&borrow.place));
    }
}
```

## パフォーマンスデバッグ

### 1. プロファイリングツール

#### perfによるプロファイリング

```bash
# CPUプロファイリング
perf record --call-graph=dwarf rustc -Z time-passes file.rs
perf report

# メモリプロファイリング
perf record -g rustc -Z time-passes file.rs
perf report --stdio | grep rustc
```

#### time-passesによる分析

```bash
# 各パスの実行時間
rustc -Z time-passes file.rs

# 詳細な時間計測
rustc -Z time-passes -Z time-llvm-passes file.rs
```

### 2. メモリ使用量の分析

#### メモリプロファイリング

```bash
# Valgrindによるメモリチェック
valgrind --tool=massif rustc file.rs
ms_print massif.out.*

# メモリ使用量の統計
rustc -Z input-stats -Z memory-stats file.rs
```

#### メモリリークの検出

```bash
# メモリリークの検出
valgrind --tool=memcheck --leak-check=full rustc file.rs
```

## 実践的なデバッグ手法

### 1. 最小再現ケースの作成

#### 問題の分離

```rust
// 元の問題コード
fn complex_function<T: Clone>(data: Vec<T>) -> Vec<T> {
    // 複雑な処理
}

// 最小化したケース
fn minimal_case() {
    let x = 5;
    let y = &x;
    let z = x; // 問題の核心
}
```

#### 自動化による最小化

```bash
# creduceによる自動最小化
creduce --n 10 test.sh original.rs
```

### 2. バイセクトによる問題特定

#### バイセクトの実行

```bash
# コンパイラのバイセクト
git bisect start
git bisect bad HEAD
git bisect good v1.50.0
git bisect run ./test.sh

# テストスクリプトの例
#!/bin/bash
./x.py build --stage 1
./build/stage1/bin/rustc test_file.rs
```

### 3. デバッグ用のコード追加

#### ログ出力の追加

```rust
// デバッグ用のログ出力
impl<'tcx> TypeChecker<'tcx> {
    fn check_expr(&mut self, expr: &Expr) {
        if log_enabled!(Debug) {
            debug!("Checking expr: {:?}", expr);
        }
        // 既存の処理
    }
}
```

#### アサーションの追加

```rust
// デバッグ用のアサーション
impl<'tcx> MirBuilder<'tcx> {
    fn assert_valid_mir(&self, body: &Body<'tcx>) {
        debug_assert!(self.validate_mir(body), "Invalid MIR: {:?}", body);
    }
}
```

## 高度なデバッグ技術

### 1. カスタムデバッグツール

#### MIRビューアの作成

```rust
// 簡単なMIRビューア
fn view_mir(mir: &Body) {
    for (bb_id, bb_data) in mir.basic_blocks.iter_enumerated() {
        println!("Basic Block {:?}", bb_id);
        for statement in &bb_data.statements {
            println!("  {:?}", statement);
        }
        if let Some(terminator) = &bb_data.terminator {
            println!("  {:?}", terminator);
        }
    }
}
```

#### 型可視化ツール

```rust
// 型の可視化
fn visualize_type(tcx: TyCtxt, ty: Ty) -> String {
    match ty.kind() {
        TyKind::Int(int_ty) => format!("int{}", int_ty.bit_width()),
        TyKind::Adt(adt, substs) => format!("{}<{}>", tcx.def_path_str(adt.did), ...),
        // 他の型の処理
    }
}
```

### 2. 自動化テスト

#### 回帰テストの作成

```rust
// tests/ui/debugging-issue.rs
fn main() {
    let x = 5;
    let y = &x;
    let z = x; // エラーを期待
    println!("{}", y);
}
```

```rust
// tests/ui/debugging-issue.stderr
error[E0505]: cannot move out of `x` because it is borrowed
 --> debugging-issue.rs:4:9
  |
3 |     let y = &x;
  |              - borrow of `x` occurs here
4 |     let z = x;
  |         ^ move out of `x` occurs here
```

## トラブルシューティング

### よくあるデバッグの課題

#### 1. 情報過多

- **問題**: 出力が多すぎて重要な情報が見つけられない
- **解決**: フィルタリングと特定の情報に焦点
- **ツール**: `grep`, `jq`, ログレベルの調整

#### 2. 再現性の問題

- **問題**: 特定の環境でのみ発生する問題
- **解決**: 環境の分離と最小化
- **ツール**: Docker, バイセクト, 最小再現ケース

#### 3. パフォーマンスの問題

- **問題**: デバッグが遅すぎる
- **解決**: 効率的なデバッグ手法の選択
- **ツール**: プロファイリング, キャッシュの活用

### デバッグのベストプラクティス

#### 1. 系統的なアプローチ

- **仮説の設定**: 問題の原因を推測
- **検証**: 仮説をテスト
- **反復**: 仮説を修正し繰り返す

#### 2. 効率的な情報収集

- **関連情報のみ**: 必要な情報に絞る
- **段階的な詳細化**: 概要から詳細へ
- **記録の保持**: 発見したことを記録

#### 3. ツールの適切な選択

- **問題に応じたツール**: 各ツールの長所を理解
- **組み合わせ**: 複数のツールを組み合わせる
- **カスタマイズ**: 独自のツールを作成

## 関連ドキュメント

より詳細な情報については、以下のドキュメントを参照してください：

- [コンパイラのデバッグ](../../compiler-debugging.md) - デバッグの詳細なガイド
- [プロファイリング](../../profiling.md) - パフォーマンス分析
- [トレーシング/ログ計装の使用](../../tracing.md) - ログとトレース
- [コンパイラのテスト](../../tests/intro.md) - テストの書き方

## 次のステップ

デバッグ手法を学習したら、次は[実践演習](./practical-exercises.md)に進みましょう。実際のコンパイラ課題に取り組むことで、学んだデバッグ技術を実践に活かすことができます。
