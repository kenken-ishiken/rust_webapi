# 要件定義管理ガイド

このドキュメントでは、GitHub Issueを使用した要件定義の作成・管理プロセスについて説明します。

## 🎯 概要

本プロジェクトでは、すべての新機能・機能拡張・バグ修正について、GitHub Issueを使用して要件定義を行います。これにより、以下の効果を実現します：

- 要件の透明性と追跡可能性の確保
- ステークホルダー間のコミュニケーション改善
- TDD（テスト駆動開発）との連携
- 開発プロセスの標準化

## 📋 要件定義テンプレート

### テンプレートの場所
`.github/ISSUE_TEMPLATE/requirement-definition.md`

### テンプレートに含まれる項目
- **基本情報**: タイトル、概要、優先度、工数見積もり
- **機能要件**: ユースケース、入力・処理・出力の詳細
- **API仕様**: エンドポイント、リクエスト・レスポンス形式
- **データベース設計**: テーブル設計、マイグレーション
- **非機能要件**: パフォーマンス、セキュリティ、可用性
- **テスト計画**: 単体・統合・パフォーマンス・セキュリティテスト
- **受け入れ条件**: 機能的・技術的・運用条件
- **実装計画**: フェーズ別の実装ステップ

## 🚀 クイックスタート

### 1. 要件定義Issueの作成

#### 自動化スクリプトを使用（推奨）
```bash
./scripts/create-requirement-issue.sh
```

#### GitHub CLI を直接使用
```bash
gh issue create \
  --title "[REQ] 機能名" \
  --template requirement-definition.md \
  --label requirement,status/draft,component/api,priority/high \
  --assignee @me
```

#### GitHub Web UI を使用
1. GitHubのIssuesページにアクセス
2. "New issue" をクリック
3. "要件定義" テンプレートを選択
4. 必要な情報を入力

### 2. 要件定義の管理

#### Issue一覧の確認
```bash
./scripts/manage-requirements.sh list
./scripts/manage-requirements.sh list draft  # ドラフト状態のみ
```

#### 状態の更新
```bash
./scripts/manage-requirements.sh review 123    # レビュー準備
./scripts/manage-requirements.sh approve 123   # 承認
./scripts/manage-requirements.sh start 123     # 実装開始
./scripts/manage-requirements.sh testing 123   # テスト開始
./scripts/manage-requirements.sh done 123      # 完了
```

#### ダッシュボードとメトリクス
```bash
./scripts/manage-requirements.sh dashboard  # 全体的な状況確認
./scripts/manage-requirements.sh metrics    # 詳細なメトリクス
```

## 🔄 ワークフロー

### フェーズ1: 要件分析 📋
**期間**: 2-5日  
**参加者**: Product Owner, Tech Lead, UI/UX Designer

1. **要件収集**
   - ステークホルダーへのヒアリング
   - ユースケースの洗い出し
   - 競合調査・技術調査
   - 制約条件の整理

2. **要件分析**
   - 機能要件の詳細化
   - 非機能要件の定義
   - API仕様の設計
   - データベース設計

3. **Issue作成**
   - 要件定義テンプレートの記入
   - 適切なラベルの設定
   - 優先度の設定
   - 担当者のアサイン

**成果物**: 要件定義Issue, API設計書, データベース設計書, テスト計画書

### フェーズ2: 設計レビュー 🔍
**期間**: 1-2日  
**参加者**: Tech Lead, Senior Developers, Product Owner, DevOps Engineer

1. **技術レビュー**
   - アーキテクチャの妥当性確認
   - パフォーマンス要件の実現可能性
   - セキュリティ要件の適切性
   - 既存システムとの整合性

2. **ビジネスレビュー**
   - ビジネス価値の確認
   - 優先度の妥当性
   - リソース配分の適切性
   - リリース計画との整合性

3. **承認プロセス**
   - レビュー結果の記録
   - 修正事項の反映
   - 最終承認の取得
   - 実装フェーズへの移行判定

**成果物**: レビュー結果レポート, 承認済み要件定義, 実装計画書

### フェーズ3: 実装 ⚙️
**期間**: 5-15日  
**参加者**: Developers, QA Engineers

1. **開発環境準備**
   - ブランチの作成
   - 開発環境のセットアップ
   - 必要な依存関係の追加
   - CI/CDパイプラインの更新

2. **TDD実装サイクル**
   - 🔴 Red: 失敗テストの作成
   - 🟢 Green: 最小限の実装
   - 🔵 Refactor: コード改善
   - 継続的な品質チェック

3. **統合テスト**
   - APIテストの実行
   - データベーステストの実行
   - パフォーマンステストの実行
   - セキュリティテストの実行

**成果物**: 実装済みコード, テストコード, テスト実行結果, API仕様書の更新

### フェーズ4: テスト・検証 🧪
**期間**: 2-5日  
**参加者**: QA Engineers, Product Owner, Developers

1. **機能テスト**
   - 受け入れ条件の確認
   - エンドツーエンドテスト
   - ユーザビリティテスト
   - 回帰テストの実行

