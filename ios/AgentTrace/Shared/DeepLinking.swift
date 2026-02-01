import Foundation

/// Handles deep link URL parsing for AgentTrace app
/// Supported URLs:
/// - agenttrace://trace/{traceId}
/// - agenttrace://span/{spanId}
/// - agenttrace://alert/{alertEventId}
/// - agenttrace://dashboard
/// - agenttrace://metrics

enum DeepLink: Equatable {
    case trace(traceId: String)
    case span(spanId: String)
    case alert(alertEventId: String)
    case dashboard
    case metrics
    case traces
    case alerts
    case settings

    /// Parse a URL into a DeepLink
    static func from(url: URL) -> DeepLink? {
        guard url.scheme == "agenttrace" else { return nil }

        let host = url.host
        let pathComponents = url.pathComponents.filter { $0 != "/" }

        switch host {
        case "trace":
            if let traceId = pathComponents.first {
                return .trace(traceId: traceId)
            }
        case "span":
            if let spanId = pathComponents.first {
                return .span(spanId: spanId)
            }
        case "alert":
            if let alertId = pathComponents.first {
                return .alert(alertEventId: alertId)
            }
        case "dashboard":
            return .dashboard
        case "metrics":
            return .metrics
        case "traces":
            return .traces
        case "alerts":
            return .alerts
        case "settings":
            return .settings
        default:
            break
        }

        return nil
    }

    /// Generate a URL for this deep link
    var url: URL {
        switch self {
        case .trace(let traceId):
            return URL(string: "agenttrace://trace/\(traceId)")!
        case .span(let spanId):
            return URL(string: "agenttrace://span/\(spanId)")!
        case .alert(let alertEventId):
            return URL(string: "agenttrace://alert/\(alertEventId)")!
        case .dashboard:
            return URL(string: "agenttrace://dashboard")!
        case .metrics:
            return URL(string: "agenttrace://metrics")!
        case .traces:
            return URL(string: "agenttrace://traces")!
        case .alerts:
            return URL(string: "agenttrace://alerts")!
        case .settings:
            return URL(string: "agenttrace://settings")!
        }
    }
}

/// Navigation coordinator that handles deep links
import SwiftUI

@Observable
final class NavigationCoordinator {
    var selectedTab: ContentView.Tab = .dashboard
    var traceNavigationPath = NavigationPath()
    var alertNavigationPath = NavigationPath()

    var pendingDeepLink: DeepLink?

    func handle(_ deepLink: DeepLink) {
        switch deepLink {
        case .dashboard:
            selectedTab = .dashboard

        case .traces:
            selectedTab = .traces
            traceNavigationPath = NavigationPath()

        case .trace(let traceId):
            selectedTab = .traces
            traceNavigationPath = NavigationPath()
            traceNavigationPath.append(TraceNavigation.detail(traceId: traceId))

        case .span(let spanId):
            // For span deep links, we need to fetch the span to get its trace ID
            // For now, store as pending and handle when we have more context
            pendingDeepLink = deepLink

        case .metrics:
            selectedTab = .metrics

        case .alerts:
            selectedTab = .alerts
            alertNavigationPath = NavigationPath()

        case .alert(let alertEventId):
            selectedTab = .alerts
            alertNavigationPath = NavigationPath()
            alertNavigationPath.append(AlertNavigation.detail(alertEventId: alertEventId))

        case .settings:
            selectedTab = .settings
        }
    }
}

/// Navigation destinations for traces
enum TraceNavigation: Hashable {
    case detail(traceId: String)
    case span(spanId: String)
}

/// Navigation destinations for alerts
enum AlertNavigation: Hashable {
    case detail(alertEventId: String)
}
