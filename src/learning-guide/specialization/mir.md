# MIR最適化

このセクションでは、MIR（中レベル中間表現）の最適化技術について深く学習します。データフロー分析、定数伝播、デッドコード削除など、コンパイラの最適化パスの実装と理論を体系的に理解しましょう。

## MIRの基本概念

### MIRの特徴と役割

#### 1. 中レベル中間表現としての特徴

```rust
// MIRの基本構造
pub struct Body<'tcx> {
    pub basic_blocks: IndexVec<BasicBlock, BasicBlockData<'tcx>>,
    pub local_decls: IndexVec<Local, LocalDecl<'tcx>>,
    pub user_type_annotations: IndexVec<UserTypeAnnotationIndex, UserTypeAnnotation>,
    pub arg_count: usize,
    pub span: Span,
    pub generator_kind: Option<GeneratorKind>,
}

// MIRの特徴
// - SSA形式: 各変数は一度だけ代入される
// - 制御フローグラフ: 基本ブロックとターミネータで構成
// - 型付き: 全ての値に型情報が付加されている
// - 低レベル: 高レベルな抽象化が取り除かれている
```

#### 2. 最適化に適した特性

- **静的単一代入**: 各基本ブロックの実行が入力値のみに依存
- **明確な制御フロー**: ジャンプと条件分岐が明確
- **型情報の豊富さ**: 最適化に必要な型情報が利用可能
- **副作用の明確化**: メモリアクセスや関数呼び出しが明確

### MIRの構造要素

#### 基本ブロックとターミネータ

```rust
// 基本ブロックの構造
pub struct BasicBlockData<'tcx> {
    pub statements: Vec<Statement<'tcx>>,
    pub terminator: Option<Terminator<'tcx>>,
    pub is_cleanup: bool,
}

// ステートメントの種類
pub enum StatementKind<'tcx> {
    Assign(Place<'tcx>, Rvalue<'tcx>), // 代入
    FakeRead(Place<'tcx>),           // 偽の読み込み
    SetDiscriminant(Place<'tcx>, VariantIdx), // 判別子の設定
    StorageLive(Local),                  // ストレージの有効化
    StorageDead(Local),                 // ストレージの無効化
    AscribeUserType(Place<'tcx>, UserTypeProjection, Variance), // ユーザー型の注釈
    Nop,                               // 何もしない
}

// ターミネータの種類
pub enum TerminatorKind<'tcx> {
    Goto { target: BasicBlock },                    // 無条件ジャンプ
    SwitchInt { discr: Operand<'tcx>, targets: SwitchTargets }, // 整数スイッチ
    Resume,                                      // 再開
    Abort,                                       // 中断
    Return,                                      // 関数からのリターン
    Unreachable,                                  // 到達不可能
    Drop { place: Place<'tcx>, target: BasicBlock, unwind: Option<BasicBlock> }, // ドロップ
    DropAndReplace { place: Place<'tcx>, value: Operand<'tcx>, target: BasicBlock, unwind: Option<BasicBlock> }, // ドロップと置換
    Call { func: Operand<'tcx>, args: Vec<Operand<'tcx>>, destination: Place<'tcx>, target: Option<BasicBlock>, cleanup: Option<BasicBlock>, from_hir_call: bool }, // 関数呼び出し
    Assert { cond: Operand<'tcx>, expected: bool, msg: Box<Constant<'tcx>>, target: BasicBlock, cleanup: Option<BasicBlock> }, // アサート
    Yield { value: Operand<'tcx>, resume: BasicBlock, drop: Option<BasicBlock> }, // 生成
    GeneratorDrop,                                 // 生成のドロップ
}
```

#### 値と場所の表現

