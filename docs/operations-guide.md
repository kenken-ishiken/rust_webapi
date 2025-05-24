# 運用ガイド

このドキュメントでは、Rust WebAPI を本番環境で運用するための情報を提供します。

## 目次

- [デプロイメント](#デプロイメント)
- [設定管理](#設定管理)
- [監視とアラート](#監視とアラート)
- [ロギング](#ロギング)
- [バックアップと復旧](#バックアップと復旧)
- [スケーリング](#スケーリング)
- [セキュリティ](#セキュリティ)
- [運用チェックリスト](#運用チェックリスト)

## デプロイメント

### コンテナイメージのビルド

```bash
# イメージをビルド
docker build -t yourregistry/rust-webapi:latest .

# イメージをプッシュ
docker push yourregistry/rust-webapi:latest
```

### Kubernetes デプロイ

このプロジェクトは Kubernetes / Istio 環境での本番運用を想定した設定を提供しています。詳細なデプロイ手順は [k8s/README.md](../k8s/README.md) を参照してください。

基本的なデプロイコマンド：

```bash
# ベースデプロイ
kubectl apply -k k8s/base

# 環境別デプロイ（開発環境）
kubectl apply -k k8s/overlays/dev

# 環境別デプロイ（本番環境）
kubectl apply -k k8s/overlays/prod
```

### アップグレード手順

1. 新しいイメージをビルドしてレジストリにプッシュ
2. overlaysディレクトリ内のkustomization.yamlでイメージタグを更新
3. `kubectl apply -k` でデプロイ
4. ロールアウト状態を確認：`kubectl rollout status deployment/rust-webapi -n api`

## 設定管理

### 環境変数

アプリケーションは以下の環境変数で設定できます：

| 変数名 | 説明 | デフォルト値 | 必須 |
|-------|------|------------|------|
| DATABASE_URL | PostgreSQL接続文字列 | - | 必須 |
| PORT | APIサーバーのポート | 8080 | オプション |
| HOST | APIサーバーのホスト | 0.0.0.0 | オプション |
| RUST_LOG | ログレベル設定 | info | オプション |
| KEYCLOAK_AUTH_SERVER_URL | Keycloak認証サーバーURL | - | 必須 |
| KEYCLOAK_REALM | Keycloakレルム名 | - | 必須 |
| KEYCLOAK_CLIENT_ID | KeycloakクライアントID | - | 必須 |
| OTEL_EXPORTER_OTLP_ENDPOINT | OpenTelemetryエンドポイント | - | オプション |

### Kubernetes ConfigMap と Secret

- **ConfigMap**: 非機密設定（Keycloak URL、ログレベルなど）
- **Secret**: 機密情報（データベース接続文字列など）

例：

```yaml
# ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: rust-webapi-config
  namespace: api
data:
  RUST_LOG: "info"
  KEYCLOAK_AUTH_SERVER_URL: "https://keycloak.example.com"
  KEYCLOAK_REALM: "rust-webapi"
  KEYCLOAK_CLIENT_ID: "api-client"

# Secret
apiVersion: v1
kind: Secret
metadata:
  name: rust-webapi-secrets
  namespace: api
type: Opaque
data:
  DATABASE_URL: "cG9zdGdyZXM6Ly91c2VyOnBhc3N3b3JkQGRiLmV4YW1wbGUuY29tOjU0MzIvZGJuYW1l"
```

## 監視とアラート

### メトリクス

アプリケーションは `/api/metrics` エンドポイントで Prometheus 形式のメトリクスを提供します：

- **api_request_duration_seconds**: HTTPリクエスト処理時間（ヒストグラム）
- **api_success_count**: 成功したAPIコール数（カウンター）
- **api_error_count**: 失敗したAPIコール数（カウンター）

### Prometheus 設定例

```yaml
scrape_configs:
  - job_name: "rust-webapi"
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_namespace, __meta_kubernetes_pod_label_app]
        regex: api;rust-webapi
        action: keep
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\\d+)?;(\\d+)
        replacement: $1:$2
        target_label: __address__
```

### アラート設定例

```yaml
groups:
- name: rust-webapi-alerts
  rules:
  - alert: HighErrorRate
    expr: sum(rate(api_error_count[5m])) / sum(rate(api_success_count[5m]) + rate(api_error_count[5m])) > 0.01
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High API error rate"
      description: "Error rate is {{ $value | humanizePercentage }} over the last 5 minutes"

  - alert: SlowAPIResponse
    expr: histogram_quantile(0.95, sum(rate(api_request_duration_seconds_bucket[5m])) by (le)) > 0.5
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Slow API response time"
      description: "95th percentile of API response time is {{ $value }} seconds"
```

## ロギング

アプリケーションは構造化JSONログを標準出力に出力します。これらのログは以下の方法で収集できます：

### Fluentd 設定例

```
<source>
  @type tail
  path /var/log/containers/rust-webapi-*.log
  pos_file /var/log/fluentd-rust-webapi.pos
  tag kubernetes.rust-webapi
  <parse>
    @type json
    time_format %Y-%m-%dT%H:%M:%S.%NZ
  </parse>
</source>

<filter kubernetes.rust-webapi>
  @type record_transformer
  <record>
    service_name rust-webapi
  </record>
</filter>

<match kubernetes.rust-webapi>
  @type elasticsearch
  host elasticsearch.logging
  port 9200
  logstash_format true
  logstash_prefix rust-webapi
  flush_interval 5s
</match>
```

### ログレベル

実行時のログレベルは `RUST_LOG` 環境変数で制御できます：

- `error`: エラーのみ
- `warn`: 警告とエラー
- `info`: 情報、警告、エラー（デフォルト）
- `debug`: デバッグ情報を含むすべてのログ
- `trace`: トレースレベルを含むすべてのログ（非常に詳細）

## バックアップと復旧

### データベースバックアップ

PostgreSQLデータベースのバックアップを定期的に実行します：

```bash
# バックアップ
pg_dump -h postgres -U postgres -d rustwebapi > backup_$(date +%Y%m%d).sql

# 復元
psql -h postgres -U postgres -d rustwebapi < backup_20250101.sql
```

### Kubernetes リソースバックアップ

```bash
# 設定のバックアップ
kubectl get configmap,secret -n api -o yaml > k8s_config_backup_$(date +%Y%m%d).yaml
```

## スケーリング

### 水平スケーリング

Kubernetes Horizontal Pod Autoscaler (HPA) を使用して自動スケーリングを設定できます：

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: rust-webapi-hpa
  namespace: api
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: rust-webapi
  minReplicas: 2
  maxReplicas: 10
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

### データベーススケーリング

PostgreSQLのコネクションプール設定は環境変数 `DATABASE_MAX_CONNECTIONS` で調整できます。

## セキュリティ

### セキュリティベストプラクティス

- すべての機密情報（DB接続文字列など）はKubernetes Secretで管理
- アプリケーションはroot以外のユーザーで実行（Dockerfileで設定済み）
- 認証エンドポイントにはレート制限を適用
- TLS終端はIstio Ingress Gatewayで処理
- アプリケーションのログには機密情報を含めない

### 脆弱性スキャン

定期的にイメージをスキャンします：

```bash
# Trivy を使用したコンテナイメージスキャン
trivy image yourregistry/rust-webapi:latest
```

## 運用チェックリスト

### デプロイ前チェックリスト

- [ ] 単体テストと統合テストが成功している
- [ ] セキュリティスキャンでクリティカルな脆弱性が検出されていない
- [ ] リソース設定（CPU/メモリ制限）が適切に設定されている
- [ ] ヘルスチェックとレディネスプローブが設定されている
- [ ] バックアップから復元するプロセスがテストされている
- [ ] ロールバック手順が準備されている

### 定期メンテナンスチェックリスト

- [ ] セキュリティアップデートの適用
- [ ] 定期的なデータベースバックアップの検証
- [ ] メトリクスとアラートの有効性確認
- [ ] リソース使用量の確認と必要に応じた調整
- [ ] ログローテーションの確認
- [ ] SSL/TLS証明書の有効期限確認
