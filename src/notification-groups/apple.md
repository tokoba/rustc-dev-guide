# Apple通知グループ

**Githubラベル:** [O-macos], [O-ios], [O-tvos], [O-watchos] および [O-visionos] <br>
**Pingコマンド:** `@rustbot ping apple`

このリストは、Apple関連の問題の診断とテストの両方の支援を求めるだけでなく、macOS/iOS/tvOS/watchOS/visionOSサポートに関する興味深い質問の解決方法についての提案を求めるために使用されます。

グループが行うことをよりよく理解するために、最善の行動方針を決定するためにグループのアドバイスを求めたであろう質問の種類の例をいくつか示します：

* 最小サポートバージョンの引き上げ（例：[#104385]）
* 追加のAppleターゲット（例：[#121419]）
* 不明瞭なXcodeリンカーの詳細（例：[#121430]）

[O-macos]: https://github.com/rust-lang/rust/labels/O-macos
[O-ios]: https://github.com/rust-lang/rust/labels/O-ios
[O-tvos]: https://github.com/rust-lang/rust/labels/O-tvos
[O-watchos]: https://github.com/rust-lang/rust/labels/O-watchos
[O-visionos]: https://github.com/rust-lang/rust/labels/O-visionos
[#104385]: https://github.com/rust-lang/rust/pull/104385
[#121419]: https://github.com/rust-lang/rust/pull/121419
[#121430]: https://github.com/rust-lang/rust/pull/121430

## デプロイメントターゲット

Appleプラットフォームには「デプロイメントターゲット」という概念があり、`*_DEPLOYMENT_TARGET`環境変数で制御され、バイナリが実行される最小OSバージョンを指定します。

`rustc`がデフォルトで使用するものよりも新しいOSバージョンからの標準ライブラリのAPIを使用すると、静的リンカーエラーまたは動的リンカーエラーのいずれかが発生します。
このため、`extern "C"` APIについては、導入されたOSバージョンを文書化するように提案し、それが`rustc`が使用する現在のデフォルトよりも新しい場合は、weak linkingの使用を提案してください。

## App StoreとプライベートAPI

Appleは文書化されていないAPIの使用について非常に保護的であるため、変更が新しい関数を使用する場合は常に、それが実際に公開APIであることを確認することが重要です。文書化されていないAPIをバイナリで言及するだけ（呼び出さなくても）でも、App Storeからのリジェクトにつながる可能性があるためです。

例えば、Darwin / XNUカーネルには実際にfutexシステムコールがありますが、公開APIではないため、`std`では使用できません。

一般的に、APIがAppleによって公開と見なされるためには、以下の条件を満たす必要があります：

* 公開ヘッダーに現れる（つまり、Xcodeと一緒に配布され、`xcrun --show-sdk-path --sdk $SDK`で特定のプラットフォーム用に見つかるもの）。
* 可用性属性がある（`__API_AVAILABLE`、`API_AVAILABLE`など）。
