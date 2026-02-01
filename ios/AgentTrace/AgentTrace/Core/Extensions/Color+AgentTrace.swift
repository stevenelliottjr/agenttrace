import SwiftUI

extension Color {
    // Brand colors
    static let agentTracePrimary = Color("Primary", bundle: nil)
    static let agentTraceSecondary = Color("Secondary", bundle: nil)

    // Semantic colors for span types
    static let spanAgent = Color.purple
    static let spanLLM = Color.blue
    static let spanTool = Color.green
    static let spanRetriever = Color.orange
    static let spanChain = Color.cyan
    static let spanEmbedding = Color.pink

    // Status colors
    static let statusSuccess = Color.green
    static let statusError = Color.red
    static let statusWarning = Color.orange
    static let statusRunning = Color.blue

    // Alert severity colors
    static let alertCritical = Color.red
    static let alertWarning = Color.orange
    static let alertInfo = Color.blue

    /// Get color for span type
    static func forSpanType(_ type: SpanType) -> Color {
        switch type {
        case .agent: return .spanAgent
        case .llm: return .spanLLM
        case .tool: return .spanTool
        case .retriever: return .spanRetriever
        case .chain: return .spanChain
        case .embedding: return .spanEmbedding
        case .unknown: return .gray
        }
    }

    /// Get color for trace status
    static func forTraceStatus(_ status: TraceStatus) -> Color {
        switch status {
        case .running: return .statusRunning
        case .completed: return .statusSuccess
        case .error: return .statusError
        }
    }

    /// Get color for alert severity
    static func forAlertSeverity(_ severity: AlertSeverity) -> Color {
        switch severity {
        case .critical: return .alertCritical
        case .warning: return .alertWarning
        case .info: return .alertInfo
        }
    }
}
