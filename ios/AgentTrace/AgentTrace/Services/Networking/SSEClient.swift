import Foundation

/// Events received from the SSE stream
enum SSEEvent {
    case span(Span)
    case traceStart(TraceSummary)
    case traceComplete(TraceSummary)
    case alert(AlertEvent)
    case connected
    case disconnected(Error?)
}

/// Client for handling Server-Sent Events from the AgentTrace backend
actor SSEClient {
    private var task: Task<Void, Never>?
    private var isConnected = false
    private let decoder: JSONDecoder

    private var eventContinuation: AsyncStream<SSEEvent>.Continuation?

    var baseURL: URL

    init(baseURL: URL = URL(string: "http://localhost:8080")!) {
        self.baseURL = baseURL
        self.decoder = JSONDecoder()
        self.decoder.dateDecodingStrategy = .custom { decoder in
            let container = try decoder.singleValueContainer()
            let dateString = try container.decode(String.self)

            let formatter = ISO8601DateFormatter()
            formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            if let date = formatter.date(from: dateString) {
                return date
            }

            formatter.formatOptions = [.withInternetDateTime]
            if let date = formatter.date(from: dateString) {
                return date
            }

            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Cannot decode date string \(dateString)"
            )
        }
    }

    func updateBaseURL(_ url: URL) {
        self.baseURL = url
    }

    /// Connect to the SSE stream and return an async stream of events
    func connect() -> AsyncStream<SSEEvent> {
        // Cancel any existing connection
        task?.cancel()

        return AsyncStream { continuation in
            self.eventContinuation = continuation

            task = Task { [weak self] in
                guard let self = self else { return }

                do {
                    await self.setConnected(true)
                    continuation.yield(.connected)

                    let url = self.baseURL.appendingPathComponent("/api/v1/stream")
                    var request = URLRequest(url: url)
                    request.setValue("text/event-stream", forHTTPHeaderField: "Accept")
                    request.timeoutInterval = .infinity

                    let (bytes, response) = try await URLSession.shared.bytes(for: request)

                    guard let httpResponse = response as? HTTPURLResponse,
                          (200...299).contains(httpResponse.statusCode) else {
                        throw APIError.invalidResponse
                    }

                    var eventType: String?
                    var dataBuffer: String = ""

                    for try await line in bytes.lines {
                        if Task.isCancelled {
                            break
                        }

                        if line.isEmpty {
                            // End of event - process it
                            if !dataBuffer.isEmpty {
                                await self.processEvent(type: eventType, data: dataBuffer, continuation: continuation)
                            }
                            eventType = nil
                            dataBuffer = ""
                        } else if line.hasPrefix("event:") {
                            eventType = String(line.dropFirst(6)).trimmingCharacters(in: .whitespaces)
                        } else if line.hasPrefix("data:") {
                            let data = String(line.dropFirst(5)).trimmingCharacters(in: .whitespaces)
                            if dataBuffer.isEmpty {
                                dataBuffer = data
                            } else {
                                dataBuffer += "\n" + data
                            }
                        }
                        // Ignore id: and retry: lines for now
                    }
                } catch {
                    if !Task.isCancelled {
                        continuation.yield(.disconnected(error))
                    }
                }

                await self.setConnected(false)
                continuation.yield(.disconnected(nil))
                continuation.finish()
            }

            continuation.onTermination = { @Sendable _ in
                Task { [weak self] in
                    await self?.disconnect()
                }
            }
        }
    }

    func disconnect() {
        task?.cancel()
        task = nil
        isConnected = false
        eventContinuation?.finish()
        eventContinuation = nil
    }

    private func setConnected(_ connected: Bool) {
        isConnected = connected
    }

    func getConnectionStatus() -> Bool {
        return isConnected
    }

    private func processEvent(type: String?, data: String, continuation: AsyncStream<SSEEvent>.Continuation) {
        guard let jsonData = data.data(using: .utf8) else { return }

        do {
            switch type {
            case "span", nil:
                let span = try decoder.decode(Span.self, from: jsonData)
                continuation.yield(.span(span))

            case "trace_start":
                let trace = try decoder.decode(TraceSummary.self, from: jsonData)
                continuation.yield(.traceStart(trace))

            case "trace_complete":
                let trace = try decoder.decode(TraceSummary.self, from: jsonData)
                continuation.yield(.traceComplete(trace))

            case "alert":
                let alert = try decoder.decode(AlertEvent.self, from: jsonData)
                continuation.yield(.alert(alert))

            default:
                // Unknown event type, try to parse as span
                if let span = try? decoder.decode(Span.self, from: jsonData) {
                    continuation.yield(.span(span))
                }
            }
        } catch {
            // Log decoding errors but don't crash
            print("SSE decoding error: \(error)")
        }
    }
}

/// Observable wrapper for SSE connection state
@Observable
final class SSEConnectionManager {
    var isConnected = false
    var lastEvent: SSEEvent?
    var recentSpans: [Span] = []
    var activeTraces: [TraceSummary] = []

    private let client: SSEClient
    private var connectionTask: Task<Void, Never>?
    private let maxRecentSpans = 100

    init(client: SSEClient = SSEClient()) {
        self.client = client
    }

    @MainActor
    func connect() {
        connectionTask?.cancel()

        connectionTask = Task {
            let stream = await client.connect()

            for await event in stream {
                await MainActor.run {
                    self.handleEvent(event)
                }
            }
        }
    }

    func disconnect() {
        connectionTask?.cancel()
        connectionTask = nil
        Task {
            await client.disconnect()
        }
        isConnected = false
    }

    func updateBaseURL(_ url: URL) async {
        await client.updateBaseURL(url)
    }

    private func handleEvent(_ event: SSEEvent) {
        lastEvent = event

        switch event {
        case .connected:
            isConnected = true

        case .disconnected:
            isConnected = false

        case .span(let span):
            recentSpans.insert(span, at: 0)
            if recentSpans.count > maxRecentSpans {
                recentSpans.removeLast()
            }

        case .traceStart(let trace):
            activeTraces.insert(trace, at: 0)

        case .traceComplete(let trace):
            activeTraces.removeAll { $0.id == trace.id }

        case .alert:
            // Alerts handled separately
            break
        }
    }
}
