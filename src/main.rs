//! Stellar-K8s Operator Entry Point
//!
//! Starts the Kubernetes controller and optional REST API server.

use std::sync::Arc;

use stellar_k8s::{controller, Error};
use tracing::{info, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use kube_leader_election::{LeaseLock, LeaseLockParams};
use tokio::sync::watch;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    info!("Starting Stellar-K8s Operator v{}", env!("CARGO_PKG_VERSION"));

    // Initialize Kubernetes client
    let client = kube::Client::try_default()
        .await
        .map_err(|e| Error::KubeError(e))?;

    info!("Connected to Kubernetes cluster");

    // Leader election configuration
    let namespace = std::env::var("POD_NAMESPACE").unwrap_or_else(|_| "default".to_string());
    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| {
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown-host".to_string())
    });

    info!("Leader election using holder ID: {}", hostname);

    let lease_name = "stellar-operator-leader";
    let lock = LeaseLock::new(
        client.clone(),
        &namespace,
        LeaseLockParams {
            lease_name: lease_name.into(),
            holder_id: hostname.clone(),
            lease_ttl: std::time::Duration::from_secs(15),
        },
    );

    // Create shared controller state
    let state = Arc::new(controller::ControllerState { client: client.clone() });

    // Start the REST API server (always running if feature enabled)
    #[cfg(feature = "rest-api")]
    {
        let api_state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = stellar_k8s::rest_api::run_server(api_state).await {
                tracing::error!("REST API server error: {:?}", e);
            }
        });
    }

    let (tx, mut leadership) = watch::channel(false);

    // Spawn leader election loop
    tokio::spawn(async move {
        loop {
            match lock.try_acquire_or_renew().await {
                Ok(res) => {
                    let _ = tx.send_replace(res.acquired_lease);
                }
                Err(e) => {
                    tracing::warn!("Leader election error: {:?}", e);
                    // On error, we don't necessarily lose leadership immediately, 
                    // but if it persists and TTL expires, we strictly aren't leader.
                    // For safety, we could downgrade, but let's stick to simple update for now.
                    // Actually, if we can't talk to API, we should probably assume strictly unsafe to reconcile.
                    let _ = tx.send_replace(false);
                }
            }
            // Renew every 5 seconds (TTL is 15s)
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    // Handle leadership changes
    loop {
        // Check current leadership state
        let is_leader = *leadership.borrow_and_update();

        if is_leader {
            info!("Leadership acquired, starting controller");
            
            // Wait for leadership loss or controller exit
            let mut controller_handle = tokio::spawn(controller::run_controller(state.clone()));
            loop {
                tokio::select! {
                    res = leadership.changed() => {
                        if res.is_err() || !*leadership.borrow() {
                            info!("Leadership lost, stopping controller");
                            controller_handle.abort();
                            break;
                        }
                    }
                    res = &mut controller_handle => {
                        match res {
                            Ok(Ok(_)) => info!("Controller exited normally"),
                            Ok(Err(e)) => tracing::error!("Controller error: {:?}", e),
                            Err(e) if e.is_cancelled() => info!("Controller aborted"),
                            Err(e) => tracing::error!("Controller task join error: {:?}", e),
                        }
                        
                        // Prevent tight loop if controller crashes immediately
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        break;
                    }
                }
            }
        } else {
            info!("In standby mode, waiting for leadership...");
            if leadership.changed().await.is_err() {
                tracing::error!("Leadership channel closed");
                break;
            }
        }
    }
    Ok(())
}
