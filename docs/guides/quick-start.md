# 🚀 GerdsenAI Socrates - Quick Start Guide

*Get productive with AI-powered coding assistance in 10 minutes*

---

## ⚡ Installation (2 minutes)

### Windows Users
1. **Download** the latest release from the repository
2. **Right-click** `INSTALL_DEPENDENCIES.BAT` → "Run as administrator"
3. **Wait** for setup to complete (5-10 minutes)
4. **Launch** with `START_APPLICATION.BAT`

### Mac/Linux Users
```bash
# Install dependencies
npm install
npm run tauri dev

# Verify setup
node scripts/verify-setup.js
```

### First Launch Checklist
- [ ] Application opens without errors
- [ ] Ollama is running (check system tray)
- [ ] AI chat responds to messages
- [ ] No red error indicators in the interface

---

## 🎯 Essential Features (8 minutes)

### 1. AI Chat Assistant (2 minutes)

**Basic Conversation**
1. Open the **Chat** tab (usually selected by default)
2. Select a model from the dropdown (try `llama3.2` for fast responses)
3. Type: *"Help me write a Python function to calculate fibonacci numbers"*
4. Watch the AI generate code with explanations

**Advanced Features**
- **Streaming responses**: See text appear in real-time
- **Code highlighting**: Automatic syntax highlighting for all languages
- **Copy code blocks**: Click the copy button on any code snippet
- **Context awareness**: The AI remembers your previous messages

### 2. Document Management & RAG (2 minutes)

**Upload Your Codebase**
1. Click the **RAG** tab
2. Click **"Add Documents"**
3. Select your project files (`.js`, `.py`, `.md`, etc.)
4. Wait for processing (vectorization happens automatically)

**Query Your Documents**
1. Type: *"What functions are defined in my authentication module?"*
2. The AI will search your uploaded documents and provide specific answers
3. Try: *"Show me all the API endpoints in my codebase"*

**Pro Tips**:
- Upload README files, documentation, and key source files
- The AI can find patterns across multiple files
- Great for understanding unfamiliar codebases

### 3. Web Search Integration (1 minute)

**Research While Coding**
1. Click the **Search** tab
2. Search: *"React 19 new features 2025"*
3. Get curated results from multiple search engines
4. Perfect for staying current with technology trends

### 4. Context Window Management (1 minute)

**Smart Context Loading**
1. In the **Chat** tab, notice the context indicator at the top
2. **Pin important files**: Click the pin icon next to relevant files
3. **Monitor token usage**: The progress bar shows how much context is being used
4. **Automatic relevance**: The AI prioritizes the most relevant files

### 5. Advanced Analysis Mode (2 minutes)

**Deep Problem Solving**
1. When facing a complex bug, click the **Analysis Mode** dropdown
2. Select **"Socratic"** for guided problem-solving
3. Ask: *"My React component is re-rendering too often, help me debug this"*
4. The AI will ask clarifying questions to help you think through the problem

**Systematic Analysis**
1. For architectural decisions, select **"Systematic"** mode
2. Ask: *"Should I use Redux or Context API for state management?"*
3. Get structured analysis with pros, cons, and recommendations

---

## 💡 Real-World Scenarios

### Scenario 1: Debugging a Node.js API

**Problem**: Your Express.js API is returning 500 errors

1. **Upload relevant files** to RAG:
   ```
   - server.js
   - routes/api.js
   - package.json
   - error logs
   ```

2. **Chat with context**:
   ```
   "My Express API is throwing 500 errors. Based on my uploaded files, 
   what could be causing this issue?"
   ```

3. **Get specific solutions**:
   - The AI will analyze your actual code
   - Suggest specific line numbers to check
   - Recommend debugging strategies

### Scenario 2: Learning a New Framework

**Goal**: Understanding Next.js for the first time

1. **Search for current information**:
   ```
   Search: "Next.js 14 tutorial 2025 best practices"
   ```

2. **Create a learning collection**:
   - Upload Next.js documentation to RAG
   - Add example projects you've found

