# CSE-Icon AutoCoder - User Guide

This guide provides instructions on how to use the CSE-Icon AutoCoder application effectively.

## Getting Started

After installing CSE-Icon AutoCoder (see [INSTALLATION.md](./INSTALLATION.md)), launch the application from your desktop shortcut or Start menu. The application will automatically dock to the side of your IDE (VS Code or Visual Studio).

## Interface Overview

The CSE-Icon AutoCoder interface consists of:

1. **Header**: Contains the CSE-Icon logo and connection status
2. **Model Selector**: Dropdown to choose your preferred Ollama model
3. **Tab Navigation**: Switch between Chat, Search, RAG, and History views
4. **Content Area**: Main interaction area that changes based on selected tab
5. **Action Bar**: Bottom bar with additional controls

## Tab Functionality

### Chat Tab

The Chat tab is your primary interface for interacting with the AI assistant:

- **Message Input**: Type your questions or code at the bottom
- **Code Highlighting**: Code blocks are automatically highlighted with proper syntax
- **Copy Button**: Easily copy code snippets to clipboard
- **Context Awareness**: The AI remembers previous messages in your conversation
- **RAG Toggle**: Enable/disable Retrieval Augmented Generation for better answers

### Search Tab

The Search tab allows you to find information from various sources:

- **Search Bar**: Enter your query to search across multiple sources
- **Source Filters**: Filter results by Documentation, Stack Overflow, GitHub, or Forums
- **Result Cards**: View search results with source information and timestamps
- **Direct Links**: Click on results to open the original source

### RAG Tab

The RAG (Retrieval Augmented Generation) tab manages your knowledge base:

- **Document Management**: Add, view, or delete documents in your knowledge base
- **Statistics**: See the number of documents and tokens in your knowledge base
- **Upload Area**: Drag and drop files or click to upload new documents
- **Document List**: Browse and manage your uploaded documents

### History Tab

The History tab shows your past conversations:

- **Session List**: Browse through previous chat sessions
- **Search**: Find specific conversations by keyword
- **Tag Filters**: Filter sessions by tags
- **Session Details**: View message count, model used, and timestamps
- **Delete Option**: Remove unwanted sessions

## Additional Features

### Light & Dark Mode

CSE-Icon AutoCoder supports both light and dark themes:

- **Auto Mode**: Automatically matches your system theme
- **Manual Toggle**: Switch between light and dark mode using the theme toggle in settings

### IDE Integration

The application is designed to integrate seamlessly with your IDE:

- **Docking**: Automatically docks to the side of VS Code or Visual Studio
- **Undocking**: Can be undocked to use as a standalone window
- **Code Context**: Can understand and reference code from your active editor

### Model Management

Manage your Ollama models directly from the application:

- **Model Selection**: Choose from available models in the dropdown
- **Parameter Adjustment**: Modify temperature, top-p, and other parameters
- **Custom URL**: Connect to remote Ollama instances

## Keyboard Shortcuts

- **Ctrl+Enter**: Send message
- **Esc**: Clear current input
- **Ctrl+L**: Clear chat history
- **Ctrl+Tab**: Switch between tabs
- **Ctrl+S**: Save current session
- **Ctrl+N**: Create new session

## Tips for Best Results

1. **Be Specific**: Provide clear, detailed questions for better responses
2. **Include Context**: When asking about code, include relevant snippets
3. **Use RAG**: Enable RAG when asking about documentation or specific libraries
4. **Iterate**: Refine your questions based on the AI's responses
5. **Save Sessions**: Tag important conversations for easy reference later

## Troubleshooting

- **Slow Responses**: Try selecting a smaller model or check your Ollama configuration
- **Connection Issues**: Verify Ollama is running and accessible
- **Missing Models**: Use the refresh button in the model selector to update the list
- **UI Glitches**: Try undocking and redocking the application

For additional help, please refer to our support website or contact our technical support team.
