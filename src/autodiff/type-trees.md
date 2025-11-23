# Autodiff用TypeTree

## TypeTreeとは？
Enzymeのためのメモリレイアウト記述子です。Enzymeに型がメモリ内でどのように構造化されているかを正確に伝えることで、効率的に導関数を計算できるようにします。

## 構造
```rust
TypeTree(Vec<Type>)

Type {
    offset: isize,  // バイトオフセット (-1 = すべて)
    size: usize,    // バイト単位のサイズ
    kind: Kind,     // Float、Integer、Pointerなど
    child: TypeTree // ネストされた構造
}
```

## 例：`fn compute(x: &f32, data: &[f32]) -> f32`

**入力0: `x: &f32`**
```rust
TypeTree(vec![Type {
    offset: -1, size: 8, kind: Pointer,
    child: TypeTree(vec![Type {
        offset: 0, size: 4, kind: Float,  // 単一の値：オフセット0を使用
        child: TypeTree::new()
    }])
}])
```

**入力1: `data: &[f32]`**
```rust
TypeTree(vec![Type {
    offset: -1, size: 8, kind: Pointer,
    child: TypeTree(vec![Type {
        offset: -1, size: 4, kind: Float,  // -1 = すべての要素
        child: TypeTree::new()
    }])
}])
```

**出力: `f32`**
```rust
TypeTree(vec![Type {
    offset: 0, size: 4, kind: Float,  // 単一のスカラー：オフセット0を使用
    child: TypeTree::new()
}])
```

## なぜ必要なのか？
- EnzymeはLLVM IRから複雑な型レイアウトを推測できない
- 遅いメモリパターン解析を防ぐ
- ネストされた構造に対する正しい導関数計算を可能にする
- メタデータと微分可能なバイトをEnzymeに伝える

## Enzymeがこの情報で何をするか：

TypeTreeなし：
```llvm
; Enzymeは汎用的なLLVM IRを見る：
define float @distance(ptr %p1, ptr %p2) {
; これらのポインタが何を指しているかを推測しなければならない
; すべてのメモリ操作の遅い解析
; 最適化の機会を逃す可能性がある
}
```

TypeTreeあり：
```llvm
define "enzyme_type"="{[-1]:Float@float}" float @distance(
    ptr "enzyme_type"="{[-1]:Pointer, [-1,0]:Float@float}" %p1,
    ptr "enzyme_type"="{[-1]:Pointer, [-1,0]:Float@float}" %p2
) {
; Enzymeは正確な型レイアウトを知っている
; 効率的な導関数コードを直接生成できる
}
```

# TypeTree - オフセットと-1の説明

## Type構造

```rust
Type {
    offset: isize, // この型がどこから始まるか
    size: usize,   // この型の大きさ
    kind: Kind,    // どんな種類のデータか (Float、Int、Pointer)
    child: TypeTree // 内部に何があるか (ポインタ/コンテナの場合)
}
```

## オフセット値

### 通常のオフセット (0, 4, 8, など)
**構造体内の特定のバイト位置**

```rust
struct Point {
    x: f32, // オフセット 0、サイズ 4
    y: f32, // オフセット 4、サイズ 4
    id: i32, // オフセット 8、サイズ 4
}
```

`&Point`のTypeTree（内部表現）：
```rust
TypeTree(vec![
    Type { offset: 0, size: 4, kind: Float },   // バイト0にx
    Type { offset: 4, size: 4, kind: Float },   // バイト4にy
    Type { offset: 8, size: 4, kind: Integer }  // バイト8にid
])
```

生成されるLLVM：
```llvm
"enzyme_type"="{[-1]:Pointer, [-1,0]:Float@float, [-1,4]:Float@float, [-1,8]:Integer, [-1,9]:Integer, [-1,10]:Integer, [-1,11]:Integer}"
```

### オフセット -1 (特別：「すべての場所」)
**「このパターンがすべての要素で繰り返される」ことを意味する**

#### 例1：直接配列 `[f32; 100]` (ポインタ間接参照なし)
```rust
TypeTree(vec![Type {
    offset: -1, // すべての位置
    size: 4,    // 各f32は4バイト
    kind: Float, // すべての要素がfloat
}])
```

生成されるLLVM：`"enzyme_type"="{[-1]:Float@float}"`

#### 例1b：配列参照 `&[f32; 100]` (ポインタ間接参照あり)
```rust
TypeTree(vec![Type {
    offset: -1, size: 8, kind: Pointer,
    child: TypeTree(vec![Type {
        offset: -1, // すべての配列要素
        size: 4,    // 各f32は4バイト
        kind: Float, // すべての要素がfloat
    }])
}])
```

生成されるLLVM：`"enzyme_type"="{[-1]:Pointer, [-1,-1]:Float@float}"`

オフセット`0,4,8,12...396`を持つ100個の個別のTypeをリストする代わりに

#### 例2：スライス `&[i32]`
```rust
// スライスデータへのポインタ
TypeTree(vec![Type {
    offset: -1, size: 8, kind: Pointer,
    child: TypeTree(vec![Type {
        offset: -1, // すべてのスライス要素
        size: 4,    // 各i32は4バイト
        kind: Integer
    }])
}])
```

生成されるLLVM：`"enzyme_type"="{[-1]:Pointer, [-1,-1]:Integer}"`

#### 例3：混合構造
```rust
struct Container {
    header: i64,        // オフセット 0
    data: [f32; 1000],  // オフセット 8、ただし要素は-1を使用
}
```

```rust
TypeTree(vec![
    Type { offset: 0, size: 8, kind: Integer }, // header
    Type { offset: 8, size: 4000, kind: Pointer,
        child: TypeTree(vec![Type {
            offset: -1, size: 4, kind: Float // すべての配列要素
        }])
    }
])
```

## 重要な区別：単一値 vs 配列

**単一値**は精度のためにオフセット`0`を使用：
- `&f32`はオフセット0に正確に1つのf32値を持つ
- -1（「すべての場所」）を使用するよりも正確
- 生成：`{[-1]:Pointer, [-1,0]:Float@float}`

**配列**は効率性のためにオフセット`-1`を使用：
- `&[f32; 100]`は同じパターンが100回繰り返される
- -1を使用することで、100個の個別のオフセットをリストすることを回避
- 生成：`{[-1]:Pointer, [-1,-1]:Float@float}`