# エンティティ関係図（ER図）

## データベーススキーマ

```mermaid
erDiagram
    products {
        uuid id PK "商品ID"
        string sku UK "商品SKU"
        string name "商品名"
        text description "商品説明"
        string status "ステータス(active/inactive/deleted)"
        decimal price_amount "価格"
        string price_currency "通貨コード"
        integer available_stock "利用可能在庫"
        integer reserved_stock "予約済み在庫"
        jsonb metadata "メタデータ"
        timestamp created_at "作成日時"
        timestamp updated_at "更新日時"
        timestamp deleted_at "削除日時"
        string deleted_by "削除者"
        string deletion_reason "削除理由"
    }

    categories {
        string id PK "カテゴリID"
        string name "カテゴリ名"
        text description "説明"
        string parent_id FK "親カテゴリID"
        integer sort_order "表示順"
        boolean is_active "有効フラグ"
        timestamp created_at "作成日時"
        timestamp updated_at "更新日時"
    }

    product_categories {
        uuid product_id PK,FK "商品ID"
        string category_id PK,FK "カテゴリID"
        timestamp created_at "作成日時"
    }

    product_tags {
        uuid product_id PK,FK "商品ID"
        string tag_name PK "タグ名"
        timestamp created_at "作成日時"
    }

    users {
        bigint id PK "ユーザーID"
        string username UK "ユーザー名"
        string email UK "メールアドレス"
        string keycloak_id UK "KeycloakユーザーID"
        jsonb profile "プロファイル情報"
        timestamp created_at "作成日時"
        timestamp updated_at "更新日時"
    }

    items {
        bigint id PK "アイテムID"
        string name "アイテム名"
        text description "説明"
        string category "カテゴリ"
        decimal price "価格"
        integer quantity "数量"
        boolean is_active "有効フラグ"
        timestamp created_at "作成日時"
        timestamp updated_at "更新日時"
    }

    deletion_logs {
        uuid id PK "ログID"
        string entity_type "エンティティタイプ"
        string entity_id "エンティティID"
        string deletion_type "削除タイプ(logical/physical/restore)"
        string deleted_by "削除実行者"
        string reason "削除理由"
        jsonb metadata "追加情報"
        timestamp deleted_at "削除日時"
    }

    products ||--o{ product_categories : "has"
    categories ||--o{ product_categories : "belongs to"
    categories ||--o{ categories : "has parent"
    products ||--o{ product_tags : "has"
    products ||--o{ deletion_logs : "logged in"
    categories ||--o{ deletion_logs : "logged in"
    users ||--o{ deletion_logs : "performed by"
```

## リレーションシップの説明

### 商品関連
- **products ↔ categories**: 多対多の関係（product_categories経由）
- **products ↔ tags**: 多対多の関係（product_tags経由）
- **products → deletion_logs**: 削除操作のログ

### カテゴリ関連
- **categories → categories**: 自己参照（階層構造）
- **categories → deletion_logs**: 削除操作のログ

### ユーザー関連
- **users → deletion_logs**: 削除操作の実行者

### 削除ログ
- **deletion_logs**: 全エンティティの削除操作を記録
  - entity_typeとentity_idで対象を特定
  - 論理削除、物理削除、復元の履歴を保持

## インデックス

```sql
-- 商品検索用
CREATE INDEX idx_products_sku ON products(sku);
CREATE INDEX idx_products_status ON products(status);
CREATE INDEX idx_products_created_at ON products(created_at DESC);

-- カテゴリ検索用
CREATE INDEX idx_categories_parent_id ON categories(parent_id);
CREATE INDEX idx_categories_is_active ON categories(is_active);

-- 関連テーブル
CREATE INDEX idx_product_categories_product_id ON product_categories(product_id);
CREATE INDEX idx_product_categories_category_id ON product_categories(category_id);
CREATE INDEX idx_product_tags_product_id ON product_tags(product_id);

-- 削除ログ検索用
CREATE INDEX idx_deletion_logs_entity ON deletion_logs(entity_type, entity_id);
CREATE INDEX idx_deletion_logs_deleted_at ON deletion_logs(deleted_at DESC);
```

## 制約

```sql
-- 外部キー制約
ALTER TABLE product_categories 
  ADD CONSTRAINT fk_product_categories_product 
  FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE;

ALTER TABLE product_categories 
  ADD CONSTRAINT fk_product_categories_category 
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE;

ALTER TABLE categories 
  ADD CONSTRAINT fk_categories_parent 
  FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE SET NULL;

-- チェック制約
ALTER TABLE products 
  ADD CONSTRAINT chk_products_status 
  CHECK (status IN ('active', 'inactive', 'deleted'));

ALTER TABLE products 
  ADD CONSTRAINT chk_products_price 
  CHECK (price_amount >= 0);

ALTER TABLE products 
  ADD CONSTRAINT chk_products_stock 
  CHECK (available_stock >= 0 AND reserved_stock >= 0);
``` 