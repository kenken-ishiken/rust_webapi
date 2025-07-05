# Datadog Integration for Rust Web API

This directory contains Datadog configuration files for monitoring the Rust Web API application.

## Overview

The monitoring setup includes:
- Comprehensive dashboard with key metrics
- Prometheus metrics scraping configuration
- Alert monitors for critical conditions
- Kubernetes integration for container metrics

## Files

- `dashboard.json` - Main monitoring dashboard configuration
- `agent-values.yaml` - Datadog Agent Helm values for Kubernetes deployment
- `configmap-prometheus.yaml` - Prometheus scraping configuration
- `monitors.json` - Alert monitor definitions

## Setup Instructions

### 1. Install Datadog Agent in Kubernetes

```bash
# Add Datadog Helm repository
helm repo add datadog https://helm.datadoghq.com
helm repo update

# Install Datadog Agent with custom values
helm install datadog-agent datadog/datadog \
  --namespace datadog \
  --create-namespace \
  -f datadog/agent-values.yaml \
  --set datadog.apiKey=<YOUR_API_KEY> \
  --set datadog.appKey=<YOUR_APP_KEY>
```

### 2. Apply Prometheus Scraping Configuration

```bash
# Apply the ConfigMap for Prometheus scraping
kubectl apply -f datadog/configmap-prometheus.yaml

# Mount the ConfigMap to Datadog Agent (add to agent-values.yaml)
# This is already configured in the provided agent-values.yaml
```

### 3. Import Dashboard

#### Using Datadog API:
```bash
curl -X POST "https://api.datadoghq.com/api/v1/dashboard" \
  -H "Content-Type: application/json" \
  -H "DD-API-KEY: <YOUR_API_KEY>" \
  -H "DD-APPLICATION-KEY: <YOUR_APP_KEY>" \
  -d @datadog/dashboard.json
```

#### Using Datadog UI:
1. Navigate to Dashboards → New Dashboard
2. Click on "Import Dashboard JSON"
3. Paste the contents of `dashboard.json`
4. Click "Import"

### 4. Create Monitors

#### Using Datadog API:
```bash
# Create each monitor
for monitor in $(cat datadog/monitors.json | jq -c '.[]'); do
  curl -X POST "https://api.datadoghq.com/api/v1/monitor" \
    -H "Content-Type: application/json" \
    -H "DD-API-KEY: <YOUR_API_KEY>" \
    -H "DD-APPLICATION-KEY: <YOUR_APP_KEY>" \
    -d "$monitor"
done
```

#### Using Terraform (recommended):
```hcl
# datadog_monitors.tf
resource "datadog_monitor" "rust_webapi_monitors" {
  for_each = { for idx, monitor in jsondecode(file("${path.module}/datadog/monitors.json")) : idx => monitor }
  
  name     = each.value.name
  type     = each.value.type
  query    = each.value.query
  message  = each.value.message
  tags     = each.value.tags
  priority = each.value.priority
  
  dynamic "monitor_thresholds" {
    for_each = can(each.value.options.thresholds) ? [each.value.options.thresholds] : []
    content {
      critical = monitor_thresholds.value.critical
      warning  = lookup(monitor_thresholds.value, "warning", null)
      ok       = lookup(monitor_thresholds.value, "ok", null)
    }
  }
  
  notify_no_data    = lookup(each.value.options, "notify_no_data", false)
  no_data_timeframe = lookup(each.value.options, "no_data_timeframe", null)
  notify_audit      = lookup(each.value.options, "notify_audit", false)
  include_tags      = lookup(each.value.options, "include_tags", true)
  
  renotify_interval = lookup(each.value.options, "renotify_interval", null)
  escalation_message = lookup(each.value.options, "escalation_message", null)
}
```

## Dashboard Widgets

The dashboard includes the following widgets:

1. **API Request Rate** - Shows successful and failed requests per second
2. **Error Rate** - Percentage of failed requests
3. **Health Check Status** - Service availability monitor
4. **Request Duration** - P50, P95, and P99 latencies by endpoint
5. **Requests by Endpoint** - Top 10 endpoints by request volume
6. **Errors by Endpoint** - Top 10 endpoints by error count
7. **Container CPU Usage** - CPU utilization by pod
8. **Container Memory Usage** - Memory consumption by pod
9. **Pod Count** - Number of running pods
10. **Database Connection Pool** - Active database connections
11. **Logs Stream** - Real-time application logs

## Monitors

The following monitors are configured:

