'use client';

import { Activity, Zap, Clock, DollarSign, AlertTriangle } from 'lucide-react';
import { Navbar } from '@/components/Navbar';
import { TraceList } from '@/components/TraceList';
import { useRecentTraces } from '@/hooks/useSpans';

export default function DashboardPage() {
  const { data: traces } = useRecentTraces(100);

  // Calculate stats
  const stats = {
    totalTraces: traces?.length || 0,
    totalSpans: traces?.reduce((sum, t) => sum + t.spans.length, 0) || 0,
    totalTokens: traces?.reduce((sum, t) => sum + t.total_tokens, 0) || 0,
    totalCost: traces?.reduce((sum, t) => sum + t.total_cost_usd, 0) || 0,
    errorCount: traces?.filter((t) => t.spans.some((s) => s.status === 'error')).length || 0,
    avgDuration:
      traces && traces.length > 0
        ? traces.reduce((sum, t) => sum + t.total_duration_ms, 0) / traces.length
        : 0,
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar />

      <main className="pt-20 pb-12 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
          <p className="text-gray-500 mt-1">
            Real-time observability for your AI agents
          </p>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
          <StatCard
            title="Total Traces"
            value={stats.totalTraces.toString()}
            icon={Activity}
            color="brand"
          />
          <StatCard
            title="Total Tokens"
            value={formatNumber(stats.totalTokens)}
            icon={Zap}
            color="purple"
          />
          <StatCard
            title="Avg Duration"
            value={formatDuration(stats.avgDuration)}
            icon={Clock}
            color="blue"
          />
          <StatCard
            title="Errors"
            value={stats.errorCount.toString()}
            icon={AlertTriangle}
            color={stats.errorCount > 0 ? 'red' : 'green'}
          />
        </div>

        {/* Recent Traces */}
        <div className="mb-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">
            Recent Traces
          </h2>
          <TraceList />
        </div>
      </main>
    </div>
  );
}

interface StatCardProps {
  title: string;
  value: string;
  icon: typeof Activity;
  color: 'brand' | 'purple' | 'blue' | 'green' | 'red';
}

function StatCard({ title, value, icon: Icon, color }: StatCardProps) {
  const colorClasses = {
    brand: 'bg-brand-100 text-brand-600',
    purple: 'bg-purple-100 text-purple-600',
    blue: 'bg-blue-100 text-blue-600',
    green: 'bg-green-100 text-green-600',
    red: 'bg-red-100 text-red-600',
  };

  return (
    <div className="bg-white rounded-xl border border-gray-200 p-6">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-gray-500">{title}</p>
          <p className="text-2xl font-bold text-gray-900 mt-1">{value}</p>
        </div>
        <div className={`p-3 rounded-lg ${colorClasses[color]}`}>
          <Icon className="w-6 h-6" />
        </div>
      </div>
    </div>
  );
}

function formatNumber(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

function formatDuration(ms: number): string {
  if (ms < 1) return '0ms';
  if (ms < 1000) return `${Math.round(ms)}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${(ms / 60000).toFixed(1)}m`;
}
