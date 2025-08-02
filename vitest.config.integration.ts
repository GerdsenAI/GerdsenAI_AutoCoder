import { defineConfig } from 'vitest/config';
import { resolve } from 'path';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['tests/integration/**/*.test.{ts,tsx}'],
    exclude: ['node_modules', 'dist', 'src-tauri'],
    setupFiles: ['./tests/integration/setup.ts'],
    testTimeout: 60000, // 60 seconds for integration tests
    hookTimeout: 30000, // 30 seconds for hooks
    reporters: ['default', 'json', 'junit'],
    outputFile: {
      json: 'test-results/integration-results.json',
      junit: 'test-results/integration-junit.xml'
    },
    pool: 'threads',
    poolOptions: {
      threads: {
        singleThread: true, // Run integration tests sequentially
      }
    }
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
});