# デプロイメント＆運用ガイド

## 目次

1. [環境構成](#環境構成)
2. [ローカル開発環境](#ローカル開発環境)
3. [Docker環境](#docker環境)
4. [Kubernetes環境](#kubernetes環境)
5. [CI/CDパイプライン](#cicdパイプライン)
6. [監視と可観測性](#監視と可観測性)
7. [セキュリティ](#セキュリティ)
8. [トラブルシューティング](#トラブルシューティング)
9. [災害復旧](#災害復旧)

## 環境構成

### 環境一覧

| 環境 | 用途 | URL | 特徴 |
|------|------|-----|------|
| 開発 (dev) | 開発・デバッグ | http://localhost:8080 | ホットリロード、詳細ログ |
| ステージング (staging) | 統合テスト | https://staging.api.example.com | 本番同等構成 |
| 本番 (production) | 実サービス | https://api.example.com | 高可用性、自動スケーリング |

### 環境変数設定

```bash
# 共通設定
RUST_LOG=info
PORT=8080

# データベース
DATABASE_URL=postgres://user:password@host:5432/dbname
DB_MAX_CONNECTIONS=32
DB_MIN_CONNECTIONS=5

# 認証（Keycloak）
KEYCLOAK_AUTH_SERVER_URL=http://localhost:8081
KEYCLOAK_REALM=rust-webapi
KEYCLOAK_CLIENT_ID=api-client
KEYCLOAK_CLIENT_SECRET=your-secret

# 監視
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
DATADOG_API_KEY=your-datadog-key
PROMETHEUS_ENDPOINT=/api/metrics

# アプリケーション設定
ENABLE_SWAGGER=true
CORS_ALLOWED_ORIGINS=http://localhost:3000,https://app.example.com
REQUEST_TIMEOUT_SECONDS=30
```

## ローカル開発環境

### 前提条件

```bash
# Rustのインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
rustup component add rustfmt clippy

# 必要なツール
cargo install cargo-watch
cargo install sqlx-cli
cargo install cargo-audit
```

### セットアップ手順

1. **リポジトリのクローン**
```bash
git clone https://github.com/your-org/rust-webapi.git
cd rust-webapi
```

2. **環境変数の設定**
```bash
cp .env.example .env
# .envファイルを編集
```

3. **データベースの起動**
```bash
docker-compose up -d postgres
```

4. **マイグレーションの実行**
```bash
sqlx migrate run
```

5. **開発サーバーの起動**
```bash
# ホットリロード付き
cargo watch -x run

# または通常起動
cargo run
```

### 開発用コマンド

```bash
# フォーマット
cargo fmt

# リント
cargo clippy --all-targets -- -D warnings

# テスト実行
cargo test

# カバレッジレポート生成
./scripts/coverage.sh

# セキュリティ監査
cargo audit
```

## Docker環境

### Dockerイメージのビルド

```dockerfile
# マルチステージビルド
FROM rust:1.77 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY crates ./crates
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust_webapi /usr/local/bin/
CMD ["rust_webapi"]
```

### Docker Compose設定

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://postgres:password@postgres:5432/rustwebapi
      - RUST_LOG=info
    depends_on:
      - postgres
      - keycloak
    networks:
      - app-network

  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
      POSTGRES_DB: rustwebapi
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./initdb:/docker-entrypoint-initdb.d
    ports:
      - "5432:5432"
    networks:
      - app-network

  keycloak:
    image: quay.io/keycloak/keycloak:23.0
    environment:
      KEYCLOAK_ADMIN: admin
      KEYCLOAK_ADMIN_PASSWORD: admin
    command: start-dev
    ports:
      - "8081:8080"
    networks:
      - app-network

volumes:
  postgres-data:

networks:
  app-network:
    driver: bridge
```

### Docker運用コマンド

```bash
# 起動
docker-compose up -d

# ログ確認
docker-compose logs -f app

# 再起動
docker-compose restart app

# スケーリング
docker-compose up -d --scale app=3

# クリーンアップ
docker-compose down -v
```

## Kubernetes環境

### アーキテクチャ

```mermaid
graph TB
    subgraph "Kubernetes Cluster"
        subgraph "Ingress"
            GW[Gateway API]
            ROUTE[HTTPRoute]
        end
        
        subgraph "Application"
            SVC[Service]
            DEP[Deployment]
            HPA[HorizontalPodAutoscaler]
            PDB[PodDisruptionBudget]
        end
        
        subgraph "Configuration"
            CM[ConfigMap]
            SEC[Secret]
        end
        
        subgraph "Data"
            PG[PostgreSQL]
            PVC[PersistentVolumeClaim]
        end
    end
    
    GW --> ROUTE
    ROUTE --> SVC
    SVC --> DEP
    HPA --> DEP
    DEP --> CM
    DEP --> SEC
    PG --> PVC
```

### Kustomizeによるデプロイ

```bash
# 基本構造
k8s/
├── base/
│   ├── kustomization.yaml
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   └── secret.yaml
└── overlays/
    ├── dev/
    ├── staging/
    └── production/
```

#### base/deployment.yaml
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-webapi
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rust-webapi
  template:
    metadata:
      labels:
        app: rust-webapi
    spec:
      containers:
      - name: app
        image: your-registry/rust-webapi:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: rust-webapi-secret
              key: database-url
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

#### HPA設定
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: rust-webapi-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: rust-webapi
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### デプロイコマンド

```bash
# 開発環境へのデプロイ
kubectl apply -k k8s/overlays/dev

# 本番環境へのデプロイ
kubectl apply -k k8s/overlays/production

# ロールアウト状況確認
kubectl rollout status deployment/rust-webapi

# ロールバック
kubectl rollout undo deployment/rust-webapi

# スケール調整
kubectl scale deployment/rust-webapi --replicas=5
```

## CI/CDパイプライン

### GitHub Actions

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - uses: Swatinem/rust-cache@v2
    
    - name: Run tests
      run: cargo test
      env:
        DATABASE_URL: postgres://postgres:postgres@localhost/test_db
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Check formatting
      run: cargo fmt -- --check

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v3
    
    - name: Build Docker image
      run: |
        docker build -t ${{ secrets.REGISTRY }}/rust-webapi:${{ github.sha }} .
        docker tag ${{ secrets.REGISTRY }}/rust-webapi:${{ github.sha }} \
                   ${{ secrets.REGISTRY }}/rust-webapi:latest
    
    - name: Push to registry
      run: |
        echo ${{ secrets.REGISTRY_PASSWORD }} | docker login -u ${{ secrets.REGISTRY_USERNAME }} --password-stdin
        docker push ${{ secrets.REGISTRY }}/rust-webapi:${{ github.sha }}
        docker push ${{ secrets.REGISTRY }}/rust-webapi:latest

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v3
    
    - name: Deploy to Kubernetes
      run: |
        kubectl set image deployment/rust-webapi \
          app=${{ secrets.REGISTRY }}/rust-webapi:${{ github.sha }} \
          --namespace=production
```

## 監視と可観測性

### メトリクス収集

```yaml
# Prometheus設定
scrape_configs:
  - job_name: 'rust-webapi'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        action: keep
        regex: rust-webapi
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
```

### ログ収集

```yaml
# Fluentd設定
<source>
  @type tail
  path /var/log/containers/rust-webapi-*.log
  pos_file /var/log/fluentd-containers.log.pos
  tag kubernetes.*
  read_from_head true
  <parse>
    @type json
    time_format %Y-%m-%dT%H:%M:%S.%NZ
  </parse>
</source>

<filter kubernetes.**>
  @type kubernetes_metadata
</filter>

<match **>
  @type elasticsearch
  host elasticsearch
  port 9200
  logstash_format true
  logstash_prefix rust-webapi
</match>
```

### アラート設定

```yaml
# PrometheusRule
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: rust-webapi-alerts
spec:
  groups:
  - name: rust-webapi
    rules:
    - alert: HighErrorRate
      expr: |
        sum(rate(http_requests_total{job="rust-webapi",status=~"5.."}[5m])) 
        / 
        sum(rate(http_requests_total{job="rust-webapi"}[5m])) > 0.05
      for: 5m
      labels:
        severity: critical
      annotations:
        summary: "High error rate detected"
        description: "Error rate is above 5% for 5 minutes"
    
    - alert: HighLatency
      expr: |
        histogram_quantile(0.95, 
          sum(rate(http_request_duration_seconds_bucket{job="rust-webapi"}[5m])) 
          by (le)
        ) > 0.5
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High latency detected"
        description: "95th percentile latency is above 500ms"
```

### ダッシュボード

```json
{
  "dashboard": {
    "title": "Rust WebAPI Monitoring",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{job='rust-webapi'}[5m])) by (method, status)"
          }
        ]
      },
      {
        "title": "Response Time",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket{job='rust-webapi'}[5m])) by (le, method))"
          }
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{job='rust-webapi',status=~'5..'}[5m])) / sum(rate(http_requests_total{job='rust-webapi'}[5m]))"
          }
        ]
      }
    ]
  }
}
```

## セキュリティ

### セキュリティチェックリスト

- [ ] HTTPS/TLSの適用
- [ ] 認証・認可の実装
- [ ] 入力値検証
- [ ] SQLインジェクション対策
- [ ] XSS対策
- [ ] CSRF対策
- [ ] レート制限
- [ ] セキュリティヘッダー設定
- [ ] 依存関係の脆弱性スキャン
- [ ] シークレット管理

### Kubernetes Security

```yaml
# NetworkPolicy
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: rust-webapi-netpol
spec:
  podSelector:
    matchLabels:
      app: rust-webapi
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: database
    ports:
    - protocol: TCP
      port: 5432
```

## トラブルシューティング

### よくある問題と解決策

#### 1. データベース接続エラー

```bash
# 症状
Error: Failed to connect to database

# 診断
kubectl exec -it deployment/rust-webapi -- nc -zv postgres 5432
kubectl logs deployment/rust-webapi | grep database

# 解決策
# 1. 接続文字列の確認
kubectl get secret rust-webapi-secret -o jsonpath='{.data.database-url}' | base64 -d

# 2. ネットワークポリシーの確認
kubectl get networkpolicy

# 3. PostgreSQLサービスの確認
kubectl get svc postgres
```

#### 2. メモリ不足

```bash
# 症状
OOMKilled

# 診断
kubectl top pods -l app=rust-webapi
kubectl describe pod <pod-name>

# 解決策
# リソース制限の調整
kubectl patch deployment rust-webapi -p '{"spec":{"template":{"spec":{"containers":[{"name":"app","resources":{"limits":{"memory":"512Mi"}}}]}}}}'
```

#### 3. 高レイテンシ

```bash
# 診断
# 1. アプリケーションメトリクスの確認
curl http://localhost:8080/api/metrics | grep http_request_duration

# 2. データベースのスロークエリ確認
kubectl exec -it postgres-0 -- psql -U postgres -c "SELECT * FROM pg_stat_statements ORDER BY total_time DESC LIMIT 10;"

# 3. CPU使用率確認
kubectl top pods

# 解決策
# 1. インデックスの追加
# 2. クエリの最適化
# 3. キャッシュの実装
# 4. 水平スケーリング
```

### デバッグツール

```bash
# ログの確認
kubectl logs -f deployment/rust-webapi --tail=100

# シェルアクセス
kubectl exec -it deployment/rust-webapi -- /bin/bash

# ポートフォワード
kubectl port-forward deployment/rust-webapi 8080:8080

# トレースの確認
kubectl exec deployment/rust-webapi -- curl http://localhost:8080/debug/pprof/trace

# メトリクスの確認
kubectl exec deployment/rust-webapi -- curl http://localhost:8080/api/metrics
```

## 災害復旧

### バックアップ戦略

```bash
# データベースバックアップ
#!/bin/bash
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
kubectl exec postgres-0 -- pg_dump -U postgres rustwebapi | gzip > backup_${TIMESTAMP}.sql.gz

# S3へのアップロード
aws s3 cp backup_${TIMESTAMP}.sql.gz s3://your-backup-bucket/postgres/
```

### リストア手順

```bash
# 1. アプリケーションの停止
kubectl scale deployment/rust-webapi --replicas=0

# 2. データベースのリストア
gunzip -c backup_20240115_120000.sql.gz | kubectl exec -i postgres-0 -- psql -U postgres rustwebapi

# 3. アプリケーションの起動
kubectl scale deployment/rust-webapi --replicas=3

# 4. ヘルスチェック
kubectl wait --for=condition=ready pod -l app=rust-webapi --timeout=300s
```

### RTO/RPO目標

| 指標 | 目標値 | 実現方法 |
|------|--------|----------|
| RTO (Recovery Time Objective) | 30分 | 自動化されたリストア手順 |
| RPO (Recovery Point Objective) | 1時間 | 1時間ごとの自動バックアップ |

### 障害シナリオ別対応

#### 1. Pod障害
- 自動再起動（Kubernetes）
- レプリカによる継続性

#### 2. ノード障害
- Pod の自動再スケジュール
- アンチアフィニティによる分散配置

#### 3. データベース障害
- レプリケーションによる自動フェイルオーバー
- バックアップからのリストア

#### 4. リージョン障害
- マルチリージョンデプロイ
- DNSフェイルオーバー