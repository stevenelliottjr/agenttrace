import Foundation

/// Severity level for alerts
enum AlertSeverity: String, Codable, CaseIterable {
    case critical = "critical"
    case warning = "warning"
    case info = "info"

    var displayName: String {
        rawValue.capitalized
    }

    var iconName: String {
        switch self {
        case .critical: return "exclamationmark.octagon.fill"
        case .warning: return "exclamationmark.triangle.fill"
        case .info: return "info.circle.fill"
        }
    }

    var colorName: String {
        switch self {
        case .critical: return "red"
        case .warning: return "orange"
        case .info: return "blue"
        }
    }
}

/// Type of alert condition
enum AlertConditionType: String, Codable, CaseIterable {
    case costThreshold = "cost_threshold"
    case latencyThreshold = "latency_threshold"
    case errorRate = "error_rate"
    case tokenBudget = "token_budget"
    case customMetric = "custom_metric"

    var displayName: String {
        switch self {
        case .costThreshold: return "Cost Threshold"
        case .latencyThreshold: return "Latency Threshold"
        case .errorRate: return "Error Rate"
        case .tokenBudget: return "Token Budget"
        case .customMetric: return "Custom Metric"
        }
    }

    var iconName: String {
        switch self {
        case .costThreshold: return "dollarsign.circle"
        case .latencyThreshold: return "clock"
        case .errorRate: return "xmark.circle"
        case .tokenBudget: return "number.circle"
        case .customMetric: return "chart.line.uptrend.xyaxis"
        }
    }
}

/// Represents an alert rule configuration
struct AlertRule: Identifiable, Codable, Hashable {
    let id: String
    let name: String
    let description: String?
    let conditionType: AlertConditionType
    let threshold: Double
    let severity: AlertSeverity
    let isEnabled: Bool
    let createdAt: Date
    let updatedAt: Date
    let filters: AlertFilters?

    enum CodingKeys: String, CodingKey {
        case id = "rule_id"
        case name
        case description
        case conditionType = "condition_type"
        case threshold
        case severity
        case isEnabled = "is_enabled"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
        case filters
    }

    var formattedThreshold: String {
        switch conditionType {
        case .costThreshold:
            return String(format: "$%.2f", threshold)
        case .latencyThreshold:
            return String(format: "%.0f ms", threshold)
        case .errorRate:
            return String(format: "%.1f%%", threshold * 100)
        case .tokenBudget:
            return String(format: "%.0f tokens", threshold)
        case .customMetric:
            return String(format: "%.2f", threshold)
        }
    }
}

/// Filters for alert rules
struct AlertFilters: Codable, Hashable {
    let services: [String]?
    let models: [String]?
    let spanTypes: [SpanType]?

    enum CodingKeys: String, CodingKey {
        case services
        case models
        case spanTypes = "span_types"
    }
}

/// Status of an alert event
enum AlertEventStatus: String, Codable {
    case triggered = "triggered"
    case acknowledged = "acknowledged"
    case resolved = "resolved"
}

/// Represents a triggered alert event
struct AlertEvent: Identifiable, Codable, Hashable {
    let id: String
    let ruleId: String
    let ruleName: String
    let severity: AlertSeverity
    let status: AlertEventStatus
    let triggeredAt: Date
    let acknowledgedAt: Date?
    let resolvedAt: Date?
    let acknowledgedBy: String?
    let currentValue: Double
    let threshold: Double
    let message: String
    let traceId: String?
    let spanId: String?

    enum CodingKeys: String, CodingKey {
        case id = "event_id"
        case ruleId = "rule_id"
        case ruleName = "rule_name"
        case severity
        case status
        case triggeredAt = "triggered_at"
        case acknowledgedAt = "acknowledged_at"
        case resolvedAt = "resolved_at"
        case acknowledgedBy = "acknowledged_by"
        case currentValue = "current_value"
        case threshold
        case message
        case traceId = "trace_id"
        case spanId = "span_id"
    }

    var timeSinceTriggered: String {
        let interval = Date().timeIntervalSince(triggeredAt)
        if interval < 60 {
            return "Just now"
        } else if interval < 3600 {
            let minutes = Int(interval / 60)
            return "\(minutes)m ago"
        } else if interval < 86400 {
            let hours = Int(interval / 3600)
            return "\(hours)h ago"
        } else {
            let days = Int(interval / 86400)
            return "\(days)d ago"
        }
    }
}
