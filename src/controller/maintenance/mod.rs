//! Maintenance Window controller for Horizon DB maintenance tasks.
//!
//! Handles scheduling and coordination of VACUUM FULL and REINDEX operations.

pub mod controller;
pub mod bloat;
pub mod coordinator;

pub use controller::MaintenanceController;
pub use bloat::BloatDetector;
pub use coordinator::MaintenanceCoordinator;
