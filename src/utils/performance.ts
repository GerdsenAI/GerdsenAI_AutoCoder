import { onCLS, onFID, onFCP, onLCP, onTTFB, Metric } from 'web-vitals';

function sendToAnalytics(metric: Metric): void {
  // Log to console in development
  if (process.env.NODE_ENV === 'development') {
    console.log('Web Vital:', metric);
  }

  // In production, you would send to your analytics service
  // Example:
  // const body = JSON.stringify({ [metric.name]: metric.value });
  // fetch('/api/analytics', { method: 'POST', body, keepalive: true });
}

export function initPerformanceMonitoring(): void {
  onCLS(sendToAnalytics);
  onFID(sendToAnalytics);
  onFCP(sendToAnalytics);
  onLCP(sendToAnalytics);
  onTTFB(sendToAnalytics);
}

export function measurePerformance(name: string, fn: () => void): void {
  const start = performance.now();
  fn();
  const end = performance.now();
  console.log(`Performance: ${name} took ${end - start}ms`);
}

export function reportWebVitals(): void {
  // This can be called from main.tsx to start monitoring
  initPerformanceMonitoring();
}
