'use client';

import { useMemo, useState } from 'react';
import { Navbar } from '@/components/Navbar';
import { CostBreakdown } from '@/components/CostBreakdown';
import { useSpans } from '@/hooks/useSpans';
import { formatCost, formatTokens, formatDate } from '@/lib/utils';
import { DollarSign, TrendingUp, TrendingDown, Calendar, PieChart, BarChart3 } from 'lucide-react';

type TimeRange = '1h' | '24h' | '7d' | '30d';
type GroupBy = 'model' | 'service' | 'day';

interface CostEntry {
  group: string;
  provider?: string;
  totalCost: number;
  totalTokensIn: number;
  totalTokensOut: number;
  callCount: number;
  percentage: number;
}

export default function CostsPage() {
  const [timeRange, setTimeRange] = useState<TimeRange>('24h');
  const [groupBy, setGroupBy] = useState<GroupBy>('model');
  const { data, isLoading } = useSpans({ limit: 1000 });

  const costData = useMemo(() => {
    if (!data?.spans) return null;

    const now = new Date();
    const rangeMs = {
      '1h': 60 * 60 * 1000,
      '24h': 24 * 60 * 60 * 1000,
      '7d': 7 * 24 * 60 * 60 * 1000,
      '30d': 30 * 24 * 60 * 60 * 1000,
    };

    // Filter spans by time range
    const filteredSpans = data.spans.filter((span) => {
      const spanTime = new Date(span.started_at).getTime();
      return now.getTime() - spanTime <= rangeMs[timeRange];
    });

    // Group by selected dimension
    const grouped = new Map<string, CostEntry>();
    let totalCost = 0;

    for (const span of filteredSpans) {
      if (span.cost_usd == null) continue;

      let key: string;
      let provider: string | undefined;

      switch (groupBy) {
        case 'model':
          key = span.model_name || 'unknown';
          provider = span.model_provider || undefined;
          break;
        case 'service':
          key = span.service_name;
          break;
        case 'day':
          key = new Date(span.started_at).toISOString().split('T')[0];
          break;
        default:
          key = 'unknown';
      }

      totalCost += span.cost_usd;

      const existing = grouped.get(key);
      if (existing) {
        existing.totalCost += span.cost_usd;
        existing.totalTokensIn += span.tokens_in || 0;
        existing.totalTokensOut += span.tokens_out || 0;
        existing.callCount++;
      } else {
        grouped.set(key, {
          group: key,
          provider,
          totalCost: span.cost_usd,
          totalTokensIn: span.tokens_in || 0,
          totalTokensOut: span.tokens_out || 0,
          callCount: 1,
          percentage: 0,
        });
      }
    }

    // Calculate percentages
    const entries = Array.from(grouped.values()).map((entry) => ({
      ...entry,
      percentage: totalCost > 0 ? (entry.totalCost / totalCost) * 100 : 0,
    }));

    // Sort by cost descending
    entries.sort((a, b) => b.totalCost - a.totalCost);

    // Calculate previous period for comparison
    const previousSpans = data.spans.filter((span) => {
      const spanTime = new Date(span.started_at).getTime();
      const age = now.getTime() - spanTime;
      return age > rangeMs[timeRange] && age <= rangeMs[timeRange] * 2;
    });
    const previousCost = previousSpans.reduce((sum, s) => sum + (s.cost_usd || 0), 0);

    return {
      entries,
      totalCost,
      previousCost,
      totalTokens: entries.reduce((sum, e) => sum + e.totalTokensIn + e.totalTokensOut, 0),
      totalCalls: entries.reduce((sum, e) => sum + e.callCount, 0),
      changePercent: previousCost > 0 ? ((totalCost - previousCost) / previousCost) * 100 : 0,
    };
  }, [data?.spans, timeRange, groupBy]);

  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar />

      <main className="pt-20 pb-12 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
        <div className="mb-8 flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-gray-900">Costs</h1>
            <p className="text-gray-500 mt-1">Track and analyze your AI spending</p>
          </div>
          <div className="flex gap-2">
            <select
              value={timeRange}
              onChange={(e) => setTimeRange(e.target.value as TimeRange)}
              className="px-3 py-2 bg-white border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
            >
              <option value="1h">Last hour</option>
              <option value="24h">Last 24 hours</option>
              <option value="7d">Last 7 days</option>
              <option value="30d">Last 30 days</option>
            </select>
            <select
              value={groupBy}
              onChange={(e) => setGroupBy(e.target.value as GroupBy)}
              className="px-3 py-2 bg-white border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
            >
              <option value="model">By Model</option>
              <option value="service">By Service</option>
              <option value="day">By Day</option>
            </select>
          </div>
        </div>

        {isLoading ? (
          <div className="animate-pulse space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
              {[1, 2, 3, 4].map((i) => (
                <div key={i} className="h-28 bg-white rounded-lg border border-gray-200"></div>
              ))}
            </div>
            <div className="h-96 bg-white rounded-lg border border-gray-200"></div>
          </div>
        ) : costData ? (
          <div className="space-y-6">
            {/* Summary Cards */}
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
              <SummaryCard
                icon={<DollarSign className="w-5 h-5" />}
                label="Total Cost"
                value={formatCost(costData.totalCost)}
                color="brand"
              />
              <SummaryCard
                icon={
                  costData.changePercent >= 0 ? (
                    <TrendingUp className="w-5 h-5" />
                  ) : (
                    <TrendingDown className="w-5 h-5" />
                  )
                }
                label="vs Previous Period"
                value={`${costData.changePercent >= 0 ? '+' : ''}${costData.changePercent.toFixed(1)}%`}
                subtitle={`Previous: ${formatCost(costData.previousCost)}`}
                color={costData.changePercent > 10 ? 'red' : costData.changePercent < 0 ? 'green' : 'gray'}
              />
              <SummaryCard
                icon={<BarChart3 className="w-5 h-5" />}
                label="Total Tokens"
                value={formatTokens(costData.totalTokens)}
                color="purple"
              />
              <SummaryCard
                icon={<Calendar className="w-5 h-5" />}
                label="Total Calls"
                value={costData.totalCalls.toString()}
                color="blue"
              />
            </div>

            {/* Cost Table */}
            <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
              <div className="px-6 py-4 border-b border-gray-200">
                <h3 className="text-lg font-semibold text-gray-900">
                  Cost Breakdown by {groupBy.charAt(0).toUpperCase() + groupBy.slice(1)}
                </h3>
              </div>
              {costData.entries.length === 0 ? (
                <div className="p-12 text-center">
                  <PieChart className="w-12 h-12 text-gray-300 mx-auto mb-4" />
                  <p className="text-gray-500">No cost data for this period</p>
                </div>
              ) : (
                <div className="overflow-x-auto">
                  <table className="w-full">
                    <thead>
                      <tr className="bg-gray-50 border-b border-gray-200">
                        <th className="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase">
                          {groupBy === 'day' ? 'Date' : groupBy === 'model' ? 'Model' : 'Service'}
                        </th>
                        {groupBy === 'model' && (
                          <th className="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase">
                            Provider
                          </th>
                        )}
                        <th className="text-right px-6 py-3 text-xs font-medium text-gray-500 uppercase">
                          Cost
                        </th>
                        <th className="text-right px-6 py-3 text-xs font-medium text-gray-500 uppercase">
                          % of Total
                        </th>
                        <th className="text-right px-6 py-3 text-xs font-medium text-gray-500 uppercase">
                          Tokens
                        </th>
                        <th className="text-right px-6 py-3 text-xs font-medium text-gray-500 uppercase">
                          Calls
                        </th>
                        <th className="text-right px-6 py-3 text-xs font-medium text-gray-500 uppercase">
                          Avg/Call
                        </th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-200">
                      {costData.entries.map((entry) => (
                        <tr key={entry.group} className="hover:bg-gray-50">
                          <td className="px-6 py-4 text-sm font-medium text-gray-900">
                            {groupBy === 'day' ? formatDate(entry.group) : entry.group}
                          </td>
                          {groupBy === 'model' && (
                            <td className="px-6 py-4 text-sm text-gray-500 capitalize">
                              {entry.provider || '-'}
                            </td>
                          )}
                          <td className="px-6 py-4 text-sm font-semibold text-gray-900 text-right">
                            {formatCost(entry.totalCost)}
                          </td>
                          <td className="px-6 py-4 text-right">
                            <div className="flex items-center justify-end gap-2">
                              <div className="w-16 h-2 bg-gray-100 rounded-full overflow-hidden">
                                <div
                                  className="h-full bg-brand-500 rounded-full"
                                  style={{ width: `${entry.percentage}%` }}
                                />
                              </div>
                              <span className="text-sm text-gray-500 w-12">
                                {entry.percentage.toFixed(1)}%
                              </span>
                            </div>
                          </td>
                          <td className="px-6 py-4 text-sm text-gray-500 text-right">
                            {formatTokens(entry.totalTokensIn + entry.totalTokensOut)}
                          </td>
                          <td className="px-6 py-4 text-sm text-gray-500 text-right">
                            {entry.callCount}
                          </td>
                          <td className="px-6 py-4 text-sm text-gray-500 text-right">
                            {formatCost(entry.totalCost / entry.callCount)}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                    <tfoot>
                      <tr className="bg-gray-50 font-semibold">
                        <td className="px-6 py-4 text-sm text-gray-900">Total</td>
                        {groupBy === 'model' && <td></td>}
                        <td className="px-6 py-4 text-sm text-gray-900 text-right">
                          {formatCost(costData.totalCost)}
                        </td>
                        <td className="px-6 py-4 text-sm text-gray-900 text-right">100%</td>
                        <td className="px-6 py-4 text-sm text-gray-500 text-right">
                          {formatTokens(costData.totalTokens)}
                        </td>
                        <td className="px-6 py-4 text-sm text-gray-500 text-right">
                          {costData.totalCalls}
                        </td>
                        <td className="px-6 py-4 text-sm text-gray-500 text-right">
                          {formatCost(costData.totalCost / costData.totalCalls)}
                        </td>
                      </tr>
                    </tfoot>
                  </table>
                </div>
              )}
            </div>

            {/* Model Breakdown Chart */}
            <CostBreakdown />
          </div>
        ) : (
          <div className="bg-white rounded-lg border border-gray-200 p-12 text-center">
            <DollarSign className="w-12 h-12 text-gray-300 mx-auto mb-4" />
            <p className="text-gray-500">No data available</p>
          </div>
        )}
      </main>
    </div>
  );
}

interface SummaryCardProps {
  icon: React.ReactNode;
  label: string;
  value: string;
  subtitle?: string;
  color: 'brand' | 'blue' | 'purple' | 'green' | 'red' | 'gray';
}

function SummaryCard({ icon, label, value, subtitle, color }: SummaryCardProps) {
  const colorClasses = {
    brand: 'bg-brand-50 text-brand-600',
    blue: 'bg-blue-50 text-blue-600',
    purple: 'bg-purple-50 text-purple-600',
    green: 'bg-green-50 text-green-600',
    red: 'bg-red-50 text-red-600',
    gray: 'bg-gray-100 text-gray-600',
  };

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4">
      <div className="flex items-center gap-3 mb-2">
        <div className={`p-2 rounded-lg ${colorClasses[color]}`}>{icon}</div>
        <span className="text-sm text-gray-500">{label}</span>
      </div>
      <p className="text-2xl font-bold text-gray-900">{value}</p>
      {subtitle && <p className="text-xs text-gray-500 mt-1">{subtitle}</p>}
    </div>
  );
}
