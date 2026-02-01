'use client';

import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Navbar } from '@/components/Navbar';
import { formatDate, formatRelativeTime, cn } from '@/lib/utils';
import {
  Bell,
  Plus,
  Trash2,
  CheckCircle,
  AlertTriangle,
  AlertCircle,
  Info,
  X,
  Play,
  History,
} from 'lucide-react';

interface AlertRule {
  id: string;
  name: string;
  description?: string;
  service_name?: string;
  model_name?: string;
  condition_type: 'threshold' | 'anomaly' | 'rate_change' | 'absence';
  metric: string;
  operator: 'gt' | 'lt' | 'eq' | 'gte' | 'lte' | 'ne';
  threshold: number;
  window_minutes: number;
  severity: 'info' | 'warning' | 'critical';
  enabled: boolean;
  last_triggered_at?: string;
  created_at: string;
}

interface AlertEvent {
  id: string;
  rule_id: string;
  triggered_at: string;
  resolved_at?: string;
  severity: string;
  status: 'active' | 'acknowledged' | 'resolved';
  message: string;
  metric_value: number;
  threshold_value: number;
  service_name?: string;
}

interface CreateRuleInput {
  name: string;
  metric: string;
  operator: string;
  threshold: number;
  severity: string;
  service_name?: string;
  condition_type: string;
}

async function fetchAlertRules(): Promise<AlertRule[]> {
  const res = await fetch('/api/v1/alerts/rules');
  if (!res.ok) throw new Error('Failed to fetch alert rules');
  return res.json();
}

async function fetchAlertEvents(): Promise<AlertEvent[]> {
  const res = await fetch('/api/v1/alerts/events?limit=50');
  if (!res.ok) throw new Error('Failed to fetch alert events');
  return res.json();
}

async function createAlertRule(input: CreateRuleInput): Promise<AlertRule> {
  const res = await fetch('/api/v1/alerts/rules', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(input),
  });
  if (!res.ok) throw new Error('Failed to create alert rule');
  return res.json();
}

async function deleteAlertRule(ruleId: string): Promise<void> {
  const res = await fetch(`/api/v1/alerts/rules/${ruleId}`, { method: 'DELETE' });
  if (!res.ok) throw new Error('Failed to delete alert rule');
}

async function testAlertRule(ruleId: string): Promise<{ would_trigger: boolean; current_value?: number }> {
  const res = await fetch(`/api/v1/alerts/rules/${ruleId}/test`, { method: 'POST' });
  if (!res.ok) throw new Error('Failed to test alert rule');
  return res.json();
}

async function acknowledgeAlert(eventId: string): Promise<void> {
  const res = await fetch(`/api/v1/alerts/events/${eventId}/acknowledge`, { method: 'POST' });
  if (!res.ok) throw new Error('Failed to acknowledge alert');
}

