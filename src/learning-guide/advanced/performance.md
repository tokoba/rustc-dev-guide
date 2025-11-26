# パフォーマンス最適化

このセクションでは、Rustコンパイラのパフォーマンス分析と最適化手法について学習します。コンパイル時間、メモリ使用量、生成コードの品質など、様々な側面からのパフォーマンス改善を体系的に理解しましょう。

## パフォーマンス最適化の概要

### 最適化の分類

#### 1. コンパイラのパフォーマンス

- **コンパイル時間**: ソースコードのコンパイルにかかる時間
- **メモリ使用量**: コンパイル中のメモリ消費量
- **並列化**: 複数コアの活用効率
- **インクリメンタルコンパイル**: 変更時の再コンパイル効率

#### 2. 生成コードのパフォーマンス

- **実行速度**: 生成されたコードの実行速度
- **コードサイズ**: 生成された実行可能ファイルのサイズ
- **最適化レベル**: 適用される最適化の度合い
- **ターゲット最適化**: 特定アーキテクチャへの最適化

#### 3. リソース使用量

- **ディスクI/O**: ファイル読み書きの効率
- **ネットワーク**: 依存関係ダウンロードの効率
- **CPU使用率**: CPUリソースの活用効率
- **メモリアロケーション**: メモリ割り当ての効率

## パフォーマンス分析の手法

### 1. プロファイリングツール

#### perfによるCPUプロファイリング

```bash
# コンパイラ全体のプロファイリング
perf record --call-graph=dwarf rustc -Z time-passes large_crate.rs
perf report

# 特定のパスのプロファイリング
perf record -e cycles,instructions,cache-misses rustc -Z time-passes
perf annotate --symbolize rustc
```

#### メモリプロファイリング

```bash
# Valgrindによるメモリプロファイリング
valgrind --tool=massif rustc large_crate.rs
ms_print massif.out.*

# heaptrackによるメモリリーク検出
heaptrack rustc large_crate.rs
heaptrack_gui heaptrack.rustc.*
```

#### 自作プロファイラ

```rust
// 簡単なプロファイラの実装
use std::time::Instant;
use std::collections::HashMap;

struct Profiler {
    timings: HashMap<String, Vec<Duration>>,
    current: Option<(String, Instant)>,
}

impl Profiler {
    pub fn start_timing(&mut self, name: &str) {
        self.current = Some((name.to_string(), Instant::now()));
    }
    
    pub fn end_timing(&mut self) {
        if let Some((name, start)) = self.current.take() {
            let duration = start.elapsed();
            self.timings.entry(name).or_insert_with(Vec::new).push(duration);
        }
    }
    
    pub fn report(&self) {
        for (name, timings) in &self.timings {
            let total: Duration = timings.iter().sum();
            let avg = total / timings.len() as u32;
            println!("{}: total={:?}, avg={:?}", name, total, avg);
        }
    }
}

// 使用例
let mut profiler = Profiler::new();

profiler.start_timing("type_check");
// 型チェックの実行
profiler.end_timing();

profiler.start_timing("mir_optimization");
// MIR最適化の実行
profiler.end_timing();

profiler.report();
```

### 2. コンパイラ組み込みの計測

#### time-passesフラグ

```bash
# 各パスの実行時間を測定
rustc -Z time-passes -Z time-llvm-passes crate.rs

# 出力例
time: 0.123s type_check
time: 0.456s mir_optimization
time: 0.789s codegen
```

#### input-statsフラグ

```bash
# 入力の統計情報を収集
rustc -Z input-stats crate.rs

# 出力例
items: 1234
functions: 567
impls: 89
traits: 45
```

#### 自動計測の実装

