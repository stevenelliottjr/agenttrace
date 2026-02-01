import Foundation
import SwiftData

/// Cached trace for offline access
@Model
final class CachedTrace {
    @Attribute(.unique) var traceId: String
    var name: String?
    var status: String
    var startTime: Date
    var endTime: Date?
    var durationUs: Int64?
    var spanCount: Int
    var errorCount: Int
    var totalTokens: Int
    var totalCost: Double
    var cachedAt: Date

    @Relationship(deleteRule: .cascade)
    var spans: [CachedSpan]?

    init(from trace: Trace) {
        self.traceId = trace.id
        self.name = trace.name
        self.status = trace.status.rawValue
        self.startTime = trace.startTime
        self.endTime = trace.endTime
        self.durationUs = trace.durationUs
        self.spanCount = trace.spanCount
        self.errorCount = trace.errorCount
        self.totalTokens = trace.totalTokens
        self.totalCost = trace.totalCost
        self.cachedAt = Date()
        self.spans = trace.spans?.map { CachedSpan(from: $0) }
    }

    init(from summary: TraceSummary) {
        self.traceId = summary.id
        self.name = summary.name
        self.status = summary.status.rawValue
        self.startTime = summary.startTime
        self.endTime = nil
        self.durationUs = summary.durationUs
        self.spanCount = summary.spanCount
        self.errorCount = summary.errorCount
        self.totalTokens = summary.totalTokens
        self.totalCost = summary.totalCost
        self.cachedAt = Date()
        self.spans = nil
    }

    func toTraceSummary() -> TraceSummary {
        TraceSummary(
            id: traceId,
            name: name,
            status: TraceStatus(rawValue: status) ?? .completed,
            startTime: startTime,
            durationUs: durationUs,
            spanCount: spanCount,
            errorCount: errorCount,
            totalTokens: totalTokens,
            totalCost: totalCost
        )
    }

    var isExpired: Bool {
        Date().timeIntervalSince(cachedAt) > 86400 // 24 hours
    }
}

/// Cached span for offline access
@Model
final class CachedSpan {
    @Attribute(.unique) var spanId: String
    var traceId: String
    var parentSpanId: String?
    var name: String
    var spanType: String
    var status: String
    var startTime: Date
    var endTime: Date?
    var durationUs: Int64?
    var model: String?
    var provider: String?
    var promptTokens: Int?
    var completionTokens: Int?
    var totalTokens: Int?
    var totalCost: Double?
    var input: String?
    var output: String?
    var errorMessage: String?
    var cachedAt: Date

    init(from span: Span) {
        self.spanId = span.id
        self.traceId = span.traceId
        self.parentSpanId = span.parentSpanId
        self.name = span.name
        self.spanType = span.spanType.rawValue
        self.status = span.status.rawValue
        self.startTime = span.startTime
        self.endTime = span.endTime
        self.durationUs = span.durationUs
        self.model = span.model
        self.provider = span.provider
        self.promptTokens = span.tokenUsage?.promptTokens
        self.completionTokens = span.tokenUsage?.completionTokens
        self.totalTokens = span.tokenUsage?.totalTokens
        self.totalCost = span.cost?.totalCost
        self.input = span.input
        self.output = span.output
        self.errorMessage = span.errorMessage
        self.cachedAt = Date()
    }

    func toSpan() -> Span {
        let tokenUsage: TokenUsage?
        if let prompt = promptTokens, let completion = completionTokens, let total = totalTokens {
            tokenUsage = TokenUsage(promptTokens: prompt, completionTokens: completion, totalTokens: total)
        } else {
            tokenUsage = nil
        }

        let cost: CostInfo?
        if let total = totalCost {
            cost = CostInfo(inputCost: 0, outputCost: 0, totalCost: total)
        } else {
            cost = nil
        }

        return Span(
            id: spanId,
            traceId: traceId,
            parentSpanId: parentSpanId,
            name: name,
            spanType: SpanType(rawValue: spanType) ?? .unknown,
            status: SpanStatus(rawValue: status) ?? .unset,
            startTime: startTime,
            endTime: endTime,
            durationUs: durationUs,
            model: model,
            provider: provider,
            tokenUsage: tokenUsage,
            cost: cost,
            input: input,
            output: output,
            errorMessage: errorMessage,
            attributes: nil
        )
    }
}

/// Cached metrics summary
@Model
final class CachedMetricsSummary {
    var totalTraces: Int
    var totalSpans: Int
    var totalTokens: Int
    var totalCost: Double
    var avgLatencyMs: Double
    var errorRate: Double
    var activeTraces: Int
    var timeRange: String
    var cachedAt: Date

    init(from summary: MetricsSummary, timeRange: TimeRange) {
        self.totalTraces = summary.totalTraces
        self.totalSpans = summary.totalSpans
        self.totalTokens = summary.totalTokens
        self.totalCost = summary.totalCost
        self.avgLatencyMs = summary.avgLatencyMs
        self.errorRate = summary.errorRate
        self.activeTraces = summary.activeTraces
        self.timeRange = timeRange.rawValue
        self.cachedAt = Date()
    }

    var isExpired: Bool {
        // Expire after 5 minutes for metrics
        Date().timeIntervalSince(cachedAt) > 300
    }
}
