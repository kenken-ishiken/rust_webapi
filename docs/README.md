# Rust WebAPI ドキュメント

このディレクトリには、Rust WebAPI プロジェクトの詳細なドキュメントが含まれています。

**最終更新**: 2024年12月（Phase 4-1: ドキュメント整備完了）

## 最新の実装改善（2024年12月）

以下のドキュメントは最新の実装内容を反映して更新されています：

- ✅ [アーキテクチャガイド](architecture-guide.md) - DI、削除統一、エラー/メトリクス統一を反映
- ✅ [API仕様書](api-documentation.md) - 統一されたエラーレスポンス形式、削除APIの更新
- ✅ [開発ガイド](development-guide.md) - DI、エラーハンドリング、メトリクス記録の方法
- ✅ [プロジェクト概要](project-overview.md) - 最新の改善内容を追加

## 利用可能なドキュメント

### API関連
- [API リファレンス](api-reference.md) - エンドポイントの詳細仕様、リクエスト・レスポンス例、curl使用例
- [API 仕様書](api-documentation.md) - REST API の包括的な仕様とエンドポイント一覧 **[更新済み]**
- [gRPC API](grpc-api.md) - Protocol Buffersを使用したgRPC API仕様

### アーキテクチャ・設計
- [アーキテクチャガイド](architecture-guide.md) - システム設計、データフロー、コンポーネント構成 **[更新済み]**
- [詳細アーキテクチャ](architecture-detailed.md) - 深掘りしたアーキテクチャ解説
- [プロジェクト概要](project-overview.md) - プロジェクト全体の概要と目標 **[更新済み]**
- [データベーススキーマ](database-schema.md) - データベース設計と関係性

### 開発・テスト
- [開発ガイド](development-guide.md) - 開発環境のセットアップ、テスト、デバッグ **[更新済み]**
- [開発ワークフロー＆テスティング](development-testing.md) - テスト戦略、コーディング規約、CI/CD
- [設定ガイド](configuration-guide.md) - 環境変数と設定管理

### 運用・デプロイ
- [運用ガイド](operations-guide.md) - デプロイ、監視、バックアップ、スケーリング
- [デプロイ・運用ガイド](deployment-operations.md) - 本番環境でのデプロイと運用
- [Keycloakセットアップガイド](keycloak-setup.md) - Keycloak認証サーバーの設定と連携方法

### プロジェクト管理
- [要件管理ガイド](requirement-management-guide.md) - 要件定義の管理方法
- [要件セットアップ例](requirement-setup-examples.md) - 実践的な要件管理の例

## その他の関連ドキュメント

- [可観測性ガイド](../o11y.md) - ログ、メトリクス、トレーシングの実装と運用
- [Kubernetesデプロイガイド](../k8s/README.md) - Kubernetes環境へのデプロイ手順
- [統合テストガイド](../tests/README.md) - Testcontainersを使用した統合テスト
- [ディレクトリ構造](../.github/directorystructure.md) - プロジェクトのディレクトリ構造と責務

## ドキュメントの更新

ドキュメントを更新する際には、以下の点に注意してください：

1. 各ドキュメントには目次を含める
2. コードスニペットには適切な構文ハイライトを使用する
3. 実際のコードベースと一致していることを確認する
4. 画像やダイアグラムを含める場合は、`docs/images/` ディレクトリに保存する

---

ドキュメントに関する提案や改善点があれば、Issueの作成やPull Requestの送信をお願いします。