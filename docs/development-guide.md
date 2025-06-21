# 開発ガイド

このドキュメントでは、Rust WebAPI プロジェクトの開発に必要な情報を提供します。

## 目次

- [開発環境のセットアップ](#開発環境のセットアップ)
- [開発ワークフロー](#開発ワークフロー)
- [アーキテクチャと設計](#アーキテクチャと設計)
- [テスト](#テスト)
- [コーディング規約](#コーディング規約)
- [エラーハンドリング](#エラーハンドリング)
- [メトリクス記録](#メトリクス記録)
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

## アーキテクチャと設計

### 依存性注入（DI）

プロジェクトでは`AppContainer`を使用した依存性注入パターンを採用しています：

```rust
// src/infrastructure/di/container.rs
pub struct AppContainer {
    pub item_handler: web::Data<ItemHandler>,
    pub user_handler: web::Data<UserHandler>,
    pub category_handler: web::Data<CategoryHandler>,
    pub product_handler: web::Data<ProductHandler>,
    // ...
}

// main.rsでの使用例
let container = AppContainer::new(&app_config).await?;
let server = build_http_server(&container, &app_config.server.bind)?;
```

### 削除操作の統一

削除操作は戦略パターンを使用して統一されています：

```rust
// 削除ファサードの使用例
let deletion_facade = /* DIコンテナから取得 */;

// 論理削除
deletion_facade.delete_item(id, DeleteKind::Logical).await?;

// 物理削除  
deletion_facade.delete_item(id, DeleteKind::Physical).await?;

// 復元
deletion_facade.delete_item(id, DeleteKind::Restore).await?;
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

# 並列実行数を制限（CI環境用）
cargo test -- --test-threads=4
```

### MockBuilder の使用

テストではMockBuilderを使用してモックを簡潔に作成できます：

```rust
use crate::tests::helpers::mock_builder::ItemMockBuilder;

#[tokio::test]
async fn test_example() {
    let mock_repo = ItemMockBuilder::new()
        .with_find_by_id(1, Some(test_item()))
        .with_create_success()
        .build();
        
    // テストコード
}
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

- **エラーハンドリング**: `unwrap()` や `expect()` は使用禁止。すべて`AppError`で統一
- **非同期処理**: すべての I/O 処理は `async/await` で記述
- **フォーマット**: 4 スペースインデント、snake_case を使用
- **ドキュメント**: 公開APIには適切なドキュメントコメントを追加
- **テスト**: 各モジュールには対応するテストモジュールを作成

## エラーハンドリング

### 統一されたエラー型

プロジェクト全体で`AppError`を使用します：

```rust
use crate::infrastructure::error::{AppError, AppResult};

// エラーの作成
AppError::not_found("User", user_id)
AppError::validation_error("Invalid email format")
AppError::internal_error("Database connection failed")

// Result型の使用
pub async fn find_user(id: u64) -> AppResult<User> {
    repository.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::not_found("User", id))
}
```

### エラーレスポンス

エラーは自動的にJSON形式でクライアントに返されます：

```json
{
  "type": "NotFound",
  "message": "User with ID 123 not found",
  "timestamp": "2024-01-15T09:30:00Z"
}
```

## メトリクス記録

### 統一されたメトリクスAPI

メトリクス記録は統一されたAPIを使用します：

```rust
use crate::infrastructure::metrics::Metrics;

// 高レベルAPI（推奨）
Metrics::with_metrics("user", "create", async {
    // 処理内容
    Ok(user)
}).await

// タイマー付き実行
Metrics::with_timer("user", "find_all", async {
    // 処理内容
    users
}).await

// 個別記録
Metrics::record_success("user", "update");
Metrics::record_error("user", "delete");
Metrics::record_duration("user", "batch_process", 1.23);
```

### メトリクスの確認

Prometheusメトリクスは以下で確認できます：

```bash
curl http://localhost:8080/api/metrics
```

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

### テストの失敗

- テストデータベースが起動しているか確認
- `docker system prune`で古いコンテナを削除
- 環境変数`TEST_LOG=debug`でテストログを確認

### メトリクスが記録されない

- メトリクスエンドポイント`/api/metrics`にアクセスできるか確認
- `Metrics::init()`が初期化時に呼ばれているか確認
- ログレベルを`debug`にしてメトリクス記録ログを確認
