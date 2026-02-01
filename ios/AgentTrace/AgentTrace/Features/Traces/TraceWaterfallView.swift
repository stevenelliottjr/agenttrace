import SwiftUI

struct TraceWaterfallView: View {
    let trace: Trace
    let onSpanSelected: (Span) -> Void

    private var spanNodes: [SpanNode] {
        trace.buildSpanTree().flatMap { $0.flatten() }
    }

    private var traceStartTime: Date {
        trace.startTime
    }

    private var traceDurationUs: Int64 {
        trace.durationUs ?? 1
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Span Timeline")
                    .font(.headline)
                Spacer()
                Text("\(spanNodes.count) spans")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            // Timeline header
            TimelineHeader(durationUs: traceDurationUs)

            // Span rows
            ScrollView {
                LazyVStack(spacing: 4) {
                    ForEach(spanNodes) { node in
                        WaterfallRow(
                            node: node,
                            traceStartTime: traceStartTime,
                            traceDurationUs: traceDurationUs,
                            onTap: { onSpanSelected(node.span) }
                        )
                    }
                }
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

// MARK: - Timeline Header

struct TimelineHeader: View {
    let durationUs: Int64

    private var markers: [String] {
        let durationMs = Double(durationUs) / 1000.0
        if durationMs < 1000 {
            return ["0", "\(Int(durationMs / 4))ms", "\(Int(durationMs / 2))ms", "\(Int(durationMs * 3 / 4))ms", "\(Int(durationMs))ms"]
        } else {
            let durationS = durationMs / 1000
            return ["0", String(format: "%.1fs", durationS / 4), String(format: "%.1fs", durationS / 2), String(format: "%.1fs", durationS * 3 / 4), String(format: "%.1fs", durationS)]
        }
    }

    var body: some View {
        GeometryReader { geometry in
            let infoWidth: CGFloat = 120
            let timelineWidth = geometry.size.width - infoWidth

            HStack(spacing: 0) {
                Rectangle()
                    .fill(Color.clear)
                    .frame(width: infoWidth)

                ZStack(alignment: .leading) {
                    // Timeline background
                    Rectangle()
                        .fill(Color(.systemGray5))
                        .frame(height: 20)

                    // Markers
                    ForEach(Array(markers.enumerated()), id: \.offset) { index, marker in
                        Text(marker)
                            .font(.system(size: 8))
                            .foregroundStyle(.secondary)
                            .position(
                                x: timelineWidth * CGFloat(index) / CGFloat(markers.count - 1),
                                y: 10
                            )
                    }
                }
                .frame(width: timelineWidth)
            }
        }
        .frame(height: 24)
    }
}

// MARK: - Waterfall Row

struct WaterfallRow: View {
    let node: SpanNode
    let traceStartTime: Date
    let traceDurationUs: Int64
    let onTap: () -> Void

    private var span: Span { node.span }

    private var offsetPercent: CGFloat {
        let spanStartUs = span.startTime.timeIntervalSince(traceStartTime) * 1_000_000
        return CGFloat(spanStartUs / Double(traceDurationUs))
    }

    private var widthPercent: CGFloat {
        guard let durationUs = span.durationUs else { return 0.01 }
        let percent = CGFloat(Double(durationUs) / Double(traceDurationUs))
        return max(percent, 0.01) // Minimum width for visibility
    }

    var body: some View {
        GeometryReader { geometry in
            let infoWidth: CGFloat = 120
            let timelineWidth = geometry.size.width - infoWidth

            HStack(spacing: 0) {
                // Span info
                HStack(spacing: 4) {
                    // Indentation
                    ForEach(0..<node.depth, id: \.self) { _ in
                        Rectangle()
                            .fill(Color(.systemGray4))
                            .frame(width: 2)
                            .padding(.leading, 4)
                    }

                    Image(systemName: span.spanType.iconName)
                        .foregroundStyle(Color.forSpanType(span.spanType))
                        .font(.caption)
                        .frame(width: 16)

                    Text(span.name)
                        .font(.caption2)
                        .lineLimit(1)
                        .truncationMode(.tail)
                }
                .frame(width: infoWidth, alignment: .leading)

                // Timeline bar
                ZStack(alignment: .leading) {
                    // Background track
                    Rectangle()
                        .fill(Color(.systemGray5))
                        .frame(height: 24)

                    // Span bar
                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color.forSpanType(span.spanType))
                        .frame(width: timelineWidth * widthPercent, height: 18)
                        .offset(x: timelineWidth * offsetPercent)
                        .overlay(alignment: .leading) {
                            if widthPercent > 0.1 {
                                Text(span.formattedDuration)
                                    .font(.system(size: 8))
                                    .foregroundStyle(.white)
                                    .padding(.leading, timelineWidth * offsetPercent + 4)
                            }
                        }

                    // Error indicator
                    if span.status == .error {
                        Image(systemName: "exclamationmark.triangle.fill")
                            .foregroundStyle(.red)
                            .font(.system(size: 10))
                            .offset(x: timelineWidth * offsetPercent + timelineWidth * widthPercent + 4)
                    }
                }
                .frame(width: timelineWidth)
            }
        }
        .frame(height: 28)
        .contentShape(Rectangle())
        .onTapGesture(perform: onTap)
    }
}

// MARK: - Span Detail Sheet

struct SpanDetailSheet: View {
    let span: Span
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    // Header
                    SpanHeaderSection(span: span)

                    // Metrics
                    SpanMetricsSection(span: span)

                    // Token usage
                    if let tokenUsage = span.tokenUsage {
                        TokenUsageSection(usage: tokenUsage, cost: span.cost)
                    }

                    // Input
                    if let input = span.input, !input.isEmpty {
                        ExpandableTextSection(title: "Input", content: input, icon: "arrow.right.circle")
                    }

                    // Output
                    if let output = span.output, !output.isEmpty {
                        ExpandableTextSection(title: "Output", content: output, icon: "arrow.left.circle")
                    }

                    // Error
                    if let errorMessage = span.errorMessage {
                        ErrorSection(message: errorMessage)
                    }

                    // Attributes
                    if let attributes = span.attributes, !attributes.isEmpty {
                        AttributesSection(attributes: attributes)
                    }
                }
                .padding()
            }
            .navigationTitle("Span Details")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") { dismiss() }
                }
            }
        }
        .presentationDetents([.medium, .large])
        .presentationDragIndicator(.visible)
    }
}

