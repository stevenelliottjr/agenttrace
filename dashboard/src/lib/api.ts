import type { Span, SpanListResponse, HealthResponse, Trace } from '@/types';

const API_BASE = '/api/v1';

async function fetchAPI<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!res.ok) {
    throw new Error(`API error: ${res.status} ${res.statusText}`);
  }

  return res.json();
}

export async function getHealth(): Promise<HealthResponse> {
  const res = await fetch('/health');
  return res.json();
}

export async function getSpans(params?: {
  trace_id?: string;
  service_name?: string;
  limit?: number;
}): Promise<SpanListResponse> {
  const searchParams = new URLSearchParams();
  if (params?.trace_id) searchParams.set('trace_id', params.trace_id);
  if (params?.service_name) searchParams.set('service_name', params.service_name);
  if (params?.limit) searchParams.set('limit', params.limit.toString());

  const query = searchParams.toString();
  return fetchAPI<SpanListResponse>(`/spans${query ? `?${query}` : ''}`);
}

export async function getSpan(spanId: string): Promise<Span> {
  return fetchAPI<Span>(`/spans/${spanId}`);
}

export async function getTraceSpans(traceId: string): Promise<Span[]> {
  const response = await getSpans({ trace_id: traceId, limit: 1000 });
  return response.spans;
}

export function buildTrace(spans: Span[]): Trace {
  if (spans.length === 0) {
    return {
      trace_id: '',
      spans: [],
      root_span: null,
      total_duration_ms: 0,
      total_tokens: 0,
      total_cost_usd: 0,
      service_names: [],
      started_at: new Date().toISOString(),
    };
  }

  // Sort by start time
  const sortedSpans = [...spans].sort(
    (a, b) => new Date(a.started_at).getTime() - new Date(b.started_at).getTime()
  );

  // Find root span (no parent)
  const rootSpan = sortedSpans.find((s) => !s.parent_span_id) || sortedSpans[0];

  // Calculate totals
  const totalTokens = spans.reduce(
    (sum, s) => sum + (s.tokens_in || 0) + (s.tokens_out || 0),
    0
  );
  const totalCost = spans.reduce((sum, s) => sum + (s.cost_usd || 0), 0);
  const serviceNames = [...new Set(spans.map((s) => s.service_name))];

  // Calculate total duration from root span or from first to last
  let totalDuration = rootSpan.duration_ms || 0;
  if (!totalDuration && sortedSpans.length > 0) {
    const firstStart = new Date(sortedSpans[0].started_at).getTime();
    const lastEnd = Math.max(
      ...sortedSpans.map((s) =>
        s.ended_at ? new Date(s.ended_at).getTime() : new Date(s.started_at).getTime()
      )
    );
    totalDuration = lastEnd - firstStart;
  }

  return {
    trace_id: rootSpan.trace_id,
    spans: sortedSpans,
    root_span: rootSpan,
    total_duration_ms: totalDuration,
    total_tokens: totalTokens,
    total_cost_usd: totalCost,
    service_names: serviceNames,
    started_at: sortedSpans[0].started_at,
  };
}

export function groupSpansByTrace(spans: Span[]): Map<string, Span[]> {
  const grouped = new Map<string, Span[]>();
  for (const span of spans) {
    const existing = grouped.get(span.trace_id) || [];
    existing.push(span);
    grouped.set(span.trace_id, existing);
  }
  return grouped;
}
