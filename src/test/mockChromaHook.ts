// Mock implementation of useChroma hook for testing
export const mockUseChroma = {
  collections: [],
  documents: [],
  isLoading: false,
  error: null,
  selectedCollection: 'default',
  stats: {
    totalDocuments: 0,
    totalCollections: 0,
    cacheHitRate: 0,
    processingQueue: 0
  },
  createCollection: vi.fn(),
  deleteCollection: vi.fn(),
  selectCollection: vi.fn(),
  addDocuments: vi.fn(),
  deleteDocument: vi.fn(),
  queryDocuments: vi.fn(),
  refreshCollections: vi.fn(),
  refreshStats: vi.fn(),
  clearCache: vi.fn(),
  invalidateCache: vi.fn()
};