# 要件定義管理システム - セットアップと使用例

## 📁 作成されたファイル一覧

```
rust_webapi/
├── .github/
│   └── ISSUE_TEMPLATE/
│       └── requirement-definition.md     # GitHub Issue テンプレート
├── docs/
│   ├── requirement-workflow.yml          # 作業フローの詳細設定
│   └── requirement-management-guide.md   # 使用ガイド
└── scripts/
    ├── create-requirement-issue.sh       # Issue作成スクリプト
    └── manage-requirements.sh           # Issue管理スクリプト
```

## 🚀 セットアップ手順

### 1. 前提条件の確認
```bash
# GitHub CLI のインストールと認証確認
gh --version
gh auth status
```

### 2. スクリプトの実行権限設定
```bash
chmod +x scripts/create-requirement-issue.sh
chmod +x scripts/manage-requirements.sh
```

### 3. 必要な依存関係のインストール
```bash
# macOS の場合
brew install jq

# Ubuntu/Debian の場合
sudo apt-get install jq
```

## 💡 使用例

### 例1: 新しいAPI機能の要件定義

#### ステップ1: Issue作成
```bash
./scripts/create-requirement-issue.sh
```

対話式で以下を入力：
- タイトル: `ユーザープロファイル取得API`
- コンポーネント: `1) API`
- 優先度: `2) High`
- 機能タイプ: `1) Feature`
- 担当者: `@me`

作成されるIssue:
- タイトル: `[REQ] ユーザープロファイル取得API`
- ラベル: `requirement,status/draft,component/api,priority/high,type/feature`

#### ステップ2: 要件の詳細記入
作成されたIssueに以下の内容を記入：

```markdown
## 概要
ユーザーが自分のプロファイル情報を取得できるAPIエンドポイントを作成する。

## 機能仕様
### 入力
- Authorization ヘッダー（JWT トークン）
- ユーザーID（パスパラメータ）

### 処理
- JWT トークンの検証
- ユーザーIDの認可チェック
- データベースからユーザー情報を取得

### 出力
- ユーザープロファイル情報（JSON）

## API仕様
### エンドポイント
```http
GET /api/v1/users/{user_id}/profile
```

### レスポンス例
```json
{
  "id": 123,
  "username": "john_doe",
  "email": "john@example.com",
  "display_name": "John Doe",
  "created_at": "2025-01-01T00:00:00Z"
}
```

## 受け入れ条件
- [ ] 認証されたユーザーのみアクセス可能
- [ ] 自分のプロファイルのみ取得可能
- [ ] レスポンス時間 < 100ms
- [ ] テストカバレッジ >= 90%
```

#### ステップ3: レビュー準備
```bash
./scripts/manage-requirements.sh review 123
```

#### ステップ4: 承認後の実装開始
```bash
./scripts/manage-requirements.sh approve 123
./scripts/manage-requirements.sh start 123

# 実装ブランチ作成
git checkout -b feature/req-123-user-profile-api
```

#### ステップ5: TDD実装
```rust
// tests/user_profile_tests.rs
#[tokio::test]
async fn test_get_user_profile_success() {
    // Given: 認証されたユーザー
    let app = create_test_app().await;
    let token = create_test_jwt_token(123).await;
    
    // When: プロファイル取得APIを呼び出し
    let response = app
        .get("/api/v1/users/123/profile")
        .header("Authorization", format!("Bearer {}", token))
        .await;
    
    // Then: ユーザー情報が返される
    assert_eq!(response.status(), 200);
    let profile: UserProfile = response.json().await;
    assert_eq!(profile.id, 123);
    assert_eq!(profile.username, "test_user");
}
```

#### ステップ6: テスト開始
```bash
./scripts/manage-requirements.sh testing 123
```

#### ステップ7: 完了
```bash
./scripts/manage-requirements.sh done 123
```

### 例2: データベース設計変更の要件定義

```bash
# Issue作成
gh issue create \
  --title "[REQ] ユーザーテーブルにロール機能追加" \
  --template requirement-definition.md \
  --label requirement,status/draft,component/database,priority/medium,type/enhancement

# Issue番号を確認（例: #124）
gh issue list --label requirement --limit 1
```

要件内容例：
```markdown
## データベース設計

### 既存テーブル変更
#### users テーブル
```sql
-- 新規カラム追加
ALTER TABLE users ADD COLUMN role VARCHAR(50) DEFAULT 'user' NOT NULL;
ALTER TABLE users ADD COLUMN role_assigned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;

-- インデックス追加
CREATE INDEX idx_users_role ON users(role);
```

### マイグレーション
```sql
-- 003_add_user_roles.sql
BEGIN;

-- カラム追加
ALTER TABLE users ADD COLUMN role VARCHAR(50) DEFAULT 'user' NOT NULL;
ALTER TABLE users ADD COLUMN role_assigned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;

