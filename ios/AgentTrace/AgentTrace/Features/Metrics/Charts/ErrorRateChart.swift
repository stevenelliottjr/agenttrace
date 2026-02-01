import SwiftUI
import Charts

struct ErrorRateChart: View {
    let data: [ErrorMetrics]
    let isLoading: Bool

    private var averageErrorRate: Double {
        guard !data.isEmpty else { return 0 }
        return data.reduce(0) { $0 + $1.errorRate } / Double(data.count)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Error Rate")
                    .font(.headline)
                Spacer()
                Text(String(format: "%.2f%% avg", averageErrorRate * 100))
                    .font(.subheadline)
                    .foregroundStyle(averageErrorRate > 0.05 ? .red : .secondary)
            }

            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if data.isEmpty {
                ContentUnavailableView(
                    "No Error Data",
                    systemImage: "exclamationmark.triangle",
                    description: Text("Error rate data will appear here")
                )
            } else {
                Chart {
                    // Error rate line
                    ForEach(data) { item in
                        LineMark(
                            x: .value("Time", item.timestamp),
                            y: .value("Error Rate", item.errorRate * 100)
                        )
                        .foregroundStyle(item.errorRate > 0.05 ? .red : .orange)
                        .lineStyle(StrokeStyle(lineWidth: 2))

                        AreaMark(
                            x: .value("Time", item.timestamp),
                            y: .value("Error Rate", item.errorRate * 100)
                        )
                        .foregroundStyle(
                            LinearGradient(
                                colors: [.red.opacity(0.3), .clear],
                                startPoint: .top,
                                endPoint: .bottom
                            )
                        )

                        PointMark(
                            x: .value("Time", item.timestamp),
                            y: .value("Error Rate", item.errorRate * 100)
                        )
                        .foregroundStyle(item.errorRate > 0.05 ? .red : .orange)
                        .symbolSize(item.errorCount > 0 ? 30 : 10)
                    }

                    // Threshold line at 5%
                    RuleMark(y: .value("Threshold", 5))
                        .foregroundStyle(.red.opacity(0.5))
                        .lineStyle(StrokeStyle(lineWidth: 1, dash: [5, 5]))
                        .annotation(position: .trailing, alignment: .leading) {
                            Text("5%")
                                .font(.caption2)
                                .foregroundStyle(.red)
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
                            if let rate = value.as(Double.self) {
                                Text(String(format: "%.1f%%", rate))
                            }
                        }
                    }
                }
                .chartYScale(domain: 0...max(10, (data.map { $0.errorRate }.max() ?? 0) * 100 * 1.2))

                // Summary stats
                HStack(spacing: 20) {
                    VStack {
                        Text("\(data.reduce(0) { $0 + $1.errorCount })")
                            .font(.title3)
                            .fontWeight(.bold)
                            .foregroundStyle(.red)
                        Text("Total Errors")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    VStack {
                        Text("\(data.reduce(0) { $0 + $1.totalCount })")
                            .font(.title3)
                            .fontWeight(.bold)
                        Text("Total Spans")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    VStack {
                        Text(String(format: "%.1f%%", (data.map { $0.errorRate }.max() ?? 0) * 100))
                            .font(.title3)
                            .fontWeight(.bold)
                            .foregroundStyle(.orange)
                        Text("Peak Rate")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                .frame(maxWidth: .infinity)
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

#Preview {
    ErrorRateChart(
        data: (0..<10).map { i in
            ErrorMetrics(
                timestamp: Date().addingTimeInterval(Double(i) * -3600),
                totalCount: Int.random(in: 100...500),
                errorCount: Int.random(in: 0...10),
                errorRate: Double.random(in: 0...0.1)
            )
        },
        isLoading: false
    )
    .frame(height: 300)
}