```rust
// 場所（左辺値）の表現
pub type Place<'tcx> = PlaceImpl<'tcx>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlaceImpl<'tcx> {
    pub local: Local,
    pub projection: Vec<PlaceElem<'tcx>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PlaceElem<'tcx> {
    Deref,                    // デリファレンス
    Field(Field),              // フィールドアクセス
    Index(Local),              // インデックスアクセス
    ConstantIndex { index: u64, min_length: u64 }, // 定数インデックス
    Subslice { from: u64, to: u64, from_end: bool }, // 部分配列
}

// 右辺値の表現
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Rvalue<'tcx> {
    Use(Operand<'tcx>),                    // 値の使用
    Repeat(Operand<'tcx>, Constant<'tcx>), // 繰り返し
    Ref(Region<'tcx>, BorrowKind, Place<'tcx>), // 参照
    AddrOf(Mutability, Place<'tcx>),      // アドレス
    Len(Place<'tcx>),                      // 長さの取得
    Cast(Operand<'tcx>, CastKind, Ty<'tcx>), // キャスト
    BinaryOp(BinOp, Operand<'tcx>, Operand<'tcx>), // 二項演算
    UnaryOp(UnOp, Operand<'tcx>),           // 単項演算
    // ... 他の右辺値
}

// オペランドの表現
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operand<'tcx> {
    Copy(Place<'tcx>),      // コピー
    Move(Place<'tcx>),      // ムーブ
    Constant(Box<Constant<'tcx>>), // 定数
}
```

## データフロー分析

### 1. 到達可能性解析

#### 基本的な到達可能性

```rust
// 到達可能性解析の実装
pub struct ReachabilityAnalysis<'tcx> {
    body: &'tcx Body<'tcx>,
    reachable: BitSet<BasicBlock>,
}

impl<'tcx> ReachabilityAnalysis<'tcx> {
    pub fn new(body: &'tcx Body<'tcx>) -> Self {
        let mut analysis = ReachabilityAnalysis {
            body,
            reachable: BitSet::new_empty(body.basic_blocks.len()),
        };
        
        // 開始基本ブロックから到達可能性を計算
        analysis.compute_reachability();
        
        analysis
    }
    
    fn compute_reachability(&mut self) {
        let mut worklist = VecDeque::new();
        worklist.push_back(START_BLOCK);
        
        while let Some(bb) = worklist.pop_front() {
            if !self.reachable.contains(bb) {
                self.reachable.insert(bb);
                
                // 後続基本ブロックをワークリストに追加
                if let Some(terminator) = &self.body.basic_blocks[bb].terminator {
                    for successor in terminator.successors() {
                        worklist.push_back(successor);
                    }
                }
            }
        }
    }
    
    pub fn is_reachable(&self, bb: BasicBlock) -> bool {
        self.reachable.contains(bb)
    }
}
```

#### 条件付き到達可能性

```rust
// 条件付き到達可能性解析
pub struct ConditionalReachability<'tcx> {
    body: &'tcx Body<'tcx>,
    reachable: Vec<BitSet<BasicBlock>>,
    conditions: Vec<Condition>,
}

#[derive(Clone, Debug)]
pub struct Condition {
    pub variable: Local,
    pub value: Constant<'tcx>,
}

impl<'tcx> ConditionalReachability<'tcx> {
    pub fn compute(&mut self) {
        // 各条件の下での到達可能性を計算
        for (i, condition) in self.conditions.iter().enumerate() {
            let mut reachable = BitSet::new_empty(self.body.basic_blocks.len());
            
            // 条件を仮定して到達可能性を計算
            self.compute_with_condition(&mut reachable, condition);
            
            self.reachable.push(reachable);
        }
    }
    
    fn compute_with_condition(&self, reachable: &mut BitSet<BasicBlock>, condition: &Condition) {
        // 条件を仮定したデータフロー解析
        let mut worklist = VecDeque::new();
        worklist.push_back(START_BLOCK);
        
        while let Some(bb) = worklist.pop_front() {
            if !reachable.contains(bb) {
                reachable.insert(bb);
                
                // 条件を考慮した後続の処理
                if let Some(terminator) = &self.body.basic_blocks[bb].terminator {
                    for successor in self.process_terminator_with_condition(terminator, condition) {
                        worklist.push_back(successor);
                    }
                }
            }
        }
    }
}
```

### 2. ライブ変数解析

#### 定義使用チェーンの構築

