# 推奨ワークフロー

完全なブートストラッププロセスにはかなりの時間がかかります。ここでは、生活を楽にするためのいくつかの提案があります。

## pre-push フックのインストール

コードの品質を保証する内部ツールである `tidy` に合格しない場合、CI は自動的にビルドを失敗させます。必要に応じて、各プッシュで `./x test tidy` を自動的に実行する [Git フック](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks) をインストールして、コードが標準に達していることを確認できます。フックが失敗した場合は、`./x test tidy --bless` を実行し、変更をコミットします。後で pre-push 動作が望ましくないと判断した場合は、`.git/hooks` の `pre-push` ファイルを削除できます。

事前ビルドされた git フックは [`src/etc/pre-push.sh`] にあります。これは、`.git/hooks` フォルダに `pre-push`（`.sh` 拡張子なし！）としてコピーできます。

`./x setup` を実行する手順の一環として、フックをインストールすることもできます！

## 設定拡張

異なるタスクで作業する際、異なるブートストラップ設定を切り替える必要がある場合があります。将来の使用のために古い設定を保持したい場合があります。しかし、ランダムなファイルに生の設定値を保存し、手動でコピーアンドペーストすると、特に長い設定履歴がある場合、すぐに乱雑になります。

複数の設定の管理を簡素化するために、設定拡張を作成できます。

たとえば、`cross.toml` という名前のシンプルな設定ファイルを作成できます：

```toml
[build]
build = "x86_64-unknown-linux-gnu"
host = ["i686-unknown-linux-gnu"]
target = ["i686-unknown-linux-gnu"]


[llvm]
download-ci-llvm = false

[target.x86_64-unknown-linux-gnu]
llvm-config = "/path/to/llvm-19/bin/llvm-config"
```

次に、これを `bootstrap.toml` に含めます：

```toml
include = ["cross.toml"]
```

拡張内に拡張を再帰的に含めることもできます。

**注意**: `include` フィールドでは、オーバーライドロジックは右から左の順序に従います。たとえば、`include = ["a.toml", "b.toml"]` では、拡張 `b.toml` が `a.toml` をオーバーライドします。また、親拡張は常に内部のものをオーバーライドします。

## `rustc` 用の `rust-analyzer` の設定

### 「library」ツリーのチェック

「library」ツリーのチェックには stage1 コンパイラが必要で、一部のコンピュータでは負荷の高いプロセスになる可能性があります。このため、ブートストラップには `--skip-std-check-if-no-download-rustc` というフラグがあり、`rust.download-rustc` が利用できない場合は「library」ツリーのチェックをスキップします。`rust-analyzer` でコンピュータに重い負荷をかけたくない場合は、`rust-analyzer` 設定の `./x check` コマンドに `--skip-std-check-if-no-download-rustc` フラグを追加できます。

### プロジェクトローカル rust-analyzer セットアップ

`rust-analyzer` は、ファイルを保存するたびにコードをチェックしてフォーマットするのに役立ちます。デフォルトでは、`rust-analyzer` は `cargo check` と `rustfmt` コマンドを実行しますが、`rustc` をハックする際に、これらのツールのより適応されたバージョンを使用するようにこれらのコマンドをオーバーライドできます。カスタムセットアップを使用すると、`rust-analyzer` は `./x check` を使用してソースをチェックし、stage 0 rustfmt を使用してフォーマットできます。

デフォルトの `rust-analyzer.check.overrideCommand` コマンドラインは、リポジトリ内のすべてのクレートとツールをチェックします。特定の部分で作業している場合は、チェック時間を節約するために、作業している部分のみをチェックするようにコマンドをオーバーライドできます。たとえば、コンパイラで作業している場合は、コンパイラ部分のみをチェックするために、コマンドを `x check compiler --json-output` にオーバーライドできます。利用可能な部分を確認するには、`x check --help --verbose` を実行できます。

`./x setup editor` を実行すると、サポートされているエディタの1つに対してプロジェクトローカル LSP 設定ファイルを作成するよう求められます。`./x setup` を実行する手順の一環として設定ファイルを作成することもできます。

### rust-analyzer 用の別のビルドディレクトリの使用

