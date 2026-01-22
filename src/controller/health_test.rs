//! Tests for health check functionality

#[cfg(test)]
mod tests {
    use super::super::health::*;

    #[test]
    fn test_health_check_result_synced() {
        let result = HealthCheckResult::synced(Some(12345));
        assert!(result.healthy);
        assert!(result.synced);
        assert_eq!(result.ledger_sequence, Some(12345));
    }

    #[test]
    fn test_health_check_result_syncing() {
        let result = HealthCheckResult::syncing("Syncing...".to_string(), Some(100));
        assert!(result.healthy);
        assert!(!result.synced);
        assert_eq!(result.ledger_sequence, Some(100));
    }

    #[test]
    fn test_health_check_result_unhealthy() {
        let result = HealthCheckResult::unhealthy("Connection failed".to_string());
        assert!(!result.healthy);
        assert!(!result.synced);
        assert_eq!(result.ledger_sequence, None);
    }

    #[test]
    fn test_health_check_result_pending() {
        let result = HealthCheckResult::pending("Pod not ready".to_string());
        assert!(!result.healthy);
        assert!(!result.synced);
        assert_eq!(result.ledger_sequence, None);
    }
}
