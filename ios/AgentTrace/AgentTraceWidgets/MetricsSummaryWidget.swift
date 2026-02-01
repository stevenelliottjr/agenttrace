import WidgetKit
import SwiftUI

// MARK: - Widget Entry

struct MetricsEntry: TimelineEntry {
    let date: Date
    let totalCost: Double
    let totalTokens: Int
    let avgLatencyMs: Double
    let errorRate: Double
    let activeTraces: Int
    let isPlaceholder: Bool

    static var placeholder: MetricsEntry {
        MetricsEntry(
            date: Date(),
            totalCost: 12.34,
            totalTokens: 125000,
            avgLatencyMs: 234,
            errorRate: 0.02,
            activeTraces: 3,
            isPlaceholder: true
        )
    }
}

// MARK: - Timeline Provider

struct MetricsTimelineProvider: TimelineProvider {
    func placeholder(in context: Context) -> MetricsEntry {
        .placeholder
    }

    func getSnapshot(in context: Context, completion: @escaping (MetricsEntry) -> Void) {
        if context.isPreview {
            completion(.placeholder)
        } else {
            Task {
                let entry = await fetchMetrics()
                completion(entry)
            }
        }
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<MetricsEntry>) -> Void) {
        Task {
            let entry = await fetchMetrics()
            // Refresh every 5 minutes
            let nextUpdate = Calendar.current.date(byAdding: .minute, value: 5, to: Date())!
            let timeline = Timeline(entries: [entry], policy: .after(nextUpdate))
            completion(timeline)
        }
    }

    private func fetchMetrics() async -> MetricsEntry {
        // Read server URL from App Group UserDefaults
        let defaults = UserDefaults(suiteName: "group.com.agenttrace.app")
        let serverURL = defaults?.string(forKey: "serverURL") ?? "http://localhost:8080"

        guard let url = URL(string: "\(serverURL)/api/v1/metrics/summary?range=24h") else {
            return .placeholder
        }

        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            let decoder = JSONDecoder()
            decoder.dateDecodingStrategy = .iso8601

            let summary = try decoder.decode(WidgetMetricsSummary.self, from: data)

            return MetricsEntry(
                date: Date(),
                totalCost: summary.totalCost,
                totalTokens: summary.totalTokens,
                avgLatencyMs: summary.avgLatencyMs,
                errorRate: summary.errorRate,
                activeTraces: summary.activeTraces,
                isPlaceholder: false
            )
        } catch {
            return .placeholder
        }
    }
}

// MARK: - Widget Response Model

struct WidgetMetricsSummary: Codable {
    let totalCost: Double
    let totalTokens: Int
    let avgLatencyMs: Double
    let errorRate: Double
    let activeTraces: Int

    enum CodingKeys: String, CodingKey {
        case totalCost = "total_cost"
        case totalTokens = "total_tokens"
        case avgLatencyMs = "avg_latency_ms"
        case errorRate = "error_rate"
        case activeTraces = "active_traces"
    }
}

// MARK: - Widget Views

struct SmallWidgetView: View {
    let entry: MetricsEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: "waveform.path.ecg")
                    .foregroundStyle(.blue)
                Text("AgentTrace")
                    .font(.caption)
                    .fontWeight(.semibold)
            }

            Spacer()

            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Image(systemName: "dollarsign.circle.fill")
                        .foregroundStyle(.green)
                        .font(.caption)
                    Text(formatCost(entry.totalCost))
                        .font(.title2)
                        .fontWeight(.bold)
                }

                HStack {
                    if entry.activeTraces > 0 {
                        Image(systemName: "bolt.fill")
                            .foregroundStyle(.yellow)
                            .font(.caption2)
                        Text("\(entry.activeTraces) active")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    } else {
                        Text("24h cost")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .containerBackground(.background, for: .widget)
    }

    private func formatCost(_ cost: Double) -> String {
        if cost < 1 {
            return String(format: "$%.2f", cost)
        } else if cost < 100 {
            return String(format: "$%.1f", cost)
        } else {
            return String(format: "$%.0f", cost)
        }
    }
}

struct MediumWidgetView: View {
    let entry: MetricsEntry

    var body: some View {
        HStack(spacing: 16) {
            // Left side - main metrics
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Image(systemName: "waveform.path.ecg")
                        .foregroundStyle(.blue)
                    Text("AgentTrace")
                        .font(.caption)
                        .fontWeight(.semibold)
                }

                Spacer()

                VStack(alignment: .leading, spacing: 2) {
                    Text(formatCost(entry.totalCost))
                        .font(.title)
                        .fontWeight(.bold)
                    Text("24h cost")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }

            Divider()

            // Right side - additional metrics
            VStack(spacing: 12) {
                MetricRow(
                    icon: "number.circle",
                    value: formatTokens(entry.totalTokens),
                    label: "Tokens"
                )

                MetricRow(
                    icon: "clock",
                    value: formatLatency(entry.avgLatencyMs),
                    label: "Avg Latency"
                )

                MetricRow(
                    icon: "exclamationmark.triangle",
                    value: formatErrorRate(entry.errorRate),
                    label: "Error Rate",
                    isError: entry.errorRate > 0.05
                )
            }
        }
        .containerBackground(.background, for: .widget)
    }

    private func formatCost(_ cost: Double) -> String {
        if cost < 1 {
            return String(format: "$%.2f", cost)
        } else {
            return String(format: "$%.1f", cost)
        }
    }

    private func formatTokens(_ count: Int) -> String {
        if count >= 1_000_000 {
            return String(format: "%.1fM", Double(count) / 1_000_000)
        } else if count >= 1_000 {
            return String(format: "%.0fK", Double(count) / 1_000)
        } else {
            return "\(count)"
        }
    }

    private func formatLatency(_ ms: Double) -> String {
        if ms < 1000 {
            return String(format: "%.0fms", ms)
        } else {
            return String(format: "%.1fs", ms / 1000)
        }
    }

    private func formatErrorRate(_ rate: Double) -> String {
        String(format: "%.1f%%", rate * 100)
    }
}

