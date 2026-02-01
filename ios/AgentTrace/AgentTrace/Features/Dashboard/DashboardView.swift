import SwiftUI

struct DashboardView: View {
    @Environment(SSEConnectionManager.self) private var sseManager
    @State private var viewModel = DashboardViewModel()

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 20) {
                    // Connection status
                    ConnectionStatusBar(isConnected: sseManager.isConnected)

                    // Time range picker
                    TimeRangePicker(selectedRange: $viewModel.selectedTimeRange)
                        .onChange(of: viewModel.selectedTimeRange) { _, newRange in
                            viewModel.updateTimeRange(newRange)
                        }

                    // Metrics cards
                    if let metrics = viewModel.metricsSummary {
                        MetricsGrid(metrics: metrics)
                    } else if viewModel.isLoading {
                        MetricsGridPlaceholder()
                    }

                    // Active alerts section
                    if !viewModel.activeAlerts.isEmpty {
                        ActiveAlertsSection(alerts: viewModel.activeAlerts)
                    }

                    // Recent traces section
                    RecentTracesSection(traces: viewModel.recentTraces)

                    // Live spans section
                    if !sseManager.recentSpans.isEmpty {
                        LiveSpansSection(spans: Array(sseManager.recentSpans.prefix(5)))
                    }
                }
                .padding()
            }
            .navigationTitle("Dashboard")
            .refreshable {
                await viewModel.refresh()
            }
            .task {
                await viewModel.loadDashboard()
            }
            .overlay {
                if let error = viewModel.error {
                    ErrorOverlay(error: error) {
                        Task {
                            await viewModel.refresh()
                        }
                    }
                }
            }
        }
    }
}

// MARK: - Connection Status Bar

struct ConnectionStatusBar: View {
    let isConnected: Bool

    var body: some View {
        HStack {
            Circle()
                .fill(isConnected ? Color.green : Color.red)
                .frame(width: 8, height: 8)

            Text(isConnected ? "Connected" : "Disconnected")
                .font(.caption)
                .foregroundStyle(.secondary)

            Spacer()

            if isConnected {
                Text("Live")
                    .font(.caption)
                    .fontWeight(.medium)
                    .foregroundStyle(.green)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }
}

// MARK: - Time Range Picker

struct TimeRangePicker: View {
    @Binding var selectedRange: TimeRange

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach(TimeRange.allCases, id: \.self) { range in
                    Button {
                        selectedRange = range
                    } label: {
                        Text(range.displayName)
                            .font(.subheadline)
                            .padding(.horizontal, 12)
                            .padding(.vertical, 6)
                            .background(selectedRange == range ? Color.accentColor : Color(.systemGray5))
                            .foregroundStyle(selectedRange == range ? .white : .primary)
                            .clipShape(Capsule())
                    }
                }
            }
        }
    }
}

// MARK: - Metrics Grid

struct MetricsGrid: View {
    let metrics: MetricsSummary

    var body: some View {
        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
            MetricCard(
                title: "Total Cost",
                value: metrics.formattedCost,
                icon: "dollarsign.circle.fill",
                color: .green
            )

            MetricCard(
                title: "Total Tokens",
                value: metrics.formattedTokens,
                icon: "number.circle.fill",
                color: .blue
            )

            MetricCard(
                title: "Avg Latency",
                value: metrics.formattedLatency,
                icon: "clock.fill",
                color: .orange
            )

            MetricCard(
                title: "Error Rate",
                value: metrics.formattedErrorRate,
                icon: "exclamationmark.triangle.fill",
                color: metrics.errorRate > 0.05 ? .red : .gray
            )

            MetricCard(
                title: "Total Traces",
                value: "\(metrics.totalTraces)",
                icon: "point.3.connected.trianglepath.dotted",
                color: .purple
            )

            MetricCard(
                title: "Active",
                value: "\(metrics.activeTraces)",
                icon: "bolt.fill",
                color: .cyan
            )
        }
    }
}

