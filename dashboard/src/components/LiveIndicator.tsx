'use client';

import { Wifi, WifiOff, Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils';

interface LiveIndicatorProps {
  isConnected: boolean;
  isConnecting: boolean;
  error?: string | null;
  spanCount?: number;
  onConnect?: () => void;
  onDisconnect?: () => void;
  className?: string;
}

export function LiveIndicator({
  isConnected,
  isConnecting,
  error,
  spanCount = 0,
  onConnect,
  onDisconnect,
  className,
}: LiveIndicatorProps) {
  const handleClick = () => {
    if (isConnected) {
      onDisconnect?.();
    } else if (!isConnecting) {
      onConnect?.();
    }
  };

  return (
    <button
      onClick={handleClick}
      className={cn(
        'flex items-center gap-2 px-3 py-1.5 rounded-full text-sm font-medium transition-all',
        isConnected
          ? 'bg-green-100 text-green-700 hover:bg-green-200'
          : isConnecting
            ? 'bg-yellow-100 text-yellow-700'
            : error
              ? 'bg-red-100 text-red-700 hover:bg-red-200'
              : 'bg-gray-100 text-gray-600 hover:bg-gray-200',
        className
      )}
      title={
        isConnected
          ? 'Connected - Click to disconnect'
          : isConnecting
            ? 'Connecting...'
            : error || 'Disconnected - Click to connect'
      }
    >
      {isConnecting ? (
        <Loader2 className="w-4 h-4 animate-spin" />
      ) : isConnected ? (
        <>
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
          </span>
          <Wifi className="w-4 h-4" />
        </>
      ) : (
        <WifiOff className="w-4 h-4" />
      )}
      <span>
        {isConnected
          ? `Live${spanCount > 0 ? ` (${spanCount})` : ''}`
          : isConnecting
            ? 'Connecting'
            : 'Offline'}
      </span>
    </button>
  );
}
