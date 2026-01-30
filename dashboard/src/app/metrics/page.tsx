'use client';

import { useMemo } from 'react';
import { Navbar } from '@/components/Navbar';
import { CostBreakdown } from '@/components/CostBreakdown';
import { useSpans } from '@/hooks/useSpans';
import { formatCost, formatTokens, formatDuration } from '@/lib/utils';
import { DollarSign, Zap, Clock, Activity, TrendingUp, AlertTriangle } from 'lucide-react';

export default function MetricsPage() {
  const { data, isLoading } = useSpans();

  const metrics = useMemo(() => {
    if (!data?.spans) return null;

    let totalCost = 0;
    let totalTokensIn = 0;
    let totalTokensOut = 0;
    let totalDuration = 0;
    let llmCallCount = 0;
    let errorCount = 0;
    const traceIds = new Set<string>();

    for (const span of data.spans) {
      traceIds.add(span.trace_id);
      if (span.cost_usd) totalCost += span.cost_usd;
      if (span.tokens_in) totalTokensIn += span.tokens_in;
      if (span.tokens_out) totalTokensOut += span.tokens_out;
      if (span.duration_ms) totalDuration += span.duration_ms;
      if (span.model_name) llmCallCount++;
      if (span.status === 'error') errorCount++;
    }

    const avgDuration = data.spans.length > 0 ? totalDuration / data.spans.length : 0;
    const errorRate = data.spans.length > 0 ? (errorCount / data.spans.length) * 100 : 0;

    return {
      totalCost,
      totalTokensIn,
      totalTokensOut,
      totalTokens: totalTokensIn + totalTokensOut,
      avgDuration,
      traceCount: traceIds.size,
      spanCount: data.spans.length,
      llmCallCount,
      errorCount,
      errorRate,
    };
  }, [data?.spans]);

  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar />

      <main className="pt-20 pb-12 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Metrics</h1>
          <p className="text-gray-500 mt-1">
            Analytics and insights for your AI agents
          </p>
        </div>

        {isLoading ? (
          <div className="animate-pulse space-y-6">
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {[1, 2, 3, 4].map((i) => (
                <div key={i} className="h-24 bg-white rounded-lg border border-gray-200"></div>
              ))}
            </div>
            <div className="h-64 bg-white rounded-lg border border-gray-200"></div>
          </div>
        ) : metrics ? (
          <div className="space-y-6">
            {/* Key Metrics */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <MetricCard
                icon={<DollarSign className="w-5 h-5" />}
                label="Total Cost"
                value={formatCost(metrics.totalCost)}
                color="brand"
              />
              <MetricCard
                icon={<Zap className="w-5 h-5" />}
                label="Total Tokens"
                value={formatTokens(metrics.totalTokens)}
                subtitle={`${formatTokens(metrics.totalTokensIn)} in / ${formatTokens(metrics.totalTokensOut)} out`}
                color="yellow"
              />
              <MetricCard
                icon={<Clock className="w-5 h-5" />}
                label="Avg Duration"
                value={formatDuration(metrics.avgDuration)}
                color="blue"
              />
              <MetricCard
                icon={<AlertTriangle className="w-5 h-5" />}
                label="Error Rate"
                value={`${metrics.errorRate.toFixed(1)}%`}
                subtitle={`${metrics.errorCount} errors`}
                color={metrics.errorRate > 5 ? 'red' : 'green'}
              />
            </div>

            {/* Secondary Metrics */}
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
              <MetricCard
                icon={<Activity className="w-5 h-5" />}
                label="Total Spans"
                value={metrics.spanCount.toString()}
                color="gray"
              />
              <MetricCard
                icon={<TrendingUp className="w-5 h-5" />}
                label="Traces"
                value={metrics.traceCount.toString()}
                color="gray"
              />
              <MetricCard
                icon={<Zap className="w-5 h-5" />}
                label="LLM Calls"
                value={metrics.llmCallCount.toString()}
                color="gray"
              />
            </div>

            {/* Cost Breakdown Chart */}
            <CostBreakdown />

            {/* Token Usage Summary */}
            <div className="rounded-lg bg-white border border-gray-200 p-6">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">Token Usage Summary</h3>
              <div className="grid md:grid-cols-3 gap-6">
                <div className="text-center p-4 bg-gray-50 rounded-lg">
                  <p className="text-sm text-gray-500 mb-1">Input Tokens</p>
                  <p className="text-3xl font-bold text-gray-900">{formatTokens(metrics.totalTokensIn)}</p>
                </div>
                <div className="text-center p-4 bg-gray-50 rounded-lg">
                  <p className="text-sm text-gray-500 mb-1">Output Tokens</p>
                  <p className="text-3xl font-bold text-gray-900">{formatTokens(metrics.totalTokensOut)}</p>
                </div>
                <div className="text-center p-4 bg-brand-50 rounded-lg">
                  <p className="text-sm text-brand-600 mb-1">Total Tokens</p>
                  <p className="text-3xl font-bold text-brand-700">{formatTokens(metrics.totalTokens)}</p>
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="rounded-lg bg-white border border-gray-200 p-12 text-center">
            <p className="text-gray-500">No data available</p>
          </div>
        )}
      </main>
    </div>
  );
}

interface MetricCardProps {
  icon: React.ReactNode;
  label: string;
  value: string;
  subtitle?: string;
  color: 'brand' | 'blue' | 'yellow' | 'green' | 'red' | 'gray';
}

function MetricCard({ icon, label, value, subtitle, color }: MetricCardProps) {
  const colorClasses = {
    brand: 'bg-brand-50 text-brand-600',
    blue: 'bg-blue-50 text-blue-600',
    yellow: 'bg-yellow-50 text-yellow-600',
    green: 'bg-green-50 text-green-600',
    red: 'bg-red-50 text-red-600',
    gray: 'bg-gray-100 text-gray-600',
  };

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4">
      <div className="flex items-center gap-2 mb-2">
        <div className={`p-2 rounded-lg ${colorClasses[color]}`}>{icon}</div>
        <span className="text-sm text-gray-500">{label}</span>
      </div>
      <p className="text-2xl font-bold text-gray-900">{value}</p>
      {subtitle && <p className="text-xs text-gray-500 mt-1">{subtitle}</p>}
    </div>
  );
}