デフォルトでは、rust-analyzer がチェックまたはフォーマットコマンドを実行すると、手動のコマンドラインビルドと同じビルドディレクトリを共有します。これは2つの理由で不便です：
- 各ビルドがビルドディレクトリをロックし、他方を待たせるため、rust-analyzer がバックグラウンドでコマンドを実行している間にコマンドラインビルドを実行することが不可能になります。
- コンパイラフラグやその他の設定の競合により、ビルドの一方が以前にビルドされたアーティファクトを削除するリスクが高まり、場合によっては追加の再ビルドが強制されます。

これらの問題を回避するには：
- エディタの rust-analyzer 設定のすべてのカスタム `x` コマンドに `--build-dir=build/rust-analyzer` を追加します。（必要に応じて別のディレクトリ名を選択してください。）
- `rust-analyzer.rustfmt.overrideCommand` 設定を変更して、その他のビルドディレクトリ内の `rustfmt` のコピーを指すようにします。
- `rust-analyzer.procMacro.server` 設定を変更して、その他のビルドディレクトリ内の `rust-analyzer-proc-macro-srv` のコピーを指すようにします。

コマンドラインビルドと rust-analyzer に別々のビルドディレクトリを使用するには、追加のディスクスペースが必要です。

### Visual Studio Code

`./x setup editor` で `vscode` を選択すると、Visual Studio Code を設定する `.vscode/settings.json` ファイルを作成するよう求められます。推奨される `rust-analyzer` 設定は [`src/etc/rust_analyzer_settings.json`] にあります。

保存時に `./x check` を実行することが不便な場合、VS Code では代わりに [Build Task] を使用できます：

```JSON
// .vscode/tasks.json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "./x check",
            "command": "./x check",
            "type": "shell",
            "problemMatcher": "$rustc",
            "presentation": { "clear": true },
            "group": { "kind": "build", "isDefault": true }
        }
    ]
}
```

[Build Task]: https://code.visualstudio.com/docs/editor/tasks


### Neovim

