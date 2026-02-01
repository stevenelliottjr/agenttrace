import SwiftUI

struct TraceListView: View {
    @State private var viewModel = TraceListViewModel()
    @State private var searchText = ""

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.traces.isEmpty {
                    ProgressView("Loading traces...")
                } else if viewModel.traces.isEmpty {
                    ContentUnavailableView(
                        "No Traces",
                        systemImage: "point.3.connected.trianglepath.dotted",
                        description: Text("Traces will appear here as your agents run")
                    )
                } else {
                    traceList
                }
            }
            .navigationTitle("Traces")
            .searchable(text: $searchText, prompt: "Search traces")
            .onChange(of: searchText) { _, newValue in
                viewModel.updateSearch(newValue)
            }
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Menu {
                        Section("Status") {
                            Button("All") { viewModel.updateStatusFilter(nil) }
                            ForEach([TraceStatus.running, .completed, .error], id: \.self) { status in
                                Button(status.rawValue.capitalized) {
                                    viewModel.updateStatusFilter(status)
                                }
                            }
                        }
                    } label: {
                        Image(systemName: "line.3.horizontal.decrease.circle")
                    }
                }
            }
            .refreshable {
                await viewModel.refresh()
            }
            .task {
                await viewModel.loadTraces()
            }
        }
    }

    private var traceList: some View {
        List {
            ForEach(viewModel.traces) { trace in
                NavigationLink {
                    TraceDetailView(traceId: trace.id)
                } label: {
                    TraceListRow(trace: trace)
                }
            }

            if viewModel.hasMore {
                HStack {
                    Spacer()
                    ProgressView()
                        .task {
                            await viewModel.loadMore()
                        }
                    Spacer()
                }
                .listRowBackground(Color.clear)
            }
        }
        .listStyle(.plain)
    }
}

struct TraceListRow: View {
    let trace: TraceSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                StatusIndicator(status: trace.status)

                Text(trace.displayName)
                    .font(.headline)

                Spacer()

                Text(trace.startTime.relativeTimeString)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            HStack(spacing: 16) {
                Label("\(trace.spanCount) spans", systemImage: "square.stack")

                Label(trace.formattedCost, systemImage: "dollarsign.circle")

                Label(trace.formattedDuration, systemImage: "clock")

                if trace.errorCount > 0 {
                    Label("\(trace.errorCount)", systemImage: "exclamationmark.triangle")
                        .foregroundStyle(.red)
                }
            }
            .font(.caption)
            .foregroundStyle(.secondary)
        }
        .padding(.vertical, 4)
    }
}

struct StatusIndicator: View {
    let status: TraceStatus

    var body: some View {
        HStack(spacing: 4) {
            Circle()
                .fill(Color.forTraceStatus(status))
                .frame(width: 8, height: 8)

            if status == .running {
                ProgressView()
                    .scaleEffect(0.5)
                    .frame(width: 12, height: 12)
            }
        }
    }
}

// MARK: - View Model

@Observable
final class TraceListViewModel {
    var traces: [TraceSummary] = []
    var isLoading = false
    var hasMore = false
    var error: Error?

    private var currentPage = 1
    private var pageSize = 20
    private var statusFilter: TraceStatus?
    private var searchQuery: String?

    private let apiClient: APIClient

    init(apiClient: APIClient = .shared) {
        self.apiClient = apiClient
    }

    @MainActor
    func loadTraces() async {
        currentPage = 1
        isLoading = true
        error = nil

        do {
            let response = try await apiClient.getTraces(
                page: currentPage,
                pageSize: pageSize,
                status: statusFilter,
                search: searchQuery
            )
            traces = response.data
            hasMore = response.hasMore
        } catch {
            self.error = error
        }

        isLoading = false
    }

    @MainActor
    func loadMore() async {
        guard hasMore, !isLoading else { return }

        isLoading = true
        currentPage += 1

        do {
            let response = try await apiClient.getTraces(
                page: currentPage,
                pageSize: pageSize,
                status: statusFilter,
                search: searchQuery
            )
            traces.append(contentsOf: response.data)
            hasMore = response.hasMore
        } catch {
            self.error = error
            currentPage -= 1
        }

        isLoading = false
    }

    @MainActor
    func refresh() async {
        await loadTraces()
    }

    func updateStatusFilter(_ status: TraceStatus?) {
        statusFilter = status
        Task {
            await loadTraces()
        }
    }

    func updateSearch(_ query: String) {
        searchQuery = query.isEmpty ? nil : query
        Task {
            await loadTraces()
        }
    }
}

#Preview {
    TraceListView()
}
