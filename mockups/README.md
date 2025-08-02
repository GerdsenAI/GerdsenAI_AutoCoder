# Mockups Directory

This directory contains visual mockups and prototypes for GerdsenAI AutoCoder features.

## Context Window Visualizer

**File**: `context-window-visualizer.html`

An interactive prototype demonstrating the improved context window management interface.

### Features Demonstrated:
- **Token Budget Bar**: Visual representation of context allocation
- **File Management**: Pin/unpin files, see relevance scores
- **AI Suggestions**: Transparent file recommendations
- **Control Panel**: Model selection and strategy presets

### Usage:
1. Open `context-window-visualizer.html` in a web browser
2. Interact with the various controls to see the UI behavior
3. Use as reference for implementing the actual React components

### Implementation Notes:
- Color scheme matches the main application's dark theme
- All interactions are client-side JavaScript for easy prototyping
- Responsive design adapts to different screen sizes
- Focus on clarity and user control over AI decisions

This mockup represents the pragmatic approach to context window management, prioritizing user understanding and control over technical complexity.

## Deep Analysis Mode Toggle

**File**: `deep-analysis-toggle.html`

An interactive prototype demonstrating the Deep Analysis Mode interface for complex problem-solving.

### Features Demonstrated:
- **Mode Toggle**: On/off switch with visual feedback
- **Analysis Modes**: Socratic and Systematic options
- **Progress Tracking**: Round indicators and time elapsed
- **Chat Integration**: Example of Socratic questioning in action

### Key Interactions:
- Toggle activates mode selection
- Mode selection triggers progress indicator
- Exit button with confirmation dialog
- Visual distinction for deep analysis conversations

This mockup shows how complex debugging features can be optional power tools rather than constant overhead.

## MCP Server Configuration

**File**: `mcp-config-ui.html`

An interactive prototype for the MCP (Model Context Protocol) server configuration interface.

### Features Demonstrated:
- **Core Services Display**: Shows built-in services (Ollama, SearXNG, ChromaDB, LSP)
- **MCP Server Management**: Add, configure, enable/disable MCP servers
- **Tool Discovery**: Display available tools from connected servers
- **Quick Add Gallery**: Popular MCP servers for one-click installation

### Key Design Decisions:
- Core services remain built-in and always available
- MCP servers are optional extensions
- Clear visual distinction between core and extended functionality
- User-friendly configuration without command-line knowledge

This design allows power users to extend AutoCoder while keeping the core experience simple and reliable.