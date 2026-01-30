'use client';

import { useEffect, useRef, useState, useCallback } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import type { Span, SpanListResponse } from '@/types';

interface UseStreamOptions {
  /** Automatically start streaming on mount */
  autoConnect?: boolean;
  /** Filter by trace ID */
  traceId?: string;
  /** Channel to subscribe to: "spans", "llm" */
  channel?: 'spans' | 'llm';
}

interface StreamState {
  isConnected: boolean;
  isConnecting: boolean;
  error: string | null;
  lastSpan: Span | null;
  spanCount: number;
}

export function useStream(options: UseStreamOptions = {}) {
  const { autoConnect = true, traceId, channel = 'spans' } = options;
  const queryClient = useQueryClient();
  const eventSourceRef = useRef<EventSource | null>(null);
  const [state, setState] = useState<StreamState>({
    isConnected: false,
    isConnecting: false,
    error: null,
    lastSpan: null,
    spanCount: 0,
  });

  const connect = useCallback(() => {
    // Don't connect if already connected
    if (eventSourceRef.current) return;

    setState((prev) => ({ ...prev, isConnecting: true, error: null }));

    // Build the URL with query params
    const params = new URLSearchParams();
    if (traceId) params.set('trace_id', traceId);
    else if (channel) params.set('channel', channel);

    const url = `/api/v1/stream${params.toString() ? `?${params}` : ''}`;

    try {
      const es = new EventSource(url);
      eventSourceRef.current = es;

      es.onopen = () => {
        setState((prev) => ({
          ...prev,
          isConnected: true,
          isConnecting: false,
          error: null,
        }));
      };

      es.addEventListener('span', (event) => {
        try {
          const span: Span = JSON.parse(event.data);

          setState((prev) => ({
            ...prev,
            lastSpan: span,
            spanCount: prev.spanCount + 1,
          }));

          // Update the React Query cache with the new span
          queryClient.setQueryData<SpanListResponse>(['spans'], (old) => {
            if (!old) return { spans: [span], total: 1 };

            // Check if span already exists (by span_id)
            const exists = old.spans.some((s) => s.span_id === span.span_id);
            if (exists) {
              // Update existing span
              return {
                ...old,
                spans: old.spans.map((s) =>
                  s.span_id === span.span_id ? span : s
                ),
              };
            }

            // Add new span at the beginning
            return {
              ...old,
              spans: [span, ...old.spans],
              total: old.total + 1,
            };
          });

          // Also invalidate trace-specific queries if applicable
          if (span.trace_id) {
            queryClient.invalidateQueries({
              queryKey: ['trace', span.trace_id],
            });
          }
        } catch (e) {
          console.error('Failed to parse span event:', e);
        }
      });

      es.onerror = () => {
        setState((prev) => ({
          ...prev,
          isConnected: false,
          isConnecting: false,
          error: 'Connection lost. Retrying...',
        }));

        // EventSource will automatically retry
      };
    } catch (e) {
      setState((prev) => ({
        ...prev,
        isConnecting: false,
        error: e instanceof Error ? e.message : 'Failed to connect',
      }));
    }
  }, [traceId, channel, queryClient]);

  const disconnect = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
      setState((prev) => ({
        ...prev,
        isConnected: false,
        isConnecting: false,
      }));
    }
  }, []);

  // Auto-connect on mount if enabled
  useEffect(() => {
    if (autoConnect) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [autoConnect, connect, disconnect]);

  return {
    ...state,
    connect,
    disconnect,
  };
}