// MARK: - Span Detail Sections

struct SpanHeaderSection: View {
    let span: Span

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: span.spanType.iconName)
                    .foregroundStyle(Color.forSpanType(span.spanType))

                Text(span.name)
                    .font(.headline)

                Spacer()

                StatusBadge(status: span.status)
            }

            if let model = span.model {
                HStack {
                    Text("Model:")
                        .foregroundStyle(.secondary)
                    Text(model)
                        .fontWeight(.medium)
                }
                .font(.caption)
            }

            if let provider = span.provider {
                HStack {
                    Text("Provider:")
                        .foregroundStyle(.secondary)
                    Text(provider)
                }
                .font(.caption)
            }

            // IDs
            Group {
                CopyableIdRow(label: "Span ID", value: span.id)
                CopyableIdRow(label: "Trace ID", value: span.traceId)
                if let parentId = span.parentSpanId {
                    CopyableIdRow(label: "Parent", value: parentId)
                }
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct StatusBadge: View {
    let status: SpanStatus

    var body: some View {
        Text(status.rawValue.uppercased())
            .font(.caption2)
            .fontWeight(.bold)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(status == .error ? Color.red : (status == .ok ? Color.green : Color.gray))
            .foregroundStyle(.white)
            .clipShape(Capsule())
    }
}

struct CopyableIdRow: View {
    let label: String
    let value: String

    var body: some View {
        HStack {
            Text("\(label):")
                .foregroundStyle(.secondary)
            Text(value)
                .monospaced()
                .lineLimit(1)
                .truncationMode(.middle)

            Button {
                UIPasteboard.general.string = value
            } label: {
                Image(systemName: "doc.on.doc")
            }
        }
        .font(.caption)
    }
}

struct SpanMetricsSection: View {
    let span: Span

    var body: some View {
        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
            MetricBadge(title: "Duration", value: span.formattedDuration, icon: "clock")
            MetricBadge(title: "Cost", value: span.formattedCost, icon: "dollarsign.circle")
        }
    }
}

struct TokenUsageSection: View {
    let usage: TokenUsage
    let cost: CostInfo?

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Token Usage")
                .font(.headline)

            HStack(spacing: 16) {
                VStack {
                    Text("\(usage.promptTokens)")
                        .font(.title3)
                        .fontWeight(.bold)
                    Text("Prompt")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Image(systemName: "plus")
                    .foregroundStyle(.secondary)

                VStack {
                    Text("\(usage.completionTokens)")
                        .font(.title3)
                        .fontWeight(.bold)
                    Text("Completion")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Image(systemName: "equal")
                    .foregroundStyle(.secondary)

                VStack {
                    Text("\(usage.totalTokens)")
                        .font(.title3)
                        .fontWeight(.bold)
                        .foregroundStyle(.blue)
                    Text("Total")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            .frame(maxWidth: .infinity)
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct ExpandableTextSection: View {
    let title: String
    let content: String
    let icon: String
    @State private var isExpanded = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Button {
                withAnimation {
                    isExpanded.toggle()
                }
            } label: {
                HStack {
                    Image(systemName: icon)
                    Text(title)
                        .font(.headline)
                    Spacer()
                    Image(systemName: isExpanded ? "chevron.up" : "chevron.down")
                        .foregroundStyle(.secondary)
                }
            }
            .buttonStyle(.plain)

            if isExpanded {
                ScrollView {
                    Text(content)
                        .font(.caption)
                        .monospaced()
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(maxHeight: 200)
                .padding(8)
                .background(Color(.systemGray5))
                .clipShape(RoundedRectangle(cornerRadius: 8))
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct ErrorSection: View {
    let message: String

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.red)
                Text("Error")
                    .font(.headline)
            }

            Text(message)
                .font(.caption)
                .foregroundStyle(.red)
                .padding(8)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.red.opacity(0.1))
                .clipShape(RoundedRectangle(cornerRadius: 8))
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct AttributesSection: View {
    let attributes: [String: AttributeValue]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Attributes")
                .font(.headline)

            ForEach(Array(attributes.keys.sorted()), id: \.self) { key in
                if let value = attributes[key] {
                    HStack(alignment: .top) {
                        Text(key)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .frame(width: 100, alignment: .leading)

                        Text(value.displayValue)
                            .font(.caption)
                            .monospaced()
                            .textSelection(.enabled)
                    }
                }
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

#Preview {
    TraceWaterfallView(
        trace: Trace(
            id: "test",
            name: "Test Trace",
            status: .completed,
            startTime: Date(),
            endTime: Date(),
            durationUs: 5000000,
            spanCount: 3,
            errorCount: 0,
            totalTokens: 1000,
            totalCost: 0.05,
            rootSpan: nil,
            spans: [],
            metadata: nil
        ),
        onSpanSelected: { _ in }
    )
}
