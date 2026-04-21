/*
 * SPDX-FileCopyrightText: Copyright 2026 LG Electronics Inc.
 * SPDX-License-Identifier: MIT
 */

//! Shutdown signal handling. See DEVELOPER_NOTES.md D-N-005.

use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::error::{TimpaniError, TimpaniResult};

/// Install SIGINT/SIGTERM handlers. Returns a token that is cancelled on either signal.
/// Must be called inside a Tokio runtime.
pub fn setup_shutdown_handlers() -> TimpaniResult<CancellationToken> {
    // Register SIGTERM before spawning to avoid a missed-signal window.
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|e| {
            error!("Failed to register SIGTERM handler: {}", e);
            TimpaniError::Signal
        })?;

    let token = CancellationToken::new();
    let cancel = token.clone();

    tokio::spawn(async move {
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                match result {
                    Ok(()) => info!("SIGINT received — initiating graceful shutdown"),
                    Err(e) => error!("Error waiting for SIGINT: {}", e),
                }
            }
            opt = sigterm.recv() => {
                if opt.is_some() {
                    info!("SIGTERM received — initiating graceful shutdown");
                } else {
                    error!("SIGTERM signal stream closed — forcing shutdown");
                }
            }
        }
        cancel.cancel();
    });

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_token_not_cancelled_on_creation() {
        let token = setup_shutdown_handlers().expect("should register signal handlers");
        assert!(!token.is_cancelled());
    }

    #[tokio::test]
    async fn test_token_cancel_propagates_to_clone() {
        let token = setup_shutdown_handlers().expect("should register signal handlers");
        let child = token.clone();
        token.cancel();
        assert!(child.is_cancelled());
    }

    #[tokio::test]
    async fn test_setup_returns_valid_token() {
        let result = setup_shutdown_handlers();
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_cancelled());
    }

    #[tokio::test]
    async fn test_token_can_be_waited_on() {
        let token = setup_shutdown_handlers().expect("should register signal handlers");
        let token_clone = token.clone();

        // Spawn a task that cancels after a delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            token.cancel();
        });

        // Wait for cancellation with timeout
        let wait_result =
            tokio::time::timeout(Duration::from_secs(1), token_clone.cancelled()).await;
        assert!(
            wait_result.is_ok(),
            "Token should be cancelled within timeout"
        );
    }

    #[tokio::test]
    async fn test_multiple_clones_all_cancelled() {
        let token = setup_shutdown_handlers().expect("should register signal handlers");
        let clone1 = token.clone();
        let clone2 = token.clone();
        let clone3 = token.clone();

        token.cancel();

        assert!(clone1.is_cancelled());
        assert!(clone2.is_cancelled());
        assert!(clone3.is_cancelled());
    }
}
