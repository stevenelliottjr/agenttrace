import SwiftUI

struct AlertsView: View {
    @State private var viewModel = AlertsViewModel()
    @State private var selectedTab: AlertTab = .events

    enum AlertTab: String, CaseIterable {
        case events = "Triggered"
        case rules = "Rules"
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // Tab picker
                Picker("Tab", selection: $selectedTab) {
                    ForEach(AlertTab.allCases, id: \.self) { tab in
                        Text(tab.rawValue).tag(tab)
                    }
                }
                .pickerStyle(.segmented)
                .padding()

                // Content
                Group {
                    switch selectedTab {
                    case .events:
                        alertEventsList
                    case .rules:
                        alertRulesList
                    }
                }
            }
            .navigationTitle("Alerts")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Menu {
                        Section("Severity") {
                            Button("All") { viewModel.severityFilter = nil }
                            ForEach(AlertSeverity.allCases, id: \.self) { severity in
                                Button {
                                    viewModel.severityFilter = severity
                                } label: {
                                    Label(severity.displayName, systemImage: severity.iconName)
                                }
                            }
                        }

                        if selectedTab == .events {
                            Section("Status") {
                                Button("All") { viewModel.statusFilter = nil }
                                ForEach([AlertEventStatus.triggered, .acknowledged, .resolved], id: \.self) { status in
                                    Button(status.rawValue.capitalized) {
                                        viewModel.statusFilter = status
                                    }
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
                await viewModel.loadAlerts()
            }
        }
    }

    private var alertEventsList: some View {
        Group {
            if viewModel.isLoading && viewModel.events.isEmpty {
                ProgressView("Loading alerts...")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if viewModel.events.isEmpty {
                ContentUnavailableView(
                    "No Alerts",
                    systemImage: "bell.slash",
                    description: Text("No alerts have been triggered")
                )
            } else {
                List {
                    ForEach(viewModel.events) { event in
                        AlertEventRow(event: event, onAcknowledge: {
                            Task {
                                await viewModel.acknowledgeAlert(eventId: event.id)
                            }
                        })
                    }

                    if viewModel.hasMoreEvents {
                        HStack {
                            Spacer()
                            ProgressView()
                                .task {
                                    await viewModel.loadMoreEvents()
                                }
                            Spacer()
                        }
                    }
                }
                .listStyle(.plain)
            }
        }
    }

    private var alertRulesList: some View {
        Group {
            if viewModel.isLoading && viewModel.rules.isEmpty {
                ProgressView("Loading rules...")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if viewModel.rules.isEmpty {
                ContentUnavailableView(
                    "No Alert Rules",
                    systemImage: "bell.badge",
                    description: Text("Configure alert rules in the web dashboard")
                )
            } else {
                List {
                    ForEach(viewModel.rules) { rule in
                        AlertRuleRow(rule: rule)
                    }
                }
                .listStyle(.plain)
            }
        }
    }
}

// MARK: - Alert Event Row

struct AlertEventRow: View {
    let event: AlertEvent
    let onAcknowledge: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: event.severity.iconName)
                    .foregroundStyle(Color.forAlertSeverity(event.severity))

                Text(event.ruleName)
                    .font(.headline)

                Spacer()

                EventStatusBadge(status: event.status)
            }

            Text(event.message)
                .font(.subheadline)
                .foregroundStyle(.secondary)

            HStack {
                Label(event.timeSinceTriggered, systemImage: "clock")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                Spacer()

                if event.status == .triggered {
                    Button("Acknowledge") {
                        onAcknowledge()
                    }
                    .font(.caption)
                    .buttonStyle(.bordered)
                    .tint(.orange)
                }
            }

            // Link to trace if available
            if let traceId = event.traceId {
                NavigationLink {
                    TraceDetailView(traceId: traceId)
                } label: {
                    Label("View Trace", systemImage: "arrow.right.circle")
                        .font(.caption)
                }
            }
        }
        .padding(.vertical, 4)
    }
}

struct EventStatusBadge: View {
    let status: AlertEventStatus

    var color: Color {
        switch status {
        case .triggered: return .red
        case .acknowledged: return .orange
        case .resolved: return .green
        }
    }

    var body: some View {
        Text(status.rawValue.uppercased())
            .font(.caption2)
            .fontWeight(.bold)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(color)
            .foregroundStyle(.white)
            .clipShape(Capsule())
    }
}

// MARK: - Alert Rule Row

struct AlertRuleRow: View {
    let rule: AlertRule

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: rule.conditionType.iconName)
                    .foregroundStyle(.blue)

                Text(rule.name)
                    .font(.headline)

                Spacer()

                if !rule.isEnabled {
                    Text("DISABLED")
                        .font(.caption2)
                        .fontWeight(.bold)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Color.gray)
                        .foregroundStyle(.white)
                        .clipShape(Capsule())
                }

                Image(systemName: rule.severity.iconName)
                    .foregroundStyle(Color.forAlertSeverity(rule.severity))
            }

            if let description = rule.description {
                Text(description)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            HStack {
                Label(rule.conditionType.displayName, systemImage: "slider.horizontal.3")
                Text("•")
                Text("Threshold: \(rule.formattedThreshold)")
            }
            .font(.caption)
            .foregroundStyle(.secondary)
        }
        .padding(.vertical, 4)
        .opacity(rule.isEnabled ? 1 : 0.6)
    }
}

// MARK: - View Model

@Observable
final class AlertsViewModel {
    var events: [AlertEvent] = []
    var rules: [AlertRule] = []
    var isLoading = false
    var hasMoreEvents = false
    var error: Error?

    var statusFilter: AlertEventStatus?
    var severityFilter: AlertSeverity?

    private var currentPage = 1
    private let pageSize = 20

    private let apiClient: APIClient

    init(apiClient: APIClient = .shared) {
        self.apiClient = apiClient
    }

    @MainActor
    func loadAlerts() async {
        isLoading = true
        error = nil
        currentPage = 1

        do {
            async let eventsTask = apiClient.getAlertEvents(
                status: statusFilter,
                severity: severityFilter,
                page: currentPage,
                pageSize: pageSize
            )
            async let rulesTask = apiClient.getAlertRules()

            let (eventsResponse, rulesData) = try await (eventsTask, rulesTask)

            events = eventsResponse.data
            hasMoreEvents = eventsResponse.hasMore
            rules = rulesData
        } catch {
            self.error = error
        }

        isLoading = false
    }

    @MainActor
    func loadMoreEvents() async {
        guard hasMoreEvents, !isLoading else { return }

        isLoading = true
        currentPage += 1

        do {
            let response = try await apiClient.getAlertEvents(
                status: statusFilter,
                severity: severityFilter,
                page: currentPage,
                pageSize: pageSize
            )
            events.append(contentsOf: response.data)
            hasMoreEvents = response.hasMore
        } catch {
            self.error = error
            currentPage -= 1
        }

        isLoading = false
    }

    @MainActor
    func refresh() async {
        await loadAlerts()
    }

    @MainActor
    func acknowledgeAlert(eventId: String) async {
        do {
            try await apiClient.acknowledgeAlert(eventId: eventId)
            // Update local state
            if let index = events.firstIndex(where: { $0.id == eventId }) {
                // Reload to get updated status
                await loadAlerts()
            }
        } catch {
            self.error = error
        }
    }
}

#Preview {
    AlertsView()
}
