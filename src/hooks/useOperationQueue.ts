import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export enum OperationType {
  AICompletion = 'AICompletion',
  FileAnalysis = 'FileAnalysis',
  DocumentIndexing = 'DocumentIndexing',
  CodeGeneration = 'CodeGeneration',
  RagQuery = 'RagQuery',
  ModelLoading = 'ModelLoading',
}

export enum OperationPriority {
  Critical = 0,
  High = 1,
  Normal = 2,
  Background = 3,
  Maintenance = 4,
}

export interface ResourceRequirements {
  cpu_units: number;
  memory_mb: number;
  io_intensity: number;
  network_kb: number;
}

export interface Operation {
  id: string;
  op_type: OperationType;
  priority: OperationPriority;
  created_at: number;
  estimated_resources: ResourceRequirements;
  timeout_ms?: number;
  cancellable: boolean;
  payload: any;
}

export type OperationStatus =
  | { Queued: {} }
  | { Running: { progress?: number } }
  | { Completed: { result?: any } }
  | { Failed: { error: string } }
  | { Cancelled: {} }
  | { TimedOut: {} };

export interface OperationInfo {
  operation: Operation;
  status: OperationStatus;
}

export function useOperationQueue() {
  const [operations, setOperations] = useState<Record<string, OperationInfo>>({});

  const enqueueOperation = useCallback(async (
    opType: OperationType,
    payload: any,
    priority: OperationPriority = OperationPriority.Normal,
    options: {
      cancellable?: boolean,
      timeoutMs?: number,
      resources?: Partial<ResourceRequirements>
    } = {}
  ) => {
    const operation: Operation = {
      id: `op-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      op_type: opType,
      priority,
      created_at: Date.now(),
      estimated_resources: {
        cpu_units: options.resources?.cpu_units || 10,
        memory_mb: options.resources?.memory_mb || 50,
        io_intensity: options.resources?.io_intensity || 5,
        network_kb: options.resources?.network_kb || 10,
      },
      timeout_ms: options.timeoutMs,
      cancellable: options.cancellable ?? true,
      payload,
    };
    try {
      const operationId = await invoke<string>('enqueue_operation', { operation });
      setOperations(prev => ({
        ...prev,
        [operationId]: {
          operation,
          status: { Queued: {} }
        }
      }));
      pollOperationStatus(operationId);
      return operationId;
    } catch (error) {
      console.error('Failed to enqueue operation:', error);
      throw error;
    }
  }, []);

  const pollOperationStatus = useCallback((operationId: string) => {
    const interval = setInterval(async () => {
      try {
        const status = await invoke<OperationStatus>('get_operation_status', { operationId });
        setOperations(prev => ({
          ...prev,
          [operationId]: {
            ...prev[operationId],
            status,
          }
        }));
        // Stop polling if operation is in terminal state
        if (
          'Completed' in status ||
          'Failed' in status ||
          'Cancelled' in status ||
          'TimedOut' in status
        ) {
          clearInterval(interval);
        }
      } catch (error) {
        console.error(`Failed to get status for operation ${operationId}:`, error);
        clearInterval(interval);
      }
    }, 500);
    // Cleanup interval on unmount
    return () => clearInterval(interval);
  }, []);

  const cancelOperation = useCallback(async (operationId: string) => {
    try {
      await invoke<void>('cancel_operation', { operationId });
      return true;
    } catch (error) {
      console.error(`Failed to cancel operation ${operationId}:`, error);
      return false;
    }
  }, []);

  return {
    operations,
    enqueueOperation,
    cancelOperation,
  };
}
