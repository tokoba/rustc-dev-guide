# 通知グループ

**通知グループ**は、大規模なプロジェクトにコミットすることなく、「少しずつ」の形でrustcに貢献するための簡単な方法です。
通知グループは**[参加が簡単](#join)**で（PRを提出するだけ！）、参加しても特定のコミットメントは必要ありません。

[通知グループに参加](#join)すると、通知グループの基準に適合する新しい問題がGitHub上で見つかるたびに、pingを受け取るリストに追加されます。興味があれば、その問題を[クレームして][claim the issue]作業を開始できます。

もちろん、新しい問題がタグ付けされるのを待つ必要はありません！お好みであれば、通知グループのGitHubラベルを使用して、まだクレームされていない既存の問題を検索できます。

[claim the issue]: https://forge.rust-lang.org/triagebot/issue-assignment.html

## 通知グループの一覧

通知グループの一覧は以下の通りです：

- [Apple](./apple.md)
- [ARM](./arm.md)
- [Emscripten](./emscripten.md)
- [RISC-V](./risc-v.md)
- [WASI](./wasi.md)
- [WebAssembly](./wasm.md)
- [Windows](./windows.md)
- [Rust for Linux](./rust-for-linux.md)

## どのような問題が通知グループに適していますか？

通知グループは、特に**中程度の優先度**の**独立した**バグについてpingを受けます：

- **独立した**とは、バグを修正するために大規模なリファクタリングが必要ないことを意味します。
- **中程度の優先度**とは、バグを修正したいと思っているものの、他のすべてを中止してまで修正するほど緊急な問題ではないことを意味します。もちろん、このようなバグの危険性は時間とともに蓄積される可能性があり、通知グループの役割はそれを防ぐことです！

<a id="join"></a>

## 通知グループへの参加

通知グループに参加するには、RustチームリポジトリのappropriateなファイルにGitHubユーザー名を追加するPRを開くだけです。
正確な手順を知り、編集するファイルを特定するには、以下の「PRの例」を参照してください。

また、まだRustチームのメンバーでない場合は、ファイルに名前を追加するだけでなく、リポジトリをチェックアウトして次のコマンドを実行する必要があります：

```bash
cargo run add-person $your_user_name
```

PRの例：

- [Appleグループに自分を追加する例。](https://github.com/rust-lang/team/pull/1434)
- [ARMグループに自分を追加する例。](https://github.com/rust-lang/team/pull/358)
- [Emscriptenグループに自分を追加する例。](https://github.com/rust-lang/team/pull/1579)
- [RISC-Vグループに自分を追加する例。](https://github.com/rust-lang/team/pull/394)
- [WASIグループに自分を追加する例。](https://github.com/rust-lang/team/pull/1580)
- [WebAssemblyグループに自分を追加する例。](https://github.com/rust-lang/team/pull/1581)
- [Windowsグループに自分を追加する例。](https://github.com/rust-lang/team/pull/348)

## 通知グループの問題にタグを付ける

通知グループに適した問題としてタグを付けるには、通知グループの名前を指定して[rustbot]に[`ping`]コマンドを発行します。例えば：

```text
@rustbot ping apple
@rustbot ping arm
@rustbot ping emscripten
@rustbot ping risc-v
@rustbot ping wasi
@rustbot ping wasm
@rustbot ping windows
```

いくつかのコマンドをより短く覚えやすくするために、[`triagebot.toml`]ファイルで定義されているエイリアスがあります。例えば、以下のコマンドはすべて同等で、Appleグループにpingします：

```text
@rustbot ping apple
@rustbot ping macos
@rustbot ping ios
```

これらのエイリアスは人間の生活を楽にするためのものであることに留意してください。
変更される可能性があります。コマンドが常に有効であることを確認する必要がある場合は、エイリアスよりも完全な呼び出しを優先してください。

**ただし、これはコンパイラチームのメンバーまたは貢献者のみが行うべきで、通常はコンパイラチームのトリアージの一環として行われます。**

[rustbot]: https://github.com/rust-lang/triagebot/
[`ping`]: https://forge.rust-lang.org/triagebot/pinging.html
[`triagebot.toml`]: https://github.com/rust-lang/rust/blob/HEAD/triagebot.toml