1. **High Error Rate** - Alerts when error rate exceeds 5%
2. **High Response Time** - Alerts when P95 latency exceeds 1 second
3. **Service Down** - Alerts when health check fails
4. **Low Pod Count** - Alerts when pods drop below minimum threshold
5. **High Memory Usage** - Alerts when memory exceeds 1GB
6. **Database Connection Issues** - Alerts on database connectivity problems
7. **Anomaly Detection** - Alerts on abnormal traffic patterns

## Metrics Collected

### Application Metrics (via Prometheus)

#### HTTP Level Metrics (Enhanced)
- `prometheus.http_requests_total` - Total HTTP requests by method, endpoint, and status
  - Labels: `method`, `endpoint`, `status`
  - Use for: Request rate, traffic patterns, error tracking
- `prometheus.http_request_duration_seconds` - HTTP request latency histogram
  - Labels: `method`, `endpoint`, `status`
  - Buckets: 0.001s to 5.0s
  - Use for: Performance monitoring, SLA tracking
- `prometheus.http_responses_total` - Response counts by status class
  - Labels: `status_class` (2xx, 4xx, 5xx)
  - Use for: Error rate calculation, health monitoring

#### Service Level Metrics (Legacy, maintained for compatibility)
- `prometheus.api_success_count` - Successful API calls by endpoint
- `prometheus.api_error_count` - Failed API calls by endpoint
- `prometheus.api_request_duration_seconds` - Request duration histogram

### Kubernetes Metrics
- Pod CPU and memory usage
- Pod count and status
- Container restart count
- Network I/O

### Database Metrics
- Connection pool size
- Query performance
- Database availability

## Example Queries

### Error Rate by Endpoint
```
sum(rate(prometheus.http_requests_total{status=~"5.."}[5m])) by (endpoint, method) / 
sum(rate(prometheus.http_requests_total[5m])) by (endpoint, method)
```

### P95 Latency
```
histogram_quantile(0.95, 
  sum(rate(prometheus.http_request_duration_seconds_bucket[5m])) by (endpoint, le)
)
```

### Request Rate
```
sum(rate(prometheus.http_requests_total[5m])) by (method, endpoint)
```

## Customization

### Adding New Metrics

1. Add metric collection in the application code:
```rust
// In src/infrastructure/metrics/mod.rs
register_histogram!(
    "my_custom_metric",
    "Description of the metric"
).unwrap();
```

2. Update the Prometheus scraping config to include the new metric
3. Add widgets to the dashboard to visualize the new metric

### Modifying Alerts

Edit `monitors.json` to adjust thresholds or add new monitors. Key fields:
- `query`: The metric query
- `thresholds`: Alert trigger values
- `message`: Notification template
- `priority`: 1 (highest) to 5 (lowest)

## Troubleshooting

### Metrics Not Appearing

1. Verify Datadog Agent is running:
```bash
kubectl get pods -n datadog
```

2. Check Agent logs:
```bash
kubectl logs -n datadog -l app=datadog-agent
```

3. Verify Prometheus endpoint is accessible:
```bash
kubectl port-forward -n rust-webapi svc/rust-webapi 8080:8080
curl http://localhost:8080/api/metrics
```

### Dashboard Import Issues

- Ensure API and APP keys have correct permissions
- Check for widget ID conflicts if importing to existing dashboard
- Validate JSON syntax: `jq . datadog/dashboard.json`

### Monitor Creation Failures

- Verify query syntax in Datadog Metric Explorer
- Check tag existence before using in monitors
- Ensure notification channels (@pagerduty, @slack) are configured

## Best Practices

1. **Use Tags Consistently**
   - Always include: `service:rust_webapi`, `env:<environment>`, `team:<team>`
   - Add component-specific tags: `endpoint:<path>`, `component:database`

2. **Set Appropriate Alert Thresholds**
   - Base thresholds on historical data
   - Start with conservative values and adjust
   - Use warning thresholds for early detection

3. **Monitor Dashboard Performance**
   - Keep widget count reasonable (< 20 per dashboard)
   - Use time aggregation for high-cardinality metrics
   - Create separate dashboards for different audiences

4. **Regular Review**
   - Review alert fatigue monthly
   - Update thresholds based on SLOs
   - Archive unused monitors

## Integration with CI/CD

Add dashboard and monitor validation to your CI pipeline:

```yaml
# .github/workflows/datadog-validation.yml
name: Validate Datadog Configs
on:
  pull_request:
    paths:
      - 'datadog/**'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Validate JSON
        run: |
          jq . datadog/dashboard.json
          jq . datadog/monitors.json
      
      - name: Check Monitor Queries
        run: |
          # Add script to validate monitor queries via Datadog API
```

## Support

For issues or questions:
1. Check Datadog Agent logs
2. Review application metrics endpoint
3. Consult Datadog documentation
4. Contact the backend team