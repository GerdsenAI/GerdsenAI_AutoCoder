# Context Window Management Implementation Guide

## Overview

This document outlines the pragmatic implementation approach for context window management in GerdsenAI AutoCoder. Instead of over-engineering, we focus on delivering 80% of the value with 20% of the complexity.

## Visual Design Reference

See `mockups/context-window-visualizer.html` for the interactive UI prototype.

## Implementation Philosophy

### Start Simple, Iterate Based on Data
1. Build MVP with basic functionality
2. Measure actual usage patterns
3. Add complexity only where proven necessary
4. Prioritize user control and transparency

## Phase 1: MVP Implementation (Week 1)

### Core Components

```rust
pub struct ContextManager {
    max_tokens: usize,
    reserved_tokens: usize,
    token_counter: Box<dyn TokenCounter>,
}

pub trait TokenCounter {
    fn count(&self, text: &str) -> usize;
}

pub struct ContextBudget {
    total: usize,
    used: usize,
    breakdown: BudgetBreakdown,
}

pub struct BudgetBreakdown {
    conversation: usize,
    rag_documents: usize,
    pinned_files: usize,
    suggested_files: usize,
    reserved: usize,
}
```

### Token Counting Strategy
- Use conservative estimates (multiply by 1.2 for safety margin)
- Cache token counts for files
- Update counts only on file changes

### Basic UI Components
1. **Token Budget Bar**
   - Visual representation of token allocation
   - Color-coded segments
   - Hover for details

2. **File Management Panel**
   - List included files with token costs
   - Pin/unpin functionality
   - Relevance scores

3. **Control Panel**
   - Model selection with context limits
   - Reserved token adjustment
   - Basic strategy presets

## Phase 2: Smart Truncation (Week 2)

### Truncation Strategies

```typescript
interface TruncationStrategy {
  truncate(content: string, maxTokens: number): TruncatedContent;
}

interface TruncatedContent {
  content: string;
  originalTokens: number;
  truncatedTokens: number;
  summary?: string;
}

// Middle truncation - keep start and end
class MiddleTruncation implements TruncationStrategy {
  truncate(content: string, maxTokens: number): TruncatedContent {
    // Keep first 40% and last 40% of tokens
    // Add "... [truncated X lines] ..." in middle
  }
}
```

### Importance Markers
- Track which sections user has edited recently
- Prioritize keeping edited sections in context
- Visual indicators for truncated content

## Phase 3: User Control & Transparency (Week 3)

### User Preferences

```typescript
interface ContextPreferences {
  pinnedFiles: string[];
  excludePatterns: string[];
  contextPresets: Map<string, ContextPreset>;
  defaultStrategy: ContextStrategy;
}

interface ContextPreset {
  name: string;
  description: string;
  pinnedFiles: string[];
  priorityPatterns: string[];
  reservedTokens: number;
}

// Built-in presets
const PRESETS = {
  debugging: {
    name: "Debugging",
    description: "Focus on error context and stack traces",
    priorityPatterns: ["*.log", "*.error", "*test*"],
    reservedTokens: 40000, // More space for responses
  },
  refactoring: {
    name: "Refactoring",
    description: "Include related files and dependencies",
    priorityPatterns: ["*.ts", "*.tsx", "package.json"],
    reservedTokens: 30000,
  },
  documentation: {
    name: "Documentation",
    description: "Focus on markdown and comments",
    priorityPatterns: ["*.md", "README*", "*.doc"],
    reservedTokens: 20000,
  },
};
```

### Transparency Features
1. **Relevance Explanation**
   - Show why each file was included
   - Display relevance score calculation
   - Allow manual score adjustment

2. **Context Diff View**
   - Show what changed when context updates
   - Highlight additions/removals
   - Provide undo functionality

## Phase 4: Performance & Scale (Week 4+)

### Optimization Strategies

```rust
pub struct OptimizedContextManager {
    embedding_cache: LruCache<String, Vec<f32>>,
    token_cache: LruCache<String, usize>,
    file_watcher: FileWatcher,
}

impl OptimizedContextManager {
    // Pre-compute embeddings during idle time
    pub async fn background_indexing(&mut self) {
        // Index files that are likely to be used
    }
    
    // Incremental updates for changed files
    pub fn handle_file_change(&mut self, path: &Path) {
        self.token_cache.remove(&path.to_string());
        self.embedding_cache.remove(&path.to_string());
        // Re-index only the changed file
    }
}
```

### Memory Management
- Strict memory budget (500MB max)
- LRU eviction for caches
- Compress older context summaries
- Stream large files instead of loading entirely

## Success Metrics

### Performance
- Context building: < 100ms for average project
- Memory usage: < 500MB even for large repos
- UI updates: < 16ms (60 FPS)

### User Experience
- Time to understand context: < 30 seconds
- Clicks to control context: < 3
- User satisfaction: > 90% find it helpful

### Quality
- Relevant context inclusion: > 85% accuracy
- Token estimation accuracy: ± 10%
- Zero context-related crashes

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_counting_accuracy() {
        let counter = ConservativeTokenCounter::new();
        assert!(counter.count("Hello world") <= 4); // Conservative
    }
    
    #[test]
    fn test_budget_allocation() {
        let manager = ContextManager::new(128_000, 25_600);
        let budget = manager.calculate_budget();
        assert_eq!(budget.available(), 102_400);
    }
}
```

### Integration Tests
- Test with various project sizes
- Verify UI updates correctly
- Test file watching and updates
- Validate preset functionality

### User Testing
- A/B test different truncation strategies
- Gather feedback on UI clarity
- Monitor actual token usage patterns
- Track performance metrics

## Implementation Checklist

### Week 1 (MVP)
- [ ] Basic token counting
- [ ] Simple context builder
- [ ] Token budget UI component
- [ ] File list with manual control
- [ ] Basic Rust integration

### Week 2 (Smart Truncation)
- [ ] Middle truncation algorithm
- [ ] File importance tracking
- [ ] Truncation indicators
- [ ] Summary generation
- [ ] Update UI with truncation info

### Week 3 (User Control)
- [ ] Preferences system
- [ ] Context presets
- [ ] Relevance explanations
- [ ] Context diff view
- [ ] Pattern-based inclusion/exclusion

### Week 4+ (Performance)
- [ ] Embedding cache implementation
- [ ] Incremental file updates
- [ ] Background indexing
- [ ] Memory optimization
- [ ] Performance monitoring

## Common Pitfalls to Avoid

1. **Over-Engineering**
   - Don't build complex features without user validation
   - Start simple, measure, then iterate

2. **Token Counting Accuracy**
   - Different models use different tokenizers
   - Always use conservative estimates
   - Test with actual model APIs when possible

3. **Memory Management**
   - Don't cache everything
   - Set strict memory limits
   - Monitor actual usage in production

4. **UI Complexity**
   - Keep the default view simple
   - Hide advanced features behind "Advanced" toggle
   - Provide sensible defaults

## Conclusion

This pragmatic approach to context window management focuses on delivering immediate value while maintaining flexibility for future enhancements. By starting simple and iterating based on real usage data, we can build a system that actually helps users rather than impressing them with complexity.

Remember: The best context window management system is the one that ships and helps users today, not the theoretically perfect one planned for tomorrow.