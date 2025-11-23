# インストール

`std::offload`は一部ナイトリービルドでユーザーが利用できます。ただし、現在のところ、すべての機能を使用するには、誰もがソースからrustcをビルドする必要があります。

## ビルド手順

まず、Rustリポジトリをクローンして設定する必要があります：
```bash
git clone git@github.com:rust-lang/rust
cd rust
./configure --enable-llvm-link-shared --release-channel=nightly --enable-llvm-assertions --enable-llvm-offload --enable-llvm-enzyme --enable-clang --enable-lld --enable-option-checking --enable-ninja --disable-docs
```

その後、次のコマンドを使用してrustcをビルドできます：
```bash
./x build --stage 1 library
```

その後、rustc toolchain linkを使用してcargoで使用できるようになります：
```
rustup toolchain link offload build/host/stage1
rustup toolchain install nightly # -Z unstable-optionsを有効にする
```



## LLVM自体のビルド手順
```bash
git clone git@github.com:llvm/llvm-project
cd llvm-project
mkdir build
cd build
cmake -G Ninja ../llvm -DLLVM_TARGETS_TO_BUILD="host,AMDGPU,NVPTX" -DLLVM_ENABLE_ASSERTIONS=ON -DLLVM_ENABLE_PROJECTS="clang;lld" -DLLVM_ENABLE_RUNTIMES="offload,openmp" -DLLVM_ENABLE_PLUGINS=ON -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=.
ninja
ninja install
```
これで動作するLLVMビルドが得られます。


## テスト
実行：
```
./x test --stage 1 tests/codegen-llvm/gpu_offload
```
