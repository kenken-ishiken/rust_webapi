# Rust WebAPI - 残りの改善タスク

## 現在の状況

### 完了済み
- ✅ エラーハンドリングの統一（unwrap/expect除去）
- ✅ 設定管理の分離（AppConfig実装）
- ✅ テストエラーの修正（全103件成功）
- ✅ 基本的なコード品質向上

### 現在の警告状況（2024年12月更新）
- **Clippy警告**: 0件（✅ redundant_closure、len_zero、missing_const_for_thread_local解消済み）
- **Dead code警告**: 3件（未使用フィールド：telemetry、service_name、log_level、is_active）
- **ビルド警告**: 0件

## Phase 1: Clippy警告の修正（優先度: 高）✅ **完了**

### 1.1 Redundant Closure警告の修正 ✅
**場所**: `tests/app_domain_repository_tests.rs`（4箇所）
```rust
// 修正前
.returning(move |item| Ok(item))

// 修正後
.returning(Ok)
```

### 1.2 Length Comparison警告の修正 ✅
**場所**: `tests/e2e_tests.rs`（2箇所）
```rust
// 修正前
assert!(items_array.len() >= 1);

// 修正後
assert!(!items_array.is_empty());
```

### 1.3 Thread Local Const警告の修正 ✅
**場所**: `tests/helpers/postgres.rs`
```rust
// 修正前
static CONTAINER: std::cell::RefCell<Option<...>> = std::cell::RefCell::new(None);

// 修正後
static CONTAINER: std::cell::RefCell<Option<...>> = const { std::cell::RefCell::new(None) };
```

### 1.4 Redundant Closure in Docker Client ✅
**場所**: `tests/helpers/postgres.rs`
```rust
// 修正前
let docker = DOCKER.get_or_init(|| testcontainers::clients::Cli::default());

// 修正後
let docker = DOCKER.get_or_init(testcontainers::clients::Cli::default);
```

## Phase 1.5: Dead Code警告の解消（優先度: 高）

### 1.5.1 未使用フィールドの処理
**対象**: 
- `CreateCategoryRequest.is_active` → 削除 or カテゴリ有効化機能実装
- `AppConfig.telemetry` → TelemetryConfigの実装完了 or 削除
- `TelemetryConfig.service_name`、`log_level` → 実際のログ設定で使用 or 削除

**完了基準**: 
- CI でdead_code警告0件
- 削除したフィールドに依存するテストの修正完了

## Phase 2-1: 依存性注入コンテナの実装（優先度: 中）

### 2.1.1 DIライブラリの選定と設計
**採用ライブラリ**: 手書きコンテナ（外部依存を避けるため）
**新規ファイル**: `src/infrastructure/di/container.rs`
- AppContainerの実装
- Repository、Service、Handlerの依存関係管理
- ライフサイクル管理（Singleton/Transient）

### 2.1.2 main.rsのリファクタリング
**目標**: main.rs < 80行（CI で行数チェック）
- 依存関係の自動解決
- 設定の注入
- サーバー起動処理の簡素化

**完了基準**:
- main.rs行数 < 80行
- 依存性注入コンテナが正常動作
- 全既存テストが成功（修正工数: 2-3時間見込み）

## Phase 2-2: リポジトリファイルの分割（優先度: 中）

### 2.2.1 ProductRepositoryの分割
**対象**: `src/infrastructure/repository/product_repository.rs`（1333行）

**分割後の構成**:
```
src/infrastructure/repository/
├── postgres/
│   └── product_repository.rs     # < 300行
├── in_memory/
│   └── product_repository.rs     # < 300行
├── product_repository.rs         # 共通トレイト < 100行
└── mod.rs
```

### 2.2.2 Contract Testの実装
**新規ファイル**: `tests/contract/product_repository_contract.rs`
- PostgresとInMemoryで共通のテスト仕様
- Fixtureによる実装切り替え
- テストの重複排除

**完了基準**:
- 各リポジトリファイル < 300行
- Contract testによる実装保証
- 既存テストの移行完了

## Phase 2-3: 削除操作の統一（優先度: 中）

### 2.3.1 Domain層でのDeletionStrategy実装
**新規ファイル**: `src/app_domain/service/deletion_service.rs`
- DeletionStrategy traitの定義
- 論理削除/物理削除の統一インターフェース
- エンティティごとの削除戦略

### 2.3.2 Application層でのFacade実装
**新規ファイル**: `src/application/service/deletion_facade.rs`
- Domain serviceの薄いラッパー
- HTTP レスポンス変換
- エラーハンドリング

**完了基準**:
- 削除関連コード30%削減（LoC比較）
- 全削除操作が統一インターフェース経由
- 削除戦略の切り替えが設定可能

## Phase 3-1: テストヘルパーの実装（優先度: 中）

### 3.1.1 MockRepositoryBuilderの実装
**新規ファイル**: `tests/helpers/mock_builder.rs`
- MockRepositoryBuilderの実装
- Fluent APIによるモック設定
- テストデータのファクトリー

