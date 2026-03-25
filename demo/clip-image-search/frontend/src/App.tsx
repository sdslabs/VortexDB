import { useState, useEffect, useCallback, useRef } from 'react'

interface SearchResult {
  image_url: string
  point_id: string
  score?: number
}

interface SearchResponse {
  results: SearchResult[]
  query: string
  vectorize_time_ms: number
  search_time_ms: number
  total_time_ms: number
}

interface Stats {
  total_images: number
  clip_vectorizer_status: string
  vortexdb_status: string
}

const SUGGESTIONS = [
  "sunset ocean",
  "sleeping cat",
  "snowy mountains",
  "city street",
  "colorful flowers",
  "dog park",
  "food plate",
  "highway cars"
]

function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value)

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedValue(value)
    }, delay)

    return () => {
      clearTimeout(timer)
    }
  }, [value, delay])

  return debouncedValue
}

function App() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [stats, setStats] = useState<Stats | null>(null)
  const [searchStats, setSearchStats] = useState<{
    vectorize_time_ms: number
    search_time_ms: number
    total_time_ms: number
  } | null>(null)
  
  const inputRef = useRef<HTMLInputElement>(null)
  const debouncedQuery = useDebounce(query, 150) // 150ms debounce for real-time feel

  // Fetch system stats on mount
  useEffect(() => {
    fetch('/api/stats')
      .then(res => res.json())
      .then(data => setStats(data))
      .catch(err => console.error('Failed to fetch stats:', err))
  }, [])

  // Perform search when debounced query changes
  const performSearch = useCallback(async (searchQuery: string) => {
    if (!searchQuery.trim()) {
      setResults([])
      setSearchStats(null)
      return
    }

    setLoading(true)
    setError(null)

    try {
      const response = await fetch('/api/search', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: searchQuery,
          limit: 24
        })
      })

      if (!response.ok) {
        throw new Error(`Search failed: ${response.statusText}`)
      }

      const data: SearchResponse = await response.json()
      setResults(data.results)
      setSearchStats({
        vectorize_time_ms: data.vectorize_time_ms,
        search_time_ms: data.search_time_ms,
        total_time_ms: data.total_time_ms
      })
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed')
      setResults([])
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    performSearch(debouncedQuery)
  }, [debouncedQuery, performSearch])

  const handleSuggestionClick = (suggestion: string) => {
    setQuery(suggestion)
    inputRef.current?.focus()
  }

  const getSpeedClass = (ms: number) => {
    if (ms < 50) return 'fast'
    if (ms < 200) return 'medium'
    return 'slow'
  }

  return (
    <div className="app">
      <header className="header">
        <div className="header-content">
          <div className="header-top">
            <div className="logo">
              <span className="logo-text">VortexDB</span>
              <span className="logo-divider">/</span>
              <span className="logo-subtitle">Image Search</span>
            </div>
            <div className="stats-badges">
              <div className="badge">
                <span 
                  className={`dot ${stats?.clip_vectorizer_status === 'online' ? 'online' : 'offline'}`}
                />
                CLIP
              </div>
              <div className="badge">
                <span 
                  className={`dot ${stats?.vortexdb_status === 'online' ? 'online' : 'offline'}`}
                />
                VortexDB
              </div>
              {stats && (
                <div className="badge">
                  {stats.total_images} images
                </div>
              )}
            </div>
          </div>

          <div className="search-container">
            <div className="search-input-wrapper">
              <span className="search-icon"></span>
              <input
                ref={inputRef}
                type="text"
                className="search-input"
                placeholder="Search images..."
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                autoFocus
              />
            </div>
          </div>

          {searchStats && (
            <div className="performance-stats">
              <div className="stat">
                <span className="stat-label">Vectorize:</span>
                <span className={`stat-value ${getSpeedClass(searchStats.vectorize_time_ms)}`}>
                  {searchStats.vectorize_time_ms.toFixed(0)}ms
                </span>
              </div>
              <div className="stat">
                <span className="stat-label">Search:</span>
                <span className={`stat-value ${getSpeedClass(searchStats.search_time_ms)}`}>
                  {searchStats.search_time_ms.toFixed(0)}ms
                </span>
              </div>
              <div className="stat">
                <span className="stat-label">Total:</span>
                <span className={`stat-value ${getSpeedClass(searchStats.total_time_ms)}`}>
                  {searchStats.total_time_ms.toFixed(0)}ms
                </span>
              </div>
              <div className="stat">
                <span className="stat-label">Results:</span>
                <span className="stat-value">{results.length}</span>
              </div>
            </div>
          )}
        </div>
      </header>

      <main className="main-content">
        {error && (
          <div className="error-state">
            <span className="error-icon">!</span>
            <p>{error}</p>
          </div>
        )}

        {loading && (
          <div className="loading-container">
            <div className="spinner" />
            <p>Searching...</p>
          </div>
        )}

        {!loading && !error && results.length > 0 && (
          <>
            <div className="results-info">
              <p className="results-count">
                Showing <strong>{results.length}</strong> results for "<strong>{debouncedQuery}</strong>"
              </p>
            </div>
            <div className="image-grid">
              {results.map((result, index) => (
                <div key={result.point_id} className="image-card">
                  <img
                    src={result.image_url}
                    alt={`Search result ${index + 1}`}
                    loading="lazy"
                    onError={(e) => {
                      (e.target as HTMLImageElement).src = 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect fill="%231a1a24" width="100" height="100"/><text x="50" y="50" text-anchor="middle" dy=".3em" fill="%23606070">No Image</text></svg>'
                    }}
                  />
                  <div className="image-card-overlay">
                    <div className="image-card-info">
                      ID: {result.point_id.slice(0, 8)}...
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </>
        )}

        {!loading && !error && results.length === 0 && !query && (
          <div className="empty-state">
            <div className="empty-state-icon"><span></span></div>
            <h2>Semantic Image Search</h2>
            <p>Describe what you're looking for. Results update as you type.</p>
            
            <div className="suggestions">
              <h3>Suggestions</h3>
              <div className="suggestion-chips">
                {SUGGESTIONS.map((suggestion) => (
                  <button
                    key={suggestion}
                    className="suggestion-chip"
                    onClick={() => handleSuggestionClick(suggestion)}
                  >
                    {suggestion}
                  </button>
                ))}
              </div>
            </div>
          </div>
        )}

        {!loading && !error && results.length === 0 && query && (
          <div className="empty-state">
            <div className="empty-state-icon empty-state-icon--empty"><span>?</span></div>
            <h2>No Results</h2>
            <p>No images matched "{query}". Try a different description.</p>
          </div>
        )}
      </main>

      <footer className="footer">
        <p>
          Powered by <strong>VortexDB</strong>
        </p>
      </footer>
    </div>
  )
}

export default App
