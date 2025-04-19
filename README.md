# Rust WebAPI サーバー

シンプルかつ拡張しやすい **Rust 製 REST API サーバー** です。  
学習用途から小規模サービスまで、Rust の高速な非同期処理を体験できます。

## 目次
- [特徴](#特徴)
- [デモ](#デモ)
- [クイックスタート](#クイックスタート)
- [API リファレンス](#api-リファレンス)
- [開発](#開発)
- [コントリビューション](#コントリビューション)
- [ライセンス](#ライセンス)

## 特徴
- **RESTful API**：CRUD 操作を HTTP/JSON で提供  
- **スキーマレス**：内部ストレージはシンプルなインメモリ DB（後から RDB 等へ差し替えも容易）  
- **高速**：`tokio` と `actix-web` による非同期 I/O  
- **型安全**：Rust の型システムでリクエスト／レスポンスを保証  
- **テスト容易**：統合テスト用ヘルパを同梱  

## デモ
```bash
# 依存関係を取得してサーバーを起動
cargo run
# 別ターミナルで
curl http://127.0.0.1:8080/
```

## クイックスタート

### 前提
- **Rust** 1.77 以上（`rustup` 推奨）
- `cargo`（Rust 同梱）
- `make`（任意：タスクランナーとして利用）

### 起動
```bash
git clone https://github.com/your-name/rust_webapi.git
cd rust_webapi
cargo run
# または
make run   # make が入っている場合
```
デフォルトで **http://127.0.0.1:8080** で待ち受けます。  
ポートを変更したい場合は環境変数 `PORT` を設定してください。

### ビルド
```bash
cargo build --release
```

## API リファレンス

| メソッド | パス | 説明 |
|----------|------|------|
| GET | `/` | ヘルスチェック |
| GET | `/api/items` | アイテム一覧取得 |
| GET | `/api/items/{id}` | 特定アイテム取得 |
| POST | `/api/items` | アイテム作成 |
| DELETE | `/api/items/{id}` | アイテム削除 |

### 例：アイテム作成
```bash
curl -X POST http://127.0.0.1:8080/api/items \
  -H "Content-Type: application/json" \
  -d '{"name":"テスト","description":"説明"}'
```

### 例：レスポンス
```json
{
  "id": 0,
  "name": "テスト",
  "description": "説明"
}
```

## 開発

### プロジェクト構成
```
src/
├── main.rs        # エントリポイント
├── api.rs         # ルーティング
├── models.rs      # データ構造
└── tests/         # 結合テスト
```

### テスト
```bash
cargo test
```

### フォーマット & Lint
```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
```

### 依存追加
`Cargo.toml` を編集後、`cargo build` を実行してください。

## コントリビューション
Issue や Pull Request は歓迎です。詳細は `CONTRIBUTING.md` を参照してください（未作成の場合は提案してください）。

## ライセンス
MIT License © 2025 Your Name