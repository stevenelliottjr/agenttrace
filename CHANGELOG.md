# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure and specification
- Core data models (Span, Trace, Metrics, Alert)
- TimescaleDB schema with hypertables and continuous aggregates
- CLI skeleton with clap argument parsing
- Configuration system
- Error handling framework

### Coming Soon
- Collector core implementation
- Python SDK
- REST API
- Web dashboard
- TUI dashboard

## [0.1.0] - TBD

Initial release.

### Added
- Rust collector with gRPC and UDP ingestion
- Python SDK with auto-instrumentation for Anthropic and OpenAI
- FastAPI REST API
- React dashboard with trace waterfall visualization
- CLI for managing the stack
- Docker Compose setup for local development
