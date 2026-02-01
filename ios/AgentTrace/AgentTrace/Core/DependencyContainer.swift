import Foundation
import SwiftUI

/// Settings manager for app configuration
@Observable
final class SettingsManager {
    var serverURL: String {
        didSet {
            UserDefaults.standard.set(serverURL, forKey: "serverURL")
        }
    }

    var defaultTimeRange: TimeRange {
        didSet {
            UserDefaults.standard.set(defaultTimeRange.rawValue, forKey: "defaultTimeRange")
        }
    }

    var enableNotifications: Bool {
        didSet {
            UserDefaults.standard.set(enableNotifications, forKey: "enableNotifications")
        }
    }

    var enableLiveActivities: Bool {
        didSet {
            UserDefaults.standard.set(enableLiveActivities, forKey: "enableLiveActivities")
        }
    }

    init() {
        self.serverURL = UserDefaults.standard.string(forKey: "serverURL") ?? "http://localhost:8080"
        let timeRangeRaw = UserDefaults.standard.string(forKey: "defaultTimeRange") ?? TimeRange.last24Hours.rawValue
        self.defaultTimeRange = TimeRange(rawValue: timeRangeRaw) ?? .last24Hours
        self.enableNotifications = UserDefaults.standard.bool(forKey: "enableNotifications")
        self.enableLiveActivities = UserDefaults.standard.bool(forKey: "enableLiveActivities")
    }

    func resetToDefaults() {
        serverURL = "http://localhost:8080"
        defaultTimeRange = .last24Hours
        enableNotifications = true
        enableLiveActivities = true
    }
}

/// Environment key for SSE connection manager
extension EnvironmentValues {
    @Entry var sseConnectionManager: SSEConnectionManager = SSEConnectionManager()
}