```rust
// パスの自動計測
pub struct TimedPass<'a> {
    name: &'a str,
    start: Option<Instant>,
}

impl<'a> TimedPass<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name, start: None }
    }
    
    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }
    
    pub fn end(self) {
        if let Some(start) = self.start {
            let duration = start.elapsed();
            eprintln!("{}: {:?}", self.name, duration);
        }
    }
}

// 使用例
fn run_type_check<'tcx>(tcx: TyCtxt<'tcx>) {
    let mut timer = TimedPass::new("type_check");
    timer.start();
    
    // 型チェックの実装
    type_check_impl(tcx);
    
    timer.end();
}
```

## コンパイラのパフォーマンス最適化

### 1. 型チェックの最適化

#### 型推論の効率化

```rust
// 効率的な型統一アルゴリズム
impl<'a, 'tcx> InferCtxt<'a, 'tcx> {
    pub fn unify_optimized<T>(&mut self, a: T, b: T) -> InferResult<'tcx>
    where 
        T: ToTrace<'tcx>,
    {
        // 1. 早期リターンの最適化
        if a == b {
            return Ok(());
        }
        
        // 2. キャッシュの活用
        let cache_key = (a.trace(), b.trace());
        if let Some(cached_result) = self.unification_cache.get(&cache_key) {
            return cached_result.clone();
        }
        
        // 3. 効率的な統一処理
        let result = self.unify_impl(a, b);
        
        // 4. 結果のキャッシュ
        self.unification_cache.insert(cache_key, result.clone());
        
        result
    }
}
```

#### 制約解決の最適化

```rust
// 制約グラフの効率的な操作
pub struct ConstraintGraph<'tcx> {
    nodes: IndexVec<ConstraintNode, ConstraintNodeData<'tcx>>,
    edges: Vec<ConstraintEdge<'tcx>>,
    worklist: Vec<ConstraintNode>,
}

impl<'tcx> ConstraintGraph<'tcx> {
    pub fn solve_optimized(&mut self) -> Solution<'tcx> {
        // 1. ワークリストアルゴリズム
        while let Some(node) = self.worklist.pop() {
            if self.process_node(node) {
                // 変更があった場合、依存ノードを追加
                self.add_dependent_nodes(node);
            }
        }
        
        // 2. 解の構築
        self.build_solution()
    }
    
    fn process_node(&mut self, node: ConstraintNode) -> bool {
        let mut changed = false;
        
        // 3. 効率的な制約処理
        for edge in &self.edges[node] {
            if self.apply_constraint(edge) {
                changed = true;
            }
        }
        
        changed
    }
}
```

### 2. MIR最適化の改善

#### デッドコード削除の最適化

```rust
// 効率的なデッドコード削除
impl<'tcx> DeadCodeElimination<'tcx> {
    pub fn run_optimized(&mut self, body: &mut Body<'tcx>) {
        // 1. ライブ変数の分析
        let mut live_locals = self.analyze_live_locals(body);
        
        // 2. 段階的な削除
        let mut changed = true;
        while changed {
            changed = false;
            
            for (bb_id, bb_data) in body.basic_blocks.iter_enumerated_mut() {
                if self.remove_dead_statements(bb_data, &live_locals) {
                    changed = true;
                }
            }
            
            // 3. ライブ変数の再計算
            if changed {
                live_locals = self.analyze_live_locals(body);
            }
        }
        
        // 4. 未使用の基本ブロックの削除
        self.remove_dead_blocks(body, &live_locals);
    }
    
    fn analyze_live_locals(&self, body: &Body<'tcx>) -> BitSet<Local> {
        // 後方解析によるライブ変数の計算
        let mut live_locals = BitSet::new_empty(body.local_decls.len());
        
        for (bb_id, bb_data) in body.basic_blocks.iter().rev() {
            // ターミネータからのライブ変数の計算
            self.process_terminator(bb_data.terminator.as_ref(), &mut live_locals);
            
            // ステートメントの処理
            for stmt in bb_data.statements.iter().rev() {
                self.process_statement(stmt, &mut live_locals);
            }
        }
        
        live_locals
    }
}
```

