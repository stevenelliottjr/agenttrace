import Foundation

/// Defines all API endpoints for the AgentTrace backend
enum Endpoint {
    // Health
    case health

    // Traces
    case traces(page: Int, pageSize: Int, status: TraceStatus?, search: String?)
    case traceDetail(traceId: String)

    // Spans
    case spans(traceId: String?, page: Int, pageSize: Int)
    case spanDetail(spanId: String)

    // Metrics
    case metricsSummary(timeRange: TimeRange)
    case costBreakdown(timeRange: TimeRange, groupBy: String)
    case latencyMetrics(timeRange: TimeRange)
    case tokenUsage(timeRange: TimeRange)
    case errorMetrics(timeRange: TimeRange)

    // Alerts
    case alertRules
    case alertRule(ruleId: String)
    case alertEvents(status: AlertEventStatus?, severity: AlertSeverity?, page: Int, pageSize: Int)
    case acknowledgeAlert(eventId: String)

    // Streaming
    case stream

    var path: String {
        switch self {
        case .health:
            return "/api/v1/health"
        case .traces:
            return "/api/v1/traces"
        case .traceDetail(let traceId):
            return "/api/v1/traces/\(traceId)"
        case .spans:
            return "/api/v1/spans"
        case .spanDetail(let spanId):
            return "/api/v1/spans/\(spanId)"
        case .metricsSummary:
            return "/api/v1/metrics/summary"
        case .costBreakdown:
            return "/api/v1/metrics/costs"
        case .latencyMetrics:
            return "/api/v1/metrics/latency"
        case .tokenUsage:
            return "/api/v1/metrics/tokens"
        case .errorMetrics:
            return "/api/v1/metrics/errors"
        case .alertRules:
            return "/api/v1/alerts/rules"
        case .alertRule(let ruleId):
            return "/api/v1/alerts/rules/\(ruleId)"
        case .alertEvents:
            return "/api/v1/alerts/events"
        case .acknowledgeAlert(let eventId):
            return "/api/v1/alerts/events/\(eventId)/acknowledge"
        case .stream:
            return "/api/v1/stream"
        }
    }

    var method: HTTPMethod {
        switch self {
        case .acknowledgeAlert:
            return .post
        default:
            return .get
        }
    }

    var queryItems: [URLQueryItem]? {
        switch self {
        case .traces(let page, let pageSize, let status, let search):
            var items = [
                URLQueryItem(name: "page", value: String(page)),
                URLQueryItem(name: "page_size", value: String(pageSize))
            ]
            if let status = status {
                items.append(URLQueryItem(name: "status", value: status.rawValue))
            }
            if let search = search, !search.isEmpty {
                items.append(URLQueryItem(name: "search", value: search))
            }
            return items

        case .spans(let traceId, let page, let pageSize):
            var items = [
                URLQueryItem(name: "page", value: String(page)),
                URLQueryItem(name: "page_size", value: String(pageSize))
            ]
            if let traceId = traceId {
                items.append(URLQueryItem(name: "trace_id", value: traceId))
            }
            return items

        case .metricsSummary(let timeRange),
             .costBreakdown(let timeRange, _),
             .latencyMetrics(let timeRange),
             .tokenUsage(let timeRange),
             .errorMetrics(let timeRange):
            var items = [URLQueryItem(name: "range", value: timeRange.rawValue)]
            if case .costBreakdown(_, let groupBy) = self {
                items.append(URLQueryItem(name: "group_by", value: groupBy))
            }
            return items

        case .alertEvents(let status, let severity, let page, let pageSize):
            var items = [
                URLQueryItem(name: "page", value: String(page)),
                URLQueryItem(name: "page_size", value: String(pageSize))
            ]
            if let status = status {
                items.append(URLQueryItem(name: "status", value: status.rawValue))
            }
            if let severity = severity {
                items.append(URLQueryItem(name: "severity", value: severity.rawValue))
            }
            return items

        default:
            return nil
        }
    }

    func url(baseURL: URL) -> URL? {
        var components = URLComponents(url: baseURL.appendingPathComponent(path), resolvingAgainstBaseURL: true)
        components?.queryItems = queryItems
        return components?.url
    }
}

enum HTTPMethod: String {
    case get = "GET"
    case post = "POST"
    case put = "PUT"
    case delete = "DELETE"
    case patch = "PATCH"
}