-- インデックス追加
CREATE INDEX idx_users_role ON users(role);

-- 既存ユーザーのロール設定
UPDATE users SET role = 'admin' WHERE email = 'admin@example.com';

COMMIT;
```

## 受け入れ条件
- [ ] 既存データの整合性が保たれる
- [ ] マイグレーションが正常に実行される
- [ ] ロールベースの認可が機能する
- [ ] パフォーマンスが劣化しない
```

### 例3: 日々の管理タスク

#### 要件定義の状況確認
```bash
# ダッシュボード表示
./scripts/manage-requirements.sh dashboard

# 出力例:
# 🎯 要件定義ダッシュボード
# ==================================
# 📊 状態別統計
# ====================
# 🗂️  ドラフト     : 3 件
# 🔍 レビュー中   : 2 件
# ✅ 承認済み     : 1 件
# ⚙️  実装中       : 4 件
# 🧪 テスト中     : 1 件
# 🎉 完了         : 12 件
```

#### 特定状態のIssue一覧
```bash
# レビュー待ちのIssue一覧
./scripts/manage-requirements.sh list review

# 実装中のIssue一覧
./scripts/manage-requirements.sh list in-progress
```

#### メトリクス確認
```bash
./scripts/manage-requirements.sh metrics

# 出力例:
# 🎯 要件定義メトリクス
# ==================================
# 📊 完了率
# ==============
# 総要件数: 23
# 完了数: 12
# 完了率: 52%
```

## 🔄 TDDとの連携例

### Red-Green-Refactorサイクルでの実装

#### 1. Red Phase（失敗テスト作成）
```rust
#[tokio::test]
async fn test_create_product_with_valid_data_should_succeed() {
    // Given
    let app = create_test_app().await;
    let product_data = CreateProductRequest {
        name: "Test Product".to_string(),
        price: 1000,
        category_id: 1,
    };
    
    // When
    let response = app.post("/api/v1/products")
        .json(&product_data)
        .await;
    
    // Then
    assert_eq!(response.status(), 201);
    let created_product: Product = response.json().await;
    assert_eq!(created_product.name, "Test Product");
    assert_eq!(created_product.price, 1000);
}
```

#### 2. Green Phase（最小実装）
```rust
pub async fn create_product(
    State(service): State<Arc<ProductService>>,
    Json(request): Json<CreateProductRequest>,
) -> Result<Json<Product>, AppError> {
    let product = service.create(request).await?;
    Ok(Json(product))
}
```

#### 3. Refactor Phase（改善）
```rust
pub async fn create_product(
    State(service): State<Arc<ProductService>>,
    Json(request): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<Product>), AppError> {
    // バリデーション追加
    request.validate()?;
    
    // 商品作成
    let product = service.create(request).await?;
    
    // 作成ログ出力
    tracing::info!("Product created: id={}, name={}", product.id, product.name);
    
    Ok((StatusCode::CREATED, Json(product)))
}
```

## 📋 チェックリスト

### Issue作成時
- [ ] 適切なテンプレートを使用している
- [ ] タイトルが明確で検索しやすい
- [ ] 必要なラベルが設定されている
- [ ] 担当者がアサインされている
- [ ] 優先度が適切に設定されている

### 要件記入時
- [ ] 概要が明確に記載されている
- [ ] 機能要件が具体的に定義されている
- [ ] 受け入れ条件が検証可能である
- [ ] 非機能要件が適切に設定されている
- [ ] テスト計画が具体的である

### 実装時
- [ ] TDDサイクルに従って開発している
- [ ] テストカバレッジが90%以上
- [ ] Clippyの警告が0個
- [ ] APIドキュメントが更新されている
- [ ] 実装ブランチが適切に命名されている

### 完了時
- [ ] すべての受け入れ条件を満たしている
- [ ] 関連ドキュメントが更新されている
- [ ] 本番環境で正常に動作している
- [ ] 監視・アラートが設定されている

## 🔗 関連コマンド集

```bash
# GitHub CLI 基本コマンド
gh issue list                           # Issue一覧
gh issue view 123                       # Issue詳細表示
gh issue create                         # Issue作成
gh issue edit 123                       # Issue編集
gh issue close 123                      # Issue クローズ

# プロジェクト固有コマンド
./scripts/create-requirement-issue.sh   # 要件定義Issue作成
./scripts/manage-requirements.sh list   # 要件一覧
./scripts/manage-requirements.sh dashboard  # ダッシュボード

# 開発関連コマンド
cargo test                              # テスト実行
cargo clippy                            # コード品質チェック
cargo fmt                               # コードフォーマット
```

このシステムを使用することで、要件定義からリリースまでの一連のプロセスを効率的かつ透明性を保って管理できます。