export default function AlertsPage() {
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [activeTab, setActiveTab] = useState<'rules' | 'events'>('rules');
  const queryClient = useQueryClient();

  const { data: rules = [], isLoading: rulesLoading } = useQuery({
    queryKey: ['alert-rules'],
    queryFn: fetchAlertRules,
  });

  const { data: events = [], isLoading: eventsLoading } = useQuery({
    queryKey: ['alert-events'],
    queryFn: fetchAlertEvents,
  });

  const deleteMutation = useMutation({
    mutationFn: deleteAlertRule,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['alert-rules'] }),
  });

  const testMutation = useMutation({
    mutationFn: testAlertRule,
  });

  const acknowledgeMutation = useMutation({
    mutationFn: acknowledgeAlert,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['alert-events'] }),
  });

  const activeAlerts = events.filter((e) => e.status === 'active');

  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar />

      <main className="pt-20 pb-12 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
        <div className="mb-8 flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-gray-900">Alerts</h1>
            <p className="text-gray-500 mt-1">Monitor your AI agents and get notified of issues</p>
          </div>
          <button
            onClick={() => setShowCreateModal(true)}
            className="flex items-center gap-2 px-4 py-2 bg-brand-600 text-white rounded-lg hover:bg-brand-700 transition-colors"
          >
            <Plus className="w-4 h-4" />
            Create Rule
          </button>
        </div>

        {/* Active Alerts Banner */}
        {activeAlerts.length > 0 && (
          <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
            <div className="flex items-center gap-3">
              <AlertCircle className="w-5 h-5 text-red-600" />
              <div>
                <p className="font-semibold text-red-800">
                  {activeAlerts.length} Active Alert{activeAlerts.length !== 1 ? 's' : ''}
                </p>
                <p className="text-sm text-red-600">
                  {activeAlerts.map((a) => a.message).join('; ')}
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Tabs */}
        <div className="border-b border-gray-200 mb-6">
          <nav className="flex gap-8">
            <button
              onClick={() => setActiveTab('rules')}
              className={cn(
                'pb-4 text-sm font-medium border-b-2 transition-colors',
                activeTab === 'rules'
                  ? 'border-brand-500 text-brand-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              )}
            >
              <div className="flex items-center gap-2">
                <Bell className="w-4 h-4" />
                Alert Rules
                <span className="px-2 py-0.5 bg-gray-100 rounded-full text-xs">
                  {rules.length}
                </span>
              </div>
            </button>
            <button
              onClick={() => setActiveTab('events')}
              className={cn(
                'pb-4 text-sm font-medium border-b-2 transition-colors',
                activeTab === 'events'
                  ? 'border-brand-500 text-brand-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              )}
            >
              <div className="flex items-center gap-2">
                <History className="w-4 h-4" />
                Alert History
                {activeAlerts.length > 0 && (
                  <span className="px-2 py-0.5 bg-red-100 text-red-700 rounded-full text-xs">
                    {activeAlerts.length}
                  </span>
                )}
              </div>
            </button>
          </nav>
        </div>

        {/* Content */}
        {activeTab === 'rules' ? (
          <div className="space-y-4">
            {rulesLoading ? (
              <div className="animate-pulse space-y-4">
                {[1, 2, 3].map((i) => (
                  <div key={i} className="h-24 bg-white rounded-lg border border-gray-200"></div>
                ))}
              </div>
            ) : rules.length === 0 ? (
              <div className="bg-white rounded-lg border border-gray-200 p-12 text-center">
                <Bell className="w-12 h-12 text-gray-300 mx-auto mb-4" />
                <p className="text-gray-500">No alert rules configured</p>
                <p className="text-sm text-gray-400 mt-1">
                  Create a rule to get notified of issues
                </p>
                <button
                  onClick={() => setShowCreateModal(true)}
                  className="mt-4 px-4 py-2 bg-brand-600 text-white rounded-lg hover:bg-brand-700 transition-colors"
                >
                  Create First Rule
                </button>
              </div>
            ) : (
              rules.map((rule) => (
                <AlertRuleCard
                  key={rule.id}
                  rule={rule}
                  onDelete={() => deleteMutation.mutate(rule.id)}
                  onTest={() => testMutation.mutate(rule.id)}
                  isDeleting={deleteMutation.isPending}
                  isTesting={testMutation.isPending}
                  testResult={
                    testMutation.data && testMutation.variables === rule.id
                      ? testMutation.data
                      : undefined
                  }
                />
              ))
            )}
          </div>
        ) : (
          <div className="space-y-4">
            {eventsLoading ? (
              <div className="animate-pulse space-y-4">
                {[1, 2, 3].map((i) => (
                  <div key={i} className="h-20 bg-white rounded-lg border border-gray-200"></div>
                ))}
              </div>
            ) : events.length === 0 ? (
              <div className="bg-white rounded-lg border border-gray-200 p-12 text-center">
                <History className="w-12 h-12 text-gray-300 mx-auto mb-4" />
                <p className="text-gray-500">No alert events yet</p>
                <p className="text-sm text-gray-400 mt-1">
                  Events will appear here when alerts are triggered
                </p>
              </div>
            ) : (
              events.map((event) => (
                <AlertEventCard
                  key={event.id}
                  event={event}
                  onAcknowledge={() => acknowledgeMutation.mutate(event.id)}
                  isAcknowledging={acknowledgeMutation.isPending}
                />
              ))
            )}
          </div>
        )}
      </main>

      {/* Create Rule Modal */}
      {showCreateModal && (
        <CreateRuleModal
          onClose={() => setShowCreateModal(false)}
          onSuccess={() => {
            setShowCreateModal(false);
            queryClient.invalidateQueries({ queryKey: ['alert-rules'] });
          }}
        />
      )}
    </div>
  );
}

interface AlertRuleCardProps {
  rule: AlertRule;
  onDelete: () => void;
  onTest: () => void;
  isDeleting: boolean;
  isTesting: boolean;
  testResult?: { would_trigger: boolean; current_value?: number };
}

function AlertRuleCard({ rule, onDelete, onTest, isDeleting, isTesting, testResult }: AlertRuleCardProps) {
  const severityColors = {
    info: 'bg-blue-50 text-blue-700 border-blue-200',
    warning: 'bg-yellow-50 text-yellow-700 border-yellow-200',
    critical: 'bg-red-50 text-red-700 border-red-200',
  };

  const operatorSymbols: Record<string, string> = {
    gt: '>',
    lt: '<',
    eq: '=',
    gte: '>=',
    lte: '<=',
    ne: '!=',
  };

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-3 mb-2">
            <h3 className="font-semibold text-gray-900">{rule.name}</h3>
            <span
              className={cn(
                'px-2 py-0.5 text-xs font-medium rounded-full border',
                severityColors[rule.severity]
              )}
            >
              {rule.severity}
            </span>
            {!rule.enabled && (
              <span className="px-2 py-0.5 text-xs font-medium rounded-full bg-gray-100 text-gray-500">
                Disabled
              </span>
            )}
          </div>
          <p className="text-sm text-gray-600 mb-2">
            {rule.metric} {operatorSymbols[rule.operator]} {rule.threshold}
            {rule.service_name && ` (service: ${rule.service_name})`}
          </p>
          <div className="flex items-center gap-4 text-xs text-gray-500">
            <span>Window: {rule.window_minutes}min</span>
            {rule.last_triggered_at && (
              <span>Last triggered: {formatRelativeTime(rule.last_triggered_at)}</span>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={onTest}
            disabled={isTesting}
            className="p-2 text-gray-400 hover:text-brand-600 hover:bg-brand-50 rounded transition-colors"
            title="Test rule"
          >
            <Play className="w-4 h-4" />
          </button>
          <button
            onClick={onDelete}
            disabled={isDeleting}
            className="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded transition-colors"
            title="Delete rule"
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      </div>
      {testResult && (
        <div
          className={cn(
            'mt-3 p-2 rounded text-sm',
            testResult.would_trigger ? 'bg-yellow-50 text-yellow-800' : 'bg-green-50 text-green-800'
          )}
        >
          {testResult.would_trigger
            ? `Would trigger (current value: ${testResult.current_value?.toFixed(4)})`
            : `Would not trigger (current value: ${testResult.current_value?.toFixed(4)})`}
        </div>
      )}
    </div>
  );
}

interface AlertEventCardProps {
  event: AlertEvent;
  onAcknowledge: () => void;
  isAcknowledging: boolean;
}

function AlertEventCard({ event, onAcknowledge, isAcknowledging }: AlertEventCardProps) {
  const statusColors = {
    active: 'bg-red-50 border-red-200',
    acknowledged: 'bg-yellow-50 border-yellow-200',
    resolved: 'bg-green-50 border-green-200',
  };

  const StatusIcon =
    event.status === 'active'
      ? AlertCircle
      : event.status === 'acknowledged'
        ? AlertTriangle
        : CheckCircle;

  const statusIconColors = {
    active: 'text-red-600',
    acknowledged: 'text-yellow-600',
    resolved: 'text-green-600',
  };

  return (
    <div className={cn('rounded-lg border p-4', statusColors[event.status])}>
      <div className="flex items-start justify-between">
        <div className="flex items-start gap-3">
          <StatusIcon className={cn('w-5 h-5 mt-0.5', statusIconColors[event.status])} />
          <div>
            <p className="font-medium text-gray-900">{event.message}</p>
            <div className="flex items-center gap-3 mt-1 text-sm text-gray-600">
              <span>Value: {event.metric_value.toFixed(4)}</span>
              <span>Threshold: {event.threshold_value}</span>
              {event.service_name && <span>Service: {event.service_name}</span>}
            </div>
            <p className="text-xs text-gray-500 mt-1">
              Triggered {formatRelativeTime(event.triggered_at)}
              {event.resolved_at && ` - Resolved ${formatRelativeTime(event.resolved_at)}`}
            </p>
          </div>
        </div>
        {event.status === 'active' && (
          <button
            onClick={onAcknowledge}
            disabled={isAcknowledging}
            className="px-3 py-1.5 bg-white border border-gray-300 rounded text-sm font-medium text-gray-700 hover:bg-gray-50 transition-colors"
          >
            Acknowledge
          </button>
        )}
      </div>
    </div>
  );
}

interface CreateRuleModalProps {
  onClose: () => void;
  onSuccess: () => void;
}

function CreateRuleModal({ onClose, onSuccess }: CreateRuleModalProps) {
  const [form, setForm] = useState({
    name: '',
    metric: 'error_rate',
    operator: 'gt',
    threshold: 5,
    severity: 'warning',
    service_name: '',
  });

  const mutation = useMutation({
    mutationFn: createAlertRule,
    onSuccess,
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    mutation.mutate({
      ...form,
      condition_type: 'threshold',
      service_name: form.service_name || undefined,
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-md mx-4">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">Create Alert Rule</h2>
          <button onClick={onClose} className="text-gray-400 hover:text-gray-600">
            <X className="w-5 h-5" />
          </button>
        </div>
        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Name</label>
            <input
              type="text"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-brand-500"
              placeholder="High Error Rate"
              required
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Metric</label>
              <select
                value={form.metric}
                onChange={(e) => setForm({ ...form, metric: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-brand-500"
              >
                <option value="error_rate">Error Rate (%)</option>
                <option value="latency_p99">P99 Latency (ms)</option>
                <option value="latency_p95">P95 Latency (ms)</option>
                <option value="cost_sum">Total Cost ($)</option>
                <option value="token_usage">Token Usage</option>
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Condition</label>
              <div className="flex gap-2">
                <select
                  value={form.operator}
                  onChange={(e) => setForm({ ...form, operator: e.target.value })}
                  className="w-20 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-brand-500"
                >
                  <option value="gt">&gt;</option>
                  <option value="gte">&gt;=</option>
                  <option value="lt">&lt;</option>
                  <option value="lte">&lt;=</option>
                  <option value="eq">=</option>
                </select>
                <input
                  type="number"
                  step="any"
                  value={form.threshold}
                  onChange={(e) => setForm({ ...form, threshold: parseFloat(e.target.value) })}
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-brand-500"
                  required
                />
              </div>
            </div>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Severity</label>
            <select
              value={form.severity}
              onChange={(e) => setForm({ ...form, severity: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-brand-500"
            >
              <option value="info">Info</option>
              <option value="warning">Warning</option>
              <option value="critical">Critical</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Service (optional)
            </label>
            <input
              type="text"
              value={form.service_name}
              onChange={(e) => setForm({ ...form, service_name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-brand-500"
              placeholder="All services"
            />
          </div>
          {mutation.isError && (
            <div className="p-3 bg-red-50 border border-red-200 rounded text-sm text-red-700">
              Failed to create rule. Please try again.
            </div>
          )}
          <div className="flex justify-end gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={mutation.isPending}
              className="px-4 py-2 bg-brand-600 text-white rounded-lg hover:bg-brand-700 transition-colors disabled:opacity-50"
            >
              {mutation.isPending ? 'Creating...' : 'Create Rule'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
