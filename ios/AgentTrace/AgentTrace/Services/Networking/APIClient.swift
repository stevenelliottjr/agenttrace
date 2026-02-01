import Foundation

/// Errors that can occur during API calls
enum APIError: Error, LocalizedError {
    case invalidURL
    case invalidResponse
    case httpError(statusCode: Int, message: String?)
    case decodingError(Error)
    case networkError(Error)
    case serverUnreachable

    var errorDescription: String? {
        switch self {
        case .invalidURL:
            return "Invalid URL"
        case .invalidResponse:
            return "Invalid response from server"
        case .httpError(let statusCode, let message):
            return "HTTP \(statusCode): \(message ?? "Unknown error")"
        case .decodingError(let error):
            return "Failed to decode response: \(error.localizedDescription)"
        case .networkError(let error):
            return "Network error: \(error.localizedDescription)"
        case .serverUnreachable:
            return "Server is unreachable"
        }
    }
}

/// Actor responsible for making API calls to the AgentTrace backend
actor APIClient {
    private let session: URLSession
    private let decoder: JSONDecoder
    private let encoder: JSONEncoder

    var baseURL: URL

    static let shared = APIClient()

    init(baseURL: URL = URL(string: "http://localhost:8080")!) {
        self.baseURL = baseURL

        let configuration = URLSessionConfiguration.default
        configuration.timeoutIntervalForRequest = 30
        configuration.timeoutIntervalForResource = 60
        self.session = URLSession(configuration: configuration)

        self.decoder = JSONDecoder()
        self.decoder.dateDecodingStrategy = .custom { decoder in
            let container = try decoder.singleValueContainer()
            let dateString = try container.decode(String.self)

            // Try ISO8601 with fractional seconds
            let formatter = ISO8601DateFormatter()
            formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            if let date = formatter.date(from: dateString) {
                return date
            }

            // Try ISO8601 without fractional seconds
            formatter.formatOptions = [.withInternetDateTime]
            if let date = formatter.date(from: dateString) {
                return date
            }

            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Cannot decode date string \(dateString)"
            )
        }

        self.encoder = JSONEncoder()
        self.encoder.dateEncodingStrategy = .iso8601
    }

    func updateBaseURL(_ url: URL) {
        self.baseURL = url
    }

    // MARK: - Generic Request Methods

    func request<T: Decodable>(_ endpoint: Endpoint) async throws -> T {
        guard let url = endpoint.url(baseURL: baseURL) else {
            throw APIError.invalidURL
        }

        var request = URLRequest(url: url)
        request.httpMethod = endpoint.method.rawValue
        request.setValue("application/json", forHTTPHeaderField: "Accept")

        do {
            let (data, response) = try await session.data(for: request)

            guard let httpResponse = response as? HTTPURLResponse else {
                throw APIError.invalidResponse
            }

            guard (200...299).contains(httpResponse.statusCode) else {
                let message = String(data: data, encoding: .utf8)
                throw APIError.httpError(statusCode: httpResponse.statusCode, message: message)
            }

            do {
                return try decoder.decode(T.self, from: data)
            } catch {
                throw APIError.decodingError(error)
            }
        } catch let error as APIError {
            throw error
        } catch {
            throw APIError.networkError(error)
        }
    }

    func requestWithBody<T: Decodable, B: Encodable>(_ endpoint: Endpoint, body: B) async throws -> T {
        guard let url = endpoint.url(baseURL: baseURL) else {
            throw APIError.invalidURL
        }

        var request = URLRequest(url: url)
        request.httpMethod = endpoint.method.rawValue
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("application/json", forHTTPHeaderField: "Accept")
        request.httpBody = try encoder.encode(body)

        do {
            let (data, response) = try await session.data(for: request)

            guard let httpResponse = response as? HTTPURLResponse else {
                throw APIError.invalidResponse
            }

            guard (200...299).contains(httpResponse.statusCode) else {
                let message = String(data: data, encoding: .utf8)
                throw APIError.httpError(statusCode: httpResponse.statusCode, message: message)
            }

            return try decoder.decode(T.self, from: data)
        } catch let error as APIError {
            throw error
        } catch let error as EncodingError {
            throw APIError.decodingError(error)
        } catch {
            throw APIError.networkError(error)
        }
    }

    func requestVoid(_ endpoint: Endpoint) async throws {
        guard let url = endpoint.url(baseURL: baseURL) else {
            throw APIError.invalidURL
        }

        var request = URLRequest(url: url)
        request.httpMethod = endpoint.method.rawValue
        request.setValue("application/json", forHTTPHeaderField: "Accept")

        do {
            let (data, response) = try await session.data(for: request)

            guard let httpResponse = response as? HTTPURLResponse else {
                throw APIError.invalidResponse
            }

            guard (200...299).contains(httpResponse.statusCode) else {
                let message = String(data: data, encoding: .utf8)
                throw APIError.httpError(statusCode: httpResponse.statusCode, message: message)
            }
        } catch let error as APIError {
            throw error
        } catch {
            throw APIError.networkError(error)
        }
    }

    // MARK: - Health Check

    func healthCheck() async throws -> Bool {
        do {
            let _: HealthResponse = try await request(.health)
            return true
        } catch {
            return false
        }
    }

    // MARK: - Traces

    func getTraces(
        page: Int = 1,
        pageSize: Int = 20,
        status: TraceStatus? = nil,
        search: String? = nil
    ) async throws -> PaginatedResponse<TraceSummary> {
        try await request(.traces(page: page, pageSize: pageSize, status: status, search: search))
    }

    func getTrace(traceId: String) async throws -> Trace {
        try await request(.traceDetail(traceId: traceId))
    }

    // MARK: - Spans

    func getSpans(traceId: String? = nil, page: Int = 1, pageSize: Int = 50) async throws -> PaginatedResponse<Span> {
        try await request(.spans(traceId: traceId, page: page, pageSize: pageSize))
    }

    func getSpan(spanId: String) async throws -> Span {
        try await request(.spanDetail(spanId: spanId))
    }

    // MARK: - Metrics

    func getMetricsSummary(timeRange: TimeRange = .last24Hours) async throws -> MetricsSummary {
        try await request(.metricsSummary(timeRange: timeRange))
    }

    func getCostBreakdown(timeRange: TimeRange = .last24Hours, groupBy: String = "model") async throws -> [CostBreakdown] {
        try await request(.costBreakdown(timeRange: timeRange, groupBy: groupBy))
    }

    func getLatencyMetrics(timeRange: TimeRange = .last24Hours) async throws -> [LatencyMetrics] {
        try await request(.latencyMetrics(timeRange: timeRange))
    }

    func getTokenUsage(timeRange: TimeRange = .last24Hours) async throws -> [TokenUsageMetrics] {
        try await request(.tokenUsage(timeRange: timeRange))
    }

    func getErrorMetrics(timeRange: TimeRange = .last24Hours) async throws -> [ErrorMetrics] {
        try await request(.errorMetrics(timeRange: timeRange))
    }

    // MARK: - Alerts

    func getAlertRules() async throws -> [AlertRule] {
        try await request(.alertRules)
    }

    func getAlertEvents(
        status: AlertEventStatus? = nil,
        severity: AlertSeverity? = nil,
        page: Int = 1,
        pageSize: Int = 20
    ) async throws -> PaginatedResponse<AlertEvent> {
        try await request(.alertEvents(status: status, severity: severity, page: page, pageSize: pageSize))
    }

    func acknowledgeAlert(eventId: String) async throws {
        try await requestVoid(.acknowledgeAlert(eventId: eventId))
    }
}

// MARK: - Response Types

struct HealthResponse: Codable {
    let status: String
    let version: String?
}
