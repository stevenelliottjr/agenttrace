export interface Span {
  id: string;
  span_id: string;
  trace_id: string;
  parent_span_id: string | null;
  operation_name: string;
  service_name: string;
  span_kind: 'internal' | 'client' | 'server' | 'producer' | 'consumer';
  started_at: string;
  ended_at: string | null;
  duration_ms: number | null;
  status: 'ok' | 'error' | 'unset';
  status_message: string | null;
  model_name: string | null;
  model_provider: string | null;
  tokens_in: number | null;
  tokens_out: number | null;
  tokens_reasoning: number | null;
  cost_usd: number | null;
  tool_name: string | null;
  tool_input: Record<string, unknown> | null;
  tool_output: Record<string, unknown> | null;
  tool_duration_ms: number | null;
  prompt_preview: string | null;
  completion_preview: string | null;
  attributes: Record<string, unknown>;
  events: SpanEvent[];
  links: SpanLink[];
}

export interface SpanEvent {
  name: string;
  timestamp: string;
  attributes: Record<string, unknown>;
}

export interface SpanLink {
  trace_id: string;
  span_id: string;
  attributes: Record<string, unknown>;
}

export interface Trace {
  trace_id: string;
  spans: Span[];
  root_span: Span | null;
  total_duration_ms: number;
  total_tokens: number;
  total_cost_usd: number;
  service_names: string[];
  started_at: string;
}

export interface SpanListResponse {
  spans: Span[];
  total: number;
}

export interface HealthResponse {
  status: string;
  version: string;
}

export type SpanType = 'llm_call' | 'tool_call' | 'agent_step' | 'retrieval' | 'embedding' | 'chain' | 'custom';

export function getSpanType(span: Span): SpanType {
  if (span.model_name) return 'llm_call';
  if (span.tool_name) return 'tool_call';
  if (span.operation_name.includes('retrieval') || span.operation_name.includes('search')) return 'retrieval';
  if (span.operation_name.includes('embed')) return 'embedding';
  if (span.operation_name.includes('chain')) return 'chain';
  if (span.operation_name.includes('agent') || span.operation_name.includes('workflow')) return 'agent_step';
  return 'custom';
}
