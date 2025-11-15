interface LoadingFallbackProps {
  message?: string;
}

export function LoadingFallback({ message = 'Loading...' }: LoadingFallbackProps) {
  return (
    <div className="loading-fallback">
      <div className="spinner" />
      <p>{message}</p>
    </div>
  );
}