```rust
// 定義使用チェーンの実装
pub struct DefUseChain<'tcx> {
    body: &'tcx Body<'tcx>,
    def_use_chains: IndexVec<Local, Vec<UseSite>>,
}

#[derive(Debug)]
pub struct UseSite {
    pub location: Location,
    pub kind: UseKind,
}

#[derive(Debug)]
pub enum UseKind {
    Definition,
    Use,
    Move,
    Borrow,
}

impl<'tcx> DefUseChain<'tcx> {
    pub fn build(&mut self) {
        // 各基本ブロックを処理
        for (bb_id, bb_data) in self.body.basic_blocks.iter_enumerated() {
            self.process_basic_block(bb_id, bb_data);
        }
    }
    
    fn process_basic_block(&mut self, bb_id: BasicBlock, bb_data: &BasicBlockData<'tcx>) {
        let mut location = Location {
            block: bb_id,
            statement_index: 0,
        };
        
        // ステートメントの処理
        for statement in &bb_data.statements {
            self.process_statement(&mut location, statement);
            location.statement_index += 1;
        }
        
        // ターミネータの処理
        if let Some(terminator) = &bb_data.terminator {
            self.process_terminator(&location, terminator);
        }
    }
    
    fn process_statement(&mut self, location: &mut Location, statement: &Statement<'tcx>) {
        match &statement.kind {
            StatementKind::Assign(place, rvalue) => {
                // 使用の記録
                self.record_uses(location, rvalue);
                
                // 定義の記録
                if let Some(local) = place.local {
                    self.def_use_chains[local].push(UseSite {
                        location: *location,
                        kind: UseKind::Definition,
                    });
                }
            }
            // ... 他のステートメント種類
        }
    }
}
```

#### ライブ変数の計算

```rust
// ライブ変数解析の実装
pub struct LivenessAnalysis<'tcx> {
    body: &'tcx Body<'tcx>,
    live_in: IndexVec<BasicBlock, BitSet<Local>>,
    live_out: IndexVec<BasicBlock, BitSet<Local>>,
}

impl<'tcx> LivenessAnalysis<'tcx> {
    pub fn compute(&mut self) {
        // 後方解析によるライブ変数の計算
        let mut worklist = VecDeque::new();
        let mut visited = BitSet::new_empty(self.body.basic_blocks.len());
        
        // 終了基本ブロックから開始
        for bb in self.body.basic_blocks.indices() {
            if self.is_exit_block(bb) {
                worklist.push_back(bb);
            }
        }
        
        while let Some(bb) = worklist.pop_front() {
            if !visited.contains(bb) {
                visited.insert(bb);
                self.compute_block_liveness(bb);
                
                // 前駆基本ブロックをワークリストに追加
                for predecessor in self.predecessors(bb) {
                    worklist.push_back(predecessor);
                }
            }
        }
    }
    
    fn compute_block_liveness(&mut self, bb: BasicBlock) {
        let bb_data = &self.body.basic_blocks[bb];
        
        // ブロックの終了時のライブ変数を初期化
        let mut live = self.live_out[bb].clone();
        
        // ターミネータの処理（逆順）
        if let Some(terminator) = &bb_data.terminator {
            self.process_terminator_liveness(&mut live, terminator);
        }
        
        // ステートメントの処理（逆順）
        for statement in bb_data.statements.iter().rev() {
            self.process_statement_liveness(&mut live, statement);
        }
        
        // ブロックの開始時のライブ変数を記録
        self.live_in[bb] = live;
    }
    
    fn process_statement_liveness(&mut self, live: &mut BitSet<Local>, statement: &Statement<'tcx>) {
        match &statement.kind {
            StatementKind::Assign(place, _) => {
                // 代入先の変数をライブセットから削除
                if let Some(local) = place.local {
                    live.remove(local);
                }
                
                // 使用されている変数をライブセットに追加
                self.add_used_variables(live, place);
            }
            // ... 他のステートメント種類
        }
    }
}
```

## 最適化パスの実装

### 1. 定数伝播

#### 定数伝播アルゴリズム

