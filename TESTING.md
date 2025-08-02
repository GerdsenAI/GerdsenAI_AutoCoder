# GerdsenAI Socrates Testing Guide

This document describes the comprehensive testing strategy for the GerdsenAI Socrates application.

## 🎉 Socratic Testing Success Story

**ACHIEVEMENT: 100% Test Success Rate** ✅
- **Total Tests**: 95 passing, 0 failing
- **Component Coverage**: All major components (ChatInterface, RAGPanel, SearchPanel, HistoryPanel)
- **Methodology Applied**: Socratic root cause analysis identified architectural issues
- **Solution**: Extracted business logic into custom hooks (useRAG, useSearch)
- **Result**: Behavior-focused testing with 100% reliability

**Key Success Metrics**:
- **RAGPanel**: 9/9 tests passing (previously failing due to complex state management)
- **SearchPanel**: 9/9 tests passing (simplified through useSearch hook)
- **ChatInterface**: 24/24 tests passing (maintained existing excellence)
- **HistoryPanel**: 24/24 tests passing (robust session management)

**Architectural Pattern Established**:
- **Custom Hooks**: Encapsulate business logic, state management, API calls
- **Clean Components**: Focus on UI rendering and user interaction
- **Behavior Testing**: Verify user experience rather than implementation details
- **Minimal Mocking**: Test observable behavior, not internal mechanics

## Test Philosophy

We follow the **Socratic Method** for testing:
- **Question First**: Why are we testing this?
- **Challenge Assumptions**: What could go wrong?
- **Root Cause Analysis**: What's the real failure mode?
- **Validate Understanding**: Does the test prove what we think?

## Test Pyramid

```
         /\        E2E Tests (10%)
        /  \       - User workflows
       /    \      - Cross-browser
      /      \     
     /--------\    Integration Tests (20%)
    /          \   - API contracts
   /            \  - Service integration
  /              \ 
 /________________\ Unit Tests (70%)
                    - Business logic
                    - Component behavior
```

## Running Tests

### Quick Start

```bash
# Run all tests
npm run test:all

# Run specific test suites
npm run test          # Unit tests
npm run test:integration  # Integration tests
npm run test:e2e      # End-to-end tests

# Run with coverage
npm run test:coverage

# Run in watch mode
npm run test:watch

# Run with UI
npm run test:ui
```

### Backend Tests (Rust)

```bash
cd src-tauri
cargo test                    # Run all tests
cargo test -- --nocapture    # Show print statements
cargo test test_name         # Run specific test
```

### Docker-Based Testing

```bash
# Start test environment
docker-compose -f docker-compose.test.yml up

# Run tests in Docker
npm run test:docker

# Build test image
npm run test:docker:build
```

## Test Structure

### Unit Tests

Located in `__tests__` directories next to the code:

```
src/
├── components/
│   ├── ChatInterface.tsx
│   └── __tests__/
│       └── ChatInterface.test.tsx
├── hooks/
│   ├── useChroma.ts
│   └── __tests__/
│       └── useChroma.test.ts
```

**Key Patterns:**
- Mock external dependencies
- Test business logic, not implementation
- Use descriptive test names
- Follow AAA pattern (Arrange, Act, Assert)

### Integration Tests

Located in `tests/integration/`:

```
tests/
└── integration/
    ├── setup.ts            # Test environment setup
    ├── api_contracts.test.ts
    └── services/
        ├── ollama.test.ts
        ├── searxng.test.ts
        └── chromadb.test.ts
```

**Key Patterns:**
- Test real service integration
- Use Docker containers for consistency
- Test error scenarios
- Verify API contracts

### E2E Tests

Located in `tests/e2e/`:

```
tests/
└── e2e/
    ├── global-setup.ts
    ├── chat-workflow.spec.ts
    ├── rag-workflow.spec.ts
    └── helpers/
        └── mocks.ts
```

**Key Patterns:**
- Test complete user workflows
- Use data-testid attributes
- Mock external services when needed
- Take screenshots on failure

## Test Data Management

### Fixtures

Test data is organized in `tests/fixtures/`:

```
tests/
└── fixtures/
    ├── ollama/
    │   └── models/       # Test model files
    ├── documents/        # Test documents for RAG
    └── responses/        # Mock API responses
```

### Test Utilities

Helper functions in `tests/utils/`:

```typescript
// Example test factory
export function createTestMessage(overrides?: Partial<ChatMessage>): ChatMessage {
  return {
    role: 'user',
    content: 'Test message',
    timestamp: new Date().toISOString(),
    ...overrides
  };
}
```

## CI/CD Integration

Tests run automatically on:
- Every push to main/develop
- Every pull request
- Nightly schedule (2 AM UTC)

### GitHub Actions Workflow

1. **Unit Tests** - Run on all platforms
2. **Integration Tests** - Run with Docker services
3. **E2E Tests** - Run with Playwright
4. **Performance Tests** - Check for regressions
5. **Security Tests** - Scan for vulnerabilities

## Performance Testing

### Load Testing with K6

```javascript
// tests/performance/load-test.js
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '30s', target: 10 },  // Ramp up
    { duration: '1m', target: 50 },   // Stay at 50 users
    { duration: '30s', target: 0 },   // Ramp down
  ],
};

export default function() {
  let res = http.post('http://localhost:11434/api/chat', {
    messages: [{ role: 'user', content: 'Hello' }]
  });
  
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
}
```

### Memory Leak Detection

