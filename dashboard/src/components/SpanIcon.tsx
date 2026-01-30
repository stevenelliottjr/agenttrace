'use client';

import {
  Brain,
  Wrench,
  Bot,
  Search,
  Database,
  Link,
  Circle,
  AlertCircle,
  CheckCircle,
} from 'lucide-react';
import type { Span, SpanType } from '@/types';
import { getSpanType } from '@/types';
import { cn } from '@/lib/utils';

const spanTypeConfig: Record<
  SpanType,
  { icon: typeof Brain; color: string; bgColor: string; label: string }
> = {
  llm_call: {
    icon: Brain,
    color: 'text-purple-600',
    bgColor: 'bg-purple-100',
    label: 'LLM Call',
  },
  tool_call: {
    icon: Wrench,
    color: 'text-blue-600',
    bgColor: 'bg-blue-100',
    label: 'Tool',
  },
  agent_step: {
    icon: Bot,
    color: 'text-green-600',
    bgColor: 'bg-green-100',
    label: 'Agent',
  },
  retrieval: {
    icon: Search,
    color: 'text-orange-600',
    bgColor: 'bg-orange-100',
    label: 'Retrieval',
  },
  embedding: {
    icon: Database,
    color: 'text-cyan-600',
    bgColor: 'bg-cyan-100',
    label: 'Embedding',
  },
  chain: {
    icon: Link,
    color: 'text-indigo-600',
    bgColor: 'bg-indigo-100',
    label: 'Chain',
  },
  custom: {
    icon: Circle,
    color: 'text-gray-600',
    bgColor: 'bg-gray-100',
    label: 'Custom',
  },
};

interface SpanIconProps {
  span: Span;
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
}

export function SpanIcon({ span, size = 'md', showLabel = false }: SpanIconProps) {
  const spanType = getSpanType(span);
  const config = spanTypeConfig[spanType];
  const Icon = config.icon;

  const sizeClasses = {
    sm: 'w-4 h-4',
    md: 'w-5 h-5',
    lg: 'w-6 h-6',
  };

  const containerClasses = {
    sm: 'p-1',
    md: 'p-1.5',
    lg: 'p-2',
  };

  return (
    <div className="flex items-center gap-2">
      <div className={cn('rounded-md', config.bgColor, containerClasses[size])}>
        <Icon className={cn(sizeClasses[size], config.color)} />
      </div>
      {showLabel && (
        <span className={cn('text-sm font-medium', config.color)}>{config.label}</span>
      )}
    </div>
  );
}

interface StatusIconProps {
  status: 'ok' | 'error' | 'unset';
  size?: 'sm' | 'md' | 'lg';
}

export function StatusIcon({ status, size = 'md' }: StatusIconProps) {
  const sizeClasses = {
    sm: 'w-3 h-3',
    md: 'w-4 h-4',
    lg: 'w-5 h-5',
  };

  if (status === 'error') {
    return <AlertCircle className={cn(sizeClasses[size], 'text-red-500')} />;
  }

  if (status === 'ok') {
    return <CheckCircle className={cn(sizeClasses[size], 'text-green-500')} />;
  }

  return <Circle className={cn(sizeClasses[size], 'text-gray-400')} />;
}
