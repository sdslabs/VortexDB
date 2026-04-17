import { useState, useRef, useEffect } from 'react';
import './App.css';

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8002';

const SUPPORTED_FORMATS = ['.pdf', '.txt', '.md', '.docx', '.csv'];
const IMAGE_FORMATS = ['.png', '.jpg', '.jpeg', '.gif', '.bmp', '.webp'];

function App() {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [uploadedFiles, setUploadedFiles] = useState(new Set());
  const [isDragging, setIsDragging] = useState(false);
  const [uploadModal, setUploadModal] = useState(null);
  const messagesEndRef = useRef(null);
  const fileInputRef = useRef(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const updateModal = (status, progress = null, detail = null) => {
    setUploadModal(prev => ({
      ...prev,
      status,
      progress,
      detail,
      timestamp: Date.now()
    }));
  };

  const handleFileSelect = async (file) => {
    const ext = '.' + file.name.split('.').pop().toLowerCase();
    
    if (IMAGE_FORMATS.includes(ext)) {
      setUploadModal({
        status: 'error',
        fileName: file.name,
        detail: `Image files are not supported. This model does not support image input. Please upload a text document (${SUPPORTED_FORMATS.join(', ')}).`
      });
      return;
    }

    if (!SUPPORTED_FORMATS.includes(ext)) {
      setUploadModal({
        status: 'error',
        fileName: file.name,
        detail: `Unsupported format: ${ext}. Please upload ${SUPPORTED_FORMATS.join(', ')} files.`
      });
      return;
    }

    setUploadModal({
      status: 'uploading',
      fileName: file.name,
      progress: 0,
      detail: 'Reading file...'
    });

    const formData = new FormData();
    formData.append('file', file);

    try {
      updateModal('uploading', 10, 'Uploading to server...');
      
      const res = await fetch(`${API_URL}/upload`, {
        method: 'POST',
        body: formData,
      });

      updateModal('processing', 50, 'Processing document...');

      const data = await res.json();

      if (!res.ok) {
        throw new Error(data.detail || 'Upload failed');
      }

      updateModal('vectors', 75, 'Creating embeddings...');

      await new Promise(resolve => setTimeout(resolve, 500));
      
      updateModal('indexing', 90, 'Indexing vectors...');
      
      await new Promise(resolve => setTimeout(resolve, 300));

      updateModal('complete', 100, `Indexed ${data.chunks} chunks successfully!`);

      setUploadedFiles(prev => new Set([...prev, file.name]));
      
      setTimeout(() => {
        setUploadModal(null);
        addMessage('system', `📄 Uploaded: ${file.name} (${data.chunks} chunks indexed)`);
      }, 1500);
      
    } catch (e) {
      setUploadModal({
        status: 'error',
        fileName: file.name,
        detail: e.message
      });
    }
  };

  const closeModal = () => {
    if (uploadModal?.status !== 'uploading' && uploadModal?.status !== 'processing') {
      setUploadModal(null);
    }
  };

  const handleDrop = (e) => {
    e.preventDefault();
    setIsDragging(false);
    const file = e.dataTransfer.files[0];
    if (file) handleFileSelect(file);
  };

  const handleDragOver = (e) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = () => {
    setIsDragging(false);
  };

  const addMessage = (role, content, sources = []) => {
    setMessages(prev => [...prev, { role, content, sources, id: Date.now() }]);
  };

  const addError = (content) => {
    setMessages(prev => [...prev, { role: 'error', content, id: Date.now() }]);
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const question = input.trim();
    setInput('');
    addMessage('user', question);
    setIsLoading(true);

    try {
      const res = await fetch(`${API_URL}/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ question }),
      });

      const data = await res.json();

      if (!res.ok) {
        throw new Error(data.detail || 'Request failed');
      }

      addMessage('assistant', data.answer, data.sources || []);
    } catch (e) {
      addError(e.message);
    } finally {
      setIsLoading(false);
    }
  };

  const clearChat = () => {
    setMessages([]);
  };

  const clearAll = async () => {
    if (!confirm('Clear all documents and chat?')) return;
    
    try {
      await fetch(`${API_URL}/clear`, { method: 'DELETE' });
      setUploadedFiles(new Set());
      setMessages([]);
    } catch (e) {
      addError('Failed to clear documents');
    }
  };

  const getStatusIcon = () => {
    if (!uploadModal) return null;
    
    switch (uploadModal.status) {
      case 'uploading':
      case 'processing':
      case 'vectors':
      case 'indexing':
        return (
          <div className="modal-spinner">
            <div className="spinner"></div>
          </div>
        );
      case 'complete':
        return (
          <div className="modal-icon success">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
              <polyline points="22 4 12 14.01 9 11.01"/>
            </svg>
          </div>
        );
      case 'error':
        return (
          <div className="modal-icon error">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="10"/>
              <line x1="15" y1="9" x2="9" y2="15"/>
              <line x1="9" y1="9" x2="15" y2="15"/>
            </svg>
          </div>
        );
      default:
        return null;
    }
  };

  const getStatusText = () => {
    if (!uploadModal) return '';
    
    switch (uploadModal.status) {
      case 'uploading': return 'Uploading';
      case 'processing': return 'Processing';
      case 'vectors': return 'Creating Embeddings';
      case 'indexing': return 'Indexing';
      case 'complete': return 'Complete';
      case 'error': return 'Error';
      default: return '';
    }
  };

  return (
    <div className="app">
      {uploadModal && (
        <div className="modal-overlay" onClick={closeModal}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-content">
              {getStatusIcon()}
              <h3>{getStatusText()}</h3>
              <p className="modal-filename">{uploadModal.fileName}</p>
              
              {(uploadModal.status === 'uploading' || uploadModal.status === 'processing' || 
                uploadModal.status === 'vectors' || uploadModal.status === 'indexing') && (
                <div className="modal-progress">
                  <div className="progress-bar">
                    <div 
                      className="progress-fill" 
                      style={{ width: `${uploadModal.progress}%` }}
                    ></div>
                  </div>
                  <span className="progress-text">{uploadModal.progress}%</span>
                </div>
              )}
              
              <p className="modal-detail">{uploadModal.detail}</p>
              
              {uploadModal.status === 'error' && (
                <button className="modal-close-btn" onClick={closeModal}>
                  Close
                </button>
              )}
            </div>
          </div>
        </div>
      )}

      <aside className="sidebar">
        <div className="sidebar-header">
          <h1>VortexDB RAG</h1>
        </div>
        
        <button className="new-chat-btn" onClick={clearChat}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 5v14M5 12h14"/>
          </svg>
          New Chat
        </button>

        <div className="upload-zone" 
             onDrop={handleDrop}
             onDragOver={handleDragOver}
             onDragLeave={handleDragLeave}>
          <input
            type="file"
            ref={fileInputRef}
            accept={SUPPORTED_FORMATS.join(',')}
            onChange={(e) => e.target.files[0] && handleFileSelect(e.target.files[0])}
          />
          <div className="upload-content" onClick={() => fileInputRef.current?.click()}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="17 8 12 3 7 8"/>
              <line x1="12" y1="3" x2="12" y2="15"/>
            </svg>
            <span>Upload Document</span>
            <span className="upload-formats">{SUPPORTED_FORMATS.join(', ')}</span>
          </div>
        </div>

        <div className="documents-list">
          <h3>Documents ({uploadedFiles.size})</h3>
          {uploadedFiles.size === 0 && (
            <p className="no-docs">No documents uploaded</p>
          )}
          {Array.from(uploadedFiles).map((file, i) => (
            <div key={i} className="document-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
              </svg>
              {file}
            </div>
          ))}
        </div>

        <button className="clear-btn" onClick={clearAll}>
          Clear All
        </button>
      </aside>

      <main className="chat-area">
        <div className="messages">
          {messages.length === 0 && (
            <div className="welcome">
              <h2>VortexDB RAG Demo</h2>
              <p>Upload documents and ask questions about them</p>
            </div>
          )}
          
          {messages.map((msg, i) => (
            <div key={msg.id || i} className={`message message-${msg.role}`}>
              <div className="message-avatar">
                {msg.role === 'user' ? (
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
                  </svg>
                ) : msg.role === 'error' ? (
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
                  </svg>
                ) : (
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                  </svg>
                )}
              </div>
              <div className="message-content">
                <div className="message-text">{msg.content}</div>
                {msg.sources && msg.sources.length > 0 && (
                  <div className="sources">
                    <h4>Sources:</h4>
                    {msg.sources.map((s, j) => (
                      <div key={j} className="source-item">
                        <span className="source-score">[{s.score}]</span>
                        <span className="source-text">{s.text}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          ))}
          
          {isLoading && (
            <div className="message message-assistant">
              <div className="message-avatar">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
              </div>
              <div className="message-content">
                <div className="message-text typing">
                  <span className="dot"></span>
                  <span className="dot"></span>
                  <span className="dot"></span>
                </div>
              </div>
            </div>
          )}
          <div ref={messagesEndRef} />
        </div>

        <form className="input-area" onSubmit={handleSubmit}>
          <div className={`input-container ${isDragging ? 'dragging' : ''}`}>
            <input
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder="Ask a question about your documents..."
              disabled={isLoading}
            />
            <button type="submit" disabled={!input.trim() || isLoading}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="22" y1="2" x2="11" y2="13"/>
                <polygon points="22 2 15 22 11 13 2 9 22 2"/>
              </svg>
            </button>
          </div>
        </form>
      </main>
    </div>
  );
}

export default App;
