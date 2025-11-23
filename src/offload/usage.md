# 使用方法

この機能は作業中であり、使用の準備ができていません。ここでの手順は、貢献者または最新の進捗状況をフォローすることに興味がある人向けです。
現在、次のRustカーネルをGPU上で起動する作業を行っています。フォローするには、これを`src/lib.rs`ファイルにコピーしてください。

```rust
#![feature(abi_gpu_kernel)]
#![no_std]

#[cfg(target_os = "linux")]
extern crate libc;
#[cfg(target_os = "linux")]
use libc::c_char;

use core::mem;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
#[inline(never)]
fn main() {
    let array_c: *mut [f64; 256] =
        unsafe { libc::calloc(256, (mem::size_of::<f64>()) as libc::size_t) as *mut [f64; 256] };
    let output = c"The first element is zero %f\n";
    let output2 = c"The first element is NOT zero %f\n";
    let output3 = c"The second element is %f\n";
    unsafe {
        let val: *const c_char = if (*array_c)[0] < 0.1 {
            output.as_ptr()
        } else {
            output2.as_ptr()
        };
        libc::printf(val, (*array_c)[0]);
    }

    unsafe {
        kernel_1(array_c);
    }
    core::hint::black_box(&array_c);
    unsafe {
        let val: *const c_char = if (*array_c)[0] < 0.1 {
            output.as_ptr()
        } else {
            output2.as_ptr()
        };
        libc::printf(val, (*array_c)[0]);
        libc::printf(output3.as_ptr(), (*array_c)[1]);
    }
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    pub fn kernel_1(array_b: *mut [f64; 256]);
}

#[cfg(not(target_os = "linux"))]
#[unsafe(no_mangle)]
#[inline(never)]
pub extern "gpu-kernel" fn kernel_1(x: *mut [f64; 256]) {
    unsafe { (*x)[0] = 21.0 };
}
```

## コンパイル手順
rustcと同じllvm上でビルドされたclangコンパイラを使用することが重要です。フルパスなしでclangを呼び出すだけでは、おそらくシステムのclangが使用され、互換性がない可能性があります。したがって、以下のclang/lld呼び出しを絶対パスで置き換えるか、`PATH`を適切に設定してください。

まず、ホスト（CPU）コードを生成します。最初のビルドは単にlibcをコンパイルするためのもので、ハッシュ化されたパスに注意してください。次に、libcアーティファクトをrustcに提供しながら、rustcを直接呼び出してホストコードをビルドします。
```
cargo +offload build -r -v
rustc +offload --edition 2024 src/lib.rs -g --crate-type cdylib -C opt-level=3 -C panic=abort -C lto=fat -L dependency=/absolute_path_to/target/release/deps --extern libc=/absolute_path_to/target/release/deps/liblibc-<HASH>.rlib --emit=llvm-bc,llvm-ir  -Zoffload=Enable -Zunstable-options
```

次に、デバイスコードを生成します。target-cpuを自分のGPUに適したコードに置き換えてください。
```
RUSTFLAGS="-Ctarget-cpu=gfx90a --emit=llvm-bc,llvm-ir -Zoffload=Enable -Zunstable-options" cargo +offload build -Zunstable-options -r -v --target amdgcn-amd-amdhsa -Zbuild-std=core
```

次に、target/amdgcn-amd-amdhsaフォルダの下にある`<libname>.ll`を見つけて、device.llファイルにコピーします（または以下のファイル名を調整してください）。
NVIDIAまたはIntel GPUで作業している場合は、名前を適切に調整し、結果（成功または失敗）を共有するためにissueを開いてください。
まず、.llファイル（手動検査に適しています）を.bcファイルにコンパイルし、残ったアーティファクトをクリーンアップします。クリーンアップは重要です。そうしないと、キャッシングが次の実行時に干渉する可能性があります。
```
opt lib.ll -o lib.bc
opt device.ll -o device.bc
rm *.o
rm bare.amdgcn.gfx90a.img*
```

