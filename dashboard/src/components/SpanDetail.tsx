'use client';

import { X, Copy, Check } from 'lucide-react';
import { useState } from 'react';
import type { Span } from '@/types';
import { SpanIcon, StatusIcon } from './SpanIcon';
import { formatDuration, formatTokens, formatCost, formatDate, cn } from '@/lib/utils';

interface SpanDetailProps {
  span: Span;
  onClose: () => void;
}

export function SpanDetail({ span, onClose }: SpanDetailProps) {
  return (
    <div className="h-full flex flex-col bg-white border-l border-gray-200">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200">
        <div className="flex items-center gap-3">
          <SpanIcon span={span} size="md" />
          <div>
            <h3 className="font-semibold text-gray-900">{span.operation_name}</h3>
            <p className="text-sm text-gray-500">{span.service_name}</p>
          </div>
        </div>
        <button
          onClick={onClose}
          className="p-1 hover:bg-gray-100 rounded-lg transition-colors"
        >
          <X className="w-5 h-5 text-gray-500" />
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {/* Status & Timing */}
        <Section title="Overview">
          <div className="grid grid-cols-2 gap-4">
            <InfoItem
              label="Status"
              value={
                <div className="flex items-center gap-2">
                  <StatusIcon status={span.status} />
                  <span className={span.status === 'error' ? 'text-red-600' : ''}>
                    {span.status}
                  </span>
                </div>
              }
            />
            <InfoItem label="Duration" value={formatDuration(span.duration_ms)} mono />
            <InfoItem label="Started" value={formatDate(span.started_at)} />
            <InfoItem
              label="Ended"
              value={span.ended_at ? formatDate(span.ended_at) : '-'}
            />
          </div>
          {span.status_message && (
            <div className="mt-3 p-3 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-sm text-red-700">{span.status_message}</p>
            </div>
          )}
        </Section>

        {/* IDs */}
        <Section title="Identifiers">
          <div className="space-y-2">
            <CopyableId label="Span ID" value={span.span_id} />
            <CopyableId label="Trace ID" value={span.trace_id} />
            {span.parent_span_id && (
              <CopyableId label="Parent Span ID" value={span.parent_span_id} />
            )}
          </div>
        </Section>

        {/* LLM Details */}
        {(span.model_name || span.tokens_in || span.tokens_out) && (
          <Section title="LLM Details">
            <div className="grid grid-cols-2 gap-4">
              {span.model_name && <InfoItem label="Model" value={span.model_name} />}
              {span.model_provider && (
                <InfoItem label="Provider" value={span.model_provider} />
              )}
              <InfoItem label="Tokens In" value={formatTokens(span.tokens_in)} mono />
              <InfoItem label="Tokens Out" value={formatTokens(span.tokens_out)} mono />
              {span.tokens_reasoning && (
                <InfoItem
                  label="Reasoning Tokens"
                  value={formatTokens(span.tokens_reasoning)}
                  mono
                />
              )}
              <InfoItem label="Cost" value={formatCost(span.cost_usd)} mono />
            </div>
          </Section>
        )}

        {/* Prompt & Completion */}
        {(span.prompt_preview || span.completion_preview) && (
          <Section title="Content">
            {span.prompt_preview && (
              <div className="mb-4">
                <label className="block text-xs font-medium text-gray-500 uppercase mb-1">
                  Prompt
                </label>
                <pre className="p-3 bg-gray-50 rounded-lg text-sm text-gray-700 whitespace-pre-wrap overflow-x-auto">
                  {span.prompt_preview}
                </pre>
              </div>
            )}
            {span.completion_preview && (
              <div>
                <label className="block text-xs font-medium text-gray-500 uppercase mb-1">
                  Completion
                </label>
                <pre className="p-3 bg-gray-50 rounded-lg text-sm text-gray-700 whitespace-pre-wrap overflow-x-auto">
                  {span.completion_preview}
                </pre>
              </div>
            )}
          </Section>
        )}

        {/* Tool Details */}
        {span.tool_name && (
          <Section title="Tool Details">
            <InfoItem label="Tool Name" value={span.tool_name} />
            {span.tool_input && (
              <div className="mt-3">
                <label className="block text-xs font-medium text-gray-500 uppercase mb-1">
                  Input
                </label>
                <pre className="p-3 bg-gray-50 rounded-lg text-sm text-gray-700 overflow-x-auto">
                  {JSON.stringify(span.tool_input, null, 2)}
                </pre>
              </div>
            )}
            {span.tool_output && (
              <div className="mt-3">
                <label className="block text-xs font-medium text-gray-500 uppercase mb-1">
                  Output
                </label>
                <pre className="p-3 bg-gray-50 rounded-lg text-sm text-gray-700 overflow-x-auto">
                  {JSON.stringify(span.tool_output, null, 2)}
                </pre>
              </div>
            )}
          </Section>
        )}

        {/* Attributes */}
        {Object.keys(span.attributes).length > 0 && (
          <Section title="Attributes">
            <div className="space-y-2">
              {Object.entries(span.attributes).map(([key, value]) => (
                <div key={key} className="flex justify-between text-sm">
                  <span className="text-gray-500">{key}</span>
                  <span className="text-gray-900 font-mono">
                    {typeof value === 'object' ? JSON.stringify(value) : String(value)}
                  </span>
                </div>
              ))}
            </div>
          </Section>
        )}

        {/* Events */}
        {span.events.length > 0 && (
          <Section title="Events">
            <div className="space-y-3">
              {span.events.map((event, index) => (
                <div key={index} className="p-3 bg-gray-50 rounded-lg">
                  <div className="flex justify-between items-start mb-2">
                    <span className="font-medium text-gray-900">{event.name}</span>
                    <span className="text-xs text-gray-500">
                      {formatDate(event.timestamp)}
                    </span>
                  </div>
                  {Object.keys(event.attributes).length > 0 && (
                    <pre className="text-xs text-gray-600 overflow-x-auto">
                      {JSON.stringify(event.attributes, null, 2)}
                    </pre>
                  )}
                </div>
              ))}
            </div>
          </Section>
        )}
      </div>
    </div>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="px-4 py-4 border-b border-gray-100">
      <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-3">
        {title}
      </h4>
      {children}
    </div>
  );
}

function InfoItem({
  label,
  value,
  mono = false,
}: {
  label: string;
  value: React.ReactNode;
  mono?: boolean;
}) {
  return (
    <div>
      <dt className="text-xs text-gray-500">{label}</dt>
      <dd className={cn('text-sm text-gray-900 mt-0.5', mono && 'font-mono')}>
        {value}
      </dd>
    </div>
  );
}

function CopyableId({ label, value }: { label: string; value: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex items-center justify-between p-2 bg-gray-50 rounded-lg">
      <div>
        <dt className="text-xs text-gray-500">{label}</dt>
        <dd className="text-sm font-mono text-gray-900">{value}</dd>
      </div>
      <button
        onClick={handleCopy}
        className="p-1.5 hover:bg-gray-200 rounded transition-colors"
        title="Copy to clipboard"
      >
        {copied ? (
          <Check className="w-4 h-4 text-green-500" />
        ) : (
          <Copy className="w-4 h-4 text-gray-400" />
        )}
      </button>
    </div>
  );
}
