# `ErrorGuaranteed`

前のセクションでは、コンパイラのユーザーが見るエラーメッセージについて説明しました。
しかし、エラーを出力することは、コンパイラソースコード内で2番目の重要な
副作用を持つこともできます：それは[`ErrorGuaranteed`][errorguar]を生成します。

`ErrorGuaranteed`は、[`rustc_errors`][rerrors]クレート外では構築不可能な
ゼロサイズ型です。エラーがユーザーに報告されるたびに生成されるため、
コンパイラコードが`ErrorGuaranteed`型の値に遭遇した場合、
コンパイルが_静的に失敗することが保証_されます。これは、
エラーコードパスが失敗につながることを静的にチェックできるため、
不健全性バグを回避するのに役立ちます。

`ErrorGuaranteed`の使用に関するいくつかの重要な考慮事項があります：

* エラーの_種類_に関する情報を伝えることは_ありません_。例えば、
  エラーは（間接的に）遅延バグまたは他のコンパイラエラーによるものかもしれません。
  したがって、エラーを出力するかどうか、またはどの種類のエラーを
  出力するかを決定する際に、`ErrorGuaranteed`に依存すべきではありません。
* `ErrorGuaranteed`は、コンパイルが将来エラーを_出力する_ことを示すために使用すべきではありません。
  エラーが_既に出力された_ことを示すために使用すべきです -- つまり、[`emit()`][emit]関数が
  既に呼び出されている必要があります。例えば、コンパイラの将来の部分がエラーを出力することを
  検出した場合、最初にエラーまたは遅延バグ自体を出力しない限り、`ErrorGuaranteed`を使用_できません_。

ありがたいことに、ほとんどの場合、`ErrorGuaranteed`を誤用することは
静的に不可能であるべきです。

[errorguar]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/struct.ErrorGuaranteed.html
[rerrors]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/index.html
[emit]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/diagnostic/struct.Diag.html#method.emit
