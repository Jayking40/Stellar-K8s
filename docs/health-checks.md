# Auto-Sync Health Checks

The Stellar-K8s operator implements automatic health checking for Horizon and Soroban RPC nodes to ensure they are fully synced before marking them as `Ready`.

## Overview

When a Horizon or Soroban RPC node is deployed, it needs time to:
1. Start up and initialize
2. Connect to Stellar Core
3. Ingest historical ledger data
4. Catch up to the current network state

The operator automatically monitors this process and only marks nodes as `Ready` when they are fully synced and operational.

## How It Works

### Health Check Flow

1. **Pod Ready Check**: First, the operator waits for the Kubernetes pod to be in a `Ready` state
2. **IP Assignment**: Verifies the pod has been assigned an IP address
3. **HTTP Health Query**: Queries the node's health endpoint at `http://<pod-ip>:8000/health`
4. **Sync Verification**: Parses the response to determine if the node is synced
5. **Status Update**: Updates the `StellarNode` status with the current phase and sync state

### Node Phases

The operator uses the following phases to track node state:

- **Pending**: Resources are being created, pod not ready yet
- **Creating**: Pod is ready but health endpoint is not responding
- **Syncing**: Node is healthy but still catching up with the network
- **Ready**: Node is fully synced and operational
- **Suspended**: Node is intentionally scaled to 0 replicas
- **Failed**: Node encountered an error during reconciliation

### Health Check Intervals

The operator adjusts its reconciliation frequency based on the node state:

- **Ready nodes**: Checked every 60 seconds
- **Syncing nodes**: Checked every 15 seconds
- **Failed nodes**: Retry based on error type (15-60 seconds)

## Horizon Health Checks

For Horizon nodes, the operator queries the `/health` endpoint and checks:

- `core_synced`: Whether Horizon is synced with Stellar Core
- `history_latest_ledger`: The latest ledger Horizon has ingested
- `core_latest_ledger`: The latest ledger from Stellar Core
- Calculates lag: `core_latest_ledger - history_latest_ledger`

### Example Horizon Health Response

```json
{
  "status": "healthy",
  "core_synced": true,
  "core_latest_ledger": 50000000,
  "history_latest_ledger": 50000000,
  "history_elder_ledger": 49000000
}
```

A Horizon node is marked as `Ready` when `core_synced` is `true`.

## Soroban RPC Health Checks

For Soroban RPC nodes, the operator queries the `/health` endpoint and checks:

- `status`: Overall health status ("healthy" or "ready")
- `ledger`: Current ledger sequence number

### Example Soroban Health Response

```json
{
  "status": "healthy",
  "ledger": 50000000
}
```

A Soroban RPC node is marked as `Ready` when `status` is "healthy" or "ready".

## Validator Nodes

Validator nodes (Stellar Core) don't have a standardized health endpoint. The operator considers them `Ready` when:

1. The StatefulSet is created
2. The pod is in `Ready` state
3. The container is running

For validators, you should monitor the Core logs and metrics separately to verify consensus participation.

## Status Conditions

The operator sets Kubernetes-standard conditions on the `StellarNode` status:

### Ready Condition

```yaml
conditions:
  - type: Ready
    status: "True"  # or "False"
    lastTransitionTime: "2026-01-21T10:30:00Z"
    reason: NodeSynced  # or NodeSyncing, NodeNotHealthy
    message: "Node is fully synced and operational"
```

### Progressing Condition

When a node is syncing, a `Progressing` condition is added:

```yaml
conditions:
  - type: Progressing
    status: "True"
    lastTransitionTime: "2026-01-21T10:25:00Z"
    reason: Syncing
    message: "Horizon is syncing: at ledger 49500000, core at 50000000 (lag: 500000)"
```

## Monitoring Sync Progress

You can monitor sync progress using `kubectl`:

```bash
# Watch node status
kubectl get stellarnodes -w

# Get detailed status
kubectl get stellarnode my-horizon -o yaml

# Check conditions
kubectl get stellarnode my-horizon -o jsonpath='{.status.conditions[?(@.type=="Ready")]}'

# View current ledger
kubectl get stellarnode my-horizon -o jsonpath='{.status.ledgerSequence}'
```

## Troubleshooting

### Node Stuck in "Syncing" Phase

If a node remains in `Syncing` for an extended period:

1. Check the lag between `history_latest_ledger` and `core_latest_ledger`
2. Verify network connectivity to Stellar Core
3. Check pod logs: `kubectl logs <pod-name>`
4. Ensure sufficient resources (CPU/memory) are allocated

### Health Endpoint Not Responding

If the operator reports "Cannot reach health endpoint":

1. Verify the pod is running: `kubectl get pods`
2. Check pod events: `kubectl describe pod <pod-name>`
3. Verify the service is created: `kubectl get svc`
4. Check if the container port is correct (8000 for Horizon/Soroban)

### False "Ready" Status

If you suspect a node is marked `Ready` but isn't actually synced:

1. Manually query the health endpoint:
   ```bash
   kubectl port-forward <pod-name> 8000:8000
   curl http://localhost:8000/health
   ```
2. Check the operator logs for health check results
3. Verify the health response format matches expectations

## Configuration

Health checks are automatically enabled for all Horizon and Soroban RPC nodes. No additional configuration is required.

The HTTP client uses a 5-second timeout for health queries. If the endpoint doesn't respond within this time, the node is considered not ready.

## Implementation Details

The health check implementation is in `src/controller/health.rs` and includes:

- `check_node_health()`: Main entry point for health checks
- `check_horizon_health()`: Horizon-specific health verification
- `check_soroban_health()`: Soroban RPC-specific health verification
- `HealthCheckResult`: Structured result with health, sync status, and ledger info

The reconciler integrates health checks in `src/controller/reconciler.rs` and updates the status with detailed sync information.
