import { useOperationQueue, OperationPriority } from '../hooks/useOperationQueue';

export function OperationMonitor() {
  const { operations, cancelOperation } = useOperationQueue();
  const operationList = Object.entries(operations).map(([id, info]) => ({ id, ...info }));

  const getStatus = (status: any) => {
    if ('Queued' in status) return 'Queued';
    if ('Running' in status) return 'Running';
    if ('Completed' in status) return 'Completed';
    if ('Failed' in status) return 'Failed';
    if ('Cancelled' in status) return 'Cancelled';
    if ('TimedOut' in status) return 'TimedOut';
    return 'Unknown';
  };

  return (
    <div style={{ padding: 16, border: '1px solid #eee', borderRadius: 8, margin: 16 }}>
      <h3>Operation Monitor</h3>
      {operationList.length === 0 && <div>No operations in queue.</div>}
      {operationList.map(op => (
        <div key={op.id} style={{ marginBottom: 12, padding: 8, border: '1px solid #ddd', borderRadius: 4 }}>
          <div><b>Type:</b> {op.operation.op_type}</div>
          <div><b>Priority:</b> {OperationPriority[op.operation.priority]}</div>
          <div><b>Status:</b> {getStatus(op.status)}</div>
          {'Running' in op.status && (
            <div>Progress: {'progress' in op.status.Running && op.status.Running.progress !== undefined ? `${Math.round(op.status.Running.progress * 100)}%` : 'Working...'}</div>
          )}
          {'Failed' in op.status && (
            <div style={{ color: 'red' }}>Error: {op.status.Failed.error}</div>
          )}
          {op.operation.cancellable && (getStatus(op.status) === 'Running' || getStatus(op.status) === 'Queued') && (
            <button onClick={() => cancelOperation(op.id)}>Cancel</button>
          )}
        </div>
      ))}
    </div>
  );
}
