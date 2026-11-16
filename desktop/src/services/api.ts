import {
  ArcadeSettings,
  BoggleEndResponse,
  BoggleState,
  BoggleSubmitResponse,
  GamesSummary,
  SpellingBeeHintsResponse,
  SpellingBeeState,
  SpellingBeeSubmitResponse,
  SystemStatus,
  UnifiedStats,
  WordleHintResponse,
  WordleState,
} from '../types/arcade';

// Vite's dev and preview servers forward these same-origin requests to Rust.
const API_BASE = '/api';

async function request<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const url = `${API_BASE}${endpoint}`;
  try {
    const res = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(options?.headers || {}),
      },
    });

    if (!res.ok) {
      const errText = await res.text();
      let message = `HTTP error ${res.status}: ${errText}`;
      try {
        const parsed = JSON.parse(errText);
        if (typeof parsed.error === 'string') message = parsed.error;
      } catch { /* Use the raw response when it is not JSON. */ }
      throw new Error(message);
    }

    const data = await res.json();
    if (data.error) {
      throw new Error(data.error);
    }
    return data as T;
  } catch (err: any) {
    console.error(`API Error on ${endpoint}:`, err);
    throw err;
  }
}

export const arcadeApi = {
  // System
  getStatus: () => request<SystemStatus>('/status'),
  getSummary: () => request<GamesSummary>('/games/summary'),

  // Wordle
  getWordleState: () => request<WordleState>('/wordle/state'),
  startWordle: (secret?: string) =>
    request<WordleState>('/wordle/start', {
      method: 'POST',
      body: secret ? JSON.stringify({ secret }) : '{}',
    }),
  submitWordleGuess: (guess: string) =>
    request<WordleState>('/wordle/guess', {
      method: 'POST',
      body: JSON.stringify({ guess }),
    }),
  getWordleHint: (strategy: 'entropy' | 'minimax' = 'entropy', top_n = 5) =>
    request<WordleHintResponse>('/wordle/hint', {
      method: 'POST',
      body: JSON.stringify({ strategy, top_n }),
    }),

  // Spelling Bee
  getSpellingBeeState: () => request<SpellingBeeState>('/spelling-bee/state'),
  startSpellingBee: () =>
    request<SpellingBeeState>('/spelling-bee/start', {
      method: 'POST',
      body: '{}',
    }),
  submitSpellingBeeWord: (word: string) =>
    request<SpellingBeeSubmitResponse>('/spelling-bee/submit', {
      method: 'POST',
      body: JSON.stringify({ word }),
    }),
  shuffleSpellingBee: () =>
    request<{ outer_letters: string[] }>('/spelling-bee/shuffle', {
      method: 'POST',
      body: '{}',
    }),
  getSpellingBeeHints: () => request<SpellingBeeHintsResponse>('/spelling-bee/hints'),

  // Boggle
  getBoggleState: () => request<BoggleState>('/boggle/state'),
  startBoggle: (size = 4) =>
    request<BoggleState>('/boggle/start', {
      method: 'POST',
      body: JSON.stringify({ size }),
    }),
  submitBoggleWord: (word: string) =>
    request<BoggleSubmitResponse>('/boggle/submit', {
      method: 'POST',
      body: JSON.stringify({ word }),
    }),
  endBoggle: () =>
    request<BoggleEndResponse>('/boggle/end', {
      method: 'POST',
      body: '{}',
    }),

  // Statistics & Settings
  getStats: () => request<UnifiedStats>('/stats'),
  resetStats: () =>
    request<{ success: boolean; message: string }>('/stats/reset', {
      method: 'POST',
      body: '{}',
    }),
  getSettings: () => request<ArcadeSettings>('/settings'),
  updateSettings: (settings: Partial<ArcadeSettings>) =>
    request<ArcadeSettings>('/settings', {
      method: 'POST',
      body: JSON.stringify(settings),
    }),
};