3. **Interactive learning**:
   ```
   Chat: "Based on the Next.js docs I uploaded, walk me through 
   creating my first API route"
   ```

### Scenario 3: Code Review & Refactoring

**Task**: Improve code quality before production

1. **Upload code to RAG**: Add files you want reviewed

2. **Systematic analysis**:
   ```
   "Using systematic analysis mode, review my authentication system 
   for security vulnerabilities and performance issues"
   ```

3. **Get actionable feedback**:
   - Security recommendations
   - Performance optimizations
   - Code style improvements

### Scenario 4: API Integration

**Challenge**: Integrating with a third-party API

1. **Research the API**:
   ```
   Search: "Stripe API integration best practices 2025"
   ```

2. **Upload API documentation** to RAG

3. **Get implementation guidance**:
   ```
   "Based on the Stripe docs I uploaded, help me implement 
   payment processing with error handling"
   ```

---

## ⚙️ Customization & Settings

### Model Selection
- **Fast responses**: Use `llama3.2:1b` or similar lightweight models
- **Better quality**: Use `llama3.2:8b` or `mistral:7b`
- **Specialized tasks**: Try domain-specific models

### Interface Preferences
1. Click the **Settings** gear icon
2. Adjust:
   - Theme (light/dark)
   - Response speed vs quality
   - Context window size
   - Default search engines

### Performance Tuning
- **RAM usage**: Smaller models use less memory
- **Response speed**: Local models are faster than API calls
- **Context size**: Larger context = better understanding but slower responses

---

## 🔥 Pro Tips

### Keyboard Shortcuts
- `Ctrl/Cmd + Enter`: Send message in chat
- `Ctrl/Cmd + K`: Quick search across all features
- `Escape`: Clear current input

### Effective Prompting
**Good**: *"Help me optimize this specific function for better performance"*
**Better**: *"This function processes 10,000 items and takes 5 seconds. Help me make it faster"*

### Context Management
- Upload your project's README and key files first
- Pin frequently referenced files
- Remove outdated documents to keep context relevant

### Workflow Integration
1. **Morning**: Upload any new documentation or code changes
2. **During coding**: Use chat for quick questions and debugging
3. **Code review**: Use systematic analysis before committing
4. **Learning**: Search for new techniques and best practices

---

## 🚨 Common Issues & Quick Fixes

### "Ollama Not Running"
```bash
# Start Ollama service
ollama serve

# Install a model if none exist
ollama pull llama3.2
```

### Slow Responses
1. Switch to a smaller model (1B parameters)
2. Reduce context window size
3. Close other memory-intensive applications

### Search Not Working
- SearXNG is optional but enhances search results
- The basic search will still work without it
- Check Windows_SETUP.md for SearXNG installation

### Documents Not Processing
- Ensure ChromaDB is running
- Check that uploaded files are in supported formats
- Try restarting the application

---

## 🎓 Next Steps

### Week 1: Core Mastery
- [ ] Complete all scenarios above
- [ ] Upload your main project to RAG
- [ ] Experiment with different models
- [ ] Customize settings for your workflow

### Week 2: Advanced Features
- [ ] Set up SearXNG for enhanced search
- [ ] Try MCP server integrations
- [ ] Create document collections for different projects
- [ ] Experiment with systematic analysis mode

### Week 3: Workflow Integration
- [ ] Integrate with your IDE using planned extensions
- [ ] Set up automated model updates
- [ ] Develop personal prompting templates
- [ ] Contribute feedback for product improvements

---

## 📚 Additional Resources

- **Detailed Setup**: See `WINDOWS_SETUP.md` for comprehensive installation help
- **Troubleshooting**: Run `node scripts/verify-setup.js` for system diagnostics
- **Advanced Usage**: Check `USER_GUIDE.md` for complete feature documentation
- **Community**: Join discussions in the project repository

---

**Ready to supercharge your coding productivity? Start with Scenario 1 above! 🚀**