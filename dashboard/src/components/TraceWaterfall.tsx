'use client';

import { useMemo, useState } from 'react';
import { ChevronRight, ChevronDown } from 'lucide-react';
import type { Span, Trace } from '@/types';
import { SpanIcon, StatusIcon } from './SpanIcon';
import { formatDuration, cn } from '@/lib/utils';

interface TraceWaterfallProps {
  trace: Trace;
  onSpanSelect?: (span: Span) => void;
  selectedSpanId?: string;
}

interface SpanNode {
  span: Span;
  children: SpanNode[];
  depth: number;
}

export function TraceWaterfall({ trace, onSpanSelect, selectedSpanId }: TraceWaterfallProps) {
  const [collapsedSpans, setCollapsedSpans] = useState<Set<string>>(new Set());

  // Build span tree
  const spanTree = useMemo(() => {
    const spanMap = new Map<string, SpanNode>();
    const rootNodes: SpanNode[] = [];

    // Create nodes
    for (const span of trace.spans) {
      spanMap.set(span.span_id, { span, children: [], depth: 0 });
    }

    // Build tree
    for (const span of trace.spans) {
      const node = spanMap.get(span.span_id)!;
      if (span.parent_span_id) {
        const parent = spanMap.get(span.parent_span_id);
        if (parent) {
          parent.children.push(node);
        } else {
          rootNodes.push(node);
        }
      } else {
        rootNodes.push(node);
      }
    }

    // Calculate depths
    function setDepth(node: SpanNode, depth: number) {
      node.depth = depth;
      for (const child of node.children) {
        setDepth(child, depth + 1);
      }
    }
    for (const root of rootNodes) {
      setDepth(root, 0);
    }

    // Sort children by start time
    function sortChildren(node: SpanNode) {
      node.children.sort(
        (a, b) =>
          new Date(a.span.started_at).getTime() - new Date(b.span.started_at).getTime()
      );
      for (const child of node.children) {
        sortChildren(child);
      }
    }
    for (const root of rootNodes) {
      sortChildren(root);
    }

    return rootNodes;
  }, [trace.spans]);

  // Calculate time bounds
  const { minTime, maxTime } = useMemo(() => {
    let min = Infinity;
    let max = -Infinity;
    for (const span of trace.spans) {
      const start = new Date(span.started_at).getTime();
      const end = span.ended_at
        ? new Date(span.ended_at).getTime()
        : start + (span.duration_ms || 0);
      min = Math.min(min, start);
      max = Math.max(max, end);
    }
    return { minTime: min, maxTime: max };
  }, [trace.spans]);

  const totalDuration = maxTime - minTime;

  const toggleCollapse = (spanId: string) => {
    setCollapsedSpans((prev) => {
      const next = new Set(prev);
      if (next.has(spanId)) {
        next.delete(spanId);
      } else {
        next.add(spanId);
      }
      return next;
    });
  };

  const renderSpanRow = (node: SpanNode, isVisible: boolean): React.ReactNode[] => {
    const { span, children, depth } = node;
    const hasChildren = children.length > 0;
    const isCollapsed = collapsedSpans.has(span.span_id);
    const isSelected = selectedSpanId === span.span_id;

    const startTime = new Date(span.started_at).getTime();
    const endTime = span.ended_at
      ? new Date(span.ended_at).getTime()
      : startTime + (span.duration_ms || 0);

    const leftPercent = ((startTime - minTime) / totalDuration) * 100;
    const widthPercent = ((endTime - startTime) / totalDuration) * 100;

    const rows: React.ReactNode[] = [];

    if (isVisible) {
      rows.push(
        <div
          key={span.span_id}
          className={cn(
            'flex items-center border-b border-gray-100 hover:bg-gray-50 transition-colors cursor-pointer',
            isSelected && 'bg-brand-50 hover:bg-brand-100'
          )}
          onClick={() => onSpanSelect?.(span)}
        >
          {/* Span info column */}
          <div className="w-80 flex-shrink-0 px-3 py-2 flex items-center gap-1">
            <div style={{ width: depth * 20 }} className="flex-shrink-0" />
            {hasChildren ? (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  toggleCollapse(span.span_id);
                }}
                className="p-0.5 hover:bg-gray-200 rounded"
              >
                {isCollapsed ? (
                  <ChevronRight className="w-4 h-4 text-gray-500" />
                ) : (
                  <ChevronDown className="w-4 h-4 text-gray-500" />
                )}
              </button>
            ) : (
              <div className="w-5" />
            )}
            <SpanIcon span={span} size="sm" />
            <span className="ml-1 text-sm font-medium text-gray-900 truncate">
              {span.operation_name}
            </span>
            <StatusIcon status={span.status} size="sm" />
          </div>

          {/* Timeline column */}
          <div className="flex-1 px-3 py-2 relative h-10">
            <div className="absolute inset-y-2 inset-x-3 bg-gray-100 rounded">
              <div
                className={cn(
                  'absolute h-full rounded transition-all',
                  span.status === 'error' ? 'bg-red-400' : 'bg-brand-500'
                )}
                style={{
                  left: `${leftPercent}%`,
                  width: `${Math.max(widthPercent, 0.5)}%`,
                }}
              />
            </div>
          </div>

          {/* Duration column */}
          <div className="w-24 flex-shrink-0 px-3 py-2 text-right text-sm font-mono text-gray-600">
            {formatDuration(span.duration_ms)}
          </div>
        </div>
      );
    }

    // Render children
    if (!isCollapsed) {
      for (const child of children) {
        rows.push(...renderSpanRow(child, isVisible));
      }
    }

    return rows;
  };

  return (
    <div className="border border-gray-200 rounded-lg overflow-hidden bg-white">
      {/* Header */}
      <div className="flex items-center border-b border-gray-200 bg-gray-50 text-xs font-medium text-gray-500 uppercase tracking-wider">
        <div className="w-80 flex-shrink-0 px-3 py-2">Operation</div>
        <div className="flex-1 px-3 py-2">Timeline</div>
        <div className="w-24 flex-shrink-0 px-3 py-2 text-right">Duration</div>
      </div>

      {/* Span rows */}
      <div className="divide-y divide-gray-100">
        {spanTree.map((node) => renderSpanRow(node, true))}
      </div>

      {/* Time scale */}
      <div className="flex items-center border-t border-gray-200 bg-gray-50 text-xs text-gray-500">
        <div className="w-80 flex-shrink-0 px-3 py-1" />
        <div className="flex-1 px-3 py-1 flex justify-between">
          <span>0ms</span>
          <span>{formatDuration(totalDuration / 4)}</span>
          <span>{formatDuration(totalDuration / 2)}</span>
          <span>{formatDuration((totalDuration * 3) / 4)}</span>
          <span>{formatDuration(totalDuration)}</span>
        </div>
        <div className="w-24 flex-shrink-0" />
      </div>
    </div>
  );
}
