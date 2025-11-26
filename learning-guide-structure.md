# rustc-dev-guide 学習ガイド ディレクトリ構造設計

## 基本方針

1. 既存のプロジェクト構造との整合性を維持
2. 学習者のレベルと目的に応じた明確な分類
3. mdBookのSUMMARY.mdとの統合を容易にする構造
4. 将来の拡張性を考慮した設計

## 提案するディレクトリ構造

```
src/
├── learning-guide/                    # 学習ガイドのルートディレクトリ
│   ├── README.md                     # 学習ガイドの概要とナビゲーション
│   ├── getting-started/               # 初級者向け学習パス
│   │   ├── README.md                 # 初級者学習パスの概要
│   │   ├── compiler-basics.md        # コンパイラの基本概念
│   │   ├── first-contribution.md     # 最初の貢献
│   │   └── build-and-debug.md        # ビルドとデバッグの基礎
│   ├── intermediate/                 # 中級者向け学習パス
│   │   ├── README.md                 # 中級者学習パスの概要
│   │   ├── compiler-architecture.md  # コンパイラアーキテクチャ
│   │   ├── source-representation.md  # ソースコード表現
│   │   └── type-system.md           # 型システム
│   ├── advanced/                     # 上級者向け学習パス
│   │   ├── README.md                 # 上級者学習パスの概要
│   │   ├── query-system.md          # クエリシステム
│   │   ├── optimization.md           # 最適化
│   │   └── code-generation.md        # コード生成
│   ├── specialization/                # 特定目的別学習パス
│   │   ├── README.md                 # 特化学習パスの概要
│   │   ├── diagnostics.md            # 診断システム
│   │   ├── testing.md                # テストインフラ
│   │   └── performance.md            # パフォーマンス最適化
│   └── resources/                    # 学習リソース
│       ├── README.md                 # リソースの概要
│       ├── exercises.md              # 演習問題
│       ├── projects.md               # 実践プロジェクト
│       └── references.md             # 参考資料
└── SUMMARY.md                        # 既存の目次（学習ガイドセクションを追加）
```

## ディレクトリ構造の詳細

### 1. learning-guide/ ルートディレクトリ

学習ガイド全体のエントリーポイントとして機能します。

- **README.md**: 学習ガイドの概要、使用方法、各学習パスへのナビゲーション
- **目的**: 学習者が自分に適した学習パスを選択できるようにする

### 2. getting-started/ 初級者向け学習パス

Rustコンパイラ開発の初心者を対象とした学習パスです。

- **対象者**: Rustコンパイラに初めて触れる開発者
- **学習目標**:
  - コンパイラの基本概念の理解
  - 簡単な貢献ができるようになる
  - ビルドとデバッグの基礎を習得
- **構成**:
  - `compiler-basics.md`: コンパイラとは何か、Rustコンパイラの特徴
  - `first-contribution.md`: 最初の貢献の具体的な手順
  - `build-and-debug.md`: ビルド環境の構築と基本的なデバッグ方法

### 3. intermediate/ 中級者向け学習パス

基本的なコンパイラ開発経験者を対象とした学習パスです。

- **対象者**: 基本的な貢献経験がある開発者
- **学習目標**:
  - コンパイラアーキテクチャの深い理解
  - 特定コンポーネントの詳細な知識習得
  - より複雑な貢献ができるようになる
- **構成**:
  - `compiler-architecture.md`: コンパイラの高レベルアーキテクチャ
  - `source-representation.md`: AST、HIR、MIRなどのソースコード表現
  - `type-system.md`: 型システムと型チェックの詳細

### 4. advanced/ 上級者向け学習パス

コンパイラ開発の専門家を対象とした学習パスです。

- **対象者**: 詳細なコンパイラ知識を持つ開発者
- **学習目標**:
  - コンパイラの内部メカニズムの完全な理解
  - 新機能の設計と実装
  - パフォーマンス最適化と高度なトピック
- **構成**:
  - `query-system.md`: クエリシステムとインクリメンタルコンパイル
  - `optimization.md`: MIR最適化とその他の最適化手法
  - `code-generation.md`: コード生成とバックエンド

### 5. specialization/ 特定目的別学習パス

特定の目的や関心を持つ学習者向けの特化パスです。

- **対象者**: 特定の分野に特化したい開発者
- **学習目標**:
  - 特定分野の専門知識の習得
  - 特定の貢献タイプへの集中
- **構成**:
  - `diagnostics.md`: 診断システムとエラーメッセージの改善
  - `testing.md`: テストインフラとテストの書き方
  - `performance.md`: パフォーマンス分析と最適化

### 6. resources/ 学習リソース

学習を支援する追加リソースです。

- **目的**: 実践的な学習機会の提供
- **構成**:
  - `exercises.md`: 各レベルに対応した演習問題
  - `projects.md`: 実践的なプロジェクト例
  - `references.md`: 追加の参考資料と外部リンク

## 既存構造との統合

### SUMMARY.mdへの統合方法

既存のSUMMARY.mdに以下のセクションを追加します：

```markdown
# 学習ガイド

- [学習ガイドの概要](./learning-guide/README.md)
  - [初級者向け学習パス](./learning-guide/getting-started/README.md)
    - [コンパイラの基本](./learning-guide/getting-started/compiler-basics.md)
    - [最初の貢献](./learning-guide/getting-started/first-contribution.md)
    - [ビルドとデバッグ](./learning-guide/getting-started/build-and-debug.md)
  - [中級者向け学習パス](./learning-guide/intermediate/README.md)
    - [コンパイラアーキテクチャ](./learning-guide/intermediate/compiler-architecture.md)
    - [ソースコード表現](./learning-guide/intermediate/source-representation.md)
    - [型システム](./learning-guide/intermediate/type-system.md)
  - [上級者向け学習パス](./learning-guide/advanced/README.md)
    - [クエリシステム](./learning-guide/advanced/query-system.md)
    - [最適化](./learning-guide/advanced/optimization.md)
    - [コード生成](./learning-guide/advanced/code-generation.md)
  - [特定目的別学習パス](./learning-guide/specialization/README.md)
    - [診断システム](./learning-guide/specialization/diagnostics.md)
    - [テストインフラ](./learning-guide/specialization/testing.md)
    - [パフォーマンス](./learning-guide/specialization/performance.md)
  - [学習リソース](./learning-guide/resources/README.md)
    - [演習問題](./learning-guide/resources/exercises.md)
    - [実践プロジェクト](./learning-guide/resources/projects.md)
    - [参考資料](./learning-guide/resources/references.md)
```

## メリット

1. **段階的学習**: 初級者から上級者まで体系的な学習パスを提供
2. **目的別学習**: 特定の目的に応じた集中学習が可能
3. **既存構造との整合性**: 既存のドキュメント構造を破壊しない
4. **拡張性**: 新しい学習パスやリソースの追加が容易
5. **保守性**: 各学習パスが独立しているため保守が容易
