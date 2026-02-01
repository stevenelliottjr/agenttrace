import SwiftUI

struct TraceDetailView: View {
    let traceId: String

    @State private var viewModel: TraceDetailViewModel
    @State private var selectedSpan: Span?
    @State private var showSpanDetail = false

    init(traceId: String) {
        self.traceId = traceId
        _viewModel = State(initialValue: TraceDetailViewModel(traceId: traceId))
    }

    var body: some View {
        Group {
            if viewModel.isLoading && viewModel.trace == nil {
                ProgressView("Loading trace...")
            } else if let trace = viewModel.trace {
                ScrollView {
                    VStack(spacing: 20) {
                        // Header
                        TraceHeaderView(trace: trace)

                        // Metrics summary
                        TraceMetricsView(trace: trace)

                        // Waterfall visualization
                        TraceWaterfallView(
                            trace: trace,
                            onSpanSelected: { span in
                                selectedSpan = span
                                showSpanDetail = true
                            }
                        )
                    }
                    .padding()
                }
            } else if let error = viewModel.error {
                ContentUnavailableView(
                    "Failed to Load",
                    systemImage: "exclamationmark.triangle",
                    description: Text(error.localizedDescription)
                )
            }
        }
        .navigationTitle("Trace Details")
        .navigationBarTitleDisplayMode(.inline)
        .refreshable {
            await viewModel.refresh()
        }
        .task {
            await viewModel.loadTrace()
        }
        .sheet(isPresented: $showSpanDetail) {
            if let span = selectedSpan {
                SpanDetailSheet(span: span)
            }
        }
    }
}

// MARK: - Trace Header

struct TraceHeaderView: View {
    let trace: Trace

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                StatusIndicator(status: trace.status)

                Text(trace.displayName)
                    .font(.title2)
                    .fontWeight(.bold)

                Spacer()
            }

            HStack(spacing: 16) {
                Label(trace.startTime.shortDateTimeString, systemImage: "calendar")

                if let endTime = trace.endTime {
                    Label(endTime.shortTimeString, systemImage: "clock")
                }
            }
            .font(.caption)
            .foregroundStyle(.secondary)

            // Trace ID
            HStack {
                Text("ID:")
                    .foregroundStyle(.secondary)
                Text(trace.id)
                    .font(.caption)
                    .monospaced()

                Button {
                    UIPasteboard.general.string = trace.id
                } label: {
                    Image(systemName: "doc.on.doc")
                        .font(.caption)
                }
            }
            .font(.caption)
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

// MARK: - Trace Metrics

struct TraceMetricsView: View {
    let trace: Trace

    var body: some View {
        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
            MetricBadge(title: "Duration", value: trace.formattedDuration, icon: "clock")
            MetricBadge(title: "Cost", value: trace.formattedCost, icon: "dollarsign.circle")
            MetricBadge(title: "Tokens", value: trace.formattedTokens, icon: "number")
            MetricBadge(title: "Spans", value: "\(trace.spanCount)", icon: "square.stack")
            MetricBadge(title: "Errors", value: "\(trace.errorCount)", icon: "exclamationmark.triangle", isError: trace.errorCount > 0)
        }
    }
}

struct MetricBadge: View {
    let title: String
    let value: String
    let icon: String
    var isError: Bool = false

    var body: some View {
        VStack(spacing: 4) {
            Image(systemName: icon)
                .foregroundStyle(isError ? .red : .secondary)

            Text(value)
                .font(.headline)
                .foregroundStyle(isError ? .red : .primary)

            Text(title)
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 12)
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }
}

// MARK: - View Model

@Observable
final class TraceDetailViewModel {
    let traceId: String
    var trace: Trace?
    var isLoading = false
    var error: Error?

    private let apiClient: APIClient

    init(traceId: String, apiClient: APIClient = .shared) {
        self.traceId = traceId
        self.apiClient = apiClient
    }

    @MainActor
    func loadTrace() async {
        isLoading = true
        error = nil

        do {
            trace = try await apiClient.getTrace(traceId: traceId)
        } catch {
            self.error = error
        }

        isLoading = false
    }

    @MainActor
    func refresh() async {
        await loadTrace()
    }
}

#Preview {
    NavigationStack {
        TraceDetailView(traceId: "test-trace-id")
    }
}
