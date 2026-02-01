import Foundation
import SwiftUI

/// View model for the Dashboard
@Observable
final class DashboardViewModel {
    var metricsSummary: MetricsSummary?
    var recentTraces: [TraceSummary] = []
    var activeAlerts: [AlertEvent] = []
    var isLoading = false
    var error: Error?
    var selectedTimeRange: TimeRange = .last24Hours

    private let apiClient: APIClient

    init(apiClient: APIClient = .shared) {
        self.apiClient = apiClient
    }

    @MainActor
    func loadDashboard() async {
        isLoading = true
        error = nil

        do {
            async let metricsTask = apiClient.getMetricsSummary(timeRange: selectedTimeRange)
            async let tracesTask = apiClient.getTraces(page: 1, pageSize: 10)
            async let alertsTask = apiClient.getAlertEvents(status: .triggered, page: 1, pageSize: 5)

            let (metrics, traces, alerts) = try await (metricsTask, tracesTask, alertsTask)

            self.metricsSummary = metrics
            self.recentTraces = traces.data
            self.activeAlerts = alerts.data
        } catch {
            self.error = error
        }

        isLoading = false
    }

    @MainActor
    func refresh() async {
        await loadDashboard()
    }

    func updateTimeRange(_ range: TimeRange) {
        selectedTimeRange = range
        Task {
            await loadDashboard()
        }
    }
}