struct MetricRow: View {
    let icon: String
    let value: String
    let label: String
    var isError: Bool = false

    var body: some View {
        HStack {
            Image(systemName: icon)
                .foregroundStyle(isError ? .red : .secondary)
                .font(.caption)
                .frame(width: 16)

            VStack(alignment: .leading, spacing: 0) {
                Text(value)
                    .font(.caption)
                    .fontWeight(.semibold)
                    .foregroundStyle(isError ? .red : .primary)
                Text(label)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            Spacer()
        }
    }
}

struct LargeWidgetView: View {
    let entry: MetricsEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            // Header
            HStack {
                Image(systemName: "waveform.path.ecg")
                    .foregroundStyle(.blue)
                Text("AgentTrace")
                    .font(.headline)
                    .fontWeight(.semibold)

                Spacer()

                if entry.activeTraces > 0 {
                    HStack(spacing: 4) {
                        Circle()
                            .fill(.green)
                            .frame(width: 6, height: 6)
                        Text("\(entry.activeTraces) active")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }

            Divider()

            // Main metrics grid
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 16) {
                LargeMetricCard(
                    icon: "dollarsign.circle.fill",
                    iconColor: .green,
                    value: formatCost(entry.totalCost),
                    label: "24h Cost"
                )

                LargeMetricCard(
                    icon: "number.circle.fill",
                    iconColor: .blue,
                    value: formatTokens(entry.totalTokens),
                    label: "Tokens"
                )

                LargeMetricCard(
                    icon: "clock.fill",
                    iconColor: .orange,
                    value: formatLatency(entry.avgLatencyMs),
                    label: "Avg Latency"
                )

                LargeMetricCard(
                    icon: "exclamationmark.triangle.fill",
                    iconColor: entry.errorRate > 0.05 ? .red : .gray,
                    value: formatErrorRate(entry.errorRate),
                    label: "Error Rate"
                )
            }

            Spacer()

            // Footer
            HStack {
                Text("Last updated: \(entry.date, style: .time)")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                Spacer()
            }
        }
        .containerBackground(.background, for: .widget)
    }

    private func formatCost(_ cost: Double) -> String {
        if cost < 1 {
            return String(format: "$%.2f", cost)
        } else {
            return String(format: "$%.1f", cost)
        }
    }

    private func formatTokens(_ count: Int) -> String {
        if count >= 1_000_000 {
            return String(format: "%.1fM", Double(count) / 1_000_000)
        } else if count >= 1_000 {
            return String(format: "%.1fK", Double(count) / 1_000)
        } else {
            return "\(count)"
        }
    }

    private func formatLatency(_ ms: Double) -> String {
        if ms < 1000 {
            return String(format: "%.0fms", ms)
        } else {
            return String(format: "%.1fs", ms / 1000)
        }
    }

    private func formatErrorRate(_ rate: Double) -> String {
        String(format: "%.1f%%", rate * 100)
    }
}

struct LargeMetricCard: View {
    let icon: String
    let iconColor: Color
    let value: String
    let label: String

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Image(systemName: icon)
                .foregroundStyle(iconColor)
                .font(.title3)

            Text(value)
                .font(.title2)
                .fontWeight(.bold)

            Text(label)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

// MARK: - Widget Definition

struct MetricsSummaryWidget: Widget {
    let kind: String = "MetricsSummaryWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: MetricsTimelineProvider()) { entry in
            switch entry {
            case let e where WidgetFamily.systemSmall == .systemSmall:
                SmallWidgetView(entry: e)
            case let e where WidgetFamily.systemMedium == .systemMedium:
                MediumWidgetView(entry: e)
            default:
                LargeWidgetView(entry: entry)
            }
        }
        .configurationDisplayName("AgentTrace Metrics")
        .description("View your AI agent metrics at a glance")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
    }
}

// MARK: - Widget Bundle

@main
struct AgentTraceWidgetBundle: WidgetBundle {
    var body: some Widget {
        MetricsSummaryWidget()
    }
}

// MARK: - Previews

#Preview("Small", as: .systemSmall) {
    MetricsSummaryWidget()
} timeline: {
    MetricsEntry.placeholder
}

#Preview("Medium", as: .systemMedium) {
    MetricsSummaryWidget()
} timeline: {
    MetricsEntry.placeholder
}

#Preview("Large", as: .systemLarge) {
    MetricsSummaryWidget()
} timeline: {
    MetricsEntry.placeholder
}
