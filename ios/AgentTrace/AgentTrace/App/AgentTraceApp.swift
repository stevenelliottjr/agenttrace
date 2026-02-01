import SwiftUI
import SwiftData

@main
struct AgentTraceApp: App {
    @State private var sseManager = SSEConnectionManager()
    @State private var settingsManager = SettingsManager()

    var sharedModelContainer: ModelContainer = {
        let schema = Schema([
            CachedTrace.self,
            CachedSpan.self,
            CachedMetricsSummary.self
        ])
        let modelConfiguration = ModelConfiguration(
            schema: schema,
            isStoredInMemoryOnly: false,
            groupContainer: .identifier("group.com.agenttrace.app")
        )

        do {
            return try ModelContainer(for: schema, configurations: [modelConfiguration])
        } catch {
            fatalError("Could not create ModelContainer: \(error)")
        }
    }()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(sseManager)
                .environment(settingsManager)
                .onAppear {
                    Task {
                        await updateAPIBaseURL()
                    }
                }
                .onChange(of: settingsManager.serverURL) { _, _ in
                    Task {
                        await updateAPIBaseURL()
                    }
                }
        }
        .modelContainer(sharedModelContainer)
    }

    private func updateAPIBaseURL() async {
        if let url = URL(string: settingsManager.serverURL) {
            await APIClient.shared.updateBaseURL(url)
            await sseManager.updateBaseURL(url)
        }
    }
}

/// Main content view with tab navigation
struct ContentView: View {
    @Environment(SSEConnectionManager.self) private var sseManager
    @State private var selectedTab: Tab = .dashboard

    enum Tab: String, CaseIterable {
        case dashboard = "Dashboard"
        case traces = "Traces"
        case metrics = "Metrics"
        case alerts = "Alerts"
        case settings = "Settings"

        var iconName: String {
            switch self {
            case .dashboard: return "square.grid.2x2"
            case .traces: return "point.3.connected.trianglepath.dotted"
            case .metrics: return "chart.xyaxis.line"
            case .alerts: return "bell"
            case .settings: return "gear"
            }
        }
    }

    var body: some View {
        TabView(selection: $selectedTab) {
            DashboardView()
                .tabItem {
                    Label(Tab.dashboard.rawValue, systemImage: Tab.dashboard.iconName)
                }
                .tag(Tab.dashboard)

            TraceListView()
                .tabItem {
                    Label(Tab.traces.rawValue, systemImage: Tab.traces.iconName)
                }
                .tag(Tab.traces)

            MetricsView()
                .tabItem {
                    Label(Tab.metrics.rawValue, systemImage: Tab.metrics.iconName)
                }
                .tag(Tab.metrics)

            AlertsView()
                .tabItem {
                    Label(Tab.alerts.rawValue, systemImage: Tab.alerts.iconName)
                }
                .tag(Tab.alerts)

            SettingsView()
                .tabItem {
                    Label(Tab.settings.rawValue, systemImage: Tab.settings.iconName)
                }
                .tag(Tab.settings)
        }
        .onAppear {
            sseManager.connect()
        }
        .onDisappear {
            sseManager.disconnect()
        }
    }
}

#Preview {
    ContentView()
        .environment(SSEConnectionManager())
        .environment(SettingsManager())
}
