# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Available Claude CLI Agents

When working on this project, use the specialized agents available:

- **research-documentation-specialist** - For comprehensive research and documentation tasks
- **qa-test-automation-engineer** - For testing strategies and quality assurance
- **performance-optimizer** - For performance analysis and optimization
- **git-operations-expert** - For Git operations and workflow management
- **code-reviewer** - For code review and quality assessment

## Development Methodology: Inquiry-Based Learning

When working on this codebase, use an inquiry-based approach to problem-solving and development:

### Core Principles
1. **Question First, Code Second** - Begin with understanding the 'why' before the 'how'
2. **Challenge Assumptions** - What are we taking for granted that might be wrong?
3. **Explore Root Causes** - Symptoms are not problems; dig deeper
4. **Test Understanding** - Can you explain it to someone else?

### Inquiry Framework for Development

#### 1. Problem Definition Phase
- What exactly are we trying to solve?
- What evidence do we have that this is the real problem?
- What would success look like?
- What constraints are we working within?

#### 2. Solution Exploration Phase
- What are the possible approaches?
- What are the trade-offs of each approach?
- What similar problems have been solved before?
- What could go wrong with our proposed solution?

#### 3. Implementation Verification Phase
- Does our implementation match our understanding?
- What edge cases haven't we considered?
- How do we know this actually solves the problem?
- What would break this solution?

### Practical Examples

#### Bug Investigation
```
Initial report: "App crashes on startup"
? Don't: Immediately start debugging the startup code
? Do: Ask "Under what conditions? Every time? After updates? For all users?"
```

#### Feature Development
```
Request: "Add dark mode"
? Don't: Start implementing CSS changes
? Do: Ask "What problem does dark mode solve for our users? How will they use it?"
```

#### Performance Optimization
```
Observation: "The app feels slow"
? Don't: Start optimizing random functions
? Do: Ask "What specific operations are slow? How are we measuring 'slow'?"
```

### Integration with Development Workflow

1. **Before coding**: Question the requirements and approach
2. **During coding**: Question your implementation choices
3. **After coding**: Question whether you've actually solved the problem
4. **During review**: Question what could be improved or might fail

### Key Questions for Every Development Task

- **Understanding**: Do I truly understand what I'm building and why?
- **Assumptions**: What am I assuming that I should verify?
- **Alternatives**: What other ways could this be done?
- **Consequences**: What happens if this fails or succeeds?
- **Learning**: What did this teach me about the system?

This inquiry-based approach leads to deeper understanding, fewer bugs, and better solutions.

### Socratic Development Success Pattern

**Proven Pattern for Component Development**:
1. **Question the Architecture**: Is this component trying to do too much?
2. **Extract Business Logic**: Move data fetching and state management to custom hooks
3. **Focus Components on Presentation**: UI rendering and user interaction only
4. **Test User Behavior**: Verify what users experience, not implementation details
5. **Measure Success**: 100% test success rate validates the approach

**Implementation Template**:
```typescript
// Custom Hook Pattern (Business Logic)
export function useFeature() {
  const [state, setState] = useState();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  
  const performAction = useCallback(async () => {
    // Handle business logic here
  }, []);
  
  return { state, loading, error, performAction };
}

// Component Pattern (Presentation)
export const FeatureComponent = ({ onCallback }) => {
  const { state, loading, error, performAction } = useFeature();
  
  const handleAction = () => {
    performAction();
  };
  
  return (
    <div>
      {/* Focus on UI and user interaction */}
    </div>
  );
};
```

**Testing Pattern**:
```typescript
// Behavior-Focused Testing
describe('FeatureComponent - User Experience', () => {
  it('shows elements users need to interact with', () => {
    render(<FeatureComponent />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });
  
  it('responds to user interactions', async () => {
    render(<FeatureComponent />);
    const button = screen.getByRole('button');
    await user.click(button);
    // Test observable behavior, not implementation
  });
});
```

## Development Commands

### Frontend (React + TypeScript + Vite)
- `npm run dev` - Start development server (Vite on port 1420)
- `npm run build` - Build frontend for production (TypeScript compilation + Vite build)
- `npm run preview` - Preview production build

