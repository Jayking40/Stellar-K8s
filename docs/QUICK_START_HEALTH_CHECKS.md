# Quick Start: Health Checks

## What's New?

The operator now automatically monitors Horizon and Soroban RPC nodes to ensure they're fully synced before marking them as `Ready`.

## Quick Example

```bash
# Deploy a Horizon node
kubectl apply -f examples/horizon-with-health-check.yaml

# Watch it sync
kubectl get stellarnodes -w

# You'll see:
# NAME         TYPE      NETWORK   REPLICAS   PHASE      AGE
# my-horizon   Horizon   Testnet   2          Creating   10s
# my-horizon   Horizon   Testnet   2          Syncing    30s
# my-horizon   Horizon   Testnet   2          Ready      5m
```

## Check Sync Status

```bash
# Current phase
kubectl get stellarnode my-horizon -o jsonpath='{.status.phase}'

# Current ledger
kubectl get stellarnode my-horizon -o jsonpath='{.status.ledgerSequence}'

# Detailed status
kubectl get stellarnode my-horizon -o yaml | grep -A 20 status:
```

## What Gets Checked?

### Horizon Nodes
- ✅ Pod is running and ready
- ✅ Health endpoint responds at `http://<pod-ip>:8000/health`
- ✅ `core_synced` field is `true`
- ✅ Ledger lag is minimal

### Soroban RPC Nodes
- ✅ Pod is running and ready
- ✅ Health endpoint responds at `http://<pod-ip>:8000/health`
- ✅ Status is "healthy" or "ready"

### Validator Nodes
- ✅ Pod is running and ready
- ℹ️ No health endpoint check (validators don't have standard health APIs)

## Phases Explained

| Phase | Meaning | Next Step |
|-------|---------|-----------|
| **Pending** | Resources being created | Wait for pod to start |
| **Creating** | Pod starting up | Wait for health endpoint |
| **Syncing** | Ingesting historical data | Wait for sync to complete |
| **Ready** | Fully synced and operational | Node is ready to use! |
| **Suspended** | Intentionally scaled to 0 | Resume by setting `suspended: false` |

## Troubleshooting

### Stuck in "Syncing"?

Check the sync progress:
```bash
kubectl get stellarnode my-horizon -o jsonpath='{.status.message}'
# Output: "Horizon is syncing: at ledger 49500000, core at 50000000 (lag: 500000)"
```

Large lag is normal for new nodes. Wait for it to catch up.

### Stuck in "Creating"?

Check pod status:
```bash
kubectl get pods -l app.kubernetes.io/instance=my-horizon
kubectl logs <pod-name>
```

### Want More Details?

See the full documentation:
- [Health Checks Guide](./health-checks.md)
- [Testing Guide](./testing-health-checks.md)

## That's It!

The operator handles everything automatically. Just deploy your nodes and monitor the status. 🚀
