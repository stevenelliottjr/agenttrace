import SwiftUI
import Charts

struct LatencyChart: View {
    let data: [LatencyMetrics]
    let isLoading: Bool

    @State private var selectedPercentile: PercentileType = .p50

    enum PercentileType: String, CaseIterable {
        case p50 = "P50"
        case p95 = "P95"
        case p99 = "P99"
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Latency Over Time")
                    .font(.headline)
                Spacer()

                // Percentile selector
                Picker("Percentile", selection: $selectedPercentile) {
                    ForEach(PercentileType.allCases, id: \.self) { type in
                        Text(type.rawValue).tag(type)
                    }
                }
                .pickerStyle(.segmented)
                .frame(width: 150)
            }

            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if data.isEmpty {
                ContentUnavailableView(
                    "No Latency Data",
                    systemImage: "clock",
                    description: Text("Latency data will appear here")
                )
            } else {
                Chart {
                    ForEach(data) { item in
                        // P50 line
                        if selectedPercentile == .p50 || selectedPercentile == .p95 || selectedPercentile == .p99 {
                            LineMark(
                                x: .value("Time", item.timestamp),
                                y: .value("P50", item.p50)
                            )
                            .foregroundStyle(.blue)
                            .lineStyle(StrokeStyle(lineWidth: 2))
                            .symbol(.circle)
                        }

                        // P95 line
                        if selectedPercentile == .p95 || selectedPercentile == .p99 {
                            LineMark(
                                x: .value("Time", item.timestamp),
                                y: .value("P95", item.p95)
                            )
                            .foregroundStyle(.orange)
                            .lineStyle(StrokeStyle(lineWidth: 2))
                            .symbol(.circle)
                        }

                        // P99 line
                        if selectedPercentile == .p99 {
                            LineMark(
                                x: .value("Time", item.timestamp),
                                y: .value("P99", item.p99)
                            )
                            .foregroundStyle(.red)
                            .lineStyle(StrokeStyle(lineWidth: 2))
                            .symbol(.circle)
                        }

                        // Area between P50 and selected percentile
                        if selectedPercentile != .p50 {
                            AreaMark(
                                x: .value("Time", item.timestamp),
                                yStart: .value("P50", item.p50),
                                yEnd: .value("Upper", selectedPercentile == .p95 ? item.p95 : item.p99)
                            )
                            .foregroundStyle(
                                LinearGradient(
                                    colors: [.blue.opacity(0.3), .clear],
                                    startPoint: .top,
                                    endPoint: .bottom
                                )
                            )
                        }
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
                            if let ms = value.as(Double.self) {
                                Text(formatLatency(ms))
                            }
                        }
                    }
                }

                // Legend
                HStack(spacing: 16) {
                    LegendItem(color: .blue, label: "P50")
                    if selectedPercentile == .p95 || selectedPercentile == .p99 {
                        LegendItem(color: .orange, label: "P95")
                    }
                    if selectedPercentile == .p99 {
                        LegendItem(color: .red, label: "P99")
                    }
                }
                .font(.caption)
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }

    private func formatLatency(_ ms: Double) -> String {
        if ms < 1000 {
            return String(format: "%.0fms", ms)
        } else {
            return String(format: "%.1fs", ms / 1000)
        }
    }
}

struct LegendItem: View {
    let color: Color
    let label: String

    var body: some View {
        HStack(spacing: 4) {
            Circle()
                .fill(color)
                .frame(width: 8, height: 8)
            Text(label)
                .foregroundStyle(.secondary)
        }
    }
}

#Preview {
    LatencyChart(
        data: (0..<10).map { i in
            LatencyMetrics(
                timestamp: Date().addingTimeInterval(Double(i) * -3600),
                p50: Double.random(in: 100...200),
                p95: Double.random(in: 200...400),
                p99: Double.random(in: 400...800),
                avg: Double.random(in: 150...250),
                min: Double.random(in: 50...100),
                max: Double.random(in: 800...1200)
            )
        },
        isLoading: false
    )
    .frame(height: 300)
}
