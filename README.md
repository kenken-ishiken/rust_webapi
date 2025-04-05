# Rust WebAPI サーバー

シンプルなREST APIを提供するRust製Webサーバーです。

## 機能

- RESTful API
- アイテムの作成・取得・一覧表示・削除
- JSON形式でのデータやり取り

## 依存関係

- Rust (最新安定版推奨)
- Cargo (Rustのパッケージマネージャー)

## 使い方

### サーバーの起動

```bash
# プロジェクトディレクトリで
cargo run
```

サーバーは http://127.0.0.1:8080 で起動します。

### APIエンドポイント

#### ルート

- `GET /` - サーバーの状態確認

#### アイテム管理

- `GET /api/items` - アイテム一覧を取得
- `GET /api/items/{id}` - 特定のアイテムを取得
- `POST /api/items` - 新しいアイテムを作成
- `DELETE /api/items/{id}` - アイテムを削除

### リクエスト例

#### アイテム作成

```bash
curl -X POST http://localhost:8080/api/items \
  -H "Content-Type: application/json" \
  -d '{"name": "テストアイテム", "description": "これはテスト用のアイテムです"}'
```

#### アイテム一覧取得

```bash
curl http://localhost:8080/api/items
```

#### 特定アイテムの取得

```bash
curl http://localhost:8080/api/items/0
```

#### アイテム削除

```bash
curl -X DELETE http://localhost:8080/api/items/0
```

## 開発

### 依存関係の追加

プロジェクトに新しい依存関係を追加する場合は、`Cargo.toml`ファイルを編集し、`cargo build`を実行してください。

### サーバーをデバッグモードで実行

```bash
RUST_LOG=debug cargo run
```

## ライセンス

MITライセンス