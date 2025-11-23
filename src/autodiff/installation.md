# インストール

近い将来、ユーザー向けのnightlyビルドで`std::autodiff`が利用可能になる予定です。ただし、コントリビューターとしては、引き続きソースからrustcをビルドする必要があります。現時点ではmsvcターゲットはサポートされていませんが、他のすべてのtier 1ターゲットは動作するはずです。サポートされているtier 1ターゲットで問題が発生した場合、または tier2/tier3ターゲットでこのプロジェクトのビルドに成功した場合は、issueを開いてください。

## ビルド手順

まず、Rustリポジトリをクローンして設定する必要があります。好みに応じて、`--enable-clang`または`--enable-lld`も有効にすることができます。
```bash
git clone git@github.com:rust-lang/rust
cd rust
./configure --release-channel=nightly --enable-llvm-enzyme --enable-llvm-link-shared --enable-llvm-assertions --enable-ninja --enable-option-checking --disable-docs --set llvm.download-ci-llvm=false
```

その後、以下を使用してrustcをビルドできます：
```bash
./x build --stage 1 library
```

その後、rustup toolchain linkを使用すると、cargo経由で使用できるようになります：
```
rustup toolchain link enzyme build/host/stage1
rustup toolchain install nightly # -Z unstable-optionsを有効にする
```

その後、テストケースを実行できます：

```bash
./x test --stage 1 tests/codegen-llvm/autodiff
./x test --stage 1 tests/pretty/autodiff
./x test --stage 1 tests/ui/autodiff
./x test --stage 1 tests/ui/feature-gates/feature-gate-autodiff.rs
```

Autodiffはまだ実験的なため、独自のプロジェクトで使用する場合は、Cargo.tomlに`lto="fat"`を追加し、
`cargo`や`cargo +nightly`の代わりに`RUSTFLAGS="-Zautodiff=Enable" cargo +enzyme`を使用する必要があります。 

## Compiler Explorerとdistビルド

同様の方法で、compiler explorerインスタンスを新しいrustcに更新できます。まず、dockerインスタンスを準備します。
```bash
docker run -it ubuntu:22.04
export CC=clang CXX=clang++
apt update
apt install wget vim python3 git curl libssl-dev pkg-config lld ninja-build cmake clang build-essential
```
次に、わずかに変更した方法でrustcをビルドします：
```bash
git clone https://github.com/rust-lang/rust
cd rust
./configure --release-channel=nightly --enable-llvm-enzyme --enable-llvm-link-shared --enable-llvm-assertions --enable-ninja --enable-option-checking --disable-docs --set llvm.download-ci-llvm=false
./x dist
```
次に、tarballをホストにコピーします。dockeridは`docker ps -a`の下の最新のエントリです。
```bash
docker cp <dockerid>:/rust/build/dist/rust-nightly-x86_64-unknown-linux-gnu.tar.gz rust-nightly-x86_64-unknown-linux-gnu.tar.gz
```
その後、EnzymeAD/rustリポジトリに新しい（プレリリース）タグを作成し、EnzymeAD/enzyme-explorerリポジトリに対してタグを更新するPRを作成できます。
PRで`tgymnich`にpingして、更新スクリプトを実行してもらうことを忘れないでください。注意：EnzymeAD/rustをアーカイブし、ここの手順を更新する必要があります。explorerは近々
公式のrustサーバーからrustcツールチェーンを取得できるようになるはずです。


## Enzyme自体のビルド手順

上記のRustビルド手順に従うと、Rustコンパイラとともに LLVMEnzyme、LLDEnzyme、ClangEnzymeがビルドされます。
これらのいずれかを使用したいだけで、cmakeの経験がない場合は、このアプローチをお勧めします。
ただし、RustなしでEnzymeのみをビルドしたい場合は、これらの手順が役立つかもしれません。

```bash
git clone git@github.com:llvm/llvm-project
cd llvm-project
mkdir build
cd build
cmake -G Ninja ../llvm -DLLVM_TARGETS_TO_BUILD="host" -DLLVM_ENABLE_ASSERTIONS=ON -DLLVM_ENABLE_PROJECTS="clang;lld" -DLLVM_ENABLE_RUNTIMES="openmp" -DLLVM_ENABLE_PLUGINS=ON -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=.
ninja
ninja install
```
これにより、動作するLLVMビルドが得られます。次に、Enzymeのビルドを続けることができます。
`llvm-project`フォルダを離れて、次のコマンドを実行します：
```bash
git clone git@github.com:EnzymeAD/Enzyme
cd Enzyme/enzyme
mkdir build
cd build
cmake .. -G Ninja -DLLVM_DIR=<YourLocalPath>/llvm-project/build/lib/cmake/llvm/ -DLLVM_EXTERNAL_LIT=<YourLocalPath>/llvm-project/llvm/utils/lit/lit.py -DCMAKE_BUILD_TYPE=Release -DCMAKE_EXPORT_COMPILE_COMMANDS=YES -DBUILD_SHARED_LIBS=ON
ninja
```
これによりEnzymeがビルドされ、`Enzyme/enzyme/build/lib/<LLD/Clang/LLVM/lib>Enzyme.so`で見つけることができます。（拡張子はOSによって異なる場合があります）。