### 3.1.2 テストコードのリファクタリング
**対象**: `tests/app_domain_repository_tests.rs`他
- 重複コードの削除
- テスト可読性の向上
- セットアップコードの共通化

**完了基準**:
- テストコード重複率50%削減（tokei LoC比較）
- MockBuilder使用率100%（該当テスト）
- テスト実行時間20%短縮

## Phase 3-2: エラー処理の統一（優先度: 中）

### 3.2.1 エラー型の統一
**改善対象**: `src/infrastructure/error.rs`
- `crate::error::AppError`での統一
- anyhow + thiserror の活用
- 外部へはactix `ResponseError`実装

### 3.2.2 エラーレスポンスの標準化
- JSON エラーレスポンスの統一
- エラーコードの体系化
- ログ記録の標準化

**完了基準**:
- 全モジュールでAppError使用率100%
- unwrap/expect使用箇所0件（CI チェック）
- エラーレスポンス形式統一

## Phase 3-3: メトリクス記録の統一（優先度: 低）

### 3.3.1 メトリクスマクロの実装
**改善対象**: `src/infrastructure/metrics/mod.rs`
- `metrics!(request_count)`マクロ実装
- tracing との統合
- パフォーマンス測定の統一

**完了基準**:
- 全APIハンドラでメトリクス記録100%
- メトリクス関連重複コード70%削減
- Prometheus形式でのメトリクス出力

## Phase 4: ドキュメントと最適化（優先度: 低）

### 4.1 ドキュメントの整備
**対象**: 
- APIドキュメントの更新（OpenAPI 3.0）
- アーキテクチャ図の作成（Mermaid）
- 開発ガイドの更新

### 4.2 パフォーマンス最適化とSLA検証
**k6テストSLA基準**:
- 95パーセンタイル応答時間 < 250ms
- エラー率 < 0.1%
- 同時接続数1000でのスループット > 500 req/s

**完了基準**:
- k6 SLAテスト合格
- データベースクエリ最適化完了
- メモリ使用量ベースライン確立

## 実装タイムライン（10週間計画）

| 週 | 主タスク | 工数見積もり | 完了基準 |
|----|----------|-------------|----------|
| 1  | Phase 1.5: Dead code解消 | 4h | CI警告0件 |
| 2  | Phase 2-1: DI設計 + PoC | 8h | コンテナ基本動作 |
| 3  | Phase 2-1: main.rsリファクタ | 6h | main.rs < 80行 |
| 4  | Phase 2-2: Repository分割(Postgres) | 8h | contract test追加 |
| 5  | Phase 2-2: Repository分割(InMemory) | 6h | 分割完了 |
| 6  | Phase 2-3: DeletionStrategy実装 | 8h | domain層実装 |
| 7  | Phase 3-1: MockBuilder実装 | 6h | テスト重複50%削減 |
| 8  | Phase 3-2: Error統一 | 8h | AppError 100%使用 |
| 9  | Phase 3-3: Metrics統一 | 4h | マクロ実装完了 |
| 10 | Phase 4: ドキュメント + SLA検証 | 6h | k6テスト合格 |

## リスク管理

### 低リスク（ロールバック容易）
- Dead code解消
- テストヘルパー追加
- ドキュメント更新
- メトリクスマクロ実装

### 中リスク（影響範囲中程度）
- 依存性注入実装
  - **影響範囲**: main.rs, 全ハンドラ
  - **ロールバック**: DIコンテナ削除、元のファクトリー関数復活
- リポジトリ分割
  - **影響範囲**: infrastructure/repository配下
  - **ロールバック**: 分割前ファイルの復元

### 高リスク（慎重な実装必要）
- エラーハンドリング統一
  - **影響範囲**: 全レイヤ（domain, application, infrastructure, presentation）
  - **ロールバック**: 段階的実装、レイヤ単位での切り戻し
  - **対策**: feature flagによる段階適用

## 定量的完了基準

### CI/CD基準
- [ ] Clippy警告: 0件
- [ ] Dead code警告: 0件
- [ ] テスト成功率: 100%
- [ ] カバレッジ: > 80%

### コード品質基準
- [ ] main.rs: < 80行
- [ ] 最大ファイルサイズ: < 500行
- [ ] テストコード重複率: -50%
- [ ] unwrap/expect使用箇所: 0件

### パフォーマンス基準
- [ ] k6テスト: 95%ile < 250ms
- [ ] エラー率: < 0.1%
- [ ] メモリ使用量: ベースライン+20%以内

### ドキュメント基準
- [ ] API仕様書: OpenAPI 3.0完備
- [ ] アーキテクチャ図: Mermaid形式
- [ ] 開発ガイド: 最新状態

---

**最終更新**: 2024年12月現在  
**次回レビュー**: Phase 1.5完了後（Week 1終了時） 