struct MetricCard: View {
    let title: String
    let value: String
    let icon: String
    let color: Color

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: icon)
                    .foregroundStyle(color)
                Spacer()
            }

            Text(value)
                .font(.title2)
                .fontWeight(.bold)

            Text(title)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct MetricsGridPlaceholder: View {
    var body: some View {
        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
            ForEach(0..<6, id: \.self) { _ in
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color(.systemGray6))
                    .frame(height: 100)
                    .redacted(reason: .placeholder)
            }
        }
    }
}

// MARK: - Active Alerts Section

struct ActiveAlertsSection: View {
    let alerts: [AlertEvent]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "bell.badge.fill")
                    .foregroundStyle(.red)
                Text("Active Alerts")
                    .font(.headline)
                Spacer()
                NavigationLink {
                    AlertsView()
                } label: {
                    Text("View All")
                        .font(.caption)
                }
            }

            ForEach(alerts) { alert in
                AlertRow(alert: alert)
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct AlertRow: View {
    let alert: AlertEvent

    var body: some View {
        HStack {
            Image(systemName: alert.severity.iconName)
                .foregroundStyle(Color.forAlertSeverity(alert.severity))

            VStack(alignment: .leading) {
                Text(alert.ruleName)
                    .font(.subheadline)
                    .fontWeight(.medium)

                Text(alert.message)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            Text(alert.timeSinceTriggered)
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
        .padding(.vertical, 4)
    }
}

// MARK: - Recent Traces Section

struct RecentTracesSection: View {
    let traces: [TraceSummary]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "clock.arrow.circlepath")
                Text("Recent Traces")
                    .font(.headline)
                Spacer()
                NavigationLink {
                    TraceListView()
                } label: {
                    Text("View All")
                        .font(.caption)
                }
            }

            if traces.isEmpty {
                ContentUnavailableView(
                    "No Recent Traces",
                    systemImage: "point.3.connected.trianglepath.dotted",
                    description: Text("Traces will appear here as your agents run")
                )
                .frame(height: 150)
            } else {
                ForEach(traces) { trace in
                    NavigationLink {
                        TraceDetailView(traceId: trace.id)
                    } label: {
                        TraceRow(trace: trace)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct TraceRow: View {
    let trace: TraceSummary

    var body: some View {
        HStack {
            Circle()
                .fill(Color.forTraceStatus(trace.status))
                .frame(width: 8, height: 8)

            VStack(alignment: .leading) {
                Text(trace.displayName)
                    .font(.subheadline)
                    .fontWeight(.medium)

                HStack(spacing: 8) {
                    Label("\(trace.spanCount)", systemImage: "square.stack")
                    Label(trace.formattedCost, systemImage: "dollarsign.circle")
                }
                .font(.caption2)
                .foregroundStyle(.secondary)
            }

            Spacer()

            VStack(alignment: .trailing) {
                Text(trace.formattedDuration)
                    .font(.caption)
                    .fontWeight(.medium)

                Text(trace.startTime.relativeTimeString)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 4)
    }
}

// MARK: - Live Spans Section

struct LiveSpansSection: View {
    let spans: [Span]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "bolt.fill")
                    .foregroundStyle(.yellow)
                Text("Live Activity")
                    .font(.headline)
                Spacer()
            }

            ForEach(spans) { span in
                LiveSpanRow(span: span)
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct LiveSpanRow: View {
    let span: Span

    var body: some View {
        HStack {
            Image(systemName: span.spanType.iconName)
                .foregroundStyle(Color.forSpanType(span.spanType))
                .frame(width: 24)

            VStack(alignment: .leading) {
                Text(span.name)
                    .font(.caption)
                    .fontWeight(.medium)
                    .lineLimit(1)

                if let model = span.model {
                    Text(model)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }

            Spacer()

            Text(span.formattedDuration)
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
    }
}

// MARK: - Error Overlay

struct ErrorOverlay: View {
    let error: Error
    let onRetry: () -> Void

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.largeTitle)
                .foregroundStyle(.red)

            Text("Failed to Load")
                .font(.headline)

            Text(error.localizedDescription)
                .font(.caption)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            Button("Retry", action: onRetry)
                .buttonStyle(.bordered)
        }
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 16))
        .padding()
    }
}

#Preview {
    DashboardView()
        .environment(SSEConnectionManager())
}
