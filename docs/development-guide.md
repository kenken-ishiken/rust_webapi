# 開発ガイド

このドキュメントでは、Rust WebAPI プロジェクトの開発に必要な情報を提供します。

## 目次

- [開発環境のセットアップ](#開発環境のセットアップ)
- [開発ワークフロー](#開発ワークフロー)
- [テスト](#テスト)
- [コーディング規約](#コーディング規約)
- [デバッグ](#デバッグ)
- [よくある問題と解決策](#よくある問題と解決策)

## 開発環境のセットアップ

### 前提条件

- **Rust** 1.77 以上（`rustup` 推奨）
- `cargo`（Rust 同梱）
- `docker` と `docker-compose`（コンテナ実行時）

### ローカル開発環境の構築

1. リポジトリをクローンします：

```bash
git clone https://github.com/kenken-ishiken/rust_webapi.git
cd rust_webapi
```

2. 環境変数ファイルを作成します：

```bash
cat > .env << EOF
DATABASE_URL=******postgres:5432/rustwebapi
KEYCLOAK_AUTH_SERVER_URL=http://localhost:8081
KEYCLOAK_REALM=rust-webapi
KEYCLOAK_CLIENT_ID=api-client
EOF
```

3. 依存関係をインストールします：

```bash
cargo build
```

### Docker Compose による開発

PostgreSQL やその他の依存サービスを含む完全な開発環境をDocker Composeで起動できます：

```bash
docker-compose up -d
```

これにより、以下のサービスが起動します：

- **API**: http://localhost:8080
- **PostgreSQL**: localhost:5432
- **Keycloak**: http://localhost:8081

## 開発ワークフロー

### サーバー起動

ローカルでRustプロセスとして実行：

```bash
cargo run
```

または、デバッグビルドで実行：

```bash
cargo run --bin rust_webapi
```

### ホットリロード開発

開発中のコード変更を自動検出して再コンパイルするには：

```bash
cargo install cargo-watch
cargo watch -x run
```

### ビルド

```bash
# 開発ビルド
cargo build

# リリースビルド（最適化あり）
cargo build --release
```

## テスト

### 単体テスト・統合テスト実行

```bash
# すべてのテストを実行
cargo test

# 特定のテストを実行
cargo test test_name

# 特定のパッケージのテストを実行
cargo test -p domain
```

### テストコンテナ

統合テストは`testcontainers-rs`を使用して分離されたPostgreSQLインスタンスで実行されます。詳細は[tests/README.md](../tests/README.md)を参照してください。

### カバレッジ計測

```bash
# rustup でツールをインストール
rustup component add llvm-tools-preview

# カバレッジ計測スクリプトを実行
./scripts/coverage.sh
```

カバレッジレポートは`target/coverage/`ディレクトリに生成されます。

## コーディング規約

### Rustフォーマット

```bash
# コードフォーマット
cargo fmt

# フォーマットチェック（CI用）
cargo fmt -- --check
```

### Lint

```bash
# clippy でコード品質チェック
cargo clippy --all-targets -- -D warnings
```

### 主要な規約

- `unwrap()` や `expect()` は避け、適切に `Result` / `Option` を扱います。
- すべての I/O 処理は `async/await` で記述します。
- 4 スペースインデント、snake_case を使用します。
- 公開APIには適切なドキュメントコメントを追加します。

## デバッグ

### ログレベル設定

ログレベルは環境変数で制御できます：

```bash
RUST_LOG=debug cargo run
```

または、細かく設定：

```bash
RUST_LOG=rust_webapi=debug,actix_web=info cargo run
```

### トレース表示

トレースを有効にしてデバッグするには：

```bash
RUST_LOG=trace cargo run
```

### デバッガー（VSCode）

`.vscode/launch.json`の設定例：

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug rust_webapi",
            "cargo": {
                "args": ["build", "--bin=rust_webapi", "--package=rust_webapi"],
                "filter": {
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}",
            "env": {
                "RUST_LOG": "debug"
            }
        }
    ]
}
```

## よくある問題と解決策

### データベース接続エラー

- `.env`ファイルの`DATABASE_URL`が正しいことを確認
- PostgreSQLが起動していることを確認
- Docker Compose環境では`docker-compose logs postgres`でログを確認

### 認証関連の問題

- Keycloakが起動しているか確認
- トークンの有効期限が切れていないか確認
- `.env`ファイルの認証設定が正しいか確認

### ビルドエラー

- `cargo clean`を実行して再ビルド
- Rustのバージョンが1.77以上か確認
- 依存クレートが競合していないか確認
