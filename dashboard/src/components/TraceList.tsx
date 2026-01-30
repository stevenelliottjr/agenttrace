'use client';

import Link from 'next/link';
import { useRecentTraces } from '@/hooks/useSpans';
import { SpanIcon, StatusIcon } from './SpanIcon';
import { formatDuration, formatTokens, formatCost, formatRelativeTime } from '@/lib/utils';
import type { Trace } from '@/types';

export function TraceList() {
  const { data: traces, isLoading, error } = useRecentTraces(100);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-brand-600" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-lg bg-red-50 p-4 text-red-700">
        Failed to load traces: {error.message}
      </div>
    );
  }

  if (!traces || traces.length === 0) {
    return (
      <div className="rounded-lg bg-gray-50 p-8 text-center text-gray-500">
        <p className="text-lg font-medium">No traces yet</p>
        <p className="mt-1 text-sm">
          Start sending spans from your agent to see them here
        </p>
      </div>
    );
  }

  return (
    <div className="overflow-hidden rounded-lg border border-gray-200 bg-white">
      <table className="min-w-full divide-y divide-gray-200">
        <thead className="bg-gray-50">
          <tr>
            <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
              Trace
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
              Service
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
              Duration
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
              Tokens
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
              Cost
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
              Spans
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
              Time
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200 bg-white">
          {traces.map((trace) => (
            <TraceRow key={trace.trace_id} trace={trace} />
          ))}
        </tbody>
      </table>
    </div>
  );
}

function TraceRow({ trace }: { trace: Trace }) {
  const hasError = trace.spans.some((s) => s.status === 'error');
  const rootSpan = trace.root_span;

  return (
    <tr className="hover:bg-gray-50 transition-colors">
      <td className="px-4 py-3">
        <Link
          href={`/traces/${trace.trace_id}`}
          className="flex items-center gap-3 group"
        >
          {rootSpan && <SpanIcon span={rootSpan} size="sm" />}
          <div>
            <div className="flex items-center gap-2">
              <span className="font-medium text-gray-900 group-hover:text-brand-600">
                {rootSpan?.operation_name || 'Unknown'}
              </span>
              <StatusIcon status={hasError ? 'error' : 'ok'} size="sm" />
            </div>
            <div className="text-xs text-gray-500 font-mono">
              {trace.trace_id.slice(0, 8)}...
            </div>
          </div>
        </Link>
      </td>
      <td className="px-4 py-3">
        <span className="inline-flex items-center rounded-full bg-gray-100 px-2.5 py-0.5 text-xs font-medium text-gray-800">
          {trace.service_names[0] || 'unknown'}
        </span>
      </td>
      <td className="px-4 py-3 text-sm text-gray-900 font-mono">
        {formatDuration(trace.total_duration_ms)}
      </td>
      <td className="px-4 py-3 text-sm text-gray-900 font-mono">
        {formatTokens(trace.total_tokens)}
      </td>
      <td className="px-4 py-3 text-sm text-gray-900 font-mono">
        {formatCost(trace.total_cost_usd)}
      </td>
      <td className="px-4 py-3">
        <span className="inline-flex items-center rounded-full bg-brand-100 px-2.5 py-0.5 text-xs font-medium text-brand-800">
          {trace.spans.length}
        </span>
      </td>
      <td className="px-4 py-3 text-sm text-gray-500">
        {formatRelativeTime(trace.started_at)}
      </td>
    </tr>
  );
}
