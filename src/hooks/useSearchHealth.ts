import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface SearchHealthStatus {
  isHealthy: boolean;
  isChecking: boolean;
  lastChecked: Date | null;
  error: string | null;
  responseTime: number | null;
}

export interface UseSearchHealthOptions {
  checkInterval?: number; // milliseconds
  enabled?: boolean;
  onHealthChange?: (status: SearchHealthStatus) => void;
}

export function useSearchHealth(options: UseSearchHealthOptions = {}) {
  const {
    checkInterval = 30000, // 30 seconds default
    enabled = true,
    onHealthChange
  } = options;

  const [status, setStatus] = useState<SearchHealthStatus>({
    isHealthy: false,
    isChecking: false,
    lastChecked: null,
    error: null,
    responseTime: null
  });

  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const mountedRef = useRef(true);

  const checkHealth = async (): Promise<SearchHealthStatus> => {
    const startTime = Date.now();
    
    try {
      const isConnected = await invoke<boolean>('check_searxng_connection');
      const responseTime = Date.now() - startTime;
      
      return {
        isHealthy: isConnected,
        isChecking: false,
        lastChecked: new Date(),
        error: null,
        responseTime
      };
    } catch (error) {
      const responseTime = Date.now() - startTime;
      
      return {
        isHealthy: false,
        isChecking: false,
        lastChecked: new Date(),
        error: error instanceof Error ? error.message : 'Unknown error',
        responseTime
      };
    }
  };

  const performHealthCheck = async () => {
    if (!mountedRef.current) return;

    setStatus(prev => ({ ...prev, isChecking: true }));
    
    try {
      const newStatus = await checkHealth();
      
      if (!mountedRef.current) return;
      
      setStatus(newStatus);
      
      if (onHealthChange) {
        onHealthChange(newStatus);
      }
    } catch (error) {
      if (!mountedRef.current) return;
      
      const errorStatus: SearchHealthStatus = {
        isHealthy: false,
        isChecking: false,
        lastChecked: new Date(),
        error: error instanceof Error ? error.message : 'Health check failed',
        responseTime: null
      };
      
      setStatus(errorStatus);
      
      if (onHealthChange) {
        onHealthChange(errorStatus);
      }
    }
  };

  // Manual health check function
  const checkHealthNow = async (): Promise<SearchHealthStatus> => {
    const newStatus = await checkHealth();
    setStatus(newStatus);
    return newStatus;
  };

  useEffect(() => {
    mountedRef.current = true;

    if (!enabled) {
      return;
    }

    // Perform initial health check
    performHealthCheck();

    // Set up periodic health checks
    if (checkInterval > 0) {
      intervalRef.current = setInterval(performHealthCheck, checkInterval);
    }

    return () => {
      mountedRef.current = false;
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [enabled, checkInterval]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      mountedRef.current = false;
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  return {
    status,
    checkHealthNow,
    isHealthy: status.isHealthy,
    isChecking: status.isChecking,
    lastChecked: status.lastChecked,
    error: status.error,
    responseTime: status.responseTime
  };
}