# GerdsenAI Socrates

![GerdsenAI Socrates](./public/assets/gerdsenai/s-logo.png)

**Advanced AI-Powered Coding Assistant with Multi-Provider Support**

[![License](https://img.shields.io/badge/license-Proprietary-blue.svg)](./LICENSE)
[![Version](https://img.shields.io/badge/version-1.0.0-green.svg)](./package.json)

---

## Overview

GerdsenAI Socrates is a powerful AI-enhanced development assistant that combines multiple AI providers (OpenAI, Anthropic, Ollama), intelligent code analysis, document management (RAG), and web search capabilities into a seamless desktop application with IDE integration.

Built with Tauri, React, and Rust, it provides a modern, performant interface for AI-assisted coding with advanced features like deep analysis mode, context window management, and extensible MCP integration.

---

## ✨ Key Features

### 🤖 Multi-AI Provider Support
- Seamlessly switch between **OpenAI GPT**, **Anthropic Claude**, and **Ollama** models
- Smart model routing based on task capabilities
- Real-time cost and performance tracking
- Intelligent provider fallback

### 🧠 AI-Enhanced Development
- **AI-powered LSP**: Intelligent code analysis, error detection, and suggestions
- **Smart completions**: Context-aware code completions
- **Intelligent hover**: AI explanations for code elements
- **Deep analysis mode**: Socratic questioning for complex problem-solving

### 📚 Document Intelligence (RAG)
- Upload and manage technical documentation
- Retrieval-Augmented Generation for context-aware responses
- Collection management with metadata
- Semantic search with relevance scoring

### 🔍 Web Search Integration
- Built-in **SearXNG** with health monitoring
- Real-time internet search toggle
- Visual status indicators with automatic failover

### 🎨 Professional UI
- Modern dark/light theme support
- Real-time health monitoring for all services
- Smooth animations and responsive design
- Multi-window desktop application support

### 🔧 Extensible Architecture
- **MCP (Model Context Protocol)** support for custom integrations
- Built-in core services (Ollama, SearXNG, ChromaDB, LSP)
- Popular tools quick-add gallery
- Plugin-ready architecture

---

## 🚀 Quick Start

### Installation

**Windows:**
```batch
# Right-click and "Run as Administrator"
install-wrapper.bat
```

**macOS/Linux:**
```bash
chmod +x install-symlink.sh
./install-symlink.sh
```

**For detailed installation instructions**, see the [Installation Guide](./docs/installation/installation-guide.md).

### First Run

1. Ensure **Ollama** is installed and running
2. Launch GerdsenAI Socrates from Start Menu (Windows) or Applications (macOS)
3. Select an AI model from the dropdown
4. Start chatting with your AI coding assistant!

---

## 📚 Documentation

### For Users
- **[Quick Start Guide](./docs/guides/quick-start.md)** - Get started in minutes
- **[Installation Guide](./docs/installation/installation-guide.md)** - Comprehensive installation instructions
- **[User Manual](./docs/usage/user-manual.md)** - Complete feature guide
- **[Troubleshooting](./docs/guides/troubleshooting.md)** - Common issues and solutions

### For Developers
- **[Contributing Guidelines](./docs/development/contributing.md)** - How to contribute
- **[Development Setup](./docs/installation/installation-guide.md#-contributing)** - Setting up your dev environment
- **[TypeScript Patterns](./docs/development/typescript-patterns.md)** - Code patterns and best practices
- **[Architecture Overview](./docs/ai/claude-integration.md#architecture-overview)** - System architecture

### Platform-Specific Setup
- **[Windows Setup](./docs/setup/windows.md)** - Windows configuration
- **[macOS Setup](./docs/setup/macos.md)** - macOS configuration

### Technical Documentation
- **[Context Window Management](./docs/CONTEXT_WINDOW_MANAGEMENT.md)** - Token management system
- **[Deep Analysis Mode](./docs/DEEP_ANALYSIS_MODE.md)** - Socratic questioning engine
- **[MCP Integration](./docs/MCP_INTEGRATION.md)** - Extensibility framework
- **[SearXNG Setup](./docs/searxng-setup.md)** - Web search configuration

📖 **[Complete Documentation Index](./docs/README.md)**

---

## 📋 System Requirements

### Minimum Requirements
- **OS**: Windows 10/11, macOS 10.15+, or Linux (Ubuntu 20.04+)
- **CPU**: 4-core processor
- **RAM**: 8GB (16GB recommended)
- **Storage**: 5GB free space
- **Network**: Internet connection for downloads

### Required Software
- **Ollama**: For AI model functionality ([download](https://ollama.ai/download))
- **Node.js 20+**: For development
- **Rust**: For building from source

### Optional Dependencies
- **Docker**: For SearXNG web search
- **VS Code** or **Visual Studio**: For IDE integration

---

## 🧪 Testing

GerdsenAI Socrates includes comprehensive production-ready testing infrastructure:

### Frontend Tests
- **24/24 tests passing** for ChatInterface (100% coverage)
- Full Tauri command mocking
- Complete user workflow testing

### Backend Tests
- **70+ comprehensive Rust tests** across critical modules
- HTTP mocking for external APIs
- Concurrency and performance testing

### Running Tests

```bash
# Frontend tests
npm test

# Backend tests
cd src-tauri && cargo test

# Full test suite
./scripts/test.sh
```

---

## 🛠️ Development

### Development Commands

```bash
# Install dependencies
npm install

# Start development server (Vite)
npm run dev

# Start Tauri development mode
npm run tauri:dev

# Build frontend
npm run build

# Build Tauri application
npm run tauri build

# Run linter
npm run lint

# Format code
npm run format

# Run quality checks
npm run quality
```

### Project Structure

```
GerdsenAI_AutoCoder/
├── src/                    # React frontend source
├── src-tauri/              # Rust backend source
├── docs/                   # Documentation
├── scripts/                # Build and utility scripts
├── extensions/             # IDE extensions
├── docker/                 # Docker configurations
└── public/                 # Static assets
```

### Development Philosophy

This project follows an **Inquiry-Based Learning** approach:

- **Question First**: Understand the 'why' before the 'how'
- **Challenge Assumptions**: Verify what we think we know
- **Explore Alternatives**: Consider multiple solutions
- **Learn Continuously**: Every bug is a learning opportunity

See [Contributing Guidelines](./docs/development/contributing.md) for detailed development practices.

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](./docs/development/contributing.md) for:

- Development setup
- Code style guidelines
- Testing requirements
- Pull request process
- Community guidelines

---

## 📝 Recent Updates

### Repository Reorganization (October 2025)
The repository has been reorganized for better maintainability:
- Documentation moved to `/docs` with logical subdirectories
- Scripts organized in `/scripts` with platform-specific folders
- Backward compatibility maintained through wrappers and symlinks

**[Migration Guide](./docs/MIGRATION.md)** - For updating references and bookmarks

---

## 📄 License

Copyright © 2025 GerdsenAI. All rights reserved.

See [LICENSE](./LICENSE) file for details.

---

## 🔗 Links

- **Documentation**: [/docs](./docs)
- **Issue Tracker**: [GitHub Issues](https://github.com/GerdsenAI/GerdsenAI_AutoCoder/issues)
- **Ollama**: [https://ollama.ai](https://ollama.ai)
- **Discord**: [Join our community](#) *(coming soon)*

---

## 🆘 Support

- **Documentation**: Check the [docs folder](./docs)
- **Troubleshooting**: See [Troubleshooting Guide](./docs/guides/troubleshooting.md)
- **Issues**: Report bugs on [GitHub Issues](https://github.com/GerdsenAI/GerdsenAI_AutoCoder/issues)
- **Community**: Join discussions on GitHub

---

**Happy coding with GerdsenAI Socrates! 🚀**

*An AI-powered development assistant that enhances your coding workflow*