### Tauri Desktop Application
- `npm run tauri dev` - Start Tauri development mode (builds frontend + launches desktop app)
- `npm run tauri build` - Build production Tauri application with installers

### Testing and Validation
- `npm test` - Run frontend tests
- `npm run test:integration` - Run integration tests
- `npm run test:e2e` - Run end-to-end tests
- `./scripts/test.sh` - Comprehensive test suite (frontend, backend, LSP, IDE extensions)
- `./scripts/validate_features.sh` - Feature validation script

### Build Scripts
- `./scripts/build.sh` - Complete cross-platform build for all components
- `./scripts/package.sh` - Package application for distribution
- `./scripts/cleanup.sh` - Clean build artifacts

### Backend (Rust)
- `cd src-tauri && cargo build --release` - Build Rust backend
- `cd src-tauri && cargo test` - Run Rust tests
- `cd src-tauri && cargo build --release --bin auto-coder-lsp` - Build LSP server

## Architecture Overview

### Core Technology Stack
- **Frontend**: React 19 with TypeScript, Vite bundler, CSS modules
- **Desktop Framework**: Tauri 2.x (Rust backend + WebView frontend)
- **State Management**: React hooks with local state, persistent storage via Tauri commands
- **Multi-AI Integration**: OpenAI GPT, Anthropic Claude, and Ollama with smart routing
- **Search**: Embedded SearXNG instance for web search
- **Vector Database**: ChromaDB for RAG (Retrieval-Augmented Generation)
- **LSP**: Custom Language Server Protocol implementation

### Application Structure
The application follows a modular architecture with clear separation of concerns:

#### Frontend Components (`src/`)
- **App.tsx**: Main application shell with tab navigation (Chat, Search, RAG, History)
- **ChatInterface**: Real-time chat with multi-AI provider support and smart routing
- **MultiAIModelSelector**: Comprehensive model selection with provider configuration
- **useMultiAI**: React hook for multi-AI provider management and state
- **RAGPanel**: Document management and vector database operations (follows Socratic architecture)
- **SearchPanel**: SearXNG integration for web search (follows Socratic architecture)
- **HistoryPanel**: Chat session persistence and management

#### Component Architecture (Socratic Design Pattern)
Following Inquiry-Based Learning principles, components are structured with clear separation of concerns:

**Custom Hooks Pattern**:
- **useRAG** (`src/hooks/useRAG.ts`): Encapsulates all RAG business logic, state management, and API calls
- **useSearch** (`src/hooks/useSearch.ts`): Handles search functionality, engine management, and health monitoring
- **useSearchHealth** (`src/hooks/useSearchHealth.ts`): Dedicated health monitoring for search services

**Component Responsibilities**:
- **Presentation Layer**: Components focus solely on UI rendering and user interaction
- **Business Logic Layer**: Custom hooks handle data fetching, state management, and business rules
- **Clean Event Handlers**: Simple, focused functions that delegate to hook methods
- **Behavior-Driven Testing**: Tests verify user experience rather than implementation details

**Benefits of Socratic Architecture**:
- Reduced component complexity (RAGPanel: 327→285 lines, cleaner structure)
- Improved testability (100% test success rate achieved)
- Better separation of concerns and maintainability
- Easier debugging and reasoning about component behavior

#### Backend Services (`src-tauri/src/`)
- **commands.rs**: Enhanced Tauri commands with comprehensive Ollama integration
- **ollama_client.rs**: Advanced HTTP client with streaming, caching, and connection pooling
- **ai_providers.rs**: Abstract AI provider system with trait-based architecture
- **openai_client.rs**: OpenAI GPT integration with streaming and cost tracking
- **anthropic_client.rs**: Anthropic Claude integration with message format support
- **ollama_provider.rs**: Ollama adapter for multi-AI system compatibility
- **multi_ai_commands.rs**: Tauri commands for multi-AI provider management
- **searxng_commands.rs**: Complete SearXNG integration with health monitoring  
- **searxng_client.rs**: SearXNG client with Docker support and error handling
- **chroma_manager.rs**: Full-featured RAG system with document management and search
- **lsp_server.rs**: AI-enhanced Language Server with debounced analysis and caching
- **window_manager.rs**: Multi-window desktop application management
- **history_manager.rs**: Chat session persistence and retrieval
- **code_analysis.rs**: AI-powered code analysis with multi-AI integration
- **doc_scraper.rs**: Documentation extraction and RAG indexing

