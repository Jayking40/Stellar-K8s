# feat: Automated Horizon DB Maintenance (Self-Healing State)

## Description
This PR implements automated database maintenance for Horizon nodes, addressing issue #252. It introduces a 'Maintenance Window' controller that monitors table bloat and automatically triggers `VACUUM FULL` and `REINDEX` operations during configured low-traffic periods.

## Key Features
- **Maintenance Window Controller**: New logic to schedule and manage maintenance tasks within user-defined time windows.
- **Automated Bloat Detection**: Logic to estimate table and index bloat using Postgres statistics.
- **Zero-Downtime Coordination**: Integration with the read-pool to ensure traffic is diverted from nodes undergoing maintenance.
- **Configurable Thresholds**: Customizable bloat percentage thresholds to trigger maintenance actions.

## CRD Changes
Added `dbMaintenanceConfig` to `StellarNodeSpec`:
```yaml
dbMaintenanceConfig:
  enabled: true
  windowStart: "02:00"
  windowDuration: "2h"
  bloatThresholdPercent: 30
  autoReindex: true
  readPoolCoordination: true
```

## Technical Details
- Added `sqlx` dependency for database interactions.
- Implemented `BloatDetector` for Postgres stats analysis.
- Implemented `MaintenanceCoordinator` for zero-downtime traffic management.
- Extended `StellarNodeSpec` with maintenance configuration fields.

## Testing
- Verified with `cargo check`.
- Unit tests for maintenance window logic.
- Mocked database interactions for bloat detection testing.

Resolves #252
