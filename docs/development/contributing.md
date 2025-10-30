# Contributing to GerdsenAI AutoCoder

Thank you for your interest in contributing to GerdsenAI AutoCoder! This guide will help you get started.

## Development Philosophy: Inquiry-Based Learning

We use an inquiry-based approach to development. This means we prioritize understanding over implementation, questioning over assuming, and learning over just coding.

### Before You Start Coding

Ask yourself:
1. **What problem am I solving?** (Not what feature am I building)
2. **Why does this problem exist?** (Root cause, not symptoms)
3. **What are the constraints?** (Technical, user experience, performance)
4. **How will I know if I've succeeded?** (Measurable outcomes)

### During Development

1. **Question your approach**: Is this the simplest solution? What are the trade-offs?
2. **Test your assumptions**: Verify that your understanding matches reality
3. **Consider edge cases**: What could go wrong? What haven't I thought of?
4. **Document your reasoning**: Help future contributors understand 'why', not just 'what'

### Code Review Process

When reviewing code, ask:
- Does this solve the stated problem?
- What assumptions does this code make?
- How could this fail?
- Is there a simpler approach?
- What did we learn from this implementation?

## Getting Started

### Prerequisites

- Node.js 20.x or later
- Rust and Cargo
- Visual Studio Build Tools 2022 (Windows)
- Ollama installed locally

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/GerdsenAI/GerdsenAI_AutoCoder.git
   cd GerdsenAI_AutoCoder
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Start development mode:
   ```bash
   npm run tauri dev
   ```

### Project Structure

- `/src` - React frontend components
- `/src-tauri` - Rust backend
- `/docs` - Documentation
- `/mockups` - UI prototypes
- `/extensions` - IDE extensions

## Making Contributions

### 1. Find an Issue

- Check existing issues or create a new one
- Discuss your approach before implementing
- Ask questions to clarify requirements

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

### 3. Implement with Inquiry

- Start with tests that define success
- Implement the simplest solution that works
- Refactor based on new understanding
- Document decisions and trade-offs

### 4. Test Thoroughly

- Unit tests for new functionality
- Integration tests for system interactions
- Manual testing for user experience
- Performance testing for efficiency

### 5. Submit Pull Request

- Clear description of what and why
- Link to related issues
- Include test results
- Be open to feedback and questions

## Code Style Guidelines

### TypeScript/React
- Use TypeScript strict mode
- Prefer functional components
- Use meaningful variable names
- Comment the 'why', not the 'what'

### Rust
- Follow Rust conventions
- Use `Result` for error handling
- Document public APIs
- Keep functions focused and small

## Testing Guidelines

### Write Tests That Question
- Test what should happen
- Test what shouldn't happen
- Test edge cases
- Test error conditions

### Example Test Structure
```typescript
describe('ContextManager', () => {
  it('should handle empty context gracefully', () => {
    // Question: What happens with no context?
  });
  
  it('should prioritize recent files', () => {
    // Question: How do we determine priority?
  });
  
  it('should respect token limits', () => {
    // Question: What happens at the limit?
  });
});
```

## Documentation

### Document Your Learning
- Explain design decisions
- Note trade-offs made
- Link to relevant discussions
- Help others understand quickly

### Update Relevant Docs
- README.md for major features
- API documentation for new endpoints
- UI mockups for interface changes
- Architecture diagrams for structural changes

## Community

### Communication Channels
- GitHub Issues for bugs and features
- Pull Requests for code discussions
- Discussions for general questions

### Be Respectful
- Welcome questions at any level
- Share knowledge generously
- Admit when you don't know
- Learn from everyone

## Release Process

1. **Question**: Is this ready for users?
2. **Verify**: All tests passing?
3. **Document**: Changes clear?
4. **Release**: Follow semantic versioning

## Need Help?

If you're stuck or unsure:
1. **First**: Check existing documentation
2. **Then**: Search closed issues
3. **Finally**: Ask in a new issue

Remember: There are no stupid questions. Every question is an opportunity to improve our documentation and help future contributors.

---

*"The important thing is not to stop questioning." - Albert Einstein*

Welcome to the GerdsenAI AutoCoder community! We're excited to learn with you.