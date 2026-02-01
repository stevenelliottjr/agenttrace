import Foundation

/// Represents the status of a trace
enum TraceStatus: String, Codable {
    case running = "running"
    case completed = "completed"
    case error = "error"
}

/// Represents a complete trace with all its spans
struct Trace: Identifiable, Codable, Hashable {
    let id: String
    let name: String?
    let status: TraceStatus
    let startTime: Date
    let endTime: Date?
    let durationUs: Int64?
    let spanCount: Int
    let errorCount: Int
    let totalTokens: Int
    let totalCost: Double
    let rootSpan: Span?
    let spans: [Span]?
    let metadata: [String: AttributeValue]?

    enum CodingKeys: String, CodingKey {
        case id = "trace_id"
        case name
        case status
        case startTime = "start_time"
        case endTime = "end_time"
        case durationUs = "duration_us"
        case spanCount = "span_count"
        case errorCount = "error_count"
        case totalTokens = "total_tokens"
        case totalCost = "total_cost"
        case rootSpan = "root_span"
        case spans
        case metadata
    }

    var durationMs: Double? {
        guard let durationUs = durationUs else { return nil }
        return Double(durationUs) / 1000.0
    }

    var formattedDuration: String {
        guard let durationMs = durationMs else {
            return "Running..."
        }
        if durationMs < 1000 {
            return String(format: "%.1f ms", durationMs)
        } else if durationMs < 60000 {
            return String(format: "%.2f s", durationMs / 1000)
        } else {
            let minutes = Int(durationMs / 60000)
            let seconds = (durationMs.truncatingRemainder(dividingBy: 60000)) / 1000
            return String(format: "%d:%04.1f", minutes, seconds)
        }
    }

    var formattedCost: String {
        if totalCost < 0.01 {
            return String(format: "$%.4f", totalCost)
        } else {
            return String(format: "$%.2f", totalCost)
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

    var displayName: String {
        name ?? "Trace \(id.prefix(8))"
    }

    /// Build a tree structure from flat spans
    func buildSpanTree() -> [SpanNode] {
        guard let spans = spans else { return [] }

        var nodeMap: [String: SpanNode] = [:]
        var rootNodes: [SpanNode] = []

        // Create nodes for all spans
        for span in spans {
            nodeMap[span.id] = SpanNode(span: span)
        }

        // Build tree structure
        for span in spans {
            let node = nodeMap[span.id]!
            if let parentId = span.parentSpanId, let parentNode = nodeMap[parentId] {
                parentNode.children.append(node)
            } else {
                rootNodes.append(node)
            }
        }

        // Sort children by start time
        func sortChildren(_ node: SpanNode) {
            node.children.sort { $0.span.startTime < $1.span.startTime }
            node.children.forEach { sortChildren($0) }
        }

        rootNodes.sort { $0.span.startTime < $1.span.startTime }
        rootNodes.forEach { sortChildren($0) }

        return rootNodes
    }
}

/// Represents a node in the span tree
class SpanNode: Identifiable {
    let id: String
    let span: Span
    var children: [SpanNode] = []
    var depth: Int = 0

    init(span: Span) {
        self.id = span.id
        self.span = span
    }

    /// Flatten the tree for display, calculating depths
    func flatten(depth: Int = 0) -> [SpanNode] {
        self.depth = depth
        var result = [self]
        for child in children {
            result.append(contentsOf: child.flatten(depth: depth + 1))
        }
        return result
    }
}

/// Summary of a trace for list display
struct TraceSummary: Identifiable, Codable, Hashable {
    let id: String
    let name: String?
    let status: TraceStatus
    let startTime: Date
    let durationUs: Int64?
    let spanCount: Int
    let errorCount: Int
    let totalTokens: Int
    let totalCost: Double

    enum CodingKeys: String, CodingKey {
        case id = "trace_id"
        case name
        case status
        case startTime = "start_time"
        case durationUs = "duration_us"
        case spanCount = "span_count"
        case errorCount = "error_count"
        case totalTokens = "total_tokens"
        case totalCost = "total_cost"
    }

    var displayName: String {
        name ?? "Trace \(id.prefix(8))"
    }

    var formattedDuration: String {
        guard let durationUs = durationUs else {
            return "Running..."
        }
        let durationMs = Double(durationUs) / 1000.0
        if durationMs < 1000 {
            return String(format: "%.0f ms", durationMs)
        } else {
            return String(format: "%.1f s", durationMs / 1000)
        }
    }

    var formattedCost: String {
        if totalCost < 0.01 {
            return String(format: "$%.4f", totalCost)
        } else {
            return String(format: "$%.2f", totalCost)
        }
    }
}