#### IDE Extensions
- **VS Code Extension** (`extensions/vscode/`): TypeScript-based extension
- **Visual Studio Extension** (`extensions/visual-studio/`): C# MEF-based extension
- Shared components in `extensions/shared/`

### Key Features Implementation Status

#### Sprint 1 Complete ✅
1. **Advanced AI Chat**: Session-based conversations with model switching and input validation
2. **AI-Enhanced LSP**: Real-time code analysis with debounced AI diagnostics, smart completions, and intelligent hover
3. **Production RAG System**: Complete document management with upload, search, collections, and metadata
4. **Operational Web Search**: Full SearXNG integration with Docker setup, health monitoring, and comprehensive testing
5. **Performance Optimized**: Response caching (5min TTL), background processing, graceful fallbacks
6. **Professional UI**: Modern interface with real-time feedback, health indicators, and smooth animations
7. **Multi-Window Support**: Independent GUI windows (docking implementation pending)
8. **Theme System**: Light/dark mode with system preference detection

#### Sprint 2 Complete ✅
1. **RAG-to-Chat Integration**: ✅ COMPLETED - Backend integration with ChromaDB, professional UI with SVG icons
2. **Context Window Management**: ✅ COMPLETED - Dynamic token-aware loading, hierarchical context system, smart chunking, model-specific adapters
3. **Comprehensive Testing Infrastructure**: ✅ COMPLETED - Production-ready test coverage for all critical components
4. **Deep Analysis Mode**: ✅ COMPLETED - Full Socratic questioning engine with 4-stage framework and systematic PDCA analysis
   - Analysis mode selector (Standard/Socratic/Systematic) with professional UI
   - Auto-saves successful debugging patterns to RAG with metadata classification
   - Smart activation triggers for complex problems with confidence scoring
   - Time-boxed questioning rounds (5 min timeout) with pattern discovery
5. **MCP Server Integration**: ✅ COMPLETED - Complete Model Context Protocol integration with extensible architecture
   - Core services remain built-in (Ollama, SearXNG, ChromaDB, LSP)
   - Settings UI for adding/configuring MCP servers with validation
   - Popular servers quick-add gallery (GitHub, Filesystem, Sequential Thinking, Brave Search)
   - Dynamic tool discovery and integration with chat interface
   - Secure API key management and process lifecycle handling

#### Sprint 3 Major Progress ✅
1. **Enterprise Performance Optimization**: ✅ COMPLETED - Production-ready performance infrastructure
   - Advanced memory management with automatic cleanup and real-time monitoring
   - Efficient streaming buffer management with overflow protection
   - Specialized worker thread pools for CPU-intensive tasks (Embedding, CodeAnalysis, FileSystem)
   - Resource management with semaphores and comprehensive performance statistics
2. **Advanced Caching & Batch Processing**: ✅ COMPLETED - Enterprise-grade scalability features  
   - Production RAG query caching with TTL (5-minute default) and hit rate tracking
   - Smart cache invalidation on data changes with configurable cleanup intervals
   - Batched embedding generation (32 documents per batch, 4 concurrent batches)
   - Sub-batch processing to prevent Ollama overload with comprehensive batch statistics
3. **Bulk Document Processing**: ✅ COMPLETED - Memory-efficient large-scale operations
   - Priority-based processing for different workloads with real-time progress monitoring
   - Batch document addition with automatic embedding generation
   - Thread-safe operations with DashMap and proper resource balancing

#### Remaining Features
4. **Repository-Wide Coding**: Advanced code analysis, automated generation, multi-file dependencies
5. **IDE Integration**: Real VS Code and Visual Studio extension implementations
6. **Service Reliability**: Health checks, error management with retry mechanisms, failover systems

