/*
 * SPDX-FileCopyrightText: Copyright 2026 LG Electronics Inc.
 * SPDX-License-Identifier: MIT
 */

//! Proto-generated gRPC modules.
//!
//! timpani-n is a **gRPC client only** — it connects to Timpani-O's
//! `NodeService` on startup and uses three RPCs:
//!
//!   • `GetSchedInfo`  — pull the task schedule assigned to this node.
//!   • `SyncTimer`     — barrier: block until all nodes are ready to start.
//!   • `ReportDMiss`   — report a deadline miss to Timpani-O.
//!
//! The proto definition lives in `proto/node_service.proto`

pub mod schedinfo_v1 {
    tonic::include_proto!("schedinfo.v1");
}