#### 定数伝播の最適化

```rust
// 効率的な定数伝播
impl<'tcx> ConstantPropagation<'tcx> {
    pub fn run_optimized(&mut self, body: &mut Body<'tcx>) {
        // 1. 定数の追跡
        let mut constants = IndexMap::new();
        
        // 2. ワークリストアルゴリズム
        let mut worklist = VecDeque::new();
        worklist.push_back(START_BLOCK);
        
        while let Some(bb_id) = worklist.pop_front() {
            let bb_data = &body.basic_blocks[bb_id];
            let mut changed = false;
            
            // 3. 基本ブロックの処理
            for stmt in &bb_data.statements {
                if self.propagate_constants(stmt, &mut constants) {
                    changed = true;
                }
            }
            
            // 4. 変更があった場合、後続ブロックを追加
            if changed {
                if let Some(terminator) = &bb_data.terminator {
                    self.add_successor_blocks(terminator, &mut worklist);
                }
            }
        }
        
        // 5. 定数の適用
        self.apply_constants(body, &constants);
    }
}
```

### 3. メモリ使用量の最適化

#### アリーナアロケータの改善

```rust
// 効率的なアリーナアロケータ
pub struct OptimizedArena<'tcx> {
    chunks: Vec<ArenaChunk>,
    current_chunk: usize,
    free_list: Vec<Ptr<'tcx>>,
    marker: PhantomData<&'tcx ()>,
}

impl<'tcx> OptimizedArena<'tcx> {
    pub fn alloc<T>(&mut self, value: T) -> &'tcx T {
        // 1. フリーリストのチェック
        if let Some(ptr) = self.free_list.pop() {
            if ptr.size() >= std::mem::size_of::<T>() {
                return unsafe { ptr.cast::<T>().write(value) };
            }
        }
        
        // 2. 効率的な割り当て
        self.alloc_in_current_chunk(value)
    }
    
    pub fn dealloc<T>(&mut self, ptr: &'tcx T) {
        // 3. 効率的な解放
        let ptr = Ptr::from_ptr(ptr);
        self.free_list.push(ptr);
    }
    
    fn alloc_in_current_chunk<T>(&mut self, value: T) -> &'tcx T {
        if !self.current_chunk().can_alloc::<T>() {
            self.allocate_new_chunk();
        }
        
        let ptr = self.current_chunk().alloc::<T>();
        unsafe { ptr.write(value) }
    }
}
```

#### インターン化の最適化

```rust
// 効率的なインターン化
pub struct Interner<'tcx> {
    map: FxHashMap<InternKey<'tcx>, InternValue<'tcx>>,
    arena: Arena<'tcx>,
}

impl<'tcx> Interner<'tcx> {
    pub fn intern<T>(&mut self, value: T) -> Interned<T>
    where 
        T: Hash + Eq + 'tcx,
    {
        // 1. ハッシュ計算のキャッシュ
        let hash = self.calculate_hash(&value);
        
        // 2. 既存の値のチェック
        if let Some(existing) = self.map.get(&InternKey::new(&value, hash)) {
            return existing.cast::<T>();
        }
        
        // 3. 新しい値の割り当て
        let allocated = self.arena.alloc(value);
        let interned = Interned::new(allocated);
        
        // 4. マップへの登録
        self.map.insert(InternKey::new(&value, hash), interned.cast());
        
        interned
    }
}
```

## 生成コードの最適化

### 1. LLVM最適化の活用

#### 最適化パスの設定

```rust
// LLVM最適化パスのカスタマイズ
impl<'ll, 'tcx> CodegenCx<'ll, 'tcx> {
    pub fn configure_optimization_passes(&self, pm: &mut PassManagerBuilder) {
        // 基本的な最適化
        pm.add_instruction_combining_pass();
        pm.add_reassociate_pass();
        pm.add_gvn_pass();
        pm.add_cfg_simplification_pass();
        
        // 高度な最適化
        pm.add_loop_vectorize_pass();
        pm.add_slp_vectorizer_pass();
        pm.add_instruction_combining_pass();
        
        // ターゲット特有の最適化
        if self.tcx.sess.target.arch == "x86_64" {
            pm.add_x86_optimization_passes();
        }
    }
}
```

