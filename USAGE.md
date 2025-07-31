# User Guide for Auto-Coder Companion

This guide provides detailed instructions on how to use Auto-Coder Companion's features effectively.

## Getting Started

After installing Auto-Coder Companion (see INSTALL.md), launch the application either as a standalone app or from your IDE.

### First Launch

1. When you first launch Auto-Coder Companion, you'll see the main interface with the model selector at the top
2. Select an Ollama model from the dropdown (requires Ollama to be running)
3. The status indicator will show green when connected successfully

## Core Features

### Chat Interface

The Chat tab is your primary interface for interacting with the AI:

1. Type your questions or code in the input field at the bottom
2. Press Enter or click the send button to submit
3. The AI will respond with text, code snippets, or suggestions
4. Code snippets include syntax highlighting and a copy button

**Tips:**
- Use clear, specific prompts for better results
- You can reference previous messages in the conversation
- Code snippets can be copied directly to your clipboard

### Code Analysis

To analyze code:

1. Paste code directly in the chat or reference open files
2. Ask specific questions about the code
3. Request explanations, optimizations, or bug fixes

In IDE extensions:
1. Select code in your editor
2. Right-click and choose "Analyze with Auto-Coder"
3. View analysis results in the Auto-Coder panel

### Repository-wide Analysis

To analyze an entire repository:

1. Click the folder icon in the bottom toolbar
2. Select a repository directory
3. Configure analysis options (file types, depth, etc.)
4. View the comprehensive analysis report

In IDE extensions:
1. Right-click on a project folder in the explorer
2. Select "Analyze Repository with Auto-Coder"

### Web Search

The Search tab provides integrated web search capabilities:

1. Enter your search query in the search field
2. Select search filters (Documentation, Stack Overflow, GitHub, Forums)
3. View search results with source information and relevance
4. Click on results to view full content

**Tips:**
- Use specific technical terms for better results
- Filter by documentation for official references
- Filter by Stack Overflow for community solutions

### RAG (Retrieval-Augmented Generation)

The RAG tab allows you to store and query documentation:

1. Add documentation by:
   - Clicking "Add Documentation" and entering a URL
   - Searching and saving results from the Search tab
   - Importing local documentation files
2. Search your documentation collection using natural language
3. View and use relevant documentation snippets in your work

**Tips:**
- Organize documentation into collections by topic
- Add tags to documentation for easier filtering
- Use specific queries to find relevant information

### History Management

The History tab provides access to past conversations:

1. View all previous chat sessions
2. Search for specific conversations by keyword
3. Filter conversations by tag or date
4. Continue any previous conversation by clicking on it

**Tips:**
- Add tags to important conversations for easier retrieval
- Use the search function to find specific code solutions
- Delete old or unnecessary conversations to keep organized

### Multi-window and Docking

Auto-Coder Companion supports flexible window management:

1. Click the "Undock" button to separate the window from your IDE
2. Drag the window to position it anywhere on your screen
3. Click "Dock" to reattach it to your IDE
4. Use multiple windows by clicking "New Window" in the menu

**Tips:**
- Undock when you need more screen space
- Use multiple windows to compare different conversations
- Dock to keep the assistant integrated with your workflow

### Model Selection

To change or configure AI models:

1. Click the model selector dropdown at the top
2. Choose from available Ollama models
3. For custom models or remote Ollama instances:
   - Click "Custom Ollama URL"
   - Enter the URL and connect

**Tips:**
- Smaller models are faster but less capable
- Larger models provide more advanced capabilities but require more resources
- Match the model to your task complexity

## Advanced Features

### Custom Commands

Auto-Coder Companion supports custom commands in the chat:

- `/analyze [file]` - Analyze a specific file
- `/fix [error]` - Generate a fix for a specific error
- `/explain [code]` - Get a detailed explanation of code
- `/generate [description]` - Generate code from description
- `/search [query]` - Perform a web search
- `/clear` - Clear the current conversation

### Keyboard Shortcuts

- `Ctrl+Enter` (Windows/Linux) or `Cmd+Enter` (macOS) - Send message
- `Ctrl+/` (Windows/Linux) or `Cmd+/` (macOS) - Focus on input field
- `Ctrl+Shift+N` (Windows/Linux) or `Cmd+Shift+N` (macOS) - New conversation
- `Ctrl+Shift+H` (Windows/Linux) or `Cmd+Shift+H` (macOS) - Show history
- `Ctrl+Shift+S` (Windows/Linux) or `Cmd+Shift+S` (macOS) - Show search
- `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (macOS) - Show RAG
- `Esc` - Cancel current operation

### Configuration Options

Access settings by clicking the gear icon:

1. **General Settings**
   - Theme (Light/Dark/System)
   - Font size
   - Window behavior

2. **AI Settings**
   - Default model
   - Temperature (creativity)
   - Context window size

3. **Search Settings**
   - SearXNG instance URL
   - Default search filters
   - Results per page

4. **RAG Settings**
   - Storage location
   - Embedding model
   - Collection management

5. **IDE Integration**
   - LSP configuration
   - Code action settings
   - Keyboard shortcuts

## Troubleshooting

### Common Issues

1. **AI Not Responding**
   - Check Ollama connection status
   - Verify model is properly loaded
   - Restart Ollama service if necessary

2. **Search Not Working**
   - Check internet connection
   - Verify SearXNG instance is accessible
   - Try a different search query

3. **IDE Integration Issues**
   - Ensure extension is properly installed
   - Check IDE compatibility
   - Restart IDE after configuration changes

4. **Performance Problems**
   - Try a smaller model
   - Close unused applications
   - Check system resource usage

### Getting Help

If you encounter issues not covered in this guide:

1. Check the FAQ section in the documentation
2. Visit the GitHub repository for known issues
3. Submit a support ticket with detailed information about your problem

## Best Practices

1. **For Code Generation**
   - Provide clear requirements and constraints
   - Include relevant context about your project
   - Specify programming language and style preferences

2. **For Code Analysis**
   - Include enough context for the AI to understand the code
   - Ask specific questions rather than general ones
   - Provide error messages when debugging

3. **For Documentation**
   - Organize documentation into logical collections
   - Add descriptive tags to improve searchability
   - Update documentation regularly as your project evolves

4. **For Workflow Integration**
   - Use keyboard shortcuts for efficiency
   - Set up custom commands for common tasks
   - Configure the interface to match your workflow
