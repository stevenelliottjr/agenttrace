'use client';

import { useState, useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import Link from 'next/link';
import { Navbar } from '@/components/Navbar';
import { SpanIcon } from '@/components/SpanIcon';
import { formatDuration, formatCost, formatTokens, formatRelativeTime, cn } from '@/lib/utils';
import type { Span, SpanListResponse } from '@/types';
import {
  Search,
  Filter,
  X,
  ChevronDown,
  ChevronUp,
  AlertCircle,
  CheckCircle,
  Clock,
  SortAsc,
  SortDesc,
} from 'lucide-react';

interface SearchFilters {
  query: string;
  service: string;
  model: string;
  status: string;
  minDuration: string;
  maxDuration: string;
  minCost: string;
  maxCost: string;
  timeRange: string;
}

interface SearchParams {
  q?: string;
  service?: string;
  model?: string;
  status?: string;
  min_duration?: number;
  max_duration?: number;
  min_cost?: number;
  max_cost?: number;
  since?: string;
  sort_by?: string;
  sort_order?: string;
  limit?: number;
  offset?: number;
}

interface SearchResponse {
  spans: Span[];
  total: number;
  limit: number;
  offset: number;
}

async function searchSpans(params: SearchParams): Promise<SearchResponse> {
  const searchParams = new URLSearchParams();

  if (params.q) searchParams.set('q', params.q);
  if (params.service) searchParams.set('service', params.service);
  if (params.model) searchParams.set('model', params.model);
  if (params.status) searchParams.set('status', params.status);
  if (params.min_duration) searchParams.set('min_duration', params.min_duration.toString());
  if (params.max_duration) searchParams.set('max_duration', params.max_duration.toString());
  if (params.min_cost) searchParams.set('min_cost', params.min_cost.toString());
  if (params.max_cost) searchParams.set('max_cost', params.max_cost.toString());
  if (params.since) searchParams.set('since', params.since);
  if (params.sort_by) searchParams.set('sort_by', params.sort_by);
  if (params.sort_order) searchParams.set('sort_order', params.sort_order);
  if (params.limit) searchParams.set('limit', params.limit.toString());
  if (params.offset) searchParams.set('offset', params.offset.toString());

  const res = await fetch(`/api/v1/search?${searchParams.toString()}`);
  if (!res.ok) throw new Error('Search failed');
  return res.json();
}

async function fetchSpansForFilters(): Promise<SpanListResponse> {
  const res = await fetch('/api/v1/spans?limit=500');
  if (!res.ok) throw new Error('Failed to fetch spans');
  return res.json();
}

export default function SearchPage() {
  const [filters, setFilters] = useState<SearchFilters>({
    query: '',
    service: '',
    model: '',
    status: '',
    minDuration: '',
    maxDuration: '',
    minCost: '',
    maxCost: '',
    timeRange: '24h',
  });
  const [showFilters, setShowFilters] = useState(true);
  const [sortBy, setSortBy] = useState('started_at');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [page, setPage] = useState(0);
  const pageSize = 50;

  // Fetch spans for filter options
  const { data: spanData } = useQuery({
    queryKey: ['spans-for-filters'],
    queryFn: fetchSpansForFilters,
  });

  // Extract unique values for filter dropdowns
  const filterOptions = useMemo(() => {
    if (!spanData?.spans) return { services: [], models: [] };

    const services = Array.from(new Set(spanData.spans.map((s) => s.service_name))).sort();
    const models = Array.from(new Set(spanData.spans.map((s) => s.model_name).filter(Boolean) as string[])).sort();

    return { services, models };
  }, [spanData?.spans]);

  // Build search params
  const searchParams: SearchParams = useMemo(() => {
    const params: SearchParams = {
      limit: pageSize,
      offset: page * pageSize,
      sort_by: sortBy,
      sort_order: sortOrder,
    };

    if (filters.query) params.q = filters.query;
    if (filters.service) params.service = filters.service;
    if (filters.model) params.model = filters.model;
    if (filters.status) params.status = filters.status;
    if (filters.minDuration) params.min_duration = parseFloat(filters.minDuration);
    if (filters.maxDuration) params.max_duration = parseFloat(filters.maxDuration);
    if (filters.minCost) params.min_cost = parseFloat(filters.minCost);
    if (filters.maxCost) params.max_cost = parseFloat(filters.maxCost);

    // Calculate since time based on range
    const rangeMs: Record<string, number> = {
      '1h': 60 * 60 * 1000,
      '6h': 6 * 60 * 60 * 1000,
      '24h': 24 * 60 * 60 * 1000,
      '7d': 7 * 24 * 60 * 60 * 1000,
      '30d': 30 * 24 * 60 * 60 * 1000,
    };
    if (filters.timeRange && rangeMs[filters.timeRange]) {
      params.since = new Date(Date.now() - rangeMs[filters.timeRange]).toISOString();
    }

    return params;
  }, [filters, page, sortBy, sortOrder]);

  // Search query
  const { data: searchResults, isLoading, isFetching } = useQuery({
    queryKey: ['search', searchParams],
    queryFn: () => searchSpans(searchParams),
    placeholderData: (prev) => prev,
  });

  const handleFilterChange = (key: keyof SearchFilters, value: string) => {
    setFilters((prev) => ({ ...prev, [key]: value }));
    setPage(0);
  };

  const clearFilters = () => {
    setFilters({
      query: '',
      service: '',
      model: '',
      status: '',
      minDuration: '',
      maxDuration: '',
      minCost: '',
      maxCost: '',
      timeRange: '24h',
    });
    setPage(0);
  };

  const hasActiveFilters = Object.entries(filters).some(
    ([key, value]) => value && key !== 'timeRange'
  );

  const toggleSort = (field: string) => {
    if (sortBy === field) {
      setSortOrder((prev) => (prev === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortBy(field);
      setSortOrder('desc');
    }
    setPage(0);
  };

  const totalPages = searchResults ? Math.ceil(searchResults.total / pageSize) : 0;

  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar />

      <main className="pt-20 pb-12 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
        <div className="mb-6">
          <h1 className="text-3xl font-bold text-gray-900">Search</h1>
          <p className="text-gray-500 mt-1">Find and filter traces and spans</p>
        </div>

        {/* Search Bar */}
        <div className="bg-white rounded-lg border border-gray-200 p-4 mb-4">
          <div className="flex gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
              <input
                type="text"
                value={filters.query}
                onChange={(e) => handleFilterChange('query', e.target.value)}
                placeholder="Search by operation name, trace ID, or content..."
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-brand-500"
              />
            </div>
            <button
              onClick={() => setShowFilters(!showFilters)}
              className={cn(
                'flex items-center gap-2 px-4 py-2 border rounded-lg transition-colors',
                showFilters || hasActiveFilters
                  ? 'border-brand-500 bg-brand-50 text-brand-700'
                  : 'border-gray-300 text-gray-700 hover:bg-gray-50'
              )}
            >
              <Filter className="w-4 h-4" />
              Filters
              {hasActiveFilters && (
                <span className="ml-1 px-1.5 py-0.5 bg-brand-500 text-white text-xs rounded-full">
                  !
                </span>
              )}
              {showFilters ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
            </button>
          </div>

          {/* Filters Panel */}
          {showFilters && (
            <div className="mt-4 pt-4 border-t border-gray-200">
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div>
                  <label className="block text-xs font-medium text-gray-500 mb-1">Service</label>
                  <select
                    value={filters.service}
                    onChange={(e) => handleFilterChange('service', e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
                  >
                    <option value="">All services</option>
                    {filterOptions.services.map((s) => (
                      <option key={s} value={s}>
                        {s}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-xs font-medium text-gray-500 mb-1">Model</label>
                  <select
                    value={filters.model}
                    onChange={(e) => handleFilterChange('model', e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
                  >
                    <option value="">All models</option>
                    {filterOptions.models.map((m) => (
                      <option key={m} value={m!}>
                        {m}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-xs font-medium text-gray-500 mb-1">Status</label>
                  <select
                    value={filters.status}
                    onChange={(e) => handleFilterChange('status', e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
                  >
                    <option value="">All statuses</option>
                    <option value="ok">OK</option>
                    <option value="error">Error</option>
                  </select>
                </div>
                <div>
                  <label className="block text-xs font-medium text-gray-500 mb-1">Time Range</label>
                  <select
                    value={filters.timeRange}
                    onChange={(e) => handleFilterChange('timeRange', e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
                  >
                    <option value="1h">Last hour</option>
                    <option value="6h">Last 6 hours</option>
                    <option value="24h">Last 24 hours</option>
                    <option value="7d">Last 7 days</option>
                    <option value="30d">Last 30 days</option>
                  </select>
                </div>
              </div>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-4">
                <div>
                  <label className="block text-xs font-medium text-gray-500 mb-1">
                    Min Duration (ms)
                  </label>
                  <input
                    type="number"
                    value={filters.minDuration}
                    onChange={(e) => handleFilterChange('minDuration', e.target.value)}
                    placeholder="0"
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-gray-500 mb-1">
                    Max Duration (ms)
                  </label>
                  <input
                    type="number"
                    value={filters.maxDuration}
                    onChange={(e) => handleFilterChange('maxDuration', e.target.value)}
                    placeholder="unlimited"
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-gray-500 mb-1">Min Cost ($)</label>
                  <input
                    type="number"
                    step="0.001"
                    value={filters.minCost}
                    onChange={(e) => handleFilterChange('minCost', e.target.value)}
                    placeholder="0"
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-gray-500 mb-1">Max Cost ($)</label>
                  <input
                    type="number"
                    step="0.001"
                    value={filters.maxCost}
                    onChange={(e) => handleFilterChange('maxCost', e.target.value)}
                    placeholder="unlimited"
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
                  />
                </div>
              </div>
              {hasActiveFilters && (
                <div className="mt-4 flex justify-end">
                  <button
                    onClick={clearFilters}
                    className="flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700"
                  >
                    <X className="w-4 h-4" />
                    Clear filters
                  </button>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Results */}
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          {/* Header */}
          <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="font-medium text-gray-900">
                {searchResults?.total || 0} results
              </span>
              {isFetching && !isLoading && (
                <div className="w-4 h-4 border-2 border-brand-500 border-t-transparent rounded-full animate-spin" />
              )}
            </div>
            <div className="flex items-center gap-2 text-sm text-gray-500">
              <span>Sort by:</span>
              {[
                { key: 'started_at', label: 'Time' },
                { key: 'duration_ms', label: 'Duration' },
                { key: 'cost_usd', label: 'Cost' },
              ].map((option) => (
                <button
                  key={option.key}
                  onClick={() => toggleSort(option.key)}
                  className={cn(
                    'flex items-center gap-1 px-2 py-1 rounded transition-colors',
                    sortBy === option.key
                      ? 'bg-brand-50 text-brand-700'
                      : 'hover:bg-gray-100'
                  )}
                >
                  {option.label}
                  {sortBy === option.key &&
                    (sortOrder === 'desc' ? (
                      <SortDesc className="w-3 h-3" />
                    ) : (
                      <SortAsc className="w-3 h-3" />
                    ))}
                </button>
              ))}
            </div>
          </div>

          {/* Results Table */}
          {isLoading ? (
            <div className="p-8">
              <div className="animate-pulse space-y-4">
                {[1, 2, 3, 4, 5].map((i) => (
                  <div key={i} className="h-16 bg-gray-100 rounded"></div>
                ))}
              </div>
            </div>
          ) : searchResults?.spans.length === 0 ? (
            <div className="p-12 text-center">
              <Search className="w-12 h-12 text-gray-300 mx-auto mb-4" />
              <p className="text-gray-500">No results found</p>
              <p className="text-sm text-gray-400 mt-1">
                Try adjusting your filters or search query
              </p>
            </div>
          ) : (
            <div className="divide-y divide-gray-100">
              {searchResults?.spans.map((span) => (
                <SearchResultRow key={span.id} span={span} />
              ))}
            </div>
          )}

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="px-6 py-4 border-t border-gray-200 flex items-center justify-between">
              <button
                onClick={() => setPage((p) => Math.max(0, p - 1))}
                disabled={page === 0}
                className="px-3 py-1.5 border border-gray-300 rounded text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Previous
              </button>
              <span className="text-sm text-gray-500">
                Page {page + 1} of {totalPages}
              </span>
              <button
                onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
                disabled={page >= totalPages - 1}
                className="px-3 py-1.5 border border-gray-300 rounded text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Next
              </button>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

function SearchResultRow({ span }: { span: Span }) {
  return (
    <Link
      href={`/traces/${span.trace_id}`}
      className="block px-6 py-4 hover:bg-gray-50 transition-colors"
    >
      <div className="flex items-start justify-between">
        <div className="flex items-start gap-3">
          <SpanIcon span={span} size="md" />
          <div>
            <div className="flex items-center gap-2">
              <span className="font-medium text-gray-900">{span.operation_name}</span>
              {span.status === 'error' ? (
                <AlertCircle className="w-4 h-4 text-red-500" />
              ) : span.status === 'ok' ? (
                <CheckCircle className="w-4 h-4 text-green-500" />
              ) : null}
            </div>
            <div className="flex items-center gap-3 mt-1 text-sm text-gray-500">
              <span>{span.service_name}</span>
              {span.model_name && (
                <>
                  <span className="text-gray-300">|</span>
                  <span>{span.model_name}</span>
                </>
              )}
              <span className="text-gray-300">|</span>
              <span className="font-mono text-xs">{span.trace_id.slice(0, 8)}...</span>
            </div>
          </div>
        </div>
        <div className="text-right text-sm">
          <div className="flex items-center gap-4">
            {span.duration_ms != null && (
              <div className="flex items-center gap-1 text-gray-500">
                <Clock className="w-4 h-4" />
                {formatDuration(span.duration_ms)}
              </div>
            )}
            {span.tokens_in != null && span.tokens_out != null && (
              <div className="text-gray-500">{formatTokens(span.tokens_in + span.tokens_out)} tok</div>
            )}
            {span.cost_usd != null && span.cost_usd > 0 && (
              <div className="font-medium text-gray-900">{formatCost(span.cost_usd)}</div>
            )}
          </div>
          <div className="text-xs text-gray-400 mt-1">{formatRelativeTime(span.started_at)}</div>
        </div>
      </div>
      {span.prompt_preview && (
        <div className="mt-2 ml-8 text-sm text-gray-500 line-clamp-1">
          {span.prompt_preview}
        </div>
      )}
    </Link>
  );
}
