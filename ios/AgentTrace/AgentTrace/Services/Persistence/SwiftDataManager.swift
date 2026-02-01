import Foundation
import SwiftData

/// Manages SwiftData persistence for offline caching
@MainActor
final class SwiftDataManager {
    static let shared = SwiftDataManager()

    private let modelContainer: ModelContainer

    private init() {
        let schema = Schema([
            CachedTrace.self,
            CachedSpan.self,
            CachedMetricsSummary.self
        ])

        let configuration = ModelConfiguration(
            schema: schema,
            isStoredInMemoryOnly: false,
            groupContainer: .identifier("group.com.agenttrace.app")
        )

        do {
            modelContainer = try ModelContainer(for: schema, configurations: [configuration])
        } catch {
            fatalError("Failed to create ModelContainer: \(error)")
        }
    }

    var context: ModelContext {
        modelContainer.mainContext
    }

    // MARK: - Trace Caching

    func cacheTraces(_ traces: [TraceSummary]) throws {
        for trace in traces {
            // Check if already cached
            let predicate = #Predicate<CachedTrace> { $0.traceId == trace.id }
            let descriptor = FetchDescriptor<CachedTrace>(predicate: predicate)

            if let existing = try context.fetch(descriptor).first {
                // Update existing
                existing.name = trace.name
                existing.status = trace.status.rawValue
                existing.durationUs = trace.durationUs
                existing.spanCount = trace.spanCount
                existing.errorCount = trace.errorCount
                existing.totalTokens = trace.totalTokens
                existing.totalCost = trace.totalCost
                existing.cachedAt = Date()
            } else {
                // Insert new
                let cached = CachedTrace(from: trace)
                context.insert(cached)
            }
        }

        try context.save()
    }

    func cacheTrace(_ trace: Trace) throws {
        let predicate = #Predicate<CachedTrace> { $0.traceId == trace.id }
        let descriptor = FetchDescriptor<CachedTrace>(predicate: predicate)

        if let existing = try context.fetch(descriptor).first {
            context.delete(existing)
        }

        let cached = CachedTrace(from: trace)
        context.insert(cached)
        try context.save()
    }

    func getCachedTraces(limit: Int = 50) throws -> [TraceSummary] {
        var descriptor = FetchDescriptor<CachedTrace>(
            sortBy: [SortDescriptor(\.startTime, order: .reverse)]
        )
        descriptor.fetchLimit = limit

        let cached = try context.fetch(descriptor)
        return cached.compactMap { $0.toTraceSummary() }
    }

    func getCachedTrace(traceId: String) throws -> Trace? {
        let predicate = #Predicate<CachedTrace> { $0.traceId == traceId }
        let descriptor = FetchDescriptor<CachedTrace>(predicate: predicate)

        guard let cached = try context.fetch(descriptor).first else {
            return nil
        }

        let spans = cached.spans?.map { $0.toSpan() } ?? []

        return Trace(
            id: cached.traceId,
            name: cached.name,
            status: TraceStatus(rawValue: cached.status) ?? .completed,
            startTime: cached.startTime,
            endTime: cached.endTime,
            durationUs: cached.durationUs,
            spanCount: cached.spanCount,
            errorCount: cached.errorCount,
            totalTokens: cached.totalTokens,
            totalCost: cached.totalCost,
            rootSpan: nil,
            spans: spans,
            metadata: nil
        )
    }

    // MARK: - Metrics Caching

    func cacheMetrics(_ summary: MetricsSummary, timeRange: TimeRange) throws {
        // Remove existing metrics for this time range
        let timeRangeValue = timeRange.rawValue
        let predicate = #Predicate<CachedMetricsSummary> { $0.timeRange == timeRangeValue }
        let descriptor = FetchDescriptor<CachedMetricsSummary>(predicate: predicate)

        for existing in try context.fetch(descriptor) {
            context.delete(existing)
        }

        let cached = CachedMetricsSummary(from: summary, timeRange: timeRange)
        context.insert(cached)
        try context.save()
    }

    func getCachedMetrics(timeRange: TimeRange) throws -> MetricsSummary? {
        let timeRangeValue = timeRange.rawValue
        let predicate = #Predicate<CachedMetricsSummary> { $0.timeRange == timeRangeValue }
        let descriptor = FetchDescriptor<CachedMetricsSummary>(predicate: predicate)

        guard let cached = try context.fetch(descriptor).first,
              !cached.isExpired else {
            return nil
        }

        return MetricsSummary(
            totalTraces: cached.totalTraces,
            totalSpans: cached.totalSpans,
            totalTokens: cached.totalTokens,
            totalCost: cached.totalCost,
            avgLatencyMs: cached.avgLatencyMs,
            errorRate: cached.errorRate,
            activeTraces: cached.activeTraces,
            period: MetricsPeriod(
                start: Date().addingTimeInterval(-86400),
                end: Date(),
                granularity: "15m"
            )
        )
    }

    // MARK: - Cache Cleanup

    func cleanupExpiredCache() throws {
        // Remove expired traces (older than 24 hours)
        let expirationDate = Date().addingTimeInterval(-86400)
        let tracePredicate = #Predicate<CachedTrace> { $0.cachedAt < expirationDate }
        let traceDescriptor = FetchDescriptor<CachedTrace>(predicate: tracePredicate)

        for trace in try context.fetch(traceDescriptor) {
            context.delete(trace)
        }

        // Remove expired metrics (older than 5 minutes)
        let metricsExpirationDate = Date().addingTimeInterval(-300)
        let metricsPredicate = #Predicate<CachedMetricsSummary> { $0.cachedAt < metricsExpirationDate }
        let metricsDescriptor = FetchDescriptor<CachedMetricsSummary>(predicate: metricsPredicate)

        for metrics in try context.fetch(metricsDescriptor) {
            context.delete(metrics)
        }

        try context.save()
    }

    func clearAllCache() throws {
        try context.delete(model: CachedTrace.self)
        try context.delete(model: CachedSpan.self)
        try context.delete(model: CachedMetricsSummary.self)
        try context.save()
    }
}
