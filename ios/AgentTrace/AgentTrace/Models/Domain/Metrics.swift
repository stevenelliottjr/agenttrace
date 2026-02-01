import Foundation

/// Summary metrics for the dashboard
struct MetricsSummary: Codable {
    let totalTraces: Int
    let totalSpans: Int
    let totalTokens: Int
    let totalCost: Double
    let avgLatencyMs: Double
    let errorRate: Double
    let activeTraces: Int
    let period: MetricsPeriod

    enum CodingKeys: String, CodingKey {
        case totalTraces = "total_traces"
        case totalSpans = "total_spans"
        case totalTokens = "total_tokens"
        case totalCost = "total_cost"
        case avgLatencyMs = "avg_latency_ms"
        case errorRate = "error_rate"
        case activeTraces = "active_traces"
        case period
    }

    var formattedCost: String {
        if totalCost < 1 {
            return String(format: "$%.4f", totalCost)
        } else if totalCost < 100 {
            return String(format: "$%.2f", totalCost)
        } else {
            return String(format: "$%.0f", totalCost)
        }
    }

    var formattedTokens: String {
        if totalTokens >= 1_000_000 {
            return String(format: "%.1fM", Double(totalTokens) / 1_000_000)
        } else if totalTokens >= 1_000 {
            return String(format: "%.1fK", Double(totalTokens) / 1_000)
        } else {
            return "\(totalTokens)"
        }
    }

    var formattedLatency: String {
        if avgLatencyMs < 1000 {
            return String(format: "%.0f ms", avgLatencyMs)
        } else {
            return String(format: "%.1f s", avgLatencyMs / 1000)
        }
    }

    var formattedErrorRate: String {
        String(format: "%.1f%%", errorRate * 100)
    }
}

/// Time period for metrics
struct MetricsPeriod: Codable {
    let start: Date
    let end: Date
    let granularity: String
}

/// Cost breakdown by model or service
struct CostBreakdown: Codable, Identifiable {
    var id: String { name }
    let name: String
    let cost: Double
    let percentage: Double
    let tokenCount: Int

    enum CodingKeys: String, CodingKey {
        case name
        case cost
        case percentage
        case tokenCount = "token_count"
    }

    var formattedCost: String {
        if cost < 0.01 {
            return String(format: "$%.4f", cost)
        } else {
            return String(format: "$%.2f", cost)
        }
    }

    var formattedPercentage: String {
        String(format: "%.1f%%", percentage * 100)
    }
}

/// Latency metrics over time
struct LatencyMetrics: Codable, Identifiable {
    var id: Date { timestamp }
    let timestamp: Date
    let p50: Double
    let p95: Double
    let p99: Double
    let avg: Double
    let min: Double
    let max: Double
}

/// Token usage metrics over time
struct TokenUsageMetrics: Codable, Identifiable {
    var id: Date { timestamp }
    let timestamp: Date
    let promptTokens: Int
    let completionTokens: Int
    let totalTokens: Int

    enum CodingKeys: String, CodingKey {
        case timestamp
        case promptTokens = "prompt_tokens"
        case completionTokens = "completion_tokens"
        case totalTokens = "total_tokens"
    }
}

/// Error rate metrics over time
struct ErrorMetrics: Codable, Identifiable {
    var id: Date { timestamp }
    let timestamp: Date
    let totalCount: Int
    let errorCount: Int
    let errorRate: Double

    enum CodingKeys: String, CodingKey {
        case timestamp
        case totalCount = "total_count"
        case errorCount = "error_count"
        case errorRate = "error_rate"
    }

    var formattedErrorRate: String {
        String(format: "%.2f%%", errorRate * 100)
    }
}

/// Response wrapper for paginated list endpoints
struct PaginatedResponse<T: Codable>: Codable {
    let data: [T]
    let total: Int
    let page: Int
    let pageSize: Int
    let hasMore: Bool

    enum CodingKeys: String, CodingKey {
        case data
        case total
        case page
        case pageSize = "page_size"
        case hasMore = "has_more"
    }
}

/// Time range options for metrics queries
enum TimeRange: String, CaseIterable {
    case lastHour = "1h"
    case last6Hours = "6h"
    case last24Hours = "24h"
    case last7Days = "7d"
    case last30Days = "30d"

    var displayName: String {
        switch self {
        case .lastHour: return "Last Hour"
        case .last6Hours: return "Last 6 Hours"
        case .last24Hours: return "Last 24 Hours"
        case .last7Days: return "Last 7 Days"
        case .last30Days: return "Last 30 Days"
        }
    }

    var granularity: String {
        switch self {
        case .lastHour: return "1m"
        case .last6Hours: return "5m"
        case .last24Hours: return "15m"
        case .last7Days: return "1h"
        case .last30Days: return "6h"
        }
    }
}