```typescript
// tests/performance/memory-test.ts
describe('Memory Usage', () => {
  it('should not leak memory on repeated operations', async () => {
    const initialMemory = process.memoryUsage().heapUsed;
    
    // Perform operations
    for (let i = 0; i < 1000; i++) {
      await performOperation();
    }
    
    // Force garbage collection
    if (global.gc) {
      global.gc();
    }
    
    const finalMemory = process.memoryUsage().heapUsed;
    const memoryIncrease = finalMemory - initialMemory;
    
    // Allow max 50MB increase
    expect(memoryIncrease).toBeLessThan(50 * 1024 * 1024);
  });
});
```

## Security Testing

### Vulnerability Scanning

```bash
# Frontend dependencies
npm audit --production

# Rust dependencies
cd src-tauri && cargo audit

# Docker images
docker scan gerdsenai-socrates:latest
```

### Input Validation Tests

```typescript
describe('Security', () => {
  it('should sanitize user input', async () => {
    const maliciousInput = '<script>alert("XSS")</script>';
    const result = await sendMessage(maliciousInput);
    expect(result).not.toContain('<script>');
  });
  
  it('should prevent path traversal', async () => {
    const maliciousPath = '../../../etc/passwd';
    await expect(readFile(maliciousPath)).rejects.toThrow();
  });
});
```

## Test Coverage

Current coverage targets:
- **Overall**: 80%
- **Critical paths**: 95%
- **New code**: 90%

View coverage reports:
```bash
npm run test:coverage
open coverage/index.html
```

## Debugging Tests

### Frontend Tests

```bash
# Debug with VS Code
# Add breakpoint in test file
# Run: Debug > JavaScript Debug Terminal
npm run test

# Debug specific test
npm run test -- --reporter=verbose ChatInterface
```

### E2E Tests

```bash
# Debug mode with Playwright Inspector
npm run test:e2e:debug

# Headed mode
npx playwright test --headed

# Specific test
npx playwright test chat-workflow
```

### Rust Tests

```bash
# Debug with print statements
cargo test -- --nocapture

# Debug with lldb
rust-lldb target/debug/deps/test_name-hash
```

## Best Practices

### 1. Test Naming

```typescript
// ❌ Bad
test('test1', () => {});

// ✅ Good
test('should display error message when API call fails', () => {});
```

### 2. Test Independence

```typescript
// ❌ Bad - depends on previous test
let sharedState;
test('first test', () => {
  sharedState = createThing();
});
test('second test', () => {
  useThing(sharedState); // Depends on first test
});

// ✅ Good - independent
test('first test', () => {
  const thing = createThing();
  // test thing
});
test('second test', () => {
  const thing = createThing(); // Create fresh
  // test thing
});
```

### 3. Mock Appropriately

```typescript
// ❌ Bad - over-mocking
vi.mock('react'); // Don't mock the framework!

// ✅ Good - mock external dependencies
vi.mock('@tauri-apps/api/core');
vi.mock('../../services/ollama');
```

### 4. Test Behavior, Not Implementation

```typescript
// ❌ Bad - testing implementation
expect(component.state.isLoading).toBe(true);

// ✅ Good - testing behavior
expect(screen.getByText('Loading...')).toBeInTheDocument();
```

### 5. Socratic Component Architecture (NEW PATTERN)

```typescript
// ✅ Proven Pattern - Custom Hook + Clean Component
// Custom Hook (Business Logic)
export function useFeature() {
  const [state, setState] = useState();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  
  const performAction = useCallback(async () => {
    // Handle all business logic here
  }, []);
  
  return { state, loading, error, performAction };
}

// Component (Presentation Only)
export const FeatureComponent = ({ onCallback }) => {
  const { state, loading, error, performAction } = useFeature();
  
  return (
    <div>
      {/* Focus purely on UI and user interaction */}
      <button onClick={performAction}>Action</button>
      {error && <div className="error">{error}</div>}
    </div>
  );
};

// Test (User Experience Focused)
describe('FeatureComponent - User Experience', () => {
  it('shows elements users need to interact with', () => {
    render(<FeatureComponent />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });
  
  it('responds appropriately to user interactions', async () => {
    render(<FeatureComponent />);
    const button = screen.getByRole('button');
    await user.click(button);
    // Test what users see, not internal state
  });
});
```

**Benefits of This Pattern**:
- **100% Test Reliability**: Proven with RAGPanel and SearchPanel
- **Reduced Complexity**: Components focus on presentation only
- **Better Maintainability**: Clear separation of concerns
- **Easier Debugging**: Business logic isolated in testable hooks

## Troubleshooting

### Common Issues

1. **Tests timeout**
   - Increase timeout: `test('name', async () => {}, 30000)`
   - Check if services are running
   - Look for unresolved promises

2. **Flaky tests**
   - Add proper wait conditions
   - Mock time-dependent operations
   - Use data-testid for reliable selection

3. **Port conflicts**
   - Use dynamic ports in tests
   - Kill processes: `npm run port:kill`
   - Check ports: `npm run port:info`

### Getting Help

- Check test output for detailed errors
- Run with verbose logging: `DEBUG=* npm test`
- Use test UI for interactive debugging: `npm run test:ui`

## Contributing

When adding new features:

1. **Write tests first** (TDD approach)
2. **Update existing tests** if behavior changes
3. **Add integration tests** for new services
4. **Document test patterns** for complex scenarios
5. **Run full test suite** before submitting PR

Remember: A feature without tests is incomplete!