```
"clang-offload-packager" "-o" "host.out" "--image=file=device.bc,triple=amdgcn-amd-amdhsa,arch=gfx90a,kind=openmp"

"clang-21" "-cc1" "-triple" "x86_64-unknown-linux-gnu" "-S" "-save-temps=cwd" "-disable-free" "-clear-ast-before-backend" "-main-file-name" "lib.rs" "-mrelocation-model" "pic" "-pic-level" "2" "-pic-is-pie" "-mframe-pointer=all" "-fmath-errno" "-ffp-contract=on" "-fno-rounding-math" "-mconstructor-aliases" "-funwind-tables=2" "-target-cpu" "x86-64" "-tune-cpu" "generic" "-resource-dir" "/<ABSOLUTE_PATH_TO>/rust/build/x86_64-unknown-linux-gnu/llvm/lib/clang/21" "-ferror-limit" "19" "-fopenmp" "-fopenmp-offload-mandatory" "-fgnuc-version=4.2.1" "-fskip-odr-check-in-gmf" "-fembed-offload-object=host.out" "-fopenmp-targets=amdgcn-amd-amdhsa" "-faddrsig" "-D__GCC_HAVE_DWARF2_CFI_ASM=1" "-o" "host.s" "-x" "ir" "lib.bc"

"clang-21" "-cc1as" "-triple" "x86_64-unknown-linux-gnu" "-filetype" "obj" "-main-file-name" "lib.rs" "-target-cpu" "x86-64" "-mrelocation-model" "pic" "-o" "host.o" "host.s"

"clang-linker-wrapper" "--should-extract=gfx90a" "--device-compiler=amdgcn-amd-amdhsa=-g" "--device-compiler=amdgcn-amd-amdhsa=-save-temps=cwd" "--device-linker=amdgcn-amd-amdhsa=-lompdevice" "--host-triple=x86_64-unknown-linux-gnu" "--save-temps" "--linker-path=/ABSOlUTE_PATH_TO/rust/build/x86_64-unknown-linux-gnu/lld/bin/ld.lld" "--hash-style=gnu" "--eh-frame-hdr" "-m" "elf_x86_64" "-pie" "-dynamic-linker" "/lib64/ld-linux-x86-64.so.2" "-o" "bare" "/lib/../lib64/Scrt1.o" "/lib/../lib64/crti.o" "/ABSOLUTE_PATH_TO/crtbeginS.o" "-L/ABSOLUTE_PATH_TO/rust/build/x86_64-unknown-linux-gnu/llvm/bin/../lib/x86_64-unknown-linux-gnu" "-L/ABSOLUTE_PATH_TO/rust/build/x86_64-unknown-linux-gnu/llvm/lib/clang/21/lib/x86_64-unknown-linux-gnu" "-L/lib/../lib64" "-L/usr/lib64" "-L/lib" "-L/usr/lib" "host.o" "-lstdc++" "-lm" "-lomp" "-lomptarget" "-L/ABSOLUTE_PATH_TO/rust/build/x86_64-unknown-linux-gnu/llvm/lib" "-lgcc_s" "-lgcc" "-lpthread" "-lc" "-lgcc_s" "-lgcc" "/ABSOLUTE_PATH_TO/crtendS.o" "/lib/../lib64/crtn.o"
```

特に最後のコマンドについては、パスを修正するのではなく、bareモードのOpenMP例をコピーして、自分のclangでコンパイルすることで再生成することをお勧めします。clang呼び出しに`-###`を追加することで、個々のステップを確認できます。
```
myclang++ -fuse-ld=lld -O3 -fopenmp  -fopenmp-offload-mandatory --offload-arch=gfx90a omp_bare.cpp -o main -###
```

最後のステップでは、バイナリを実行できます

```
./main
The first element is zero 0.000000
The first element is NOT zero 21.000000
The second element is  0.000000
```

メモリ転送に関する詳細情報を受け取るには、情報出力を有効にできます
```
LIBOMPTARGET_INFO=-1  ./main
```
