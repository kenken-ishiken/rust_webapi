# データベーススキーマ設計書

## 目次

1. [概要](#概要)
2. [ER図](#er図)
3. [テーブル定義](#テーブル定義)
4. [インデックス設計](#インデックス設計)
5. [制約とビジネスルール](#制約とビジネスルール)
6. [トリガーと自動処理](#トリガーと自動処理)
7. [パフォーマンス考慮事項](#パフォーマンス考慮事項)

## 概要

本データベースは、PostgreSQL 15以上を使用し、ECコマースシステムの商品管理、カテゴリ管理、ユーザー管理を中心とした設計となっています。

### 設計原則

- **正規化**: 第3正規形を基本とし、パフォーマンス要件に応じて適切に非正規化
- **拡張性**: JSON型やカスタム属性テーブルによる柔軟な拡張
- **データ整合性**: 外部キー制約とチェック制約による厳密な整合性管理
- **パフォーマンス**: 適切なインデックスとパーティショニング戦略
- **監査性**: 履歴テーブルによる変更追跡

## ER図

```mermaid
erDiagram
    CATEGORIES ||--o{ CATEGORIES : "has_parent"
    CATEGORIES ||--o{ PRODUCTS : "contains"
    PRODUCTS ||--|| PRODUCT_INVENTORY : "has"
    PRODUCTS ||--o{ PRODUCT_PRICES : "has"
    PRODUCTS ||--o{ PRODUCT_IMAGES : "has"
    PRODUCTS ||--o{ PRODUCT_TAGS : "has"
    PRODUCTS ||--o{ PRODUCT_ATTRIBUTES : "has"
    PRODUCTS ||--o{ PRODUCT_HISTORY : "tracks"
    
    CATEGORIES {
        varchar id PK
        varchar name
        text description
        varchar parent_id FK
        integer sort_order
        boolean is_active
        timestamp created_at
        timestamp updated_at
    }
    
    PRODUCTS {
        varchar id PK
        varchar name
        text description
        varchar sku UK
        varchar brand
        varchar status
        varchar category_id FK
        decimal width
        decimal height
        decimal depth
        decimal weight
        varchar shipping_class
        boolean free_shipping
        decimal shipping_fee
        timestamp created_at
        timestamp updated_at
    }
    
    PRODUCT_INVENTORY {
        bigserial id PK
        varchar product_id FK UK
        integer quantity
        integer reserved_quantity
        integer alert_threshold
        boolean track_inventory
        boolean allow_backorder
        timestamp created_at
        timestamp updated_at
    }
    
    PRODUCT_PRICES {
        bigserial id PK
        varchar product_id FK
        decimal selling_price
        decimal list_price
        decimal discount_price
        varchar currency
        boolean tax_included
        timestamp effective_from
        timestamp effective_until
        timestamp created_at
        timestamp updated_at
    }
    
    PRODUCT_IMAGES {
        varchar id PK
        varchar product_id FK
        text url
        text alt_text
        integer sort_order
        boolean is_main
        timestamp created_at
        timestamp updated_at
    }
    
    USERS {
        bigint id PK
        varchar username
        varchar email
    }
    
    ITEMS {
        bigint id PK
        varchar name
        text description
    }
```

## テーブル定義

### 1. categories - カテゴリマスタ

商品の分類を階層構造で管理するテーブル。

| カラム名 | データ型 | NULL | デフォルト | 説明 |
|---------|----------|------|-----------|------|
| id | VARCHAR(255) | NO | - | カテゴリID (PK) |
| name | VARCHAR(100) | NO | - | カテゴリ名 |
| description | TEXT | YES | NULL | カテゴリ説明 |
| parent_id | VARCHAR(255) | YES | NULL | 親カテゴリID (FK) |
| sort_order | INTEGER | NO | 0 | 表示順序 |
| is_active | BOOLEAN | NO | true | 有効フラグ |
| created_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 作成日時 |
| updated_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 更新日時 |

**制約:**
- `UNIQUE(name, parent_id)`: 同一親カテゴリ内での名前の重複を防止
- `CHECK(sort_order >= 0)`: 表示順序は非負
- `CHECK(LENGTH(TRIM(name)) > 0)`: 名前は空文字不可

### 2. products - 商品マスタ

商品の基本情報を管理するテーブル。

| カラム名 | データ型 | NULL | デフォルト | 説明 |
|---------|----------|------|-----------|------|
| id | VARCHAR(255) | NO | - | 商品ID (PK) |
| name | VARCHAR(200) | NO | - | 商品名 |
| description | TEXT | YES | NULL | 商品説明 |
| sku | VARCHAR(50) | NO | - | 在庫管理単位 (UK) |
| brand | VARCHAR(100) | YES | NULL | ブランド名 |
| status | VARCHAR(20) | NO | 'Draft' | ステータス |
| category_id | VARCHAR(255) | YES | NULL | カテゴリID (FK) |
| width | DECIMAL(10,1) | YES | NULL | 幅 (cm) |
| height | DECIMAL(10,1) | YES | NULL | 高さ (cm) |
| depth | DECIMAL(10,1) | YES | NULL | 奥行き (cm) |
| weight | DECIMAL(10,1) | YES | NULL | 重量 (kg) |
| shipping_class | VARCHAR(50) | YES | 'standard' | 配送クラス |
| free_shipping | BOOLEAN | YES | false | 送料無料フラグ |
| shipping_fee | DECIMAL(10,2) | YES | 0 | 送料 |
| created_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 作成日時 |
| updated_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 更新日時 |

**ステータス値:**
- `Active`: 販売中
- `Inactive`: 販売停止
- `Draft`: 下書き
- `Discontinued`: 廃番

### 3. product_prices - 商品価格

時間軸での価格管理と複数通貨対応。

| カラム名 | データ型 | NULL | デフォルト | 説明 |
|---------|----------|------|-----------|------|
| id | BIGSERIAL | NO | - | 価格ID (PK) |
| product_id | VARCHAR(255) | NO | - | 商品ID (FK) |
| selling_price | DECIMAL(12,2) | NO | - | 販売価格 |
| list_price | DECIMAL(12,2) | YES | NULL | 定価 |
| discount_price | DECIMAL(12,2) | YES | NULL | 割引価格 |
| currency | VARCHAR(3) | NO | 'JPY' | 通貨コード |
| tax_included | BOOLEAN | NO | true | 税込みフラグ |
| effective_from | TIMESTAMP WITH TIME ZONE | YES | NULL | 有効開始日時 |
| effective_until | TIMESTAMP WITH TIME ZONE | YES | NULL | 有効終了日時 |
| created_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 作成日時 |
| updated_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 更新日時 |

**価格ルール:**
- 販売価格 ≤ 定価
- 割引価格 ≤ 販売価格

### 4. product_inventory - 在庫情報

在庫数と予約管理。

| カラム名 | データ型 | NULL | デフォルト | 説明 |
|---------|----------|------|-----------|------|
| id | BIGSERIAL | NO | - | 在庫ID (PK) |
| product_id | VARCHAR(255) | NO | - | 商品ID (FK, UK) |
| quantity | INTEGER | NO | 0 | 在庫数 |
| reserved_quantity | INTEGER | NO | 0 | 予約数 |
| alert_threshold | INTEGER | YES | NULL | 在庫アラート閾値 |
| track_inventory | BOOLEAN | NO | true | 在庫追跡フラグ |
| allow_backorder | BOOLEAN | NO | false | 取り寄せ許可フラグ |
| created_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 作成日時 |
| updated_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 更新日時 |

**在庫ルール:**
- 在庫数 ≥ 0
- 予約数 ≤ 在庫数
- 利用可能在庫 = 在庫数 - 予約数

### 5. product_images - 商品画像

商品画像の管理。

| カラム名 | データ型 | NULL | デフォルト | 説明 |
|---------|----------|------|-----------|------|
| id | VARCHAR(255) | NO | - | 画像ID (PK) |
| product_id | VARCHAR(255) | NO | - | 商品ID (FK) |
| url | TEXT | NO | - | 画像URL |
| alt_text | TEXT | YES | NULL | 代替テキスト |
| sort_order | INTEGER | NO | 0 | 表示順序 |
| is_main | BOOLEAN | NO | false | メイン画像フラグ |
| created_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 作成日時 |
| updated_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 更新日時 |

**画像ルール:**
- 1商品につき1つのメイン画像のみ（ユニークインデックスで制約）

### 6. product_tags - 商品タグ

商品の検索性向上のためのタグ管理。

| カラム名 | データ型 | NULL | デフォルト | 説明 |
|---------|----------|------|-----------|------|
| id | BIGSERIAL | NO | - | タグID (PK) |
| product_id | VARCHAR(255) | NO | - | 商品ID (FK) |
| tag | VARCHAR(100) | NO | - | タグ名 |
| created_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 作成日時 |

**制約:**
- `UNIQUE(product_id, tag)`: 同一商品への同じタグの重複防止

### 7. product_attributes - 商品属性

商品の柔軟な属性管理（色、サイズなど）。

| カラム名 | データ型 | NULL | デフォルト | 説明 |
|---------|----------|------|-----------|------|
| id | BIGSERIAL | NO | - | 属性ID (PK) |
| product_id | VARCHAR(255) | NO | - | 商品ID (FK) |
| attribute_name | VARCHAR(100) | NO | - | 属性名 |
| attribute_value | TEXT | NO | - | 属性値 |
| created_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 作成日時 |
| updated_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 更新日時 |

**制約:**
- `UNIQUE(product_id, attribute_name)`: 同一商品での属性名の重複防止

### 8. product_history - 商品変更履歴

商品情報の変更を追跡する監査ログ。

| カラム名 | データ型 | NULL | デフォルト | 説明 |
|---------|----------|------|-----------|------|
| id | BIGSERIAL | NO | - | 履歴ID (PK) |
| product_id | VARCHAR(255) | NO | - | 商品ID (FK) |
| field_name | VARCHAR(100) | NO | - | 変更フィールド名 |
| old_value | TEXT | YES | NULL | 変更前の値 |
| new_value | TEXT | YES | NULL | 変更後の値 |
| changed_by | VARCHAR(255) | YES | NULL | 変更者ID |
| reason | TEXT | YES | NULL | 変更理由 |
| changed_at | TIMESTAMP WITH TIME ZONE | NO | CURRENT_TIMESTAMP | 変更日時 |

### 9. users - ユーザーマスタ

システムユーザーの基本情報。

| カラム名 | データ型 | NULL | デフォルト | 説明 |
|---------|----------|------|-----------|------|
| id | BIGINT | NO | - | ユーザーID (PK) |
| username | VARCHAR(255) | NO | - | ユーザー名 |
| email | VARCHAR(255) | NO | - | メールアドレス |

### 10. items - アイテムマスタ

汎用的なアイテム管理（レガシー互換用）。

| カラム名 | データ型 | NULL | デフォルト | 説明 |
|---------|----------|------|-----------|------|
| id | BIGINT | NO | - | アイテムID (PK) |
| name | VARCHAR(255) | NO | - | アイテム名 |
| description | TEXT | YES | NULL | 説明 |

## インデックス設計

### パフォーマンス最適化のためのインデックス

#### 1. 主要検索パターン用インデックス

```sql
-- 商品検索の高速化
CREATE INDEX idx_products_sku ON products(sku);
CREATE INDEX idx_products_status ON products(status);
CREATE INDEX idx_products_brand ON products(brand);
CREATE INDEX idx_products_category_id ON products(category_id);

-- カテゴリ階層の高速化
CREATE INDEX idx_categories_parent_id ON categories(parent_id);
CREATE INDEX idx_categories_parent_sort ON categories(parent_id, sort_order);

-- 価格検索の高速化
CREATE INDEX idx_product_prices_product_id ON product_prices(product_id);
CREATE INDEX idx_product_prices_effective_dates ON product_prices(effective_from, effective_until);

-- タグ検索の高速化
CREATE INDEX idx_product_tags_tag ON product_tags(tag);
```

#### 2. ソート・フィルタ用インデックス

```sql
-- 新着商品表示
CREATE INDEX idx_products_created_at ON products(created_at);

-- カテゴリ内商品の表示順
CREATE INDEX idx_categories_sort_order ON categories(sort_order);
CREATE INDEX idx_product_images_sort_order ON product_images(sort_order);

-- アクティブな商品のみ表示
CREATE INDEX idx_categories_is_active ON categories(is_active);
```

#### 3. 特殊制約用インデックス

```sql
-- メイン画像の一意性保証
CREATE UNIQUE INDEX idx_product_images_main_unique 
    ON product_images(product_id) 
    WHERE is_main = true;
```

## 制約とビジネスルール

### 1. データ整合性制約

#### 外部キー制約
- `categories.parent_id → categories.id`: 親カテゴリの存在保証
- `products.category_id → categories.id`: カテゴリの存在保証
- `product_*.product_id → products.id`: 商品の存在保証

#### チェック制約
- 数値の非負制約（在庫数、価格、寸法など）
- 文字列の空文字制約（名前、SKUなど）
- ステータス値の制限
- 価格の大小関係（定価 ≥ 販売価格 ≥ 割引価格）

### 2. ビジネスルール実装

#### 在庫管理ルール
```sql
-- 予約数は在庫数を超えない
CONSTRAINT check_reserved_not_exceeds_quantity 
    CHECK (reserved_quantity <= quantity)
```

#### 価格設定ルール
```sql
-- 価格の論理的整合性
CONSTRAINT check_price_relationship CHECK (
    (list_price IS NULL OR selling_price <= list_price) AND
    (discount_price IS NULL OR discount_price <= selling_price)
)
```

#### 日付管理ルール
```sql
-- 有効期間の整合性
CONSTRAINT check_effective_dates CHECK (
    effective_from IS NULL OR 
    effective_until IS NULL OR 
    effective_from <= effective_until
)
```

## トリガーと自動処理

### 1. 更新日時の自動更新

```sql
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 各テーブルへの適用
CREATE TRIGGER update_categories_updated_at 
    BEFORE UPDATE ON categories 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();
```

### 2. 履歴記録の自動化（将来実装）

商品情報の変更を自動的に`product_history`テーブルに記録するトリガーの実装を検討。

## パフォーマンス考慮事項

### 1. インデックス戦略

- **カバリングインデックス**: 頻繁にアクセスされるカラムの組み合わせ
- **部分インデックス**: 特定条件下でのみ使用されるインデックス
- **インデックスの定期的な再構築**: VACUUMとREINDEXの実行

### 2. パーティショニング戦略

大量データに対する将来的な対応：

- `product_history`: 日付によるパーティショニング
- `product_prices`: 通貨別パーティショニング

### 3. クエリ最適化

- 適切なJOIN順序の選択
- EXISTS句の活用
- CTEの適切な使用

### 4. 接続プーリング

```yaml
推奨設定:
- max_connections: 100
- connection_pool_size: 32
- idle_timeout: 600秒
```

## データ移行とバックアップ

### 1. バックアップ戦略

- **日次バックアップ**: pg_dumpによる論理バックアップ
- **継続的バックアップ**: WALアーカイブによるPITR
- **レプリケーション**: ストリーミングレプリケーション

### 2. データ移行

- スキーマバージョン管理: sqlxマイグレーション
- ゼロダウンタイムマイグレーション戦略
- ロールバック手順の準備

## 監視とメンテナンス

### 1. 監視項目

- テーブルサイズとインデックスサイズ
- クエリパフォーマンス（pg_stat_statements）
- デッドロックとロック待機
- インデックスの使用状況

### 2. 定期メンテナンス

- VACUUM ANALYZE: 週次
- インデックス再構築: 月次
- パーティション管理: 必要に応じて
- 統計情報の更新: 日次