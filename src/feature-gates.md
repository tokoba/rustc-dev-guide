# フィーチャーゲート

この章は、フィーチャーゲートの追加、削除、変更のための基本的なヘルプを提供することを目的としています。

これは*言語*フィーチャーゲートに固有のものであることに注意してください。*ライブラリ*フィーチャーゲートは[異なるメカニズム][libs-gate]を使用します。

[libs-gate]: ./stability.md

## フィーチャーゲートの追加

手順については、「新機能の実装」セクションの["コードでの安定性"][adding]を参照してください。

[adding]: ./implementing_new_features.md#stability-in-code

## フィーチャーゲートの削除

[removing]: #removing-a-feature-gate

フィーチャーゲートを削除するには、次の手順に従います：

1. `rustc_feature/src/unstable.rs` のフィーチャーゲート宣言を削除します。次のようになります：

   ```rust,ignore
   /// フィーチャーの説明
   (unstable, $feature_name, "$version", Some($tracking_issue_number))
   ```

2. 削除したばかりのフィーチャーゲート宣言の変更版を `rustc_feature/src/removed.rs` に追加します：

   ```rust,ignore
   /// フィーチャーの説明
   (removed, $old_feature_name, "$version", Some($tracking_issue_number),
    Some("$why_it_was_removed"))
   ```


## フィーチャーゲートの名前変更

[renaming]: #renaming-a-feature-gate

フィーチャーゲートの名前を変更するには、次の手順に従います（最初の2つは[フィーチャーゲートを削除する][removing]ときと同じ手順です）：

1. `rustc_feature/src/unstable.rs` の古いフィーチャーゲート宣言を削除します。次のようになります：

   ```rust,ignore
   /// フィーチャーの説明
   (unstable, $old_feature_name, "$version", Some($tracking_issue_number))
   ```

2. 削除したばかりの古いフィーチャーゲート宣言の変更版を `rustc_feature/src/removed.rs` に追加します：

   ```rust,ignore
   /// フィーチャーの説明
   /// `$new_feature_name` に名前変更
   (removed, $old_feature_name, "$version", Some($tracking_issue_number),
    Some("renamed to `$new_feature_name`"))
   ```

3. 新しい名前のフィーチャーゲート宣言を `rustc_feature/src/unstable.rs` に追加します。古い宣言と非常に似ているはずです：

   ```rust,ignore
   /// フィーチャーの説明
   (unstable, $new_feature_name, "$version", Some($tracking_issue_number))
   ```


## フィーチャーの安定化

手順については、「フィーチャーの安定化」章の["フィーチャーゲートリストの更新"]を参照してください。宣言を更新するだけでなく、他にも必要な手順があります！


["Stability in code"]: ./implementing_new_features.md#stability-in-code
["フィーチャーゲートリストの更新"]: ./stabilization_guide.md#updating-the-feature-gate-listing
