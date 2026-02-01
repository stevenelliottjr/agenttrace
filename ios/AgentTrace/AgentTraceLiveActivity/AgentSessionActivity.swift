import ActivityKit
import SwiftUI
import WidgetKit

// MARK: - Live Activity Attributes

struct AgentSessionAttributes: ActivityAttributes {
    public struct ContentState: Codable, Hashable {
        var traceName: String
        var spanCount: Int
        var tokenCount: Int
        var cost: Double
        var elapsedSeconds: Int
        var currentSpanName: String?
        var status: SessionStatus

        enum SessionStatus: String, Codable {
            case running
            case completed
            case error
        }
    }

    var traceId: String
    var startTime: Date
}

// MARK: - Live Activity Views

struct AgentSessionLiveActivity: Widget {
    var body: some WidgetConfiguration {
        ActivityConfiguration(for: AgentSessionAttributes.self) { context in
            // Lock screen / banner presentation
            LockScreenView(context: context)
        } dynamicIsland: { context in
            DynamicIsland {
                // Expanded regions
                DynamicIslandExpandedRegion(.leading) {
                    HStack {
                        Image(systemName: statusIcon(context.state.status))
                            .foregroundStyle(statusColor(context.state.status))
                        VStack(alignment: .leading) {
                            Text(context.state.traceName)
                                .font(.caption)
                                .fontWeight(.semibold)
                            if let spanName = context.state.currentSpanName {
                                Text(spanName)
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }

                DynamicIslandExpandedRegion(.trailing) {
                    VStack(alignment: .trailing) {
                        Text(formatCost(context.state.cost))
                            .font(.caption)
                            .fontWeight(.bold)
                            .foregroundStyle(.green)
                        Text("\(context.state.tokenCount) tokens")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }

                DynamicIslandExpandedRegion(.center) {
                    // Empty
                }

                DynamicIslandExpandedRegion(.bottom) {
                    HStack {
                        Label("\(context.state.spanCount) spans", systemImage: "square.stack")
                        Spacer()
                        Label(formatDuration(context.state.elapsedSeconds), systemImage: "clock")
                    }
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                }
            } compactLeading: {
                HStack(spacing: 4) {
                    Image(systemName: "waveform.path.ecg")
                        .foregroundStyle(.blue)
                }
            } compactTrailing: {
                Text(formatCost(context.state.cost))
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundStyle(.green)
            } minimal: {
                Image(systemName: statusIcon(context.state.status))
                    .foregroundStyle(statusColor(context.state.status))
            }
        }
    }

    private func statusIcon(_ status: AgentSessionAttributes.ContentState.SessionStatus) -> String {
        switch status {
        case .running: return "bolt.fill"
        case .completed: return "checkmark.circle.fill"
        case .error: return "exclamationmark.triangle.fill"
        }
    }

    private func statusColor(_ status: AgentSessionAttributes.ContentState.SessionStatus) -> Color {
        switch status {
        case .running: return .blue
        case .completed: return .green
        case .error: return .red
        }
    }

    private func formatCost(_ cost: Double) -> String {
        if cost < 0.01 {
            return String(format: "$%.4f", cost)
        } else if cost < 1 {
            return String(format: "$%.3f", cost)
        } else {
            return String(format: "$%.2f", cost)
        }
    }

    private func formatDuration(_ seconds: Int) -> String {
        if seconds < 60 {
            return "\(seconds)s"
        } else if seconds < 3600 {
            return "\(seconds / 60)m \(seconds % 60)s"
        } else {
            return "\(seconds / 3600)h \((seconds % 3600) / 60)m"
        }
    }
}

// MARK: - Lock Screen View

struct LockScreenView: View {
    let context: ActivityViewContext<AgentSessionAttributes>

    var body: some View {
        VStack(spacing: 12) {
            // Header
            HStack {
                Image(systemName: "waveform.path.ecg")
                    .foregroundStyle(.blue)
                Text("AgentTrace")
                    .font(.caption)
                    .fontWeight(.semibold)

                Spacer()

                StatusBadge(status: context.state.status)
            }

            Divider()

            // Trace info
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(context.state.traceName)
                        .font(.headline)

                    if let spanName = context.state.currentSpanName {
                        HStack {
                            Image(systemName: "arrow.right")
                                .font(.caption2)
                            Text(spanName)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                }

                Spacer()

                VStack(alignment: .trailing) {
                    Text(formatCost(context.state.cost))
                        .font(.title3)
                        .fontWeight(.bold)
                        .foregroundStyle(.green)

                    Text(formatDuration(context.state.elapsedSeconds))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }

            // Metrics bar
            HStack(spacing: 20) {
                MetricPill(icon: "square.stack", value: "\(context.state.spanCount)", label: "spans")
                MetricPill(icon: "number", value: formatTokens(context.state.tokenCount), label: "tokens")
            }
        }
        .padding()
        .activityBackgroundTint(.black.opacity(0.8))
    }

    private func formatCost(_ cost: Double) -> String {
        if cost < 0.01 {
            return String(format: "$%.4f", cost)
        } else if cost < 1 {
            return String(format: "$%.3f", cost)
        } else {
            return String(format: "$%.2f", cost)
        }
    }

    private func formatDuration(_ seconds: Int) -> String {
        if seconds < 60 {
            return "\(seconds)s"
        } else if seconds < 3600 {
            return "\(seconds / 60)m \(seconds % 60)s"
        } else {
            return "\(seconds / 3600)h \((seconds % 3600) / 60)m"
        }
    }

    private func formatTokens(_ count: Int) -> String {
        if count >= 1000 {
            return String(format: "%.1fK", Double(count) / 1000)
        }
        return "\(count)"
    }
}

struct StatusBadge: View {
    let status: AgentSessionAttributes.ContentState.SessionStatus

    var body: some View {
        HStack(spacing: 4) {
            Circle()
                .fill(statusColor)
                .frame(width: 6, height: 6)
            Text(status.rawValue.capitalized)
                .font(.caption2)
                .fontWeight(.medium)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(statusColor.opacity(0.2))
        .clipShape(Capsule())
    }

    var statusColor: Color {
        switch status {
        case .running: return .blue
        case .completed: return .green
        case .error: return .red
        }
    }
}

struct MetricPill: View {
    let icon: String
    let value: String
    let label: String

    var body: some View {
        HStack(spacing: 4) {
            Image(systemName: icon)
                .font(.caption2)
            Text("\(value) \(label)")
                .font(.caption)
        }
        .foregroundStyle(.secondary)
    }
}

// MARK: - Live Activity Manager

@MainActor
class LiveActivityManager: ObservableObject {
    static let shared = LiveActivityManager()

    private var currentActivity: Activity<AgentSessionAttributes>?

    private init() {}

    func startActivity(traceId: String, traceName: String) async {
        guard ActivityAuthorizationInfo().areActivitiesEnabled else {
            print("Live Activities are not enabled")
            return
        }

        let attributes = AgentSessionAttributes(
            traceId: traceId,
            startTime: Date()
        )

        let initialState = AgentSessionAttributes.ContentState(
            traceName: traceName,
            spanCount: 0,
            tokenCount: 0,
            cost: 0,
            elapsedSeconds: 0,
            currentSpanName: nil,
            status: .running
        )

        do {
            let activity = try Activity.request(
                attributes: attributes,
                content: .init(state: initialState, staleDate: nil),
                pushType: nil
            )
            currentActivity = activity
            print("Started Live Activity: \(activity.id)")
        } catch {
            print("Failed to start Live Activity: \(error)")
        }
    }

    func updateActivity(
        spanCount: Int,
        tokenCount: Int,
        cost: Double,
        currentSpanName: String?
    ) async {
        guard let activity = currentActivity else { return }

        let elapsedSeconds = Int(Date().timeIntervalSince(activity.attributes.startTime))

        let updatedState = AgentSessionAttributes.ContentState(
            traceName: activity.content.state.traceName,
            spanCount: spanCount,
            tokenCount: tokenCount,
            cost: cost,
            elapsedSeconds: elapsedSeconds,
            currentSpanName: currentSpanName,
            status: .running
        )

        await activity.update(
            ActivityContent(state: updatedState, staleDate: nil)
        )
    }

    func endActivity(status: AgentSessionAttributes.ContentState.SessionStatus) async {
        guard let activity = currentActivity else { return }

        let elapsedSeconds = Int(Date().timeIntervalSince(activity.attributes.startTime))

        let finalState = AgentSessionAttributes.ContentState(
            traceName: activity.content.state.traceName,
            spanCount: activity.content.state.spanCount,
            tokenCount: activity.content.state.tokenCount,
            cost: activity.content.state.cost,
            elapsedSeconds: elapsedSeconds,
            currentSpanName: nil,
            status: status
        )

        await activity.end(
            ActivityContent(state: finalState, staleDate: nil),
            dismissalPolicy: .after(.now + 30)
        )

        currentActivity = nil
    }
}

// MARK: - Previews

#Preview("Lock Screen", as: .content, using: AgentSessionAttributes(
    traceId: "test-123",
    startTime: Date()
)) {
    AgentSessionLiveActivity()
} contentStates: {
    AgentSessionAttributes.ContentState(
        traceName: "Customer Support Agent",
        spanCount: 12,
        tokenCount: 4532,
        cost: 0.0234,
        elapsedSeconds: 45,
        currentSpanName: "Generating response",
        status: .running
    )

    AgentSessionAttributes.ContentState(
        traceName: "Customer Support Agent",
        spanCount: 24,
        tokenCount: 12500,
        cost: 0.0567,
        elapsedSeconds: 120,
        currentSpanName: nil,
        status: .completed
    )
}
