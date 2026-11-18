// Type definitions matching arcade-server JSON responses

export type GameTab = 'home' | 'wordle' | 'spelling-bee' | 'boggle' | 'timer' | 'stats' | 'settings';

export interface SystemStatus {
  status: string;
  version: string;
  answers_count: number;
  legal_guess_count: number;
  dictionary_loaded: boolean;
}

export interface GamesSummary {
  wordle_active: boolean;
  spelling_bee_active: boolean;
  boggle_active: boolean;
}

// Wordle Types
export type LetterFeedbackType = 'Correct' | 'Present' | 'Absent';
export type KeyFeedbackType = 'Correct' | 'Present' | 'Absent' | 'Unknown';
export type WordleStatus = 'NotStarted' | 'InProgress' | 'Won' | 'Lost';

export interface WordleGuess {
  word: string;
  feedback: LetterFeedbackType[];
}

export interface WordleState {
  attempts_remaining: number;
  max_attempts: number;
  guesses: WordleGuess[];
  game_status: WordleStatus;
  secret: string | null;
  keyboard_state: Record<string, KeyFeedbackType>;
  remaining_candidates_count: number;
}

export interface WordleSuggestion {
  word: string;
  expected_info_bits: number;
  expected_remaining: number;
  worst_case_remaining: number;
  is_possible_answer: boolean;
}

export interface WordleHintResponse {
  remaining_candidates: number;
  is_single_candidate: boolean;
  suggestions: WordleSuggestion[];
}

// Spelling Bee Types
export interface SpellingBeeState {
  main_letter: string;
  outer_letters: string[];
  score: number;
  max_possible_score: number;
  words_found: string[];
  total_solutions: number;
  pangrams_found: number;
  is_queen_bee: boolean;
  rank: string;
}

export interface SpellingBeeSubmitResponse {
  result:
    | 'Accepted'
    | 'TooShort'
    | 'InvalidCharacters'
    | 'MissingMainLetter'
    | 'InvalidLetters'
    | 'NotInSolutions'
    | 'AlreadyGuessed';
  message: string;
  points: number;
  is_pangram?: boolean;
  score?: number;
  rank?: string;
  state?: SpellingBeeState;
}

export interface SpellingBeeHintsResponse {
  total_words: number;
  total_points: number;
  pangram_count: number;
  length_counts: Record<string, number>;
  first_letter_counts: Record<string, number>;
  prefix_counts: Record<string, number>;
}

// Boggle Types
export interface BoggleState {
  size: number;
  cells: string[];
  grid: string[][];
  score: number;
  accepted_words: string[];
  total_possible_words: number;
  max_possible_score: number;
  is_active: boolean;
  ends_at_unix_ms: number | null;
}

export interface BoggleSubmitResponse {
  result:
    | 'Accepted'
    | 'TooShort'
    | 'InvalidCharacters'
    | 'NotInDictionary'
    | 'NotOnBoard'
    | 'AlreadyGuessed';
  message: string;
  points: number;
  score?: number;
  word?: string;
  state?: BoggleState;
}

export interface BoggleEndResponse {
  score: number;
  total_possible_words: number;
  max_possible_score: number;
  accepted_count: number;
  missed_sample: string[];
  state: BoggleState;
}

// Statistics Types
export interface WordleStats {
  games_played: number;
  games_won: number;
  current_streak: number;
  max_streak: number;
  guess_distribution: [number, number, number, number, number, number];
  win_rate: number;
  average_guesses: number;
}

export interface SpellingBeeStats {
  games_played: number;
  highest_score: number;
  total_words_found: number;
  pangrams_found: number;
  queen_bees: number;
}

export interface BoggleStats {
  games_played: number;
  highest_score: number;
  total_words_found: number;
  average_score: number;
  best_words: string[];
}

export interface UnifiedStats {
  wordle: WordleStats;
  spelling_bee: SpellingBeeStats;
  boggle: BoggleStats;
}

// Settings
export interface ArcadeSettings {
  sound_enabled: boolean;
  music_enabled: boolean;
  reduced_motion: boolean;
}
