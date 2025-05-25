# Contributing Guide

このドキュメントでは、新規参加者が知っておくべきプロジェクトの概要と基本的な作業手順をまとめています。

## リポジトリ構造

主要なディレクトリと役割は以下のとおりです。

```
├── src/                # メインアプリケーション
│   ├── main.rs         # エントリポイント
│   ├── application/    # アプリケーション層（DTO、サービス）
│   ├── app_domain/     # ドメイン層（モデル、リポジトリインターフェース）
│   ├── infrastructure/ # インフラ層（DB、認証、ロギング）
│   └── presentation/   # プレゼンテーション層（API ハンドラ）
├── crates/domain/      # ドメイン層サブクレート
├── docs/               # 詳細ドキュメント
├── k8s/                # Kubernetes マニフェスト
├── initdb/             # DB 初期化 SQL
├── scripts/            # 補助スクリプト
├── tests/              # 統合テスト
```

詳細な責務は `.github/directorystructure.md` を参照してください。

## 開発環境のセットアップ

1. Rust 1.77 以上、Docker と docker-compose をインストールしてください。
2. `.env` ファイルを作成し、`DATABASE_URL` などの環境変数を設定します。
3. 以下のコマンドでコンテナを起動します。

```bash
docker-compose up -d
```

ローカル実行だけなら `cargo run` でも起動できます。

## 主要な開発コマンド

```bash
# テスト実行
cargo test

# フォーマット & Lint
cargo fmt
cargo clippy --all-targets -- -D warnings
```

可観測性の仕組みやトレース設定の詳細は `docs/operations-guide.md` および `o11y.md` を参照してください。

## コーディング規約

- `unwrap()` や `expect()` は避け、適切に `Result` / `Option` を扱います。
- すべての I/O 処理は `async/await` で記述します。
- 4 スペースインデント、snake_case を使用します。

## さらに学ぶべきこと

- 詳細なドキュメントは `docs/` ディレクトリを参照してください。
- 認証機構：`src/infrastructure/auth/` の実装を確認してください。
- テスト：`tests/` ディレクトリには統合テストの例があります。
- Kubernetes デプロイ：`k8s/README.md` に本番環境向けの手順があります。

以上を一通り把握すると、プロジェクトの全体像が掴みやすくなります。Issue や Pull Request は歓迎です。