```rust
// 定数伝播の実装
pub struct ConstantPropagation<'tcx> {
    body: &'tcx Body<'tcx>,
    constants: IndexVec<Local, Option<Constant<'tcx>>>,
    worklist: VecDeque<BasicBlock>,
}

impl<'tcx> ConstantPropagation<'tcx> {
    pub fn run(&mut self) {
        // 定数の初期化
        self.initialize_constants();
        
        // ワークリストアルゴリズム
        while let Some(bb) = self.worklist.pop_front() {
            if self.process_basic_block(bb) {
                // 変更があった場合、後続ブロックを追加
                self.add_successors_to_worklist(bb);
            }
        }
        
        // 定数の適用
        self.apply_constants();
    }
    
    fn process_basic_block(&mut self, bb: BasicBlock) -> bool {
        let mut changed = false;
        let bb_data = &self.body.basic_blocks[bb];
        
        // ステートメントの処理
        for statement in &bb_data.statements {
            if self.process_statement(statement) {
                changed = true;
            }
        }
        
        // ターミネータの処理
        if let Some(terminator) = &bb_data.terminator {
            if self.process_terminator(terminator) {
                changed = true;
            }
        }
        
        changed
    }
    
    fn process_statement(&mut self, statement: &Statement<'tcx>) -> bool {
        match &statement.kind {
            StatementKind::Assign(place, rvalue) => {
                // 右辺値の定数評価
                if let Some(constant) = self.evaluate_rvalue(rvalue) {
                    // 左辺変数に定数を設定
                    if let Some(local) = place.local {
                        if self.constants[local] != Some(constant) {
                            self.constants[local] = Some(constant);
                            return true;
                        }
                    }
                }
            }
            // ... 他のステートメント種類
        }
        false
    }
    
    fn evaluate_rvalue(&self, rvalue: &Rvalue<'tcx>) -> Option<Constant<'tcx>> {
        match rvalue {
            Rvalue::Use(operand) => self.evaluate_operand(operand),
            Rvalue::BinaryOp(op, left, right) => {
                if let (Some(left_const), Some(right_const)) = (
                    self.evaluate_operand(left),
                    self.evaluate_operand(right),
                ) {
                    self.evaluate_binary_op(*op, left_const, right_const)
                } else {
                    None
                }
            }
            // ... 他の右辺値
            _ => None,
        }
    }
    
    fn apply_constants(&mut self) {
        // 定数をMIRに適用
        for bb_data in &mut self.body.basic_blocks {
            for statement in &mut bb_data.statements {
                self.fold_constants_in_statement(statement);
            }
            
            if let Some(terminator) = &mut bb_data.terminator {
                self.fold_constants_in_terminator(terminator);
            }
        }
    }
}
```

### 2. デッドコード削除

#### デッドコードの検出

```rust
// デッドコード削除の実装
pub struct DeadCodeElimination<'tcx> {
    body: &'tcx Body<'tcx>,
    live_locals: BitSet<Local>,
    used_blocks: BitSet<BasicBlock>,
}

impl<'tcx> DeadCodeElimination<'tcx> {
    pub fn run(&mut self) {
        // ライブ変数解析の実行
        let mut liveness = LivenessAnalysis::new(self.body);
        liveness.compute();
        
        // 到達可能性解析の実行
        let mut reachability = ReachabilityAnalysis::new(self.body);
        reachability.compute_reachability();
        
        // デッドコードの削除
        self.remove_dead_statements(&liveness);
        self.remove_dead_blocks(&reachability);
        self.remove_dead_locals(&liveness);
    }
    
    fn remove_dead_statements(&mut self, liveness: &LivenessAnalysis<'tcx>) {
        for bb_data in &mut self.body.basic_blocks {
            // ライブでない変数への代入を削除
            bb_data.statements.retain(|statement| {
                match &statement.kind {
                    StatementKind::Assign(place, _) => {
                        if let Some(local) = place.local {
                            // 変数が後で使用されているかチェック
                            liveness.is_live_after(local, statement.location)
                        } else {
                            true
                        }
                    }
                    _ => true,
                }
            });
        }
    }
    
    fn remove_dead_blocks(&mut self, reachability: &ReachabilityAnalysis<'tcx>) {
        // 到達不可能な基本ブロックを削除
        for bb in self.body.basic_blocks.indices() {
            if !reachability.is_reachable(bb) {
                self.remove_block(bb);
            }
        }
    }
    
    fn remove_dead_locals(&mut self, liveness: &LivenessAnalysis<'tcx>) {
        // 使用されていないローカル変数を削除
        for (local, decl) in self.body.local_decls.iter_enumerated() {
            if !liveness.is_ever_used(local) {
                decl.local_info = LocalInfo::Dead;
            }
        }
    }
}
```