### Data Flow
1. User interactions trigger React components
2. Components invoke Tauri commands via `@tauri-apps/api`
3. Rust backend processes requests through specialized modules
4. External services (Ollama, SearXNG, ChromaDB) handle AI/search operations
5. Results flow back through the same chain with real-time updates

### Configuration Files
- **tauri.conf.json**: Tauri application configuration, window settings, build targets
- **vite.config.ts**: Frontend build configuration with Tauri integration
- **tsconfig.json**: TypeScript configuration with path aliases (`@/*` → `src/*`)
- **Cargo.toml**: Rust dependencies and build configuration

### Installation and Dependencies
- Windows-focused with automated setup via `INSTALL_DEPENDENCIES.BAT`
- Requires Ollama, SearXNG, and ChromaDB as external services
- Uses `START_APPLICATION.BAT` for simplified application launching
- Comprehensive setup documentation in `WINDOWS_SETUP.md`

## Development Notes

### Testing Strategy
The project has comprehensive production-ready testing infrastructure:

#### Frontend Testing (React + TypeScript)
**Socratic Testing Methodology Applied**: ✅ **COMPLETE**
- **100% Test Success Rate Achieved**: 95 passing tests, 0 failing tests
- **Behavior-Driven Testing**: Tests focus on user experience rather than implementation details
- **Component Test Suites**:
  - **ChatInterface**: 24/24 tests passing (complete chat workflow coverage)
  - **RAGPanel**: 9/9 tests passing (user interaction and accessibility focused)
  - **SearchPanel**: 9/9 tests passing (search interface and engine selection)
  - **HistoryPanel**: 24/24 tests passing (session management and persistence)
- **Testing Philosophy**: "Test what users experience, not what code does"
- **Testing Framework**: Vitest with React Testing Library
- **Mock Strategy**: Minimal mocking, focused on user-observable behavior

#### Backend Testing (Rust)
- **ollama_client.rs**: 25+ comprehensive tests with HTTP mocking via mockito
  - Connection handling, retries, timeouts, and streaming responses
  - Concurrent requests, caching behavior, and large payload handling
  - Network failure scenarios and malformed JSON recovery
- **operation_manager.rs**: 20+ tests for critical task management
  - Operation enqueueing, priority queues, and concurrent limiting
  - Cancellation lifecycle, high-load scenarios, and resource constraints
- **context_manager.rs**: 25+ tests for memory-critical operations
  - Token counting/caching, file pinning with concurrency
  - Budget calculations, context building, and memory bounds

#### Quality Assurance Approach
**Socratic Methodology Success Story**: ✅ **PROVEN EFFECTIVE**
- **Root Cause Analysis Applied**: Identified overly complex components as source of test brittleness
- **Architectural Solution**: Extracted business logic into custom hooks (useRAG, useSearch)
- **Test Quality Improvement**: Shifted from implementation-testing to behavior-testing
- **Measurable Results**: Achieved 100% test success rate (95 passing, 0 failing)

**Established QA Practices**:
- **IBL-Driven Design**: Question-first methodology, assumption challenging, root cause focus
- **Production-Ready Error Handling**: Network failures, race conditions, edge cases
- **Behavior-Focused Testing**: User experience validation over implementation verification
- **Component Architecture Standards**: Clean separation of presentation and business logic
- **Performance Testing**: Timeout handling, memory bounds, high-load scenarios

#### Additional Testing (Legacy)
- LSP server integration tests
- IDE extension compatibility tests
- End-to-end application workflows (pending enhancement)

### Build System
Cross-platform build system supports:
- Tauri desktop applications (Windows MSI, Linux AppImage, macOS DMG)
- VS Code extension packaging (VSIX)
- Visual Studio extension compilation
- LSP server binary compilation

### External Service Dependencies
- **Ollama**: Must be running for AI chat functionality
- **SearXNG**: Optional for web search features
- **ChromaDB**: Required for RAG document storage and retrieval
