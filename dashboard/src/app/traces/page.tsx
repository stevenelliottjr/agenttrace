'use client';

import { useState } from 'react';
import { Search, Filter, RefreshCw } from 'lucide-react';
import { Navbar } from '@/components/Navbar';
import { TraceList } from '@/components/TraceList';
import { LiveIndicator } from '@/components/LiveIndicator';
import { useQueryClient } from '@tanstack/react-query';
import { useStream } from '@/hooks/useStream';

export default function TracesPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const queryClient = useQueryClient();
  const stream = useStream({ autoConnect: true });

  const handleRefresh = () => {
    queryClient.invalidateQueries({ queryKey: ['spans'] });
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar />

      <main className="pt-20 pb-12 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-3xl font-bold text-gray-900">Traces</h1>
            <p className="text-gray-500 mt-1">
              Browse and search through your agent traces
            </p>
          </div>
          <div className="flex items-center gap-3">
            <LiveIndicator
              isConnected={stream.isConnected}
              isConnecting={stream.isConnecting}
              error={stream.error}
              spanCount={stream.spanCount}
              onConnect={stream.connect}
              onDisconnect={stream.disconnect}
            />
            <button
              onClick={handleRefresh}
              className="flex items-center gap-2 px-4 py-2 bg-white border border-gray-200 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50 transition-colors"
            >
              <RefreshCw className="w-4 h-4" />
              Refresh
            </button>
          </div>
        </div>

        {/* Filters */}
        <div className="flex items-center gap-4 mb-6">
          <div className="relative flex-1 max-w-md">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
            <input
              type="text"
              placeholder="Search by trace ID, operation, or service..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500 focus:border-transparent"
            />
          </div>
          <button className="flex items-center gap-2 px-4 py-2 bg-white border border-gray-200 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50 transition-colors">
            <Filter className="w-4 h-4" />
            Filters
          </button>
        </div>

        {/* Trace List */}
        <TraceList />
      </main>
    </div>
  );
}
