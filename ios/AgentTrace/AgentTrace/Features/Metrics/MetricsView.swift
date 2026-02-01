import SwiftUI
import Charts

struct MetricsView: View {
    @State private var viewModel = MetricsViewModel()
    @State private var selectedChart: ChartType = .cost

    enum ChartType: String, CaseIterable {
        case cost = "Cost"
        case latency = "Latency"
        case tokens = "Tokens"
        case errors = "Errors"

        var icon: String {
            switch self {
            case .cost: return "dollarsign.circle"
            case .latency: return "clock"
            case .tokens: return "number"
            case .errors: return "exclamationmark.triangle"
            }
        }
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 20) {
                    // Time range picker
                    TimeRangePicker(selectedRange: $viewModel.selectedTimeRange)
                        .onChange(of: viewModel.selectedTimeRange) { _, _ in
                            Task {
                                await viewModel.loadMetrics()
                            }
                        }

                    // Chart type selector
                    chartTypeSelector

                    // Selected chart
                    Group {
                        switch selectedChart {
                        case .cost:
                            CostBreakdownChart(data: viewModel.costBreakdown, isLoading: viewModel.isLoading)
                        case .latency:
                            LatencyChart(data: viewModel.latencyMetrics, isLoading: viewModel.isLoading)
                        case .tokens:
                            TokenUsageChart(data: viewModel.tokenUsage, isLoading: viewModel.isLoading)
                        case .errors:
                            ErrorRateChart(data: viewModel.errorMetrics, isLoading: viewModel.isLoading)
                        }
                    }
                    .frame(height: 300)
                }
                .padding()
            }
            .navigationTitle("Metrics")
            .refreshable {
                await viewModel.loadMetrics()
            }
            .task {
                await viewModel.loadMetrics()
            }
        }
    }

    private var chartTypeSelector: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 12) {
                ForEach(ChartType.allCases, id: \.self) { type in
                    Button {
                        withAnimation {
                            selectedChart = type
                        }
                    } label: {
                        Label(type.rawValue, systemImage: type.icon)
                            .font(.subheadline)
                            .padding(.horizontal, 16)
                            .padding(.vertical, 10)
                            .background(selectedChart == type ? Color.accentColor : Color(.systemGray5))
                            .foregroundStyle(selectedChart == type ? .white : .primary)
                            .clipShape(RoundedRectangle(cornerRadius: 10))
                    }
                }
            }
        }
    }
}

// MARK: - View Model

@Observable
final class MetricsViewModel {
    var selectedTimeRange: TimeRange = .last24Hours
    var costBreakdown: [CostBreakdown] = []
    var latencyMetrics: [LatencyMetrics] = []
    var tokenUsage: [TokenUsageMetrics] = []
    var errorMetrics: [ErrorMetrics] = []
    var isLoading = false
    var error: Error?

    private let apiClient: APIClient

    init(apiClient: APIClient = .shared) {
        self.apiClient = apiClient
    }

    @MainActor
    func loadMetrics() async {
        isLoading = true
        error = nil

        do {
            async let costsTask = apiClient.getCostBreakdown(timeRange: selectedTimeRange)
            async let latencyTask = apiClient.getLatencyMetrics(timeRange: selectedTimeRange)
            async let tokensTask = apiClient.getTokenUsage(timeRange: selectedTimeRange)
            async let errorsTask = apiClient.getErrorMetrics(timeRange: selectedTimeRange)

            let (costs, latency, tokens, errors) = try await (costsTask, latencyTask, tokensTask, errorsTask)

            self.costBreakdown = costs
            self.latencyMetrics = latency
            self.tokenUsage = tokens
            self.errorMetrics = errors
        } catch {
            self.error = error
        }

        isLoading = false
    }
}

#Preview {
    MetricsView()
}
