import SwiftUI

struct SettingsView: View {
    @Environment(SettingsManager.self) private var settings
    @Environment(SSEConnectionManager.self) private var sseManager
    @State private var isTestingConnection = false
    @State private var connectionTestResult: ConnectionTestResult?

    enum ConnectionTestResult {
        case success
        case failure(String)
    }

    var body: some View {
        NavigationStack {
            Form {
                // Server Configuration
                Section {
                    @Bindable var bindableSettings = settings
                    TextField("Server URL", text: $bindableSettings.serverURL)
                        .textContentType(.URL)
                        .keyboardType(.URL)
                        .autocapitalization(.none)
                        .autocorrectionDisabled()

                    Button {
                        Task {
                            await testConnection()
                        }
                    } label: {
                        HStack {
                            Text("Test Connection")
                            Spacer()
                            if isTestingConnection {
                                ProgressView()
                            } else if let result = connectionTestResult {
                                switch result {
                                case .success:
                                    Image(systemName: "checkmark.circle.fill")
                                        .foregroundStyle(.green)
                                case .failure:
                                    Image(systemName: "xmark.circle.fill")
                                        .foregroundStyle(.red)
                                }
                            }
                        }
                    }
                    .disabled(isTestingConnection)

                    if case .failure(let message) = connectionTestResult {
                        Text(message)
                            .font(.caption)
                            .foregroundStyle(.red)
                    }
                } header: {
                    Text("Server")
                } footer: {
                    Text("The URL of your AgentTrace collector server")
                }

                // Connection Status
                Section("Connection") {
                    HStack {
                        Text("Real-time Stream")
                        Spacer()
                        HStack(spacing: 4) {
                            Circle()
                                .fill(sseManager.isConnected ? .green : .red)
                                .frame(width: 8, height: 8)
                            Text(sseManager.isConnected ? "Connected" : "Disconnected")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }

                    Button(sseManager.isConnected ? "Disconnect" : "Connect") {
                        if sseManager.isConnected {
                            sseManager.disconnect()
                        } else {
                            sseManager.connect()
                        }
                    }
                }

                // Preferences
                Section("Preferences") {
                    @Bindable var bindableSettings = settings
                    Picker("Default Time Range", selection: $bindableSettings.defaultTimeRange) {
                        ForEach(TimeRange.allCases, id: \.self) { range in
                            Text(range.displayName).tag(range)
                        }
                    }

                    Toggle("Enable Notifications", isOn: $bindableSettings.enableNotifications)

                    Toggle("Enable Live Activities", isOn: $bindableSettings.enableLiveActivities)
                }

                // About
                Section("About") {
                    HStack {
                        Text("Version")
                        Spacer()
                        Text(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0.0")
                            .foregroundStyle(.secondary)
                    }

                    HStack {
                        Text("Build")
                        Spacer()
                        Text(Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "1")
                            .foregroundStyle(.secondary)
                    }

                    Link(destination: URL(string: "https://github.com/agenttrace/agenttrace")!) {
                        HStack {
                            Text("GitHub Repository")
                            Spacer()
                            Image(systemName: "arrow.up.right.square")
                                .foregroundStyle(.secondary)
                        }
                    }

                    Link(destination: URL(string: "https://agenttrace.dev/docs")!) {
                        HStack {
                            Text("Documentation")
                            Spacer()
                            Image(systemName: "arrow.up.right.square")
                                .foregroundStyle(.secondary)
                        }
                    }
                }

                // Reset
                Section {
                    Button("Reset to Defaults", role: .destructive) {
                        settings.resetToDefaults()
                        connectionTestResult = nil
                    }
                }

                // Debug Info (only in debug builds)
                #if DEBUG
                Section("Debug") {
                    HStack {
                        Text("Recent Spans")
                        Spacer()
                        Text("\(sseManager.recentSpans.count)")
                            .foregroundStyle(.secondary)
                    }

                    HStack {
                        Text("Active Traces")
                        Spacer()
                        Text("\(sseManager.activeTraces.count)")
                            .foregroundStyle(.secondary)
                    }

                    Button("Clear Cache") {
                        // TODO: Implement cache clearing
                    }
                }
                #endif
            }
            .navigationTitle("Settings")
        }
    }

    private func testConnection() async {
        isTestingConnection = true
        connectionTestResult = nil

        do {
            if let url = URL(string: settings.serverURL) {
                await APIClient.shared.updateBaseURL(url)
            }

            let isHealthy = try await APIClient.shared.healthCheck()
            connectionTestResult = isHealthy ? .success : .failure("Server returned unhealthy status")
        } catch {
            connectionTestResult = .failure(error.localizedDescription)
        }

        isTestingConnection = false
    }
}

#Preview {
    SettingsView()
        .environment(SettingsManager())
        .environment(SSEConnectionManager())
}