2. **非機能テスト**
   - パフォーマンステスト
   - セキュリティテスト
   - 可用性テスト
   - スケーラビリティテスト

3. **受け入れテスト**
   - Product Ownerによる確認
   - ステークホルダーレビュー
   - 本番環境での動作確認
   - 運用手順の確認

**成果物**: テスト実行レポート, 不具合レポート, 受け入れテスト結果, 運用手順書

### フェーズ5: デプロイメント 🚀
**期間**: 1-2日  
**参加者**: DevOps Engineer, Tech Lead, Support Team

1. **デプロイ準備**
   - デプロイメント計画の確認
   - ロールバック手順の準備
   - 監視・アラートの設定
   - ドキュメントの更新

2. **デプロイ実行**
   - ステージング環境での最終確認
   - 本番環境へのデプロイ
   - 動作確認テストの実行
   - 監視メトリクスの確認

3. **リリース後対応**
   - リリースノートの公開
   - ユーザーへの通知
   - サポート体制の準備
   - フィードバック収集の開始

**成果物**: リリースノート, デプロイメントレポート, 監視ダッシュボード, サポートドキュメント

## 🏷️ ラベル管理

### 状態ラベル
- `status/draft` - ドラフト状態
- `status/review` - レビュー中
- `status/approved` - 承認済み
- `status/in-progress` - 実装中
- `status/testing` - テスト中
- `status/done` - 完了

### タイプラベル
- `type/feature` - 新機能
- `type/enhancement` - 機能拡張
- `type/bugfix` - バグ修正
- `type/refactor` - リファクタリング

### 優先度ラベル
- `priority/critical` - クリティカル
- `priority/high` - 高優先度
- `priority/medium` - 中優先度
- `priority/low` - 低優先度

### コンポーネントラベル
- `component/api` - API関連
- `component/database` - データベース関連
- `component/auth` - 認証関連
- `component/frontend` - フロントエンド関連

## 📊 品質ゲート

### 要件定義の品質チェック
- [ ] すべての必須項目が記載されている
- [ ] 受け入れ条件が具体的である
- [ ] 非機能要件が適切に定義されている
- [ ] 実装計画が現実的である
- [ ] ステークホルダーの承認を得ている

### 実装の品質チェック
- [ ] テストカバレッジ >= 90%
- [ ] Clippyの警告 = 0
- [ ] パフォーマンステスト合格
- [ ] セキュリティテスト合格
- [ ] コードレビューの完了

## 🔧 ツールとコマンド

### 必要なツール
- [GitHub CLI](https://cli.github.com/) - Issueの作成・管理
- [jq](https://stedolan.github.io/jq/) - JSONデータの処理

### よく使用するコマンド

#### Issue作成
```bash
# 対話式Issue作成
./scripts/create-requirement-issue.sh

# 直接作成
gh issue create --title "[REQ] 新機能名" --template requirement-definition.md
```

#### Issue管理
```bash
# Issue一覧表示
gh issue list --label requirement

# 特定の状態のIssue表示
gh issue list --label "requirement,status/review"

# Issue詳細表示
gh issue view 123

# Issue編集
gh issue edit 123 --add-label priority/high
```

#### 状態変更
```bash
# レビュー準備
gh issue edit 123 --remove-label status/draft --add-label status/review

# 承認
gh issue edit 123 --remove-label status/review --add-label status/approved

# 実装開始
gh issue edit 123 --remove-label status/approved --add-label status/in-progress

# 完了（クローズ）
gh issue edit 123 --remove-label status/testing --add-label status/done --state closed
```

## 🔍 トラブルシューティング

### よくある問題と解決方法

#### 要件が不明確で実装に進めない
**解決策:**
- ステークホルダーと再度詳細を確認
- プロトタイプを作成して要件を明確化
- 受け入れ条件を具体化

#### 実装中に要件が拡大する（スコープクリープ）
**解決策:**
- 変更管理プロセスの適用
- 影響分析の実施
- 優先度の再評価

#### 品質要件を満たせない
**解決策:**
- テスト戦略の見直し
- 追加のコードレビュー
- リファクタリングの実施

## 📚 参考資料

### 内部ドキュメント
- [TDD設定ドキュメント](./tdd-config.yml)
- [要件定義ワークフロー](./requirement-workflow.yml)
- [プロジェクト概要](./project-overview.md)
- [開発ガイド](./development-guide.md)

### 外部リソース
- [GitHub Issues](https://docs.github.com/en/issues)
- [GitHub CLI](https://cli.github.com/manual/)
- [アジャイル要件定義のベストプラクティス](https://agilealliance.org/agile101/)

## 📞 サポート

要件定義プロセスに関する質問や改善提案がある場合は、以下の方法でお問い合わせください：

- Slack: `#requirements` チャンネル
- Email: tech-lead@example.com
- GitHub Discussion: プロジェクトのDiscussionsページ

---

最終更新: 2025年6月1日  
メンテナー: Development Team