#### ターゲット特有の最適化

```rust
// x86_64ターゲットの最適化
pub struct X86_64Optimizer;

impl X86_64Optimizer {
    pub fn optimize_for_x86_64(&self, llfunc: &'ll llvm::Value) {
        // 1. SIMD命令の活用
        self.optimize_for_simd(llfunc);
        
        // 2. キャッシュの考慮
        self.optimize_for_cache(llfunc);
        
        // 3. 分岐予測の最適化
        self.optimize_for_branch_prediction(llfunc);
    }
    
    fn optimize_for_simd(&self, llfunc: &'ll llvm::Value) {
        // SIMD命令への変換
        self.vectorize_loops(llfunc);
        self.replace_scalar_with_vector(llfunc);
    }
    
    fn optimize_for_cache(&self, llfunc: &'ll llvm::Value) {
        // キャッシュフレンドリーなコード生成
        self.optimize_memory_access_patterns(llfunc);
        self.reduce_cache_misses(llfunc);
    }
}
```

### 2. コードサイズの最適化

#### LTO（Link Time Optimization）の活用

```rust
// LTOの設定と最適化
impl<'tcx> LtoOptimizer<'tcx> {
    pub fn optimize_with_lto(&self, modules: Vec<Module>) -> Module {
        // 1. モジュールのマージ
        let merged_module = self.merge_modules(modules);
        
        // 2. クロスモジュール最適化
        self.optimize_across_modules(&merged_module);
        
        // 3. デッドコードの削除
        self.remove_dead_code_across_modules(&merged_module);
        
        merged_module
    }
    
    fn optimize_across_modules(&self, module: &mut Module) {
        // インライン化の拡大
        self.aggressive_inlining(module);
        
        // 定数伝播の拡大
        self.interprocedural_constant_propagation(module);
        
        // デッドコード削除の拡大
        self.interprocedural_dead_code_elimination(module);
    }
}
```

## 並列化の最適化

### 1. クエリの並列化

#### 依存関係分析の最適化

```rust
// 効率的な並列クエリ実行
pub struct ParallelQueryExecutor<'tcx> {
    dependency_graph: DependencyGraph<'tcx>,
    thread_pool: ThreadPool,
    work_queue: WorkQueue<QueryJob<'tcx>>,
}

impl<'tcx> ParallelQueryExecutor<'tcx> {
    pub fn execute_parallel(&mut self, queries: Vec<QueryJob<'tcx>>) {
        // 1. 依存関係の分析
        let execution_groups = self.dependency_graph.topological_sort(queries);
        
        // 2. グループごとの並列実行
        for group in execution_groups {
            self.execute_group_parallel(group);
        }
    }
    
    fn execute_group_parallel(&mut self, group: Vec<QueryJob<'tcx>>) {
        // 3. ワークスチーリング
        let chunks = self.chunk_work(group);
        
        // 4. 並列実行
        let handles: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                self.thread_pool.spawn(move || {
                    self.execute_chunk(chunk)
                })
            })
            .collect();
        
        // 5. 結果の収集
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
```

### 2. MIR最適化の並列化

#### 基本ブロックの並列処理