### 3. 共通部分式の削除

#### CSEアルゴリズム

```rust
// 共通部分式の削除の実装
pub struct CommonSubexpressionElimination<'tcx> {
    body: &'tcx Body<'tcx>,
    expressions: HashMap<ExpressionHash, Local>,
    replacements: HashMap<Local, Local>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExpressionHash {
    pub kind: ExpressionKind,
    pub types: Vec<Ty<'tcx>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExpressionKind {
    BinaryOp(BinOp, Local, Local),
    UnaryOp(UnOp, Local),
    Load(Local),
    // ... 他の式の種類
}

impl<'tcx> CommonSubexpressionElimination<'tcx> {
    pub fn run(&mut self) {
        // 式のハッシュ計算
        self.compute_expression_hashes();
        
        // 共通部分式の検出
        self.find_common_subexpressions();
        
        // 置換の適用
        self.apply_replacements();
    }
    
    fn compute_expression_hashes(&mut self) {
        for bb_data in &self.body.basic_blocks {
            for statement in &bb_data.statements {
                if let StatementKind::Assign(_, rvalue) = &statement.kind {
                    if let Some(hash) = self.hash_rvalue(rvalue) {
                        self.expressions.insert(hash, statement.location.local);
                    }
                }
            }
        }
    }
    
    fn find_common_subexpressions(&mut self) {
        // 複数回出現する式を検出
        let mut expression_counts = HashMap::new();
        
        for (hash, local) in &self.expressions {
            *expression_counts.entry(hash.clone()).or_insert(0) += 1;
        }
        
        // 共通部分式の置換を計算
        for (hash, count) in &expression_counts {
            if *count > 1 {
                if let Some(original_local) = self.expressions.get(hash) {
                    // 最初の式を保持し、後続の式を置換
                    for (other_hash, other_local) in &self.expressions {
                        if other_hash == hash && other_local != original_local {
                            self.replacements.insert(*other_local, *original_local);
                        }
                    }
                }
            }
        }
    }
    
    fn apply_replacements(&mut self) {
        // 置換をMIRに適用
        for bb_data in &mut self.body.basic_blocks {
            for statement in &mut bb_data.statements {
                self.replace_in_statement(statement);
            }
            
            if let Some(terminator) = &mut bb_data.terminator {
                self.replace_in_terminator(terminator);
            }
        }
    }
}
```

## 高度な最適化技術

### 1. ループ最適化

#### ループ不変式の検出

```rust
// ループ不変式の検出
pub struct LoopInvariantAnalysis<'tcx> {
    body: &'tcx Body<'tcx>,
    loops: Vec<LoopInfo>,
}

#[derive(Debug)]
pub struct LoopInfo {
    pub header: BasicBlock,
    pub body: Vec<BasicBlock>,
    pub exits: Vec<BasicBlock>,
    pub invariants: Vec<Invariant>,
}

#[derive(Debug)]
pub struct Invariant {
    pub variable: Local,
    pub value: Constant<'tcx>,
    pub location: Location,
}

impl<'tcx> LoopInvariantAnalysis<'tcx> {
    pub fn detect_loops(&mut self) {
        // 自然ループの検出
        self.find_natural_loops();
        
        // 各ループの不変式を検出
        for loop_info in &mut self.loops {
            self.find_loop_invariants(loop_info);
        }
    }
    
    fn find_natural_loops(&mut self) {
        // 支配木の構築
        let dominator_tree = self.build_dominator_tree();
        
        // 後方エッジの検出
        let back_edges = self.find_back_edges(&dominator_tree);
        
        // ループの構築
        for back_edge in back_edges {
            let loop_info = self.build_loop_info(back_edge, &dominator_tree);
            self.loops.push(loop_info);
        }
    }
    
    fn find_loop_invariants(&mut self, loop_info: &mut LoopInfo) {
        // ループ内の定義を収集
        let mut definitions = HashMap::new();
        self.collect_definitions_in_loop(loop_info, &mut definitions);
        
        // ループ入口での値を記録
        let mut entry_values = HashMap::new();
        self.record_entry_values(loop_info, &entry_values);
        
        // 不変式の検出
        for (variable, entry_value) in entry_values {
            if let Some(loop_value) = definitions.get(&variable) {
                if entry_value == loop_value {
                    loop_info.invariants.push(Invariant {
                        variable,
                        value: entry_value.clone(),
                        location: Location::loop_header(loop_info.header),
                    });
                }
            }
        }
    }
}
```

