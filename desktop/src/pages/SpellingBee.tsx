import React, { useCallback, useEffect, useState } from 'react';
import {
  RotateCcw,
  Shuffle,
} from 'lucide-react';
import {
  SpellingBeeHintsResponse,
  SpellingBeeState,
} from '../types/arcade';
import { arcadeApi } from '../services/api';
import { CozyButton } from '../components/CozyButton';
import { CozyModal } from '../components/CozyModal';
import { sound } from '../services/sound';
import { BennyCat } from '../components/stickers/CustomCats';
import {
  BeeGameIcon,
  CrownQueenIcon,
  PawPrintIcon,
} from '../components/stickers/CustomIcons';
import {
  DoodleStars,
  PostageStamp,
  PressedFlower,
  ScrawlyNote,
  WashiTape,
} from '../components/stickers/DiaryStickers';

interface SpellingBeePageProps {
  onGameComplete?: () => void;
}

const HexagonTile: React.FC<{
  letter: string;
  isCenter?: boolean;
  onClick: () => void;
}> = ({ letter, isCenter, onClick }) => (
  <button
    onClick={onClick}
    className={`hex-tile-btn ${isCenter ? 'center' : ''}`}
    title={isCenter ? 'Mandatory Center Letter' : `Letter ${letter}`}
    style={{
      width: '76px',
      height: '88px',
    }}
  >
    <svg
      width="76"
      height="88"
      viewBox="0 0 100 115"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <polygon
        points="50,5 95.5,31.25 95.5,83.75 50,110 4.5,83.75 4.5,31.25"
        fill={isCenter ? '#FFD56D' : '#FFF8ED'}
        stroke="#60433B"
        strokeWidth="6"
        strokeLinejoin="round"
      />
      <text
        x="50"
        y="69"
        textAnchor="middle"
        fontSize={isCenter ? '42' : '38'}
        fontWeight="900"
        fontFamily="var(--font-heading)"
        fill={isCenter ? '#4a3000' : 'var(--text-main)'}
      >
        {letter}
      </text>
    </svg>
  </button>
);