```rust
// 並列MIR最適化
pub struct ParallelMirOptimizer<'tcx> {
    thread_pool: ThreadPool,
    work_divider: WorkDivider,
}

impl<'tcx> ParallelMirOptimizer<'tcx> {
    pub fn optimize_parallel(&self, body: &mut Body<'tcx>) {
        // 1. 依存関係の分析
        let independent_blocks = self.analyze_independence(body);
        
        // 2. 並列最適化
        let chunks = self.work_divider.divide_blocks(independent_blocks);
        
        let handles: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                self.thread_pool.spawn(move || {
                    self.optimize_chunk(chunk)
                })
            })
            .collect();
        
        // 3. 結果の統合
        for handle in handles {
            let optimized_chunk = handle.join().unwrap();
            self.merge_optimized_chunk(body, optimized_chunk);
        }
    }
    
    fn analyze_independence(&self, body: &Body<'tcx>) -> Vec<BasicBlock> {
        // 基本ブロック間の依存関係を分析
        let mut independent_blocks = Vec::new();
        let mut dependencies = BitSet::new_empty(body.basic_blocks.len());
        
        for (bb_id, bb_data) in body.basic_blocks.iter() {
            if !self.has_dependencies(bb_id, &dependencies) {
                independent_blocks.push(bb_id);
                self.mark_dependencies(bb_data, &mut dependencies);
            }
        }
        
        independent_blocks
    }
}
```

## ベンチマーキングと測定

### 1. ベンチマークの設計

#### 包括的なベンチマーク

```rust
// ベンチマークの実装
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_type_check(c: &mut Criterion) {
    let large_code = generate_large_code();
    
    c.bench_function("type_check", |b| {
        b.iter(|| {
            let tcx = create_typeck_context(&large_code);
            type_check(black_box(tcx))
        })
    });
}

fn benchmark_mir_optimization(c: &mut Criterion) {
    let mir = generate_complex_mir();
    
    c.bench_function("mir_optimization", |b| {
        b.iter(|| {
            let mut optimized_mir = mir.clone();
            optimize_mir(black_box(&mut optimized_mir))
        })
    });
}

fn benchmark_code_generation(c: &mut Criterion) {
    let mir = generate_optimized_mir();
    
    c.bench_function("code_generation", |b| {
        b.iter(|| {
            generate_llvm_ir(black_box(&mir))
        })
    });
}

criterion_group!(
    benches,
    benchmark_type_check,
    benchmark_mir_optimization,
    benchmark_code_generation
);
criterion_main!(benches);
```

### 2. リグレッション検出

#### 自動リグレッションテスト

```rust
// リグレッション検出の実装
pub struct RegressionDetector {
    baseline_metrics: HashMap<String, f64>,
    tolerance: f64,
}

impl RegressionDetector {
    pub fn check_regressions(&self, current_metrics: &HashMap<String, f64>) -> Vec<Regression> {
        let mut regressions = Vec::new();
        
        for (metric, current_value) in current_metrics {
            if let Some(baseline_value) = self.baseline_metrics.get(metric) {
                let regression_ratio = (current_value - baseline_value) / baseline_value;
                
                if regression_ratio > self.tolerance {
                    regressions.push(Regression {
                        metric: metric.clone(),
                        baseline: *baseline_value,
                        current: *current_value,
                        regression_ratio,
                    });
                }
            }
        }
        
        regressions
    }
}

#[derive(Debug)]
pub struct Regression {
    pub metric: String,
    pub baseline: f64,
    pub current: f64,
    pub regression_ratio: f64,
}
```

## 関連ドキュメント

より詳細な情報については、以下のドキュメントを参照してください：

- [プロファイリング](../../profiling.md) - パフォーマンス分析の詳細
- [MIR最適化](../../mir/optimizations.md) - MIR最適化の詳細
- [並列コンパイル](../../parallel-rustc.md) - 並列化の実装
- [プロファイルガイド最適化](../../profile-guided-optimization.md) - PGOの実装
- [コンパイラのテスト](../../tests/perf.md) - パフォーマンステスト

## 次のステップ

パフォーマンス最適化を学習したら、次は[特化パス](../specialization/README.md)で特定の分野に特化することを検討しましょう。より専門的な知識を深めることで、特定の分野での貢献ができるようになります。