#### ループ展開の実装

```rust
// ループ展開の実装
pub struct LoopUnrolling<'tcx> {
    body: &'tcx Body<'tcx>,
    unroll_factor: usize,
}

impl<'tcx> LoopUnrolling<'tcx> {
    pub fn unroll_loops(&mut self) {
        // 展開可能なループを検出
        let loops = self.find_unrollable_loops();
        
        // 各ループを展開
        for loop_info in loops {
            if self.should_unroll(&loop_info) {
                self.unroll_loop(&loop_info);
            }
        }
    }
    
    fn unroll_loop(&mut self, loop_info: &LoopInfo) {
        // ループの展開
        let unrolled_body = self.create_unrolled_body(loop_info);
        
        // 元のループを置換
        self.replace_loop_with_unrolled_body(loop_info, unrolled_body);
    }
    
    fn create_unrolled_body(&self, loop_info: &LoopInfo) -> Body<'tcx> {
        let mut unrolled_body = self.body.clone();
        
        // 展開されたループ本体の作成
        for i in 1..self.unroll_factor {
            let iteration_body = self.clone_loop_body(loop_info, i);
            self.append_iteration(&mut unrolled_body, iteration_body);
        }
        
        unrolled_body
    }
}
```

### 2. 関数インライン化

#### インライン化の決定

```rust
// 関数インライン化の実装
pub struct Inlining<'tcx> {
    body: &'tcx Body<'tcx>,
    inline_threshold: usize,
}

impl<'tcx> Inlining<'tcx> {
    pub fn inline_functions(&mut self) {
        // インライン化候補の検出
        let candidates = self.find_inlining_candidates();
        
        // 各候補を評価しインライン化
        for candidate in candidates {
            if self.should_inline(&candidate) {
                self.inline_function(&candidate);
            }
        }
    }
    
    fn find_inlining_candidates(&self) -> Vec<InliningCandidate> {
        let mut candidates = Vec::new();
        
        // 関数呼び出しの検出
        for bb_data in &self.body.basic_blocks {
            if let Some(terminator) = &bb_data.terminator {
                if let TerminatorKind::Call { func, .. } = &terminator.kind {
                    if let Some(function_id) = self.get_function_id(func) {
                        let function_info = self.get_function_info(function_id);
                        candidates.push(InliningCandidate {
                            function_id,
                            call_site: terminator.location,
                            function_info,
                        });
                    }
                }
            }
        }
        
        candidates
    }
    
    fn should_inline(&self, candidate: &InliningCandidate) -> bool {
        // インライン化の決定基準
        let function_size = candidate.function_info.size;
        let call_frequency = candidate.function_info.call_count;
        
        // 小さな関数は積極的にインライン化
        if function_size < 10 {
            return true;
        }
        
        // 呼び出し頻度の高い関数をインライン化
        if call_frequency > 5 {
            return true;
        }
        
        // しきい値以下の関数のみインライン化
        function_size < self.inline_threshold
    }
    
    fn inline_function(&mut self, candidate: &InliningCandidate) {
        // 関数のインライン化
        let function_body = self.get_function_body(candidate.function_id);
        let inlined_body = self.create_inlined_body(function_body, candidate);
        
        // 呼び出しをインライン化された本体で置換
        self.replace_call_with_inlined_body(candidate, inlined_body);
    }
}
```

