import { useState, useRef, useEffect } from 'react';
import './App.css';
import { chat, getHealth } from './api';

const STORAGE_KEY_CHATS = 'vortexdb_rag_chats';
const STORAGE_KEY_ACTIVE = 'vortexdb_rag_active_chat_id';

const createSession = () => ({
  id: `chat_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`,
  title: 'New Chat',
  createdAt: Date.now(),
  messages: [],
});

function App() {
  const [chats, setChats] = useState(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY_CHATS);
      if (saved) {
        const parsed = JSON.parse(saved);
        if (Array.isArray(parsed) && parsed.length > 0) {
          return parsed;
        }
      }
    } catch (e) {
      console.error('Failed to load chats from localStorage', e);
    }
    return [createSession()];
  });

  const [activeChatId, setActiveChatId] = useState(() => {
    try {
      const savedId = localStorage.getItem(STORAGE_KEY_ACTIVE);
      if (savedId) return savedId;
    } catch (e) {
      console.error(e);
    }
    return chats[0]?.id || '';
  });

  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [backendStatus, setBackendStatus] = useState('checking');
  const messagesEndRef = useRef(null);

  // Sync activeChatId if invalid
  useEffect(() => {
    if (!chats.some((c) => c.id === activeChatId) && chats.length > 0) {
      setActiveChatId(chats[0].id);
    }
  }, [chats, activeChatId]);

  // Persist chats and activeChatId
  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY_CHATS, JSON.stringify(chats));
    } catch (e) {
      console.error('Failed to save chats to localStorage', e);
    }
  }, [chats]);

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY_ACTIVE, activeChatId);
    } catch (e) {
      console.error('Failed to save active chat ID', e);
    }
  }, [activeChatId]);

  const activeChat = chats.find((c) => c.id === activeChatId) || chats[0] || createSession();
  const messages = activeChat.messages || [];

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, isLoading]);

  useEffect(() => {
    getHealth()
      .then((data) => setBackendStatus(data.status === 'ok' ? 'ready' : 'degraded'))
      .catch(() => setBackendStatus('offline'));
  }, []);

  const updateActiveChatMessages = (updater, newTitle = null) => {
    setChats((prevChats) =>
      prevChats.map((c) => {
        if (c.id === activeChatId) {
          const updatedMessages = typeof updater === 'function' ? updater(c.messages) : updater;
          return {
            ...c,
            title: newTitle || c.title,
            messages: updatedMessages,
          };
        }
        return c;
      })
    );
  };

  const addMessage = (role, content, sources = []) => {
    const newMsg = {
      role,
      content,
      sources,
      id: `${role}_${Date.now()}_${Math.random().toString(36).substring(2, 6)}`,
    };
    updateActiveChatMessages((prev) => [...prev, newMsg]);
  };

  const addError = (content) => {
    const errorMsg = {
      role: 'error',
      content,
      id: `err_${Date.now()}_${Math.random().toString(36).substring(2, 6)}`,
    };
    updateActiveChatMessages((prev) => [...prev, errorMsg]);
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const question = input.trim();
    setInput('');

    // Generate title from first message if title is 'New Chat'
    let updatedTitle = null;
    if (activeChat.messages.length === 0 && activeChat.title === 'New Chat') {
      updatedTitle = question.length > 32 ? question.slice(0, 32).trim() + '...' : question;
    }

    const userMsg = {
      role: 'user',
      content: question,
      sources: [],
      id: `user_${Date.now()}_${Math.random().toString(36).substring(2, 6)}`,
    };

    updateActiveChatMessages((prev) => [...prev, userMsg], updatedTitle);
    setIsLoading(true);

    try {
      const data = await chat(question);
      const assistantMsg = {
        role: 'assistant',
        content: data.answer,
        sources: data.sources || [],
        id: `asst_${Date.now()}_${Math.random().toString(36).substring(2, 6)}`,
      };
      updateActiveChatMessages((prev) => [...prev, assistantMsg]);
    } catch (err) {
      addError(err.message);
    } finally {
      setIsLoading(false);
    }
  };

  const startNewChat = () => {
    // If current chat is already empty, just stay on it
    if (activeChat && activeChat.messages.length === 0) {
      return;
    }

    const newChat = createSession();
    setChats((prev) => [newChat, ...prev]);
    setActiveChatId(newChat.id);
  };

  const deleteChat = (e, chatIdToDelete) => {
    e.stopPropagation();
    setChats((prev) => {
      const filtered = prev.filter((c) => c.id !== chatIdToDelete);
      if (filtered.length === 0) {
        const fresh = createSession();
        setActiveChatId(fresh.id);
        return [fresh];
      }
      if (activeChatId === chatIdToDelete) {
        setActiveChatId(filtered[0].id);
      }
      return filtered;
    });
  };

  const statusLabel = {
    checking: 'Checking...',
    ready: 'Ready',
    degraded: 'Degraded',
    offline: 'Offline',
  }[backendStatus];

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="sidebar-header">
          <h1>VortexDB RAG</h1>
          <div className={`backend-status status-${backendStatus}`}>
            <span className="status-dot" />
            Backend: {statusLabel}
          </div>
        </div>

        <button className="new-chat-btn" onClick={startNewChat}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 5v14M5 12h14" />
          </svg>
          New Chat
        </button>

        <div className="chat-history">
          <div className="chat-history-header">Previous Chats</div>
          <div className="chat-list">
            {chats.map((c) => (
              <div
                key={c.id}
                className={`chat-list-item ${c.id === activeChatId ? 'active' : ''}`}
                onClick={() => setActiveChatId(c.id)}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                </svg>
                <span className="chat-title" title={c.title}>
                  {c.title}
                </span>
                {chats.length > 1 && (
                  <button
                    className="delete-chat-btn"
                    onClick={(e) => deleteChat(e, c.id)}
                    title="Delete chat"
                    aria-label="Delete chat"
                  >
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                    </svg>
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>
      </aside>

      <main className="chat-area">
        <div className="messages">
          {messages.length === 0 && (
            <div className="welcome">
              <h2>VortexDB RAG Demo</h2>
              <p>Ask questions about the pre-ingested documents</p>
            </div>
          )}

          {messages.map((msg, i) => (
            <div key={msg.id || i} className={`message message-${msg.role}`}>
              <div className="message-avatar">
                {msg.role === 'user' ? (
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" />
                  </svg>
                ) : msg.role === 'error' ? (
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z" />
                  </svg>
                ) : (
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
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
                        <span className="source-index">[{j + 1}]</span>
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
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                </svg>
              </div>
              <div className="message-content">
                <div className="message-text typing">
                  <span className="dot" />
                  <span className="dot" />
                  <span className="dot" />
                </div>
              </div>
            </div>
          )}
          <div ref={messagesEndRef} />
        </div>

        <form className="input-area" onSubmit={handleSubmit}>
          <div className="input-container">
            <input
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder="Ask a question about your documents..."
              disabled={isLoading || backendStatus === 'offline'}
            />
            <button type="submit" disabled={!input.trim() || isLoading || backendStatus === 'offline'}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="22" y1="2" x2="11" y2="13" />
                <polygon points="22 2 15 22 11 13 2 9 22 2" />
              </svg>
            </button>
          </div>
        </form>
      </main>
    </div>
  );
}

export default App;
