'use client';

import { Settings } from 'lucide-react';
import { Navbar } from '@/components/Navbar';

export default function SettingsPage() {
  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar />

      <main className="pt-20 pb-12 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Settings</h1>
          <p className="text-gray-500 mt-1">
            Configure your AgentTrace instance
          </p>
        </div>

        {/* Placeholder */}
        <div className="rounded-lg bg-white border border-gray-200 p-12 text-center">
          <div className="mx-auto w-12 h-12 bg-gray-100 rounded-full flex items-center justify-center mb-4">
            <Settings className="w-6 h-6 text-gray-400" />
          </div>
          <h3 className="text-lg font-medium text-gray-900 mb-2">
            Settings Coming Soon
          </h3>
          <p className="text-gray-500 max-w-md mx-auto">
            Configuration options for alerts, retention policies, API keys, and
            integrations will be available here.
          </p>
        </div>
      </main>
    </div>
  );
}
