# デプロイメント図

## Kubernetes構成

```mermaid
graph TB
    subgraph "Internet"
        Client[クライアント]
    end

    subgraph "Kubernetes Cluster"
        subgraph "Gateway"
            GW[Gateway<br/>HTTPRoute]
        end

        subgraph "rust-webapi namespace"
            subgraph "Deployment"
                Pod1[Pod 1<br/>rust-webapi]
                Pod2[Pod 2<br/>rust-webapi]
                Pod3[Pod 3<br/>rust-webapi]
            end

            SVC[Service<br/>ClusterIP]
            HPA[HorizontalPodAutoscaler<br/>min: 3, max: 10]
            PDB[PodDisruptionBudget<br/>minAvailable: 2]

            subgraph "ConfigMaps & Secrets"
                CM[ConfigMap<br/>app-config]
                Secret[Secret<br/>app-secret]
            end

            subgraph "Database"
                PG[PostgreSQL<br/>StatefulSet]
                PVC[PersistentVolumeClaim<br/>10Gi]
            end
        end

        subgraph "Monitoring"
            Prometheus[Prometheus]
            Grafana[Grafana]
        end

        subgraph "Observability"
            DD[Datadog Agent<br/>DaemonSet]
        end
    end

    subgraph "External Services"
        KC[Keycloak<br/>認証サーバー]
    end

    Client --> GW
    GW --> SVC
    SVC --> Pod1
    SVC --> Pod2
    SVC --> Pod3
    
    Pod1 --> PG
    Pod2 --> PG
    Pod3 --> PG
    
    Pod1 --> KC
    Pod2 --> KC
    Pod3 --> KC
    
    CM -.-> Pod1
    CM -.-> Pod2
    CM -.-> Pod3
    
    Secret -.-> Pod1
    Secret -.-> Pod2
    Secret -.-> Pod3
    
    PG --> PVC
    
    HPA --> Pod1
    HPA --> Pod2
    HPA --> Pod3
    
    Pod1 --> Prometheus
    Pod2 --> Prometheus
    Pod3 --> Prometheus
    
    DD --> Pod1
    DD --> Pod2
    DD --> Pod3
    
    Prometheus --> Grafana
```

## コンポーネント説明

### アプリケーション層
- **Pods**: rust-webapiコンテナを実行
  - 最小3レプリカ、最大10レプリカ
  - リソース制限: CPU 500m, Memory 512Mi
  - ヘルスチェック: /api/health

### ネットワーク層
- **Gateway**: 外部からのHTTPSトラフィックを受信
- **Service**: ClusterIPでPod間の負荷分散
- **NetworkPolicy**: 必要な通信のみ許可

### データ層
- **PostgreSQL**: StatefulSetで実行
- **PersistentVolume**: 10GiBのストレージ

### 監視・観測性
- **Prometheus**: メトリクス収集
- **Grafana**: メトリクス可視化
- **Datadog**: APM、ログ、メトリクス統合監視

### 外部サービス
- **Keycloak**: OAuth2/OIDC認証プロバイダー

## デプロイメントフロー

```mermaid
sequenceDiagram
    participant Dev as 開発者
    participant Git as GitHub
    participant CI as GitHub Actions
    participant Reg as Container Registry
    participant K8s as Kubernetes

    Dev->>Git: git push
    Git->>CI: Trigger workflow
    
    CI->>CI: cargo test
    CI->>CI: cargo build --release
    CI->>CI: docker build
    
    CI->>Reg: docker push
    CI->>K8s: kubectl apply -k k8s/overlays/staging
    
    K8s->>K8s: Rolling update
    K8s-->>CI: Deployment complete
    CI-->>Dev: Success notification
```

## 環境別設定

### Development
```yaml
replicas: 1
resources:
  limits:
    cpu: 200m
    memory: 256Mi
database:
  size: 1Gi
```

### Staging
```yaml
replicas: 2
resources:
  limits:
    cpu: 500m
    memory: 512Mi
database:
  size: 5Gi
```

### Production
```yaml
replicas: 3-10 (HPA)
resources:
  limits:
    cpu: 1000m
    memory: 1Gi
database:
  size: 50Gi
  backup: enabled
```

## セキュリティ設定

```mermaid
graph LR
    subgraph "Security Layers"
        TLS[TLS/HTTPS]
        Auth[JWT認証]
        RBAC[Kubernetes RBAC]
        NetPol[NetworkPolicy]
        PSP[PodSecurityPolicy]
    end
    
    TLS --> Auth
    Auth --> RBAC
    RBAC --> NetPol
    NetPol --> PSP
```

### セキュリティポリシー
- すべての通信はTLS暗号化
- JWT認証必須（ヘルスチェック除く）
- 最小権限の原則（RBAC）
- ネットワーク分離（NetworkPolicy）
- コンテナセキュリティ（非root実行） 