import { useQuery } from '@tanstack/react-query';
import { getSpans, getTraceSpans, buildTrace, groupSpansByTrace } from '@/lib/api';
import type { Trace } from '@/types';

export function useSpans(params?: {
  trace_id?: string;
  service_name?: string;
  limit?: number;
}) {
  return useQuery({
    queryKey: ['spans', params],
    queryFn: () => getSpans(params),
    refetchInterval: 5000, // Refresh every 5 seconds
  });
}

export function useTraceSpans(traceId: string | undefined) {
  return useQuery({
    queryKey: ['trace', traceId],
    queryFn: () => getTraceSpans(traceId!),
    enabled: !!traceId,
  });
}

export function useTrace(traceId: string | undefined) {
  const { data: spans, ...rest } = useTraceSpans(traceId);

  const trace = spans ? buildTrace(spans) : undefined;

  return {
    data: trace,
    ...rest,
  };
}

export function useRecentTraces(limit: number = 50) {
  const { data, ...rest } = useSpans({ limit });

  const traces: Trace[] = [];
  if (data?.spans) {
    const grouped = groupSpansByTrace(data.spans);
    for (const [, spans] of grouped) {
      traces.push(buildTrace(spans));
    }
    // Sort by most recent
    traces.sort(
      (a, b) => new Date(b.started_at).getTime() - new Date(a.started_at).getTime()
    );
  }

  return {
    data: traces,
    ...rest,
  };
}
