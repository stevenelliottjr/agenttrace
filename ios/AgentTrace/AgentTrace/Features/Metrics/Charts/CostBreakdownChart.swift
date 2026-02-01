import SwiftUI
import Charts

struct CostBreakdownChart: View {
    let data: [CostBreakdown]
    let isLoading: Bool

    private var totalCost: Double {
        data.reduce(0) { $0 + $1.cost }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Cost by Model")
                    .font(.headline)
                Spacer()
                Text(String(format: "$%.2f total", totalCost))
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }

            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if data.isEmpty {
                ContentUnavailableView(
                    "No Cost Data",
                    systemImage: "dollarsign.circle",
                    description: Text("Cost data will appear here")
                )
            } else {
                HStack(spacing: 20) {
                    // Pie chart
                    Chart(data) { item in
                        SectorMark(
                            angle: .value("Cost", item.cost),
                            innerRadius: .ratio(0.5),
                            angularInset: 1.0
                        )
                        .foregroundStyle(by: .value("Model", item.name))
                        .cornerRadius(4)
                    }
                    .chartLegend(.hidden)
                    .frame(width: 150, height: 150)

                    // Legend with values
                    VStack(alignment: .leading, spacing: 8) {
                        ForEach(data) { item in
                            CostLegendRow(item: item)
                        }
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

struct CostLegendRow: View {
    let item: CostBreakdown

    var body: some View {
        HStack {
            Circle()
                .fill(colorForModel(item.name))
                .frame(width: 10, height: 10)

            Text(item.name)
                .font(.caption)
                .lineLimit(1)

            Spacer()

            VStack(alignment: .trailing) {
                Text(item.formattedCost)
                    .font(.caption)
                    .fontWeight(.medium)

                Text(item.formattedPercentage)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private func colorForModel(_ name: String) -> Color {
        // Generate consistent colors for models
        let hash = abs(name.hashValue)
        let colors: [Color] = [.blue, .green, .orange, .purple, .pink, .cyan, .red, .yellow]
        return colors[hash % colors.count]
    }
}

#Preview {
    CostBreakdownChart(
        data: [
            CostBreakdown(name: "gpt-4", cost: 1.50, percentage: 0.6, tokenCount: 10000),
            CostBreakdown(name: "gpt-3.5-turbo", cost: 0.50, percentage: 0.2, tokenCount: 50000),
            CostBreakdown(name: "claude-3-opus", cost: 0.50, percentage: 0.2, tokenCount: 5000)
        ],
        isLoading: false
    )
    .frame(height: 300)
}
