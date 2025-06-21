# OpenAPI Documentation

このディレクトリには、Rust WebAPIのOpenAPI 3.1仕様とSwagger UIが含まれています。

## ファイル構成

- `openapi.yaml` - OpenAPI 3.1仕様ファイル
- `swagger-ui.html` - Swagger UIを使用した対話的なAPIドキュメント
- `README.md` - このファイル

## 使用方法

### Swagger UIの表示

1. ローカルサーバーを起動:
   ```bash
   # Pythonを使用する場合
   cd docs/openapi
   python -m http.server 8000
   
   # または、Node.jsのhttp-serverを使用する場合
   npx http-server -p 8000
   ```

2. ブラウザで以下のURLにアクセス:
   ```
   http://localhost:8000/swagger-ui.html
   ```

### OpenAPI仕様の検証

```bash
# npxを使用してOpenAPI仕様を検証
npx @apidevtools/swagger-cli validate openapi.yaml

# または、Spectralを使用
npx @stoplight/spectral-cli lint openapi.yaml
```

### コード生成

OpenAPI仕様からクライアントコードやサーバースタブを生成できます：

```bash
# TypeScriptクライアントの生成
npx @openapitools/openapi-generator-cli generate \
  -i openapi.yaml \
  -g typescript-axios \
  -o ./generated/typescript-client

# Rustサーバースタブの生成
npx @openapitools/openapi-generator-cli generate \
  -i openapi.yaml \
  -g rust-server \
  -o ./generated/rust-server
```

## 仕様の特徴

### 認証

- Bearer Token (JWT)を使用
- すべてのAPIエンドポイント（ヘルスチェックとメトリクスを除く）で認証が必要

### エラーハンドリング

統一されたエラーレスポンス形式：
```json
{
  "type": "ErrorType",
  "message": "エラーメッセージ",
  "timestamp": "2024-01-15T09:30:00Z",
  "details": {}
}
```

### ページネーション

リスト系APIでは標準的なページネーションパラメータをサポート：
- `page` - ページ番号（1から開始）
- `per_page` - 1ページあたりの件数（最大100）
- `sort` - ソート項目
- `order` - ソート順（asc/desc）

### 削除操作

統一された削除インターフェース：
- `kind` - 削除種別（logical/physical/restore）
- `reason` - 削除理由（オプション）

## 更新方法

1. `openapi.yaml`を編集
2. 検証を実行して仕様の正しさを確認
3. 必要に応じてコード生成を実行
4. 変更をコミット

## 関連ドキュメント

- [API Documentation](../api-documentation.md) - API仕様の詳細説明
- [Architecture Guide](../architecture-guide.md) - システムアーキテクチャ
- [Development Guide](../development-guide.md) - 開発ガイド 