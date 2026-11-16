import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  AlertCircle,
  RotateCcw,
} from 'lucide-react';
import {
  KeyFeedbackType,
  LetterFeedbackType,
  WordleHintResponse,
  WordleState,
} from '../types/arcade';
import { arcadeApi } from '../services/api';
import { CozyButton } from '../components/CozyButton';
import { CozyModal } from '../components/CozyModal';
import { sound } from '../services/sound';
import { PrincessCat } from '../components/stickers/CustomCats';
import {
  CookieEasterEgg,
  PawPrintIcon,
  WordleGameIcon,
} from '../components/stickers/CustomIcons';
import {
  DoodleStars,
  PostageStamp,
  PressedFlower,
  ScrawlyNote,
  WashiTape,
} from '../components/stickers/DiaryStickers';

interface WordlePageProps {
  onGameComplete?: () => void;
  active?: boolean;
}

export const WordlePage: React.FC<WordlePageProps> = ({ onGameComplete, active = true }) => {
  const initialized = useRef(false);
  const pending = useRef(false);
  const [gameState, setGameState] = useState<WordleState | null>(null);
  const [currentInput, setCurrentInput] = useState('');
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isShaking, setIsShaking] = useState(false);
  const [loading, setLoading] = useState(false);

  // Helper/Solver state
  const [solverStrategy, setSolverStrategy] = useState<'entropy' | 'minimax'>('entropy');
  const [hintData, setHintData] = useState<WordleHintResponse | null>(null);
  const [hintLoading, setHintLoading] = useState(false);

  // End of game modal
  const [showEndModal, setShowEndModal] = useState(false);

  // Load existing state or start a new game
  const fetchState = useCallback(async () => {
    try {
      setLoading(true);
      let state = await arcadeApi.getWordleState();
      if (state.game_status === 'NotStarted') {
        state = await arcadeApi.startWordle();
      }
      setGameState(state);
      if (state.game_status === 'Won' || state.game_status === 'Lost') {
        setShowEndModal(true);
      }
    } catch (err: any) {
      setErrorMessage(err.message || 'Failed to connect to Wordle engine');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (initialized.current) return;
    initialized.current = true;
    void fetchState();
  }, [fetchState]);

  // Restart game
  const handleNewGame = async () => {
    if (pending.current) return;
    pending.current = true;
    try {
      setLoading(true);
      setCurrentInput('');
      setErrorMessage(null);
      setHintData(null);
      setShowEndModal(false);
      const state = await arcadeApi.startWordle();
      setGameState(state);
      sound.playSuccess();
    } catch (err: any) {
      setErrorMessage(err.message || 'Could not start new Wordle game');
    } finally {
      pending.current = false;
      setLoading(false);
    }
  };

  // Submit current guess
  const handleSubmitGuess = async () => {
    if (!gameState || gameState.game_status !== 'InProgress') return;
    if (currentInput.length !== 5) {
      triggerError('Word must be 5 letters!');
      return;
    }

    if (pending.current) return;
    pending.current = true;
    try {
      setErrorMessage(null);
      const updated = await arcadeApi.submitWordleGuess(currentInput);
      setGameState(updated);
      setCurrentInput('');
      setHintData(null);

      if (updated.game_status === 'Won') {
        sound.playSuccess();
        setShowEndModal(true);
        onGameComplete?.();
      } else if (updated.game_status === 'Lost') {
        sound.playInvalid();
        setShowEndModal(true);
        onGameComplete?.();
      } else {
        sound.playPop(500);
      }
    } catch (err: any) {
      triggerError(err.message || 'Invalid guess');
    } finally {
      pending.current = false;
    }
  };

  const triggerError = (msg: string) => {
    sound.playInvalid();
    setErrorMessage(msg);
    setIsShaking(true);
    setTimeout(() => setIsShaking(false), 500);
    setTimeout(() => setErrorMessage(null), 3000);
  };

  // Handle key typing
  const handleKeyPress = useCallback(
    (key: string) => {
      if (!gameState || gameState.game_status !== 'InProgress') return;

      const upper = key.toUpperCase();
      if (upper === 'ENTER') {
        handleSubmitGuess();
      } else if (upper === 'BACKSPACE' || upper === 'DELETE' || upper === '⌫') {
        sound.playClick();
        setCurrentInput((prev) => prev.slice(0, -1));
      } else if (/^[A-Z]$/.test(upper)) {
        if (currentInput.length < 5) {
          sound.playClick();
          setCurrentInput((prev) => prev + upper);
        }
      }
    },
    [currentInput, gameState]
  );

  // Physical keyboard listener
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (!active || loading || pending.current || e.ctrlKey || e.altKey || e.metaKey || e.isComposing) return;
      if (e.target instanceof HTMLElement && e.target.closest('input, textarea, select, [contenteditable="true"], [role="dialog"]')) return;
      if (document.querySelector('[role="dialog"]')) return;
      if (e.target instanceof HTMLElement && e.target.closest('button') && (e.key === 'Enter' || e.key === ' ')) return;
      if (e.key === 'Enter' || e.key === 'Backspace' || e.key === 'Delete' || /^[a-zA-Z]$/.test(e.key)) e.preventDefault();
      handleKeyPress(e.key);
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [handleKeyPress, active, loading]);

  // Request hint from Rust solver
  const handleRequestHint = async () => {
    try {
      setHintLoading(true);
      sound.playClick();
      const res = await arcadeApi.getWordleHint(solverStrategy, 5);
      setHintData(res);
    } catch (err: any) {
      setErrorMessage(err.message || 'Solver hint failed');
    } finally {
      setHintLoading(false);
    }
  };

  // Helper for tile color
  const getTileStyles = (
    fb?: LetterFeedbackType,
    letter?: string
  ): React.CSSProperties => {
    if (!fb) {
      return {
        backgroundColor: letter ? '#FFF8EE' : '#FFF0D9',
        color: 'var(--text-main)',
        borderColor: letter ? 'var(--outline-cocoa)' : '#d5c4b1',
      };
    }
    switch (fb) {
      case 'Correct':
        return {
          backgroundColor: 'var(--color-success)',
          color: '#ffffff',
          borderColor: 'var(--outline-cocoa)',
        };
      case 'Present':
        return {
          backgroundColor: 'var(--color-present)',
          color: '#523c00',
          borderColor: 'var(--outline-cocoa)',
        };
      case 'Absent':
        return {
          backgroundColor: 'var(--color-absent)',
          color: '#ffffff',
          borderColor: 'var(--outline-cocoa)',
        };
    }
  };

  const getKeyColor = (status?: KeyFeedbackType) => {
    switch (status) {
      case 'Correct':
        return { bg: 'var(--color-success)', text: '#ffffff' };
      case 'Present':
        return { bg: 'var(--color-present)', text: '#523c00' };
      case 'Absent':
        return { bg: 'var(--color-absent)', text: '#ffffff' };
      default:
        return { bg: '#FFF0D9', text: 'var(--text-main)' };
    }
  };

  if (!gameState) {
    return (
      <div style={{ textAlign: 'center', padding: '60px' }}>
        <h2 style={{ fontSize: '1.4rem' }}>Opening Wordle diary spread...</h2>
        <p role="alert">{errorMessage}</p>
        <CozyButton onClick={fetchState} disabled={loading}>Retry connection</CozyButton>
      </div>
    );
  }

  const guesses = gameState.guesses || [];
  const currentRowIndex = guesses.length;

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: '1.1fr 0.9fr',
        minHeight: '560px',
        position: 'relative',
      }}
    >
      {/* ============================================================== */}
      {/* LEFT DIARY PAGE: 5x6 Board & Illustrated Keyboard             */}
      {/* ============================================================== */}
      <div
        style={{
          padding: '24px 20px 24px 28px',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'space-between',
          borderRight: '1px solid rgba(96,67,59,0.15)',
          position: 'relative',
        }}
      >
        {/* Header Ribbon & Attempt Indicator */}
        <div
          style={{
            width: '100%',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            marginBottom: '8px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <WordleGameIcon size={26} />
            <h2 style={{ fontSize: '1.5rem', color: 'var(--text-main)', margin: 0 }}>
              WORDLE
            </h2>
          </div>

          <div
            style={{
              fontFamily: 'var(--font-handwritten)',
              fontSize: '1.2rem',
              color: '#6E4E37',
              fontWeight: 700,
            }}
          >
            Attempt {Math.min(guesses.length + 1, 6)} of 6
          </div>
        </div>

        {/* Error notification */}
        {errorMessage && (
          <div
            className="animate-pop"
            style={{
              backgroundColor: '#FFD3D1',
              color: '#701e1a',
              border: '2px solid #a83d38',
              borderRadius: 'var(--radius-pill)',
              padding: '4px 16px',
              fontWeight: 800,
              fontSize: '0.85rem',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              marginBottom: '6px',
            }}
          >
            <AlertCircle size={15} />
            <span>{errorMessage}</span>
          </div>
        )}

        {/* 5x6 Grid */}
        <div
          className={isShaking ? 'animate-shake' : ''}
          style={{
            display: 'grid',
            gridTemplateRows: 'repeat(6, 1fr)',
            gap: '6px',
            padding: '12px',
            backgroundColor: '#F5E4CE',
            border: '3px solid var(--outline-cocoa)',
            borderRadius: '16px',
            boxShadow: '0 4px 0 var(--outline-cocoa)',
          }}
        >
          {Array.from({ length: 6 }).map((_, rowIdx) => {
            const isSubmitted = rowIdx < guesses.length;
            const isCurrent = rowIdx === currentRowIndex;
            const guessRecord = isSubmitted ? guesses[rowIdx] : null;

            return (
              <div
                key={rowIdx}
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(5, 1fr)',
                  gap: '6px',
                }}
              >
                {Array.from({ length: 5 }).map((_, colIdx) => {
                  let letter = '';
                  let feedback: LetterFeedbackType | undefined;

                  if (isSubmitted && guessRecord) {
                    letter = guessRecord.word[colIdx] || '';
                    feedback = guessRecord.feedback[colIdx];
                  } else if (isCurrent) {
                    letter = currentInput[colIdx] || '';
                  }

                  const style = getTileStyles(feedback, letter);

                  return (
                    <div
                      key={colIdx}
                      style={{
                        width: '46px',
                        height: '46px',
                        border: '2.5px solid',
                        borderRadius: '10px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        fontFamily: 'var(--font-heading)',
                        fontSize: '1.6rem',
                        fontWeight: 900,
                        boxShadow: '0 2.5px 0 var(--outline-cocoa)',
                        ...style,
                      }}
                    >
                      {letter}
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>

        {/* Virtual Keyboard */}
        <div
          style={{
            width: '100%',
            maxWidth: '440px',
            display: 'flex',
            flexDirection: 'column',
            gap: '5px',
            backgroundColor: '#FAF2E3',
            padding: '10px 8px',
            borderRadius: '14px',
            border: '2.5px solid var(--outline-cocoa)',
            boxShadow: '0 3px 0 var(--outline-cocoa)',
            marginTop: '8px',
          }}
        >
          {[
            ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
            ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'],
            ['ENTER', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', '⌫'],
          ].map((row, rowIdx) => (
            <div key={rowIdx} style={{ display: 'flex', justifyContent: 'center', gap: '4px' }}>
              {row.map((k) => {
                const status = gameState.keyboard_state[k];
                const { bg, text } = getKeyColor(status);
                const isWide = k === 'ENTER' || k === '⌫';

                return (
                  <button
                    key={k}
                    onClick={() => handleKeyPress(k)}
                    style={{
                      minWidth: isWide ? '48px' : '30px',
                      height: '38px',
                      backgroundColor: bg,
                      color: text,
                      border: '2px solid var(--outline-cocoa)',
                      borderRadius: '6px',
                      boxShadow: '0 2px 0 var(--outline-cocoa)',
                      fontFamily: 'var(--font-heading)',
                      fontSize: isWide ? '0.75rem' : '1.05rem',
                      fontWeight: 800,
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      userSelect: 'none',
                    }}
                  >
                    {k}
                  </button>
                );
              })}
            </div>
          ))}
        </div>
      </div>

      {/* ============================================================== */}
      {/* RIGHT DIARY PAGE: Solver Notebook, Custom Notes & Stickers   */}
      {/* ============================================================== */}
      <div
        style={{
          padding: '24px 28px 24px 22px',
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'space-between',
          position: 'relative',
        }}
      >
        {/* Top Header with Princess Cat Icon & New Word Button */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            borderBottom: '2px dashed var(--outline-cocoa)',
            paddingBottom: '8px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div
              onClick={() => sound.playPurr()}
              style={{ cursor: 'pointer', display: 'flex', alignItems: 'center' }}
              title="Tap Princess to hear her purr!"
            >
              <PrincessCat size={42} />
            </div>
            <div>
              <h3 style={{ fontSize: '1.2rem', margin: 0, color: 'var(--text-main)' }}>
                Little Helper
              </h3>
              <span
                style={{
                  fontFamily: 'var(--font-handwritten)',
                  fontSize: '0.95rem',
                  color: '#6E4E37',
                }}
              >
                Rust Shannon Entropy Solver
              </span>
            </div>
          </div>

          <CozyButton variant="yellow" size="sm" onClick={handleNewGame}>
            <RotateCcw size={14} />
            <span>New Word</span>
          </CozyButton>
        </div>

        {/* Handwritten Scrawly Memo Tape Note */}
        <div style={{ margin: '8px 0' }}>
          <ScrawlyNote
            title="Princess's Strategy Note"
            text="Green means exact letter spot! Yellow means in the word elsewhere. Check candidate entropy bits below!"
            author="Princess Cat"
            tapeColor="mint"
            rotation={1}
            width={240}
          />
        </div>

        {/* Solver Controls */}
        <div
          style={{
            backgroundColor: '#FAF5EE',
            border: '2px solid var(--outline-cocoa)',
            borderRadius: '12px',
            padding: '10px 14px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: '8px',
            flexWrap: 'wrap',
          }}
        >
          <div>
            <div style={{ fontSize: '0.75rem', fontWeight: 800, color: 'var(--text-muted)' }}>
              Remaining Viable Words:
            </div>
            <div style={{ fontSize: '1.25rem', fontWeight: 900, color: 'var(--text-main)' }}>
              {gameState.remaining_candidates_count} words
            </div>
          </div>

          <CozyButton
            variant="yellow"
            size="sm"
            onClick={handleRequestHint}
            disabled={hintLoading || gameState.game_status !== 'InProgress'}
          >
            {hintLoading ? 'Thinking...' : 'Get Solver Hint'}
          </CozyButton>
        </div>

        {/* Solver Suggestions List */}
        {hintData ? (
          <div
            className="animate-pop"
            style={{
              backgroundColor: '#FFFDF5',
              border: '2px solid var(--outline-cocoa)',
              borderRadius: '10px',
              padding: '10px',
              marginTop: '6px',
            }}
          >
            {hintData.is_single_candidate ? (
              <div
                style={{
                  fontFamily: 'var(--font-handwritten)',
                  fontSize: '1.25rem',
                  color: '#2b8a3e',
                  textAlign: 'center',
                  fontWeight: 700,
                }}
              >
                ★ Only 1 word left: {hintData.suggestions[0]?.word}! ★
              </div>
            ) : (
              <div>
                <div style={{ fontSize: '0.8rem', fontWeight: 800, marginBottom: '6px' }}>
                  Best Next Guesses (by Information Gain):
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                  {hintData.suggestions.slice(0, 4).map((s) => (
                    <div
                      key={s.word}
                      style={{
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        fontSize: '0.85rem',
                        padding: '3px 6px',
                        backgroundColor: '#FAF0DE',
                        borderRadius: '6px',
                        border: '1px solid #d4c2ab',
                      }}
                    >
                      <span style={{ fontWeight: 900, letterSpacing: '1px' }}>{s.word}</span>
                      <span
                        style={{
                          fontSize: '0.7rem',
                          padding: '1px 5px',
                          borderRadius: '4px',
                          backgroundColor: s.is_possible_answer ? '#d4edda' : '#e2e3e5',
                          fontWeight: 700,
                        }}
                      >
                        {s.is_possible_answer ? 'Candidate' : 'Explorer'}
                      </span>
                      <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                        +{s.expected_info_bits.toFixed(1)} bits
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        ) : (
          /* Decorative Doodle Space Filler when no hint is open */
          <div
            style={{
              textAlign: 'center',
              padding: '10px',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              gap: '6px',
            }}
          >
            <div style={{ display: 'flex', gap: '8px' }}>
              <PostageStamp label="10¢" illustration="cat" rotation={-6} size={44} />
              <DoodleStars size={24} />
              <PressedFlower size={38} rotation={6} />
            </div>
            <div
              style={{
                fontFamily: 'var(--font-handwritten)',
                fontSize: '1.05rem',
                color: '#8C6E5A',
              }}
            >
              "The solver prunes candidates using information theory!"
            </div>
          </div>
        )}

        {/* Botanical sticker at bottom right corner */}
        <div style={{ alignSelf: 'flex-end', marginTop: '4px' }}>
          <WashiTape color="lavender" width={60} rotation={2} />
        </div>
      </div>

      {/* End-Of-Game Modal */}
      <CozyModal
        isOpen={active && showEndModal}
        onClose={() => setShowEndModal(false)}
        title={gameState.game_status === 'Won' ? 'Splendid Victory!' : 'Game Over!'}
        isVictory={gameState.game_status === 'Won'}
      >
        <div style={{ textAlign: 'center', padding: '10px 0' }}>
          {gameState.game_status === 'Won' ? (
            <div>
              <p style={{ fontSize: '1.1rem', marginBottom: '8px' }}>
                Solved in <strong>{gameState.guesses.length}</strong>{' '}
                {gameState.guesses.length === 1 ? 'attempt' : 'attempts'}!
              </p>
              <div
                style={{
                  display: 'inline-block',
                  backgroundColor: 'var(--highlight-yellow)',
                  border: '2.5px solid var(--outline-cocoa)',
                  borderRadius: 'var(--radius-pill)',
                  padding: '8px 24px',
                  fontSize: '1.6rem',
                  fontWeight: 900,
                  letterSpacing: '3px',
                  marginBottom: '16px',
                }}
              >
                {gameState.secret}
              </div>
            </div>
          ) : (
            <div>
              <p style={{ fontSize: '1.05rem', marginBottom: '8px' }}>
                The secret word was:
              </p>
              <div
                style={{
                  display: 'inline-block',
                  backgroundColor: 'var(--panel-peach)',
                  border: '2.5px solid var(--outline-cocoa)',
                  borderRadius: 'var(--radius-pill)',
                  padding: '8px 24px',
                  fontSize: '1.6rem',
                  fontWeight: 900,
                  letterSpacing: '3px',
                  color: 'var(--text-main)',
                  marginBottom: '16px',
                }}
              >
                {gameState.secret}
              </div>
            </div>
          )}

          <div style={{ display: 'flex', justifyContent: 'center', gap: '12px', marginTop: '16px' }}>
            <CozyButton variant="coral" onClick={handleNewGame}>
              PLAY AGAIN
            </CozyButton>
          </div>
        </div>
      </CozyModal>
    </div>
  );
};
