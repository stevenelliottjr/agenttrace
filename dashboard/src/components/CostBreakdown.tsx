'use client';

import { useMemo } from 'react';
import { useSpans } from '@/hooks/useSpans';
import { formatCost, formatTokens } from '@/lib/utils';

interface ModelCost {
  model: string;
  provider: string;
  totalCost: number;
  totalTokensIn: number;
  totalTokensOut: number;
  count: number;
}

export function CostBreakdown() {
  const { data, isLoading } = useSpans();

  const costByModel = useMemo(() => {
    if (!data?.spans) return [];

    const modelCosts = new Map<string, ModelCost>();

    for (const span of data.spans) {
      if (!span.model_name || span.cost_usd == null) continue;

      const existing = modelCosts.get(span.model_name);
      if (existing) {
        existing.totalCost += span.cost_usd;
        existing.totalTokensIn += span.tokens_in || 0;
        existing.totalTokensOut += span.tokens_out || 0;
        existing.count++;
      } else {
        modelCosts.set(span.model_name, {
          model: span.model_name,
          provider: span.model_provider || 'unknown',
          totalCost: span.cost_usd,
          totalTokensIn: span.tokens_in || 0,
          totalTokensOut: span.tokens_out || 0,
          count: 1,
        });
      }
    }

    return Array.from(modelCosts.values()).sort((a, b) => b.totalCost - a.totalCost);
  }, [data?.spans]);

  const totalCost = useMemo(() => {
    return costByModel.reduce((sum, model) => sum + model.totalCost, 0);
  }, [costByModel]);

  if (isLoading) {
    return (
      <div className="rounded-lg bg-white border border-gray-200 p-6">
        <div className="animate-pulse">
          <div className="h-6 bg-gray-200 rounded w-1/3 mb-4"></div>
          <div className="space-y-3">
            {[1, 2, 3].map((i) => (
              <div key={i} className="h-12 bg-gray-100 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  if (costByModel.length === 0) {
    return (
      <div className="rounded-lg bg-white border border-gray-200 p-8 text-center">
        <p className="text-gray-500">No cost data available yet</p>
        <p className="text-sm text-gray-400 mt-1">
          Start sending spans with token usage to see costs
        </p>
      </div>
    );
  }

  const maxCost = Math.max(...costByModel.map((m) => m.totalCost));

  return (
    <div className="rounded-lg bg-white border border-gray-200">
      <div className="p-4 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-gray-900">Cost by Model</h3>
          <div className="text-right">
            <p className="text-sm text-gray-500">Total Cost</p>
            <p className="text-2xl font-bold text-brand-600">{formatCost(totalCost)}</p>
          </div>
        </div>
      </div>
      <div className="p-4">
        <div className="space-y-4">
          {costByModel.map((model) => {
            const percentage = (model.totalCost / maxCost) * 100;
            const providerColor = getProviderColor(model.provider);

            return (
              <div key={model.model}>
                <div className="flex items-center justify-between mb-1">
                  <div className="flex items-center gap-2">
                    <span
                      className="w-2 h-2 rounded-full"
                      style={{ backgroundColor: providerColor }}
                    />
                    <span className="text-sm font-medium text-gray-900">
                      {model.model}
                    </span>
                    <span className="text-xs text-gray-500 capitalize">
                      {model.provider}
                    </span>
                  </div>
                  <span className="text-sm font-semibold text-gray-900">
                    {formatCost(model.totalCost)}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="flex-1 h-2 bg-gray-100 rounded-full overflow-hidden">
                    <div
                      className="h-full rounded-full transition-all"
                      style={{
                        width: `${percentage}%`,
                        backgroundColor: providerColor,
                      }}
                    />
                  </div>
                </div>
                <div className="flex justify-between text-xs text-gray-500 mt-1">
                  <span>{model.count} call{model.count !== 1 ? 's' : ''}</span>
                  <span>
                    {formatTokens(model.totalTokensIn)} in / {formatTokens(model.totalTokensOut)} out
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function getProviderColor(provider: string): string {
  const colors: Record<string, string> = {
    anthropic: '#D97706',
    openai: '#10B981',
    google: '#3B82F6',
    mistral: '#8B5CF6',
    unknown: '#6B7280',
  };
  return colors[provider.toLowerCase()] || colors.unknown;
}