export const SpellingBeePage: React.FC<SpellingBeePageProps> = ({ onGameComplete }) => {
  const [gameState, setGameState] = useState<SpellingBeeState | null>(null);
  const [currentInput, setCurrentInput] = useState('');
  const [feedbackMessage, setFeedbackMessage] = useState<{ text: string; isError: boolean } | null>(
    null
  );
  const [isShaking, setIsShaking] = useState(false);
  const [loading, setLoading] = useState(false);

  // Hints drawer
  const [showHints, setShowHints] = useState(false);
  const [hintsData, setHintsData] = useState<SpellingBeeHintsResponse | null>(null);
  const [hintsLoading, setHintsLoading] = useState(false);

  // Celebrations
  const [showQueenModal, setShowQueenModal] = useState(false);
  const [showPangramToast, setShowPangramToast] = useState(false);

  // Fetch or start game
  const fetchState = useCallback(async () => {
    try {
      setLoading(true);
      let state = await arcadeApi.getSpellingBeeState();
      if (!state.main_letter) {
        state = await arcadeApi.startSpellingBee();
      }
      setGameState(state);
      if (state.is_queen_bee) {
        setShowQueenModal(true);
      }
    } catch (err: any) {
      setFeedbackMessage({ text: err.message || 'Error connecting to Spelling Bee', isError: true });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchState();
  }, [fetchState]);

  // Restart puzzle
  const handleNewPuzzle = async () => {
    try {
      setLoading(true);
      setCurrentInput('');
      setFeedbackMessage(null);
      setHintsData(null);
      const state = await arcadeApi.startSpellingBee();
      setGameState(state);
      sound.playSuccess();
    } catch (err: any) {
      setFeedbackMessage({ text: err.message || 'Could not start new puzzle', isError: true });
    } finally {
      setLoading(false);
    }
  };

  // Shuffle outer letters
  const handleShuffle = async () => {
    try {
      sound.playClick();
      const res = await arcadeApi.shuffleSpellingBee();
      if (gameState) {
        setGameState({ ...gameState, outer_letters: res.outer_letters });
      }
    } catch (e) {
      // fallback
    }
  };

  // Submit word
  const handleSubmitWord = async () => {
    if (!gameState || !currentInput.trim()) return;

    try {
      const res = await arcadeApi.submitSpellingBeeWord(currentInput);
      if (res.result === 'Accepted') {
        if (res.is_pangram) {
          sound.playPangram();
          setShowPangramToast(true);
          setTimeout(() => setShowPangramToast(false), 3500);
        } else {
          sound.playSuccess();
        }

        setFeedbackMessage({ text: res.message, isError: false });
        if (res.state) {
          setGameState(res.state);
          if (res.state.is_queen_bee) {
            setShowQueenModal(true);
            onGameComplete?.();
          }
        }
      } else {
        sound.playInvalid();
        setIsShaking(true);
        setTimeout(() => setIsShaking(false), 450);
        setFeedbackMessage({ text: res.message, isError: true });
      }
      setCurrentInput('');
      setTimeout(() => setFeedbackMessage(null), 3000);
    } catch (err: any) {
      sound.playInvalid();
      setFeedbackMessage({ text: err.message || 'Submission failed', isError: true });
      setCurrentInput('');
    }
  };

  const handleLetterClick = (letter: string) => {
    sound.playClick();
    setCurrentInput((prev) => prev + letter);
  };

  const handleBackspace = () => {
    sound.playClick();
    setCurrentInput((prev) => prev.slice(0, -1));
  };

  // Keyboard navigation
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.altKey || e.metaKey) return;
      if (e.key === 'Enter') {
        handleSubmitWord();
      } else if (e.key === 'Backspace' || e.key === 'Delete') {
        handleBackspace();
      } else if (/^[a-zA-Z]$/.test(e.key)) {
        handleLetterClick(e.key.toUpperCase());
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [currentInput, gameState]);

  // Load Hints
  const handleToggleHints = async () => {
    if (!showHints && !hintsData) {
      try {
        setHintsLoading(true);
        const data = await arcadeApi.getSpellingBeeHints();
        setHintsData(data);
      } catch (err) {
        // error loading hints
      } finally {
        setHintsLoading(false);
      }
    }
    setShowHints(!showHints);
  };

  if (!gameState) {
    return (
      <div style={{ textAlign: 'center', padding: '60px' }}>
        <h2 style={{ fontSize: '1.4rem' }}>Opening Spelling Bee diary spread...</h2>
      </div>
    );
  }

  const progressPercent =
    gameState.max_possible_score > 0
      ? Math.min((gameState.score / gameState.max_possible_score) * 100, 100)
      : 0;

  const outer = gameState.outer_letters;

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: '1.05fr 0.95fr',
        minHeight: '560px',
        position: 'relative',
      }}
    >
      {/* ============================================================== */}
      {/* LEFT DIARY PAGE: Honeycomb, Input, Action Buttons             */}
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
        {/* Header */}
        <div
          style={{
            width: '100%',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <BeeGameIcon size={28} />
            <h2 style={{ fontSize: '1.5rem', color: 'var(--text-main)', margin: 0 }}>
              SPELLING BEE
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
            Center: [ {gameState.main_letter} ]
          </div>
        </div>

        {/* Word Input Display Box */}
        <div
          className={isShaking ? 'animate-shake' : ''}
          style={{
            minWidth: '220px',
            height: '46px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            borderBottom: '3px solid var(--outline-cocoa)',
            fontFamily: 'var(--font-heading)',
            fontSize: '1.9rem',
            fontWeight: 900,
            letterSpacing: '3px',
            color: 'var(--text-main)',
            margin: '4px 0',
          }}
        >
          {currentInput || (
            <span style={{ opacity: 0.35, fontSize: '1.3rem' }}>Click or type...</span>
          )}
        </div>

        {/* Feedback Message */}
        {feedbackMessage && (
          <div
            className="animate-pop"
            style={{
              padding: '4px 14px',
              borderRadius: 'var(--radius-pill)',
              border: '2px solid var(--outline-cocoa)',
              backgroundColor: feedbackMessage.isError ? '#FFD3D1' : '#D4EDDA',
              color: feedbackMessage.isError ? '#721C24' : '#155724',
              fontWeight: 800,
              fontSize: '0.85rem',
            }}
          >
            {feedbackMessage.text}
          </div>
        )}

        {/* HONEYCOMB ARRANGEMENT: 7 Hexagonal Letter Tiles */}
        <div className="honeycomb-container" style={{ margin: '8px 0' }}>
          {/* Row 1 (2 outer letters) */}
          <div className="honeycomb-row">
            <HexagonTile letter={outer[0] || ''} onClick={() => handleLetterClick(outer[0] || '')} />
            <HexagonTile letter={outer[1] || ''} onClick={() => handleLetterClick(outer[1] || '')} />
          </div>

          {/* Row 2 (1 outer, 1 CENTER, 1 outer) */}
          <div className="honeycomb-row">
            <HexagonTile letter={outer[2] || ''} onClick={() => handleLetterClick(outer[2] || '')} />
            <HexagonTile
              letter={gameState.main_letter}
              isCenter
              onClick={() => handleLetterClick(gameState.main_letter)}
            />
            <HexagonTile letter={outer[3] || ''} onClick={() => handleLetterClick(outer[3] || '')} />
          </div>

          {/* Row 3 (2 outer letters) */}
          <div className="honeycomb-row">
            <HexagonTile letter={outer[4] || ''} onClick={() => handleLetterClick(outer[4] || '')} />
            <HexagonTile letter={outer[5] || ''} onClick={() => handleLetterClick(outer[5] || '')} />
          </div>
        </div>

        {/* Action Buttons: Delete, Shuffle, Enter */}
        <div style={{ display: 'flex', gap: '10px' }}>
          <CozyButton variant="cream" size="sm" onClick={handleBackspace}>
            Delete
          </CozyButton>

          <CozyButton variant="yellow" size="sm" onClick={handleShuffle}>
            <Shuffle size={14} />
            <span>Shuffle</span>
          </CozyButton>

          <CozyButton variant="coral" size="sm" onClick={handleSubmitWord}>
            Enter
          </CozyButton>
        </div>

        {/* Scrawly Handwritten Reminder at Bottom of Left Page */}
        <div
          style={{
            fontFamily: 'var(--font-handwritten)',
            fontSize: '1.05rem',
            color: '#7A5E44',
            textAlign: 'center',
            marginTop: '8px',
          }}
        >
          ~ Letters can be used more than once! ~
        </div>
      </div>

      {/* ============================================================== */}
      {/* RIGHT DIARY PAGE: Progress, Found Words List & Bee Hints       */}
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
        {/* Top Header with Progress & New Puzzle */}
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
            <BennyCat size={44} />
            <div>
              <div style={{ fontWeight: 900, fontSize: '1.1rem', color: 'var(--text-main)' }}>
                Rank: {gameState.rank}
              </div>
              <div style={{ fontSize: '0.8rem', fontWeight: 800, color: 'var(--text-muted)' }}>
                Score: {gameState.score} / {gameState.max_possible_score} pts
              </div>
            </div>
          </div>

          <CozyButton variant="yellow" size="sm" onClick={handleNewPuzzle}>
            <RotateCcw size={14} />
            <span>New Bee</span>
          </CozyButton>
        </div>

        {/* Queen Bee Progress Bar */}
        <div style={{ margin: '8px 0' }}>
          <div
            style={{
              height: '14px',
              backgroundColor: '#F5E4CE',
              borderRadius: 'var(--radius-pill)',
              border: '2px solid var(--outline-cocoa)',
              overflow: 'hidden',
            }}
          >
            <div
              style={{
                height: '100%',
                width: `${progressPercent}%`,
                backgroundColor: '#FFD56D',
                borderRadius: 'var(--radius-pill)',
                transition: 'width 0.4s ease',
              }}
            />
          </div>
        </div>

        {/* Discovered Words List (3 Columns on Notebook Paper) */}
        <div
          style={{
            backgroundColor: '#FAF5EE',
            border: '2px solid var(--outline-cocoa)',
            borderRadius: '12px',
            padding: '12px',
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-between',
            maxHeight: '260px',
          }}
        >
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              borderBottom: '1px dashed #d8c3a5',
              paddingBottom: '4px',
              marginBottom: '6px',
            }}
          >
            <span style={{ fontWeight: 800, fontSize: '0.9rem' }}>
              Words Found ({gameState.words_found.length} / {gameState.total_solutions})
            </span>
            <span
              style={{
                fontFamily: 'var(--font-handwritten)',
                fontSize: '1rem',
                color: '#b8860b',
                fontWeight: 700,
              }}
            >
              Pangrams: {gameState.pangrams_found}
            </span>
          </div>

          {gameState.words_found.length === 0 ? (
            <div
              style={{
                textAlign: 'center',
                padding: '24px 8px',
                fontFamily: 'var(--font-handwritten)',
                fontSize: '1.2rem',
                color: '#8C6E5A',
              }}
            >
              "No words found yet! Try building words around [{gameState.main_letter}]"
            </div>
          ) : (
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(3, 1fr)',
                gap: '4px 8px',
                overflowY: 'auto',
                maxHeight: '170px',
                paddingRight: '4px',
              }}
            >
              {gameState.words_found.map((w) => (
                <div
                  key={w}
                  style={{
                    fontSize: '0.85rem',
                    fontWeight: 800,
                    color: 'var(--text-main)',
                    padding: '2px 6px',
                    backgroundColor: '#FAF0DE',
                    border: '1px solid #d8c3a5',
                    borderRadius: '4px',
                    textAlign: 'center',
                  }}
                >
                  {w}
                </div>
              ))}
            </div>
          )}

          {/* Bee Hints Toggle */}
          <div style={{ textAlign: 'center', paddingTop: '6px', borderTop: '1px dashed #d8c3a5' }}>
            <button
              onClick={handleToggleHints}
              style={{
                fontFamily: 'var(--font-handwritten)',
                fontSize: '1.05rem',
                color: '#554120',
                fontWeight: 700,
                cursor: 'pointer',
                textDecoration: 'underline',
              }}
            >
              {showHints ? 'Hide Solver Hints ▲' : 'Open Bee Solver Hints ▼'}
            </button>
          </div>
        </div>

        {/* Bee Hints Drawer */}
        {showHints && hintsData && (
          <div
            className="animate-pop"
            style={{
              backgroundColor: '#FFFDF5',
              border: '2px solid var(--outline-cocoa)',
              borderRadius: '10px',
              padding: '8px 12px',
              marginTop: '6px',
              fontSize: '0.8rem',
            }}
          >
            <div style={{ display: 'flex', gap: '10px', fontWeight: 800, marginBottom: '4px' }}>
              <span>Total Words: {hintsData.total_words}</span>
              <span>Points: {hintsData.total_points}</span>
              <span>Pangrams: {hintsData.pangram_count}</span>
            </div>
            <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
              {Object.entries(hintsData.length_counts).map(([len, count]) => (
                <span
                  key={len}
                  style={{
                    backgroundColor: '#FAF0DE',
                    border: '1px solid #d8c3a5',
                    borderRadius: '3px',
                    padding: '1px 5px',
                    fontWeight: 700,
                  }}
                >
                  {len}L: {count}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* Pangram Memo Tape Note filling bottom blank space */}
        <div style={{ marginTop: '8px' }}>
          <ScrawlyNote
            text="Pangrams use all 7 letters at least once and award +7 bonus points!"
            author="Benny"
            tapeColor="yellow"
            rotation={1.5}
            width={240}
          />
        </div>
      </div>

      {/* Queen Bee Celebration Modal */}
      <CozyModal
        isOpen={showQueenModal}
        onClose={() => setShowQueenModal(false)}
        title="QUEEN BEE ACHIEVED!"
        isVictory={true}
      >
        <div style={{ textAlign: 'center', padding: '16px 0' }}>
          <p style={{ fontSize: '1.15rem', marginBottom: '12px' }}>
            Incredible! You discovered every single valid word in this puzzle!
          </p>
          <div
            style={{
              fontSize: '2rem',
              fontWeight: 900,
              color: '#b8860b',
              marginBottom: '16px',
            }}
          >
            Total Score: {gameState.score} Points
          </div>
          <CozyButton variant="yellow" onClick={handleNewPuzzle}>
            PLAY ANOTHER PUZZLE
          </CozyButton>
        </div>
      </CozyModal>
    </div>
  );
};
