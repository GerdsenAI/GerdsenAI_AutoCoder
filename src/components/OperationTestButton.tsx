import { useOperationQueue, OperationType, OperationPriority } from '../hooks/useOperationQueue';

export function OperationTestButton() {
  const { enqueueOperation } = useOperationQueue();
  return (
    <button
      style={{ margin: 16, padding: 8 }}
      onClick={() => {
        enqueueOperation(
          OperationType.AICompletion,
          { prompt: 'Test prompt' },
          OperationPriority.Normal,
          { cancellable: true }
        );
      }}
    >
      Enqueue Test Operation
    </button>
  );
}
