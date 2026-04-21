/*
 * SPDX-FileCopyrightText: Copyright 2026 LG Electronics Inc.
 * SPDX-License-Identifier: MIT
 */

//! Integration tests for timpani-n
//!
//! Tests marked `#[ignore]` require a live Timpani-O instance.
//! Run them with: `cargo test -p timpani-n -- --ignored`

use timpani_n::config::Config;
use timpani_n::context::Context;
use timpani_n::run_app;

#[tokio::test]
#[ignore = "requires live Timpani-O on 127.0.0.1:50054"]
async fn test_run_app_integration() {
    let config = Config::default();
    assert!(run_app(config).await.is_ok());
}

#[tokio::test]
#[ignore = "requires live Timpani-O on 127.0.0.1:50054"]
async fn test_full_lifecycle_with_various_configs() {
    // Test with CPU affinity
    let config = Config {
        cpu: 2,
        ..Default::default()
    };
    assert!(run_app(config).await.is_ok());

    // Test with priority
    let config = Config {
        prio: 50,
        ..Default::default()
    };
    assert!(run_app(config).await.is_ok());
}

#[test]
fn test_context_lifecycle() {
    let config = Config::default();
    let mut ctx = Context::new(config);

    // Initialize
    assert!(ctx.initialize().is_ok());

    // Cleanup
    ctx.cleanup();
}

#[test]
fn test_multiple_context_instances() {
    // Test creating multiple context instances
    for i in 0..5 {
        let config = Config {
            node_id: format!("node-{}", i),
            ..Default::default()
        };
        let mut ctx = Context::new(config);
        assert!(ctx.initialize().is_ok());
        ctx.cleanup();
    }
}
