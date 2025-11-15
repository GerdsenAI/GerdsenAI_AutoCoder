import React, { Suspense } from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { ErrorBoundary } from './components/ErrorBoundary';
import { LoadingFallback } from './components/LoadingFallback';
import { reportWebVitals } from './utils/performance';
import './styles.css';

// Initialize performance monitoring
reportWebVitals();

const rootElement = document.getElementById('root');
if (!rootElement) throw new Error('Failed to find the root element');

ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <Suspense fallback={<LoadingFallback message="Loading GerdsenAI Socrates..." />}>
        <App />
      </Suspense>
    </ErrorBoundary>
  </React.StrictMode>,
);
