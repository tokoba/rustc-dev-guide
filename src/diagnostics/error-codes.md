# エラーコード

一般的に、各エラーメッセージに`E0123`のような一意のコードを割り当てようとしています。
これらのコードは、各クレートの`diagnostics.rs`ファイルで定義されており、
基本的にはマクロで構成されています。すべてのエラーコードには関連する
説明があります：新しいエラーコードにはそれらを含める必要があります。すべての_歴史的_
（もはや出力されない）エラーコードに説明があるわけではないことに注意してください。

## エラー説明

説明はMarkdownで書かれており（構文の詳細については[CommonMark Spec]を参照）、
それらすべては[`rustc_error_codes`]クレートにリンクされています。長いエラー
コードのフォーマットと書き方の詳細については、[RFC 1567]をお読みください。
<!-- date-check --> 2023年2月現在、この大部分時代遅れのRFCを
新しいより柔軟な標準に置き換える取り組み[^new-explanations]があります。

エラー説明は、エラーメッセージを拡張し、エラーが発生する_理由_に関する詳細を
提供する必要があります。ユーザーがクイックフィックスをコピーペーストすることは役に立ちません；
説明は、コンパイラがコードを受け入れることができない理由をユーザーが理解するのを
助けるべきです。Rustは役立つエラーメッセージを誇りにしており、長形式の
説明も例外ではありません。ただし、エラー説明が刷新される[^new-explanations]前に、
正確にどのように書くべきかは少し開かれています。いつものように：レビュアーに尋ねるか、
Rust Zulipで尋ねてください。

[^new-explanations]: ドラフトRFCは[ここ][new-explanations-rfc]を参照してください。

[`rustc_error_codes`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_error_codes/index.html
[CommonMark Spec]: https://spec.commonmark.org/current/
[RFC 1567]: https://github.com/rust-lang/rfcs/blob/master/text/1567-long-error-codes-explanation-normalization.md
[new-explanations-rfc]: https://github.com/rust-lang/rfcs/pull/3370

## 新しいコードの割り当て

エラーコードは`compiler/rustc_error_codes`に保存されています。

新しいエラーを作成するには、まず次に利用可能な
コードを見つける必要があります。`tidy`で見つけることができます：

```
./x test tidy
```

これにより、tidyスクリプトが呼び出されます。これは一般的に、コードが
コーディング規約に従っているかどうかをチェックします。これらのジョブの一部は、エラーコードをチェックし、
重複がないことなどを確認します（tidyチェックは
`src/tools/tidy/src/error_codes.rs`で定義されています）。それが完了すると、tidyは
使用されている最も高いエラーコードを出力します：

```
...
tidy check
Found 505 error codes
Highest error code: `E0591`
...
```

ここでは、使用中の最も高いエラーコードが`E0591`であることがわかるので、_おそらく_
`E0592`が必要です。確実にするには、`rg E0592`を実行してチェックし、参照が表示されないことを確認します。

エラーの詳細説明を書く必要があります。
これは`rustc_error_codes/src/error_codes/E0592.md`に入ります。
エラーを登録するには、`rustc_error_codes/src/error_codes.rs`を開いて、
コード（適切な数値順序で）を`register_diagnostics!`マクロに追加します。次のようになります：

```rust
register_diagnostics! {
    ...
    E0592: include_str!("./error_codes/E0592.md"),
}
```

実際にエラーを発行するには、`struct_span_code_err!`マクロを使用できます：

```rust
struct_span_code_err!(self.dcx(), // ここにある`DiagCtxt`へのパス
                 span, // ソースで必要なスパン
                 E0592, // 新しいエラーコード
                 fluent::example::an_error_message)
    .emit() // 実際にエラーを発行
```

ノートや他のスニペットを追加したい場合は、`.emit()`を呼び出す前にメソッドを
呼び出すことができます：

```rust
struct_span_code_err!(...)
    .span_label(another_span, fluent::example::example_label)
    .span_note(another_span, fluent::example::separate_note)
    .emit()
```

エラーコードを追加するPRの例については、[#76143]を参照してください。

[#76143]: https://github.com/rust-lang/rust/pull/76143

## エラーコードのdoctestsの実行

`rustc_error_codes/src/error_codes`に追加された例をテストするには、
次を使用してエラーインデックスジェネレーターを実行します：

```
./x test ./src/tools/error_index_generator
```
