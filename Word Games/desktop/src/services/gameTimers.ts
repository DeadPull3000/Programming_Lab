// Custom game countdown timer configurations persisted to localStorage

export interface GameTimerSettings {
  boggleSecs: number; // 0 = untimed, default 180
  spellingBeeSecs: number; // 0 = untimed (default), e.g. 180
  wordleSecs: number; // 0 = untimed (default), e.g. 120
}

const STORAGE_KEY = 'arcade_custom_game_timers';

export const getGameTimers = (): GameTimerSettings => {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      return {
        boggleSecs: typeof parsed.boggleSecs === 'number' ? parsed.boggleSecs : 180,
        spellingBeeSecs: typeof parsed.spellingBeeSecs === 'number' ? parsed.spellingBeeSecs : 0,
        wordleSecs: typeof parsed.wordleSecs === 'number' ? parsed.wordleSecs : 0,
      };
    }
  } catch {
    // fallback
  }
  return {
    boggleSecs: 180,
    spellingBeeSecs: 0,
    wordleSecs: 0,
  };
};

export const setGameTimer = (game: 'boggle' | 'spelling-bee' | 'wordle', secs: number): void => {
  const current = getGameTimers();
  if (game === 'boggle') current.boggleSecs = Math.max(0, secs);
  if (game === 'spelling-bee') current.spellingBeeSecs = Math.max(0, secs);
  if (game === 'wordle') current.wordleSecs = Math.max(0, secs);
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(current));
  } catch {
    // ignore
  }
};
