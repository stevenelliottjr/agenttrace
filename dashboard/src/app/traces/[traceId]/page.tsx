'use client';

import { useState } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';
import { ArrowLeft, Clock, Zap, DollarSign, Layers } from 'lucide-react';
import { Navbar } from '@/components/Navbar';
import { TraceWaterfall } from '@/components/TraceWaterfall';
import { SpanDetail } from '@/components/SpanDetail';
import { useTrace } from '@/hooks/useSpans';
import { formatDuration, formatTokens, formatCost, formatDate } from '@/lib/utils';
import type { Span } from '@/types';

export default function TraceDetailPage() {
  const params = useParams();
  const traceId = params.traceId as string;
  const { data: trace, isLoading, error } = useTrace(traceId);
  const [selectedSpan, setSelectedSpan] = useState<Span | null>(null);

  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-50">
        <Navbar />
        <div className="pt-20 flex items-center justify-center h-[calc(100vh-5rem)]">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-brand-600" />
        </div>
      </div>
    );
  }

  if (error || !trace) {
    return (
      <div className="min-h-screen bg-gray-50">
        <Navbar />
        <main className="pt-20 pb-12 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
          <div className="rounded-lg bg-red-50 p-8 text-center">
            <p className="text-red-700 font-medium">
              {error ? `Failed to load trace: ${error.message}` : 'Trace not found'}
            </p>
            <Link
              href="/traces"
              className="inline-flex items-center gap-2 mt-4 text-brand-600 hover:text-brand-700"
            >
              <ArrowLeft className="w-4 h-4" />
              Back to traces
            </Link>
          </div>
        </main>
      </div>
    );
  }

  const hasError = trace.spans.some((s) => s.status === 'error');

  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar />

      <main className="pt-20 pb-12">
        <div className="px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
          {/* Back link */}
          <Link
            href="/traces"
            className="inline-flex items-center gap-2 text-sm text-gray-500 hover:text-gray-700 mb-4"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to traces
          </Link>

          {/* Header */}
          <div className="flex items-start justify-between mb-6">
            <div>
              <div className="flex items-center gap-3">
                <h1 className="text-2xl font-bold text-gray-900">
                  {trace.root_span?.operation_name || 'Unknown Trace'}
                </h1>
                {hasError && (
                  <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                    Error
                  </span>
                )}
              </div>
              <p className="text-gray-500 mt-1 font-mono text-sm">{traceId}</p>
            </div>
            <div className="text-right text-sm text-gray-500">
              {formatDate(trace.started_at)}
            </div>
          </div>

          {/* Stats */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <div className="flex items-center gap-2 text-gray-500 text-sm mb-1">
                <Clock className="w-4 h-4" />
                Duration
              </div>
              <div className="text-xl font-semibold text-gray-900">
                {formatDuration(trace.total_duration_ms)}
              </div>
            </div>
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <div className="flex items-center gap-2 text-gray-500 text-sm mb-1">
                <Layers className="w-4 h-4" />
                Spans
              </div>
              <div className="text-xl font-semibold text-gray-900">
                {trace.spans.length}
              </div>
            </div>
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <div className="flex items-center gap-2 text-gray-500 text-sm mb-1">
                <Zap className="w-4 h-4" />
                Tokens
              </div>
              <div className="text-xl font-semibold text-gray-900">
                {formatTokens(trace.total_tokens)}
              </div>
            </div>
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <div className="flex items-center gap-2 text-gray-500 text-sm mb-1">
                <DollarSign className="w-4 h-4" />
                Cost
              </div>
              <div className="text-xl font-semibold text-gray-900">
                {formatCost(trace.total_cost_usd)}
              </div>
            </div>
          </div>

          {/* Services */}
          <div className="flex items-center gap-2 mb-6">
            <span className="text-sm text-gray-500">Services:</span>
            {trace.service_names.map((service) => (
              <span
                key={service}
                className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800"
              >
                {service}
              </span>
            ))}
          </div>
        </div>

        {/* Waterfall with detail panel */}
        <div className="flex">
          <div
            className={`px-4 sm:px-6 lg:px-8 transition-all ${
              selectedSpan ? 'w-[calc(100%-400px)]' : 'w-full max-w-7xl mx-auto'
            }`}
          >
            <h2 className="text-lg font-semibold text-gray-900 mb-4">
              Trace Timeline
            </h2>
            <TraceWaterfall
              trace={trace}
              onSpanSelect={setSelectedSpan}
              selectedSpanId={selectedSpan?.span_id}
            />
          </div>

          {/* Span Detail Panel */}
          {selectedSpan && (
            <div className="w-[400px] flex-shrink-0 h-[calc(100vh-5rem)] sticky top-20">
              <SpanDetail
                span={selectedSpan}
                onClose={() => setSelectedSpan(null)}
              />
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
