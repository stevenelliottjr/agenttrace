import Foundation

/// Represents the type of a span in a trace
enum SpanType: String, Codable, CaseIterable {
    case agent = "agent"
    case llm = "llm"
    case tool = "tool"
    case retriever = "retriever"
    case chain = "chain"
    case embedding = "embedding"
    case unknown = "unknown"

    var displayName: String {
        switch self {
        case .agent: return "Agent"
        case .llm: return "LLM"
        case .tool: return "Tool"
        case .retriever: return "Retriever"
        case .chain: return "Chain"
        case .embedding: return "Embedding"
        case .unknown: return "Unknown"
        }
    }

    var iconName: String {
        switch self {
        case .agent: return "cpu"
        case .llm: return "brain"
        case .tool: return "wrench.and.screwdriver"
        case .retriever: return "magnifyingglass"
        case .chain: return "link"
        case .embedding: return "cube"
        case .unknown: return "questionmark.circle"
        }
    }

    var color: String {
        switch self {
        case .agent: return "purple"
        case .llm: return "blue"
        case .tool: return "green"
        case .retriever: return "orange"
        case .chain: return "cyan"
        case .embedding: return "pink"
        case .unknown: return "gray"
        }
    }
}

/// Represents the status of a span
enum SpanStatus: String, Codable {
    case ok = "ok"
    case error = "error"
    case unset = "unset"
}

/// Token usage information for LLM spans
struct TokenUsage: Codable, Hashable {
    let promptTokens: Int
    let completionTokens: Int
    let totalTokens: Int

    enum CodingKeys: String, CodingKey {
        case promptTokens = "prompt_tokens"
        case completionTokens = "completion_tokens"
        case totalTokens = "total_tokens"
    }
}

/// Cost information for a span
struct CostInfo: Codable, Hashable {
    let inputCost: Double
    let outputCost: Double
    let totalCost: Double

    enum CodingKeys: String, CodingKey {
        case inputCost = "input_cost"
        case outputCost = "output_cost"
        case totalCost = "total_cost"
    }
}

/// Represents a single span in a trace
struct Span: Identifiable, Codable, Hashable {
    let id: String
    let traceId: String
    let parentSpanId: String?
    let name: String
    let spanType: SpanType
    let status: SpanStatus
    let startTime: Date
    let endTime: Date?
    let durationUs: Int64?
    let model: String?
    let provider: String?
    let tokenUsage: TokenUsage?
    let cost: CostInfo?
    let input: String?
    let output: String?
    let errorMessage: String?
    let attributes: [String: AttributeValue]?

    enum CodingKeys: String, CodingKey {
        case id = "span_id"
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

    var durationMs: Double? {
        guard let durationUs = durationUs else { return nil }
        return Double(durationUs) / 1000.0
    }

    var formattedDuration: String {
        guard let durationMs = durationMs else { return "-" }
        if durationMs < 1000 {
            return String(format: "%.1f ms", durationMs)
        } else {
            return String(format: "%.2f s", durationMs / 1000)
        }
    }

    var formattedCost: String {
        guard let cost = cost else { return "-" }
        return String(format: "$%.4f", cost.totalCost)
    }
}

/// Represents a flexible attribute value that can be various types
enum AttributeValue: Codable, Hashable {
    case string(String)
    case int(Int)
    case double(Double)
    case bool(Bool)
    case array([AttributeValue])

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        if let stringValue = try? container.decode(String.self) {
            self = .string(stringValue)
        } else if let intValue = try? container.decode(Int.self) {
            self = .int(intValue)
        } else if let doubleValue = try? container.decode(Double.self) {
            self = .double(doubleValue)
        } else if let boolValue = try? container.decode(Bool.self) {
            self = .bool(boolValue)
        } else if let arrayValue = try? container.decode([AttributeValue].self) {
            self = .array(arrayValue)
        } else {
            self = .string("")
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .string(let value): try container.encode(value)
        case .int(let value): try container.encode(value)
        case .double(let value): try container.encode(value)
        case .bool(let value): try container.encode(value)
        case .array(let value): try container.encode(value)
        }
    }

    var displayValue: String {
        switch self {
        case .string(let value): return value
        case .int(let value): return String(value)
        case .double(let value): return String(format: "%.4f", value)
        case .bool(let value): return value ? "true" : "false"
        case .array(let value): return "[\(value.map { $0.displayValue }.joined(separator: ", "))]"
        }
    }
}
