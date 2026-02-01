import Foundation

/// Manages data shared between the main app, widgets, and Live Activities
/// via App Group container
final class AppGroupStorage {
    static let shared = AppGroupStorage()

    private let suiteName = "group.com.agenttrace.app"
    private let defaults: UserDefaults?

    private init() {
        defaults = UserDefaults(suiteName: suiteName)
    }

    // MARK: - Server Configuration

    var serverURL: String {
        get {
            defaults?.string(forKey: Keys.serverURL) ?? "http://localhost:8080"
        }
        set {
            defaults?.set(newValue, forKey: Keys.serverURL)
        }
    }

    // MARK: - Cached Metrics (for widgets)

    var cachedMetrics: CachedWidgetMetrics? {
        get {
            guard let data = defaults?.data(forKey: Keys.cachedMetrics) else { return nil }
            return try? JSONDecoder().decode(CachedWidgetMetrics.self, from: data)
        }
        set {
            if let newValue = newValue {
                let data = try? JSONEncoder().encode(newValue)
                defaults?.set(data, forKey: Keys.cachedMetrics)
            } else {
                defaults?.removeObject(forKey: Keys.cachedMetrics)
            }
        }
    }

    // MARK: - Active Trace (for Live Activities)

    var activeTraceId: String? {
        get {
            defaults?.string(forKey: Keys.activeTraceId)
        }
        set {
            defaults?.set(newValue, forKey: Keys.activeTraceId)
        }
    }

    var activeTraceName: String? {
        get {
            defaults?.string(forKey: Keys.activeTraceName)
        }
        set {
            defaults?.set(newValue, forKey: Keys.activeTraceName)
        }
    }

    // MARK: - Keys

    private enum Keys {
        static let serverURL = "serverURL"
        static let cachedMetrics = "cachedMetrics"
        static let activeTraceId = "activeTraceId"
        static let activeTraceName = "activeTraceName"
    }
}

/// Metrics data cached for widget display
struct CachedWidgetMetrics: Codable {
    let totalCost: Double
    let totalTokens: Int
    let avgLatencyMs: Double
    let errorRate: Double
    let activeTraces: Int
    let cachedAt: Date

    var isExpired: Bool {
        // Consider expired after 10 minutes
        Date().timeIntervalSince(cachedAt) > 600
    }
}
