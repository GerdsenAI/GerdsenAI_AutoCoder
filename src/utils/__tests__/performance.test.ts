import { beforeEach, expect, it, vi } from 'vitest';
import { onCLS, onFCP, onINP, onLCP, onTTFB } from 'web-vitals';
import { initPerformanceMonitoring } from '../performance';

vi.mock('web-vitals', () => ({
  onCLS: vi.fn(), onFCP: vi.fn(), onINP: vi.fn(),
  onLCP: vi.fn(), onTTFB: vi.fn(),
}));

beforeEach(() => vi.clearAllMocks());

it('registers the supported web-vitals 5 metrics, including INP', () => {
  initPerformanceMonitoring();
  for (const register of [onCLS, onFCP, onINP, onLCP, onTTFB]) {
    expect(register).toHaveBeenCalledTimes(1);
    expect(register).toHaveBeenCalledWith(expect.any(Function));
  }
});
