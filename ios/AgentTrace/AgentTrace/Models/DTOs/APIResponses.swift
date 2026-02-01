import Foundation

/// Generic API error response
struct APIErrorResponse: Codable {
    let error: String
    let message: String?
    let code: String?
}

/// Health check response
struct HealthCheckResponse: Codable {
    let status: String
    let version: String?
    let uptime: Int?
    let database: DatabaseStatus?

    struct DatabaseStatus: Codable {
        let connected: Bool
        let latencyMs: Double?

        enum CodingKeys: String, CodingKey {
            case connected
            case latencyMs = "latency_ms"
        }
    }
}

/// Trace list response
struct TraceListResponse: Codable {
    let data: [TraceSummaryDTO]
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

/// Trace summary DTO matching API response format
struct TraceSummaryDTO: Codable {
    let traceId: String
    let name: String?
    let status: String
    let startTime: String
    let endTime: String?
    let durationUs: Int64?
    let spanCount: Int
    let errorCount: Int
    let totalTokens: Int
    let totalCost: Double

    enum CodingKeys: String, CodingKey {
        case traceId = "trace_id"
        case name
        case status
        case startTime = "start_time"
        case endTime = "end_time"
        case durationUs = "duration_us"
        case spanCount = "span_count"
        case errorCount = "error_count"
        case totalTokens = "total_tokens"
        case totalCost = "total_cost"
    }

    func toDomain() -> TraceSummary? {
        guard let status = TraceStatus(rawValue: status),
              let startTime = parseDate(startTime) else {
            return nil
        }

        return TraceSummary(
            id: traceId,
            name: name,
            status: status,
            startTime: startTime,
            durationUs: durationUs,
            spanCount: spanCount,
            errorCount: errorCount,
            totalTokens: totalTokens,
            totalCost: totalCost
        )
    }
}

/// Full trace detail DTO
struct TraceDetailDTO: Codable {
    let traceId: String
    let name: String?
    let status: String
    let startTime: String
    let endTime: String?
    let durationUs: Int64?
    let spanCount: Int
    let errorCount: Int
    let totalTokens: Int
    let totalCost: Double
    let spans: [SpanDTO]?
    let metadata: [String: AnyCodable]?

    enum CodingKeys: String, CodingKey {
        case traceId = "trace_id"
        case name
        case status
        case startTime = "start_time"
        case endTime = "end_time"
        case durationUs = "duration_us"
        case spanCount = "span_count"
        case errorCount = "error_count"
        case totalTokens = "total_tokens"
        case totalCost = "total_cost"
        case spans
        case metadata
    }
}

/// Span DTO matching API response format
struct SpanDTO: Codable {
    let spanId: String
    let traceId: String
    let parentSpanId: String?
    let name: String
    let spanType: String
    let status: String
    let startTime: String
    let endTime: String?
    let durationUs: Int64?
    let model: String?
    let provider: String?
    let tokenUsage: TokenUsageDTO?
    let cost: CostDTO?
    let input: String?
    let output: String?
    let errorMessage: String?
    let attributes: [String: AnyCodable]?

    enum CodingKeys: String, CodingKey {
        case spanId = "span_id"
        case traceId = "trace_id"
        case parentSpanId = "parent_span_id"
        case name
        case spanType = "span_type"
        case status
        case startTime = "start_time"
        case endTime = "end_time"
        case durationUs = "duration_us"
        case model
        case provider
        case tokenUsage = "token_usage"
        case cost
        case input
        case output
        case errorMessage = "error_message"
        case attributes
    }
}

struct TokenUsageDTO: Codable {
    let promptTokens: Int
    let completionTokens: Int
    let totalTokens: Int

    enum CodingKeys: String, CodingKey {
        case promptTokens = "prompt_tokens"
        case completionTokens = "completion_tokens"
        case totalTokens = "total_tokens"
    }
}

struct CostDTO: Codable {
    let inputCost: Double
    let outputCost: Double
    let totalCost: Double

    enum CodingKeys: String, CodingKey {
        case inputCost = "input_cost"
        case outputCost = "output_cost"
        case totalCost = "total_cost"
    }
}

/// Metrics summary DTO
struct MetricsSummaryDTO: Codable {
    let totalTraces: Int
    let totalSpans: Int
    let totalTokens: Int
    let totalCost: Double
    let avgLatencyMs: Double
    let errorRate: Double
    let activeTraces: Int
    let period: MetricsPeriodDTO

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
}

struct MetricsPeriodDTO: Codable {
    let start: String
    let end: String
    let granularity: String
}

/// Alert event DTO
struct AlertEventDTO: Codable {
    let eventId: String
    let ruleId: String
    let ruleName: String
    let severity: String
    let status: String
    let triggeredAt: String
    let acknowledgedAt: String?
    let resolvedAt: String?
    let acknowledgedBy: String?
    let currentValue: Double
    let threshold: Double
    let message: String
    let traceId: String?
    let spanId: String?

    enum CodingKeys: String, CodingKey {
        case eventId = "event_id"
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
}

// MARK: - Helper Types

/// Type-erased Codable for handling dynamic JSON
struct AnyCodable: Codable {
    let value: Any

    init(_ value: Any) {
        self.value = value
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        if container.decodeNil() {
            self.value = NSNull()
        } else if let bool = try? container.decode(Bool.self) {
            self.value = bool
        } else if let int = try? container.decode(Int.self) {
            self.value = int
        } else if let double = try? container.decode(Double.self) {
            self.value = double
        } else if let string = try? container.decode(String.self) {
            self.value = string
        } else if let array = try? container.decode([AnyCodable].self) {
            self.value = array.map { $0.value }
        } else if let dict = try? container.decode([String: AnyCodable].self) {
            self.value = dict.mapValues { $0.value }
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unable to decode value")
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        switch value {
        case is NSNull:
            try container.encodeNil()
        case let bool as Bool:
            try container.encode(bool)
        case let int as Int:
            try container.encode(int)
        case let double as Double:
            try container.encode(double)
        case let string as String:
            try container.encode(string)
        case let array as [Any]:
            try container.encode(array.map { AnyCodable($0) })
        case let dict as [String: Any]:
            try container.encode(dict.mapValues { AnyCodable($0) })
        default:
            throw EncodingError.invalidValue(value, .init(codingPath: [], debugDescription: "Unable to encode value"))
        }
    }
}

// MARK: - Date Parsing Helper

private func parseDate(_ string: String) -> Date? {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    if let date = formatter.date(from: string) {
        return date
    }

    formatter.formatOptions = [.withInternetDateTime]
    return formatter.date(from: string)
}
