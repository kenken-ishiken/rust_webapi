# Kubernetes Deployment for Rust WebAPI

This directory contains Kubernetes manifests for deploying the Rust WebAPI application with Istio Ingress using Gateway API.

## Architecture

The deployment consists of the following components:

1. **Rust WebAPI Application**:
   - Deployment with 2 replicas
   - Service exposing port 80
   - ConfigMap for Keycloak configuration
   - Secret for database credentials

2. **PostgreSQL Database**:
   - Deployment with 1 replica
   - Service exposing port 5432
   - PersistentVolumeClaim for data persistence

3. **Istio Ingress with Gateway API**:
   - Gateway resource specifying Istio as the implementation
   - HTTPRoute for routing traffic to the API

## Namespaces

- `api`: Contains the Rust WebAPI application
- `database`: Contains the PostgreSQL database

Both namespaces have Istio sidecar injection enabled.

## Prerequisites

1. Kubernetes cluster with Istio installed
2. Gateway API CRDs installed
3. Istio configured as a Gateway API controller
4. Docker registry with the Rust WebAPI image

## Deployment Instructions

1. Update the Docker registry in the deployment.yaml file:
   ```
   sed -i 's|${DOCKER_REGISTRY}|your-registry.example.com|g' k8s/base/deployment.yaml
   ```

2. Create the Secret with your actual database URL:
   ```
   kubectl create secret generic rust-webapi-secrets \
     --namespace api \
     --from-literal=database-url='postgres://postgres:password@postgres.database:5432/rustwebapi'
   ```

3. Update the ConfigMap with your actual values:
   - Update `keycloak-realm`, `keycloak-auth-server-url`, and `keycloak-client-id` in `configmap.yaml`

3. Apply the manifests using Kustomize:
   ```
   kubectl apply -k k8s/base
   ```

4. Verify the deployment:
   ```
   kubectl get pods -n api
   kubectl get pods -n database
   kubectl get gateway -n api
   kubectl get httproute -n api
   ```

5. Access the API through the Istio Ingress Gateway:
   ```
   export INGRESS_HOST=$(kubectl -n istio-system get service istio-ingressgateway -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
   export INGRESS_PORT=$(kubectl -n istio-system get service istio-ingressgateway -o jsonpath='{.spec.ports[?(@.name=="http2")].port}')
   curl http://$INGRESS_HOST:$INGRESS_PORT/api/health
   ```

## Customization

For different environments (dev, staging, prod), you can create overlay directories with Kustomize:

```
k8s/
├── base/
│   └── ... (base manifests)
├── overlays/
│   ├── dev/
│   │   └── kustomization.yaml
│   ├── staging/
│   │   └── kustomization.yaml
│   └── prod/
│       └── kustomization.yaml
```

Example overlay kustomization.yaml:
```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - ../../base

namespace: api-prod

patches:
  - path: deployment-patch.yaml
```
