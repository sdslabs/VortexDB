const API_URL = import.meta.env.VITE_API_URL || '/api';

export function parseApiError(data, status) {
  if (status === 429) {
    return data.error || 'Rate limit exceeded. Try again in a minute.';
  }
  if (typeof data.detail === 'string') {
    return data.detail;
  }
  if (Array.isArray(data.detail)) {
    return data.detail.map((e) => e.msg).join(', ');
  }
  return 'Request failed';
}

export async function chat(query) {
  const res = await fetch(`${API_URL}/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query }),
  });
  const data = await res.json();
  if (!res.ok) {
    throw new Error(parseApiError(data, res.status));
  }
  return data;
}

export async function getHealth() {
  const res = await fetch(`${API_URL}/health`);
  const data = await res.json();
  if (!res.ok) {
    throw new Error(data.message || 'Health check failed');
  }
  return data;
}
