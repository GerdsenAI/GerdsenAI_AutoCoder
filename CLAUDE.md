# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Available Claude CLI Agents

When working on this project, use the specialized agents available:

- **research-documentation-specialist** - For comprehensive research and documentation tasks
- **qa-test-automation-engineer** - For testing strategies and quality assurance
- **performance-optimizer** - For performance analysis and optimization
- **git-operations-expert** - For Git operations and workflow management
- **code-reviewer** - For code review and quality assessment

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
- **AI Integration**: Ollama client for local/network LLM models
- **Search**: Embedded SearXNG instance for web search
- **Vector Database**: ChromaDB for RAG (Retrieval-Augmented Generation)
- **LSP**: Custom Language Server Protocol implementation

### Application Structure
The application follows a modular architecture with clear separation of concerns:

#### Frontend Components (`src/`)
- **App.tsx**: Main application shell with tab navigation (Chat, Search, RAG, History)
- **ChatInterface**: Real-time chat with Ollama models, session management
- **ModelSelector**: Dynamic model selection from local/network Ollama instances
- **RAGPanel**: Document management and vector database operations
- **SearchPanel**: SearXNG integration for web search
- **HistoryPanel**: Chat session persistence and management

#### Backend Services (`src-tauri/src/`)
- **commands.rs**: Enhanced Tauri commands with comprehensive Ollama integration
- **ollama_client.rs**: Advanced HTTP client with streaming, caching, and connection pooling
- **searxng_commands.rs**: Complete SearXNG integration with health monitoring  
- **searxng_client.rs**: SearXNG client with Docker support and error handling
- **chroma_manager.rs**: Full-featured RAG system with document management and search
- **lsp_server.rs**: AI-enhanced Language Server with debounced analysis and caching
- **window_manager.rs**: Multi-window desktop application management
- **history_manager.rs**: Chat session persistence and retrieval
- **code_analysis.rs**: AI-powered code analysis with Ollama integration
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

#### Sprint 2 In Progress 🚀
1. **RAG-to-Chat Integration**: ✅ COMPLETED - Backend integration with ChromaDB, professional UI with SVG icons
2. **Future-Proof Context Window Management**: 🎯 NEXT PRIORITY - Dynamic token-aware loading, hierarchical context system, smart chunking, model-specific adapters
3. **Deep Analysis Mode**: Optional Socratic/systematic problem-solving for complex debugging
   - Toggle between standard/socratic/systematic approaches
   - Auto-saves successful debugging patterns to RAG
   - Smart activation on complex problems
   - Time-boxed questioning rounds
4. **MCP Server Integration**: User-configurable extensions via Model Context Protocol
   - Core services remain built-in (Ollama, SearXNG, ChromaDB, LSP)
   - Add/configure MCP servers through Settings UI
   - Popular servers quick-add gallery
   - Dynamic tool discovery and integration
   - Secure API key management
5. **Repository-Wide Coding**: Advanced code analysis, automated generation, multi-file dependencies
4. **IDE Integration**: Real VS Code and Visual Studio extension implementations

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
The project has comprehensive testing at multiple levels:
- Frontend unit tests with Jest/Vitest
- Rust backend tests with `cargo test`
- LSP server integration tests
- IDE extension compatibility tests
- End-to-end application tests

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
