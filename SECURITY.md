# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.x.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in AgentTrace, please report it responsibly.

### How to Report

1. **Do NOT** open a public GitHub issue for security vulnerabilities
2. Email security concerns to: [INSERT EMAIL] (or use GitHub's private vulnerability reporting)
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: We will acknowledge receipt within 48 hours
- **Assessment**: We will assess the vulnerability and determine its severity
- **Updates**: We will keep you informed of our progress
- **Resolution**: We aim to resolve critical vulnerabilities within 7 days
- **Credit**: We will credit you in the release notes (unless you prefer anonymity)

## Security Considerations

### Data Privacy

AgentTrace is designed with privacy in mind:

- **Local-first**: All data stays on your machine by default
- **No telemetry**: We do not collect usage data or send data to external servers
- **Sensitive data**: Agent traces may contain prompts, code, and other sensitive information. AgentTrace stores this locally in your TimescaleDB instance.

### Best Practices

When using AgentTrace:

1. **Database security**: Secure your TimescaleDB instance with strong passwords
2. **Network exposure**: Don't expose the collector or API ports to the public internet without authentication
3. **Prompt content**: Be aware that prompts and responses are stored in traces
4. **Access control**: In production, implement proper authentication for the dashboard

### Threat Model

AgentTrace assumes:
- The local machine is trusted
- Network traffic between components is on localhost or a trusted network
- Users have appropriate access to view trace data

AgentTrace does NOT currently provide:
- Built-in authentication (planned for future releases)
- Encryption at rest (relies on database-level encryption)
- Multi-tenancy isolation
