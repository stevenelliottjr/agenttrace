import SwiftUI
import Charts

struct TokenUsageChart: View {
    let data: [TokenUsageMetrics]
    let isLoading: Bool

    private var totalTokens: Int {
        data.reduce(0) { $0 + $1.totalTokens }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Token Usage")
                    .font(.headline)
                Spacer()
                Text(formatTokens(totalTokens) + " total")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }

            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if data.isEmpty {
                ContentUnavailableView(
                    "No Token Data",
                    systemImage: "number",
                    description: Text("Token usage data will appear here")
                )
            } else {
                Chart {
                    ForEach(data) { item in
                        // Prompt tokens (bottom)
                        BarMark(
                            x: .value("Time", item.timestamp),
                            y: .value("Prompt", item.promptTokens)
                        )
                        .foregroundStyle(.blue)

                        // Completion tokens (stacked on top)
                        BarMark(
                            x: .value("Time", item.timestamp),
                            y: .value("Completion", item.completionTokens)
                        )
                        .foregroundStyle(.green)
                    }
                }
                .chartXAxis {
                    AxisMarks(values: .automatic(desiredCount: 5)) { _ in
                        AxisGridLine()
                        AxisValueLabel(format: .dateTime.hour().minute())
                    }
                }
                .chartYAxis {
                    AxisMarks { value in
                        AxisGridLine()
                        AxisValueLabel {
                            if let tokens = value.as(Int.self) {
                                Text(formatTokens(tokens))
                            }
                        }
                    }
                }

                // Legend
                HStack(spacing: 16) {
                    LegendItem(color: .blue, label: "Prompt Tokens")
                    LegendItem(color: .green, label: "Completion Tokens")
                }
                .font(.caption)
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }

    private func formatTokens(_ count: Int) -> String {
        if count >= 1_000_000 {
            return String(format: "%.1fM", Double(count) / 1_000_000)
        } else if count >= 1_000 {
            return String(format: "%.1fK", Double(count) / 1_000)
        } else {
            return "\(count)"
        }
    }
}

#Preview {
    TokenUsageChart(
        data: (0..<10).map { i in
            TokenUsageMetrics(
                timestamp: Date().addingTimeInterval(Double(i) * -3600),
                promptTokens: Int.random(in: 1000...5000),
                completionTokens: Int.random(in: 500...2000),
                totalTokens: Int.random(in: 1500...7000)
            )
        },
        isLoading: false
    )
    .frame(height: 300)
}