Neovim ユーザーには、いくつかのオプションがあります。最も簡単な方法は、[neoconf.nvim](https://github.com/folke/neoconf.nvim/) を使用することです。これにより、ネイティブ LSP でプロジェクトローカル設定ファイルが可能になります。使用方法の手順は以下のとおりです。これらには、rust-analyzer がすでに Neovim で設定されている必要があることに注意してください。これの手順は [こちら](https://rust-analyzer.github.io/manual.html#nvim-lsp) にあります。

1. まず、プラグインをインストールします。これは、README の手順に従って行うことができます。
2. `./x setup editor` を実行し、`vscode` を選択して `.vscode/settings.json` ファイルを作成します。`neoconf` は、このファイルが検出されたときにプロジェクトが開かれたときに、rust-analyzer 設定を自動的に読み取って更新できます。

`coc.nvim` を使用している場合は、`./x setup editor` を実行し、`vim` を選択して `.vim/coc-settings.json` を作成できます。設定は `:CocLocalConfig` で編集できます。推奨設定は [`src/etc/rust_analyzer_settings.json`] にあります。

別の方法は、プラグインなしで、設定に独自のロジックを作成することです。以下のコードは、rust-lang/rust の任意のチェックアウト（2025年2月以降）で機能します：

```lua
local function expand_config_variables(option)
    local var_placeholders = {
        ['${workspaceFolder}'] = function(_)
            return vim.lsp.buf.list_workspace_folders()[1]
        end,
    }

    if type(option) == "table" then
        local mt = getmetatable(option)
        local result = {}
        for k, v in pairs(option) do
            result[expand_config_variables(k)] = expand_config_variables(v)
        end
        return setmetatable(result, mt)
    end
    if type(option) ~= "string" then
        return option
    end
    local ret = option
    for key, fn in pairs(var_placeholders) do
        ret = ret:gsub(key, fn)
    end
    return ret
end
lspconfig.rust_analyzer.setup {
    root_dir = function()
        local default = lspconfig.rust_analyzer.config_def.default_config.root_dir()
        -- the default root detection uses the cargo workspace root.
        -- but for rust-lang/rust, the standard library is in its own workspace.
        -- use the git root instead.
        local compiler_config = vim.fs.joinpath(default, "../src/bootstrap/defaults/config.compiler.toml")
        if vim.fs.basename(default) == "library" and vim.uv.fs_stat(compiler_config) then
            return vim.fs.dirname(default)
        end
        return default
    end,
    on_init = function(client)
        local path = client.workspace_folders[1].name
        local config = vim.fs.joinpath(path, "src/etc/rust_analyzer_zed.json")
        if vim.uv.fs_stat(config) then
            -- load rust-lang/rust settings
            local file = io.open(config)
            local json = vim.json.decode(file:read("*a"))
            client.config.settings["rust-analyzer"] = expand_config_variables(json.lsp["rust-analyzer"].initialization_options)
            client.notify("workspace/didChangeConfiguration", { settings = client.config.settings })
        end
        return true
    end
}
```

上記で説明したビルドタスクを使用したい場合は、設定で独自のコマンドを作成するか、VSCode の `task.json` ファイルを [読み取れる](https://github.com/stevearc/overseer.nvim/blob/master/doc/guides.md#vs-code-tasks) [overseer.nvim](https://github.com/stevearc/overseer.nvim) などのプラグインをインストールして、上記と同じ手順に従うことができます。

### Emacs

Emacs は、[Eglot](https://www.gnu.org/software/emacs/manual/html_node/eglot/) を介してプロジェクトローカル設定で rust-analyzer をサポートします。Eglot を rust-analyzer でセットアップする手順は [こちら](https://rust-analyzer.github.io/manual.html#eglot) にあります。一般的に Rust 開発用に Emacs と Eglot をセットアップしたら、`./x setup editor` を実行し、`emacs` を選択できます。これにより、Eglot の推奨設定で `.dir-locals.el` を作成するよう求められます。推奨設定は [`src/etc/rust_analyzer_eglot.el`] にあります。プロジェクト固有の Eglot 設定の詳細については、[マニュアル](https://www.gnu.org/software/emacs/manual/html_node/eglot/Project_002dspecific-configuration.html) を参照してください。

### Helix

Helix には、組み込みの LSP と rust-analyzer サポートが付属しています。[こちら](https://docs.helix-editor.com/languages.html) で説明されているように、`languages.toml` を介して設定できます。`./x setup editor` を実行し、`helix` を選択すると、Helix の推奨設定で `languages.toml` を作成するよう求められます。推奨設定は [`src/etc/rust_analyzer_helix.toml`] にあります。

### Zed

Zed には、組み込みの LSP と rust-analyzer サポートが付属しています。[こちら](https://zed.dev/docs/configuring-languages) で説明されているように、`.zed/settings.json` を介して設定できます。`./x setup editor` で `zed` を選択すると、推奨設定で Zed を設定する `.zed/settings.json` ファイルを作成するよう求められます。推奨される `rust-analyzer` 設定は [`src/etc/rust_analyzer_zed.json`] にあります。

## Check、check、そして再び check

シンプルなリファクタリングを行う場合、`./x check` を継続的に実行すると便利です。上記のように `rust-analyzer` をセットアップした場合、ファイルを保存するたびにこれが自動的に実行されます。ここでは、コンパイラが **ビルド** できるかどうかをチェックしているだけですが、多くの場合それで十分です（たとえば、メソッドの名前を変更する場合）。実際にテストを実行する必要があるときに `./x build` を実行できます。

実際には、コードが 100% 機能することを確信していない場合でも、テストを延期することが有用な場合があります。次に、リファクタリングコミットを積み重ねて、後の時点でのみテストを実行できます。次に、`git bisect` を使用して、**正確に** どのコミットが問題を引き起こしたかを追跡できます。このスタイルの良い副作用は、最終的にかなり細かいコミットのセットが残り、すべてがビルドされてテストに合格することです。これはレビューに役立つことがよくあります。

## `rustup` を nightly を使用するように設定する

ブートストラッププロセスの一部は、rustfmt などのツールのピン留めされた nightly バージョンを使用します。リポジトリで `cargo fmt` などを正しく動作させるには、次を実行します

```console
cd <path to rustc repo>
rustup override set nightly
```

`rustup` で [nightly ツールチェーンをインストール] した後。[worktree をセットアップした] すべてのディレクトリに対してこれを行うことを忘れないでください。`src/stage0` からピン留めされた nightly バージョンを使用する必要がある場合がありますが、多くの場合、通常の `nightly` チャネルが機能します。

**注意** `x` が使用する実際の rustfmt でそれを設定する方法については、[vscode のセクション] を参照してください。ブートストラップされたコンパイラ用に `rustup` ツールチェーンをセットアップする方法については、[rustup のセクション] を参照してください

**注意** これは、cargo で直接 `rustc` をビルドすることを許可するものでは _ありません_。コンパイラや標準ライブラリで作業するには、引き続き `x` を使用する必要があります。これは単に `cargo fmt` を使用できるようにするだけです。

[nightly ツールチェーンをインストール]: https://rust-lang.github.io/rustup/concepts/channels.html?highlight=nightl#working-with-nightly-rust
[worktree をセットアップした]: ./suggested.md#working-on-multiple-branches-at-the-same-time
[vscode のセクション]: suggested.md#configuring-rust-analyzer-for-rustc
[rustup のセクション]: how-to-build-and-run.md?highlight=rustup#creating-a-rustup-toolchain

## CI-rustc でのより高速なビルド

コンパイラで作業していない場合、多くの場合、コンパイラツリーをビルドする必要はありません。たとえば、コンパイラのビルドをスキップして、`library` ツリーまたは `src/tools` 下のツールのみをビルドできます。これを実現するには、設定で `download-rustc` オプションを設定してこれを有効にする必要があります。これにより、ブートストラップは `stage > 0` ステップに最新の nightly コンパイラを使用するように指示されます。つまり、2つの事前コンパイルされたコンパイラ、stage0 コンパイラと `stage > 0` ステップ用の `download-rustc` コンパイラを持つことになります。このようにして、インツリーコンパイラをビルドする必要がなくなります。その結果、インツリーコンパイラをビルドしないことで、ビルド時間が大幅に短縮されます。

## `--keep-stage-std` でのより高速な再ビルド

時には、コンパイラがビルドされるかどうかをチェックするだけでは十分ではありません。一般的な例は、状態の値を検査したり、問題をよりよく理解したりするために、`debug!` ステートメントを追加する必要がある場合です。その場合、実際には完全なビルドは必要ありません。ブートストラップのキャッシュ無効化をバイパスすることで、これらのビルドを非常に高速に（たとえば、約 30 秒で）完了させることができます。唯一の注意点は、これには少しのごまかしが必要で、機能しないコンパイラを生成する可能性があることです（ただし、それは簡単に検出して修正できます）。

必要なコマンドのシーケンスは次のとおりです：

- 初期ビルド：`./x build library`
- 後続のビルド：`./x build library --keep-stage-std=1`
  - ここで `--keep-stage-std=1` フラグを追加したことに注意してください

前述のように、`--keep-stage-std=1` の効果は、古い標準ライブラリを再利用できると _仮定する_ だけです。コンパイラを編集している場合、これは多くの場合真実です：結局のところ、標準ライブラリを変更していません。しかし、時にはそうではありません：たとえば、コンパイラが型や他の状態を `rlib` ファイルにエンコードする方法を制御する「メタデータ」部分を編集している場合、またはメタデータに含まれるもの（MIR の定義など）を編集している場合です。

**要するに、`--keep-stage-std=1` を使用してコンパイルすると、奇妙な動作が発生する可能性があります** -- たとえば、奇妙な [ICE](../appendix/glossary.html#ice) やその他のパニック。その場合は、単にコマンドから `--keep-stage-std=1` を削除して再ビルドすればよいでしょう。それで問題が解決するはずです。

テストを実行する際にも `--keep-stage-std=1` を使用できます。次のようなものです：

- 初期テスト実行：`./x test tests/ui`
- 後続のテスト実行：`./x test tests/ui --keep-stage-std=1`

## インクリメンタルコンパイルの使用

`--incremental` フラグをさらに有効にして、後続の再ビルドで追加の時間を節約できます：

```bash
./x test tests/ui --incremental --test-args issue-1234
```

すべてのコマンドでフラグを含めたくない場合は、`bootstrap.toml` で有効にできます：

```toml
[rust]
incremental = true
```

インクリメンタルコンパイルは、通常よりも多くのディスクスペースを使用することに注意してください。ディスクスペースが懸念事項である場合は、時々 `build` ディレクトリのサイズを確認することをお勧めします。

## 最適化の微調整

`optimize = false` を設定すると、コンパイラがテストには遅すぎます。ただし、テストサイクルを改善するために、再ビルドする必要があるクレートに対してのみ選択的に最適化を無効にできます（[ソース](https://rust-lang.zulipchat.com/#narrow/stream/131828-t-compiler/topic/incremental.20compilation.20question/near/202712165)）。たとえば、`rustc_mir_build` で作業している場合、`rustc_mir_build` と `rustc_driver` クレートがインクリメンタルに再ビルドするのに最も時間がかかります。したがって、ルート `Cargo.toml` に次のように設定できます：

```toml
[profile.release.package.rustc_mir_build]
opt-level = 0
[profile.release.package.rustc_driver]
opt-level = 0
```

## 同時に複数のブランチで作業する

並行して複数のブランチで作業すると、少し面倒になる可能性があります。1つのブランチでコンパイラをビルドすると、古いビルドとインクリメンタルコンパイルキャッシュが上書きされるためです。1つの解決策は、リポジトリの複数のクローンを持つことですが、それは Git メタデータを複数回保存し、各クローンを個別に更新する必要があることを意味します。

幸いなことに、Git には [worktrees] と呼ばれるより良いソリューションがあります。これにより、すべて同じ Git データベースを共有する複数の「作業ツリー」を作成できます。さらに、すべての worktree が同じオブジェクトデータベースを共有しているため、いずれかでブランチ（例：`main`）を更新すると、任意の worktree から新しいコミットを使用できます。ただし、1つの注意点は、サブモジュールは共有されないということです。それらは依然として複数回クローンされます。

[worktrees]: https://git-scm.com/docs/git-worktree

Rust リポジトリのルートディレクトリ内にいる場合、次のコマンドを実行することで、新しい「rust2」ディレクトリに「リンクされた作業ツリー」を作成できます：

```bash
git worktree add ../rust2
```

`main` に基づく新しいブランチの新しい worktree を作成するには、次のようになります：

```bash
git worktree add -b my-feature ../rust2 main
```

次に、その rust2 フォルダを `rustc` を変更およびビルドするための別のワークスペースとして使用できます！

## nix での作業

いくつかの nix 設定が `src/tools/nix-dev-shell` で定義されています。

direnv を使用している場合は、`src/tools/nix-dev-shell/envrc-flake` または `src/tools/nix-dev-shell/envrc-shell` へのシンボリックリンクを作成できます

```bash
ln -s ./src/tools/nix-dev-shell/envrc-flake ./.envrc # Use flake
```
または
```bash
ln -s ./src/tools/nix-dev-shell/envrc-shell ./.envrc # Use nix-shell
```

### 注意

NixOS 以外のディストリビューションで nix を使用する場合、**`bootstrap.toml` で `patch-binaries-for-nix = true` を設定する** 必要がある場合があることに注意してください。ブートストラップは nix で実行されているかどうかを検出し、パッチ適用を自動的に有効にしようとしますが、この検出には誤検知がある可能性があります。

nix シェルを使用して `bootstrap.toml` を管理することもできます：

```nix
let
  config = pkgs.writeText "rustc-config" ''
    # Your bootstrap.toml content goes here
  ''
pkgs.mkShell {
  /* ... */
  # This environment variable tells bootstrap where our bootstrap.toml is.
  RUST_BOOTSTRAP_CONFIG = config;
}
```

## シェル補完

Bash、Zsh、Fish、または PowerShell を使用している場合、[`src/etc/completions`](https://github.com/rust-lang/rust/tree/HEAD/src/etc/completions) で `x.py` 用の自動生成されたシェル補完スクリプトを見つけることができます。

`source ./src/etc/completions/x.py.<extension>` を使用して、選択したシェルの補完をロードできます。または、PowerShell の場合は `& .\src\etc\completions\x.py.ps1` を使用します。これをシェルのスタートアップスクリプト（例：`.bashrc`）に追加すると、この補完が自動的にロードされます。

[`src/etc/rust_analyzer_settings.json`]: https://github.com/rust-lang/rust/blob/HEAD/src/etc/rust_analyzer_settings.json
[`src/etc/rust_analyzer_eglot.el`]: https://github.com/rust-lang/rust/blob/HEAD/src/etc/rust_analyzer_eglot.el
[`src/etc/rust_analyzer_helix.toml`]: https://github.com/rust-lang/rust/blob/HEAD/src/etc/rust_analyzer_helix.toml
[`src/etc/rust_analyzer_zed.json`]: https://github.com/rust-lang/rust/blob/HEAD/src/etc/rust_analyzer_zed.json
[`src/etc/pre-push.sh`]: https://github.com/rust-lang/rust/blob/HEAD/src/etc/pre-push.sh
