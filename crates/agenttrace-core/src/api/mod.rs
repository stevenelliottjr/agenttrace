//! REST API implementation
//!
//! This module provides the HTTP API for AgentTrace.

pub mod handlers;
pub mod middleware;
pub mod routes;

// TODO: Implement API
// - GET /api/v1/traces - List traces
// - GET /api/v1/traces/:id - Get trace detail
// - POST /api/v1/spans - Ingest spans
// - GET /api/v1/metrics - Get metrics
// - GET /api/v1/costs - Get cost breakdown
// - GET/POST/DELETE /api/v1/alerts - Manage alerts
// - GET /api/v1/health - Health check