## 最適化の評価と測定

### 1. 最適化効果の測定

#### パフォーマンスベンチマーク

```rust
// 最適化効果の測定
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_optimizations(c: &mut Criterion) {
    let mir = generate_complex_mir();
    
    // 最適化前のベンチマーク
    c.bench_function("before_optimization", |b| {
        b.iter(|| {
            let mut optimized_mir = mir.clone();
            // 最適化なしで実行
            execute_mir(black_box(&mut optimized_mir))
        })
    });
    
    // 各最適化パスのベンチマーク
    c.bench_function("constant_propagation", |b| {
        b.iter(|| {
            let mut optimized_mir = mir.clone();
            let mut const_prop = ConstantPropagation::new(&optimized_mir);
            const_prop.run();
            execute_mir(black_box(&mut optimized_mir))
        })
    });
    
    c.bench_function("dead_code_elimination", |b| {
        b.iter(|| {
            let mut optimized_mir = mir.clone();
            let mut dce = DeadCodeElimination::new(&optimized_mir);
            dce.run();
            execute_mir(black_box(&mut optimized_mir))
        })
    });
    
    // 全ての最適化のベンチマーク
    c.bench_function("full_optimization", |b| {
        b.iter(|| {
            let mut optimized_mir = mir.clone();
            optimize_mir_fully(black_box(&mut optimized_mir));
            execute_mir(black_box(&mut optimized_mir))
        })
    });
}

criterion_group!(benches, benchmark_optimizations);
criterion_main!(benches);
```

### 2. 最適化の品質評価

#### コードサイズの測定

```rust
// コードサイズの測定
pub struct CodeSizeAnalyzer<'tcx> {
    body: &'tcx Body<'tcx>,
}

impl<'tcx> CodeSizeAnalyzer<'tcx> {
    pub fn analyze(&self) -> CodeSizeMetrics {
        let mut metrics = CodeSizeMetrics::default();
        
        // 基本ブロック数の計算
        metrics.basic_block_count = self.body.basic_blocks.len();
        
        // ステートメント数の計算
        metrics.statement_count = self.body.basic_blocks
            .iter()
            .map(|bb| bb.statements.len())
            .sum();
        
        // ターミネータ数の計算
        metrics.terminator_count = self.body.basic_blocks
            .iter()
            .filter(|bb| bb.terminator.is_some())
            .count();
        
        // ローカル変数数の計算
        metrics.local_count = self.body.local_decls.len();
        
        // 複雑度の計算
        metrics.cyclomatic_complexity = self.calculate_complexity();
        
        metrics
    }
    
    fn calculate_complexity(&self) -> usize {
        let mut complexity = 0;
        
        for bb_data in &self.body.basic_blocks {
            if let Some(terminator) = &bb_data.terminator {
                complexity += match &terminator.kind {
                    TerminatorKind::SwitchInt { .. } => 1,
                    TerminatorKind::If { .. } => 1,
                    TerminatorKind::Call { .. } => 1,
                    _ => 0,
                };
            }
        }
        
        complexity
    }
}

#[derive(Default, Debug)]
pub struct CodeSizeMetrics {
    pub basic_block_count: usize,
    pub statement_count: usize,
    pub terminator_count: usize,
    pub local_count: usize,
    pub cyclomatic_complexity: usize,
}
```

## 関連ドキュメント

より詳細な情報については、以下のドキュメントを参照してください：

- [MIR最適化](../../mir/optimizations.md) - MIR最適化の詳細
- [MIRの構築](../../mir/construction.md) - MIR構築のプロセス
- [MIRのビジターと走査](../../mir/visitor.md) - MIR操作のAPI
- [MIRのクエリとパス](../../mir/passes.md) - MIRパスの実装
- [プロファイリング](../../profiling.md) - パフォーマンス分析

## 次のステップ

MIR最適化を学習したら、次は[型推論・トレイト解決](./type-inference.md)を学びましょう。型システムの深い理解を得ることで、より高度なコンパイラ機能の実装ができるようになります。
