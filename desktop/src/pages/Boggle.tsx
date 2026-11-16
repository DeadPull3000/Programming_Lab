import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  Check,
  Clock,
  RotateCcw,
  StopCircle,
  X,
} from 'lucide-react';
import { BoggleEndResponse, BoggleState } from '../types/arcade';
import { arcadeApi } from '../services/api';
import { CozyButton } from '../components/CozyButton';
import { CozyModal } from '../components/CozyModal';
import { sound } from '../services/sound';
import { SnowyCat } from '../components/stickers/CustomCats';
import {
  BoggleGameIcon,
  PawPrintIcon,
} from '../components/stickers/CustomIcons';
import {
  DoodleStars,
  PostageStamp,
  PressedFern,
  ScrawlyNote,
  WashiTape,
} from '../components/stickers/DiaryStickers';

interface BogglePageProps {
  onGameComplete?: () => void;
  active?: boolean;
}

interface TileCoord {
  row: number;
  col: number;
  letter: string;
}

export const BogglePage: React.FC<BogglePageProps> = ({ onGameComplete, active = true }) => {
  const initialized = useRef(false);
  const pending = useRef(false);
  const [gameState, setGameState] = useState<BoggleState | null>(null);
  const [selectedBoardSize, setSelectedBoardSize] = useState<number>(4);
  const [currentInput, setCurrentInput] = useState('');
  const [selectedPath, setSelectedPath] = useState<TileCoord[]>([]);
  const traceMoved = useRef(false);
  const isTracingRef = useRef<boolean>(false);
  const selectedPathRef = useRef<TileCoord[]>([]);
  const [feedback, setFeedback] = useState<{ text: string; isError: boolean } | null>(null);
  const [isShaking, setIsShaking] = useState(false);
  const [loading, setLoading] = useState(false);

  // Timer: 180 seconds round timer
  const [timeLeft, setTimeLeft] = useState<number>(180);
  const ending = useRef(false);

  // End of round results
  const [endResults, setEndResults] = useState<BoggleEndResponse | null>(null);
  const [showEndModal, setShowEndModal] = useState(false);

  // Fetch or start game
  const fetchState = useCallback(async () => {
    try {
      setLoading(true);
      let state = await arcadeApi.getBoggleState();
      if (state.cells.length === 0) {
        state = await arcadeApi.startBoggle(4);
      }
      setGameState(state);
      setSelectedBoardSize(state.size);
    } catch (err: any) {
      setFeedback({ text: err.message || 'Error connecting to Boggle engine', isError: true });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (initialized.current) return;
    initialized.current = true;
    void fetchState();
  }, [fetchState]);

  // Derive time from the server deadline so refreshes and background tabs cannot reset it.
  useEffect(() => {
    if (!gameState?.is_active || !gameState.ends_at_unix_ms) { setTimeLeft(0); return; }
    const deadline = gameState.ends_at_unix_ms;
    const tick = () => {
      const seconds = Math.max(0, Math.ceil((deadline - Date.now()) / 1000));
      setTimeLeft(seconds);
      if (seconds === 0 && !ending.current && !pending.current) void handleEndRound();
    };
    tick();
    const interval = window.setInterval(tick, 250);
    return () => window.clearInterval(interval);
  }, [gameState?.is_active, gameState?.ends_at_unix_ms]);

  // Start new round
  const handleStartNewRound = async (size = selectedBoardSize) => {
    if (pending.current) return;
    pending.current = true;
    try {
      setLoading(true);
      setCurrentInput('');
      setSelectedPath([]);
      setFeedback(null);
      setEndResults(null);
      setShowEndModal(false);
      setTimeLeft(180);
      selectedPathRef.current = [];
      isTracingRef.current = false;

      const state = await arcadeApi.startBoggle(size);
      setGameState(state);
      setSelectedBoardSize(size);
      sound.playSuccess();
    } catch (err: any) {
      setFeedback({ text: err.message || 'Could not start new Boggle board', isError: true });
    } finally {
      pending.current = false;
      setLoading(false);
    }
  };

  // Submit word from input or selected path
  const handleSubmitWord = async (overrideWord?: string) => {
    const wordToSubmit = (
      overrideWord ||
      currentInput.trim() ||
      selectedPathRef.current.map((p) => p.letter).join('')
    ).toUpperCase();

    if (!wordToSubmit || !gameState?.is_active) return;

    if (pending.current) return;
    pending.current = true;
    try {
      const res = await arcadeApi.submitBoggleWord(wordToSubmit);
      if (res.result === 'Accepted') {
        sound.playSuccess();
        setFeedback({ text: res.message, isError: false });
        if (res.state) {
          setGameState(res.state);
        }
      } else {
        sound.playInvalid();
        setIsShaking(true);
        setTimeout(() => setIsShaking(false), 450);
        setFeedback({ text: res.message, isError: true });
      }
      setCurrentInput('');
      selectedPathRef.current = [];
      setSelectedPath([]);
      setTimeout(() => setFeedback(null), 3000);
    } catch (err: any) {
      sound.playInvalid();
      setFeedback({ text: err.message || 'Submission error', isError: true });
      setCurrentInput('');
      selectedPathRef.current = [];
      setSelectedPath([]);
    } finally {
      pending.current = false;
    }
  };

  // End round
  const handleEndRound = async () => {
    if (!gameState || !gameState.is_active || ending.current || pending.current) return;
    ending.current = true;
    try {
      const res = await arcadeApi.endBoggle();
      setEndResults(res);
      setGameState(res.state);
      setShowEndModal(true);
      sound.playSuccess();
      onGameComplete?.();
    } catch (err: any) {
      setFeedback({ text: err.message || 'Failed to end round', isError: true });
    } finally {
      ending.current = false;
    }
  };

  // A tap extends the path; dragging additionally submits it on release.
  const handleTraceStart = (row: number, col: number, letter: string) => {
    if (!gameState?.is_active || pending.current) return;
    traceMoved.current = false;
    isTracingRef.current = true;
    handleTileClick(row, col, letter);
  };

  // Move pointer over adjacent tiles to trace the word path
  const handleTraceEnter = (row: number, col: number, letter: string) => {
    if (!isTracingRef.current || !gameState?.is_active) return;
    const path = selectedPathRef.current;
    if (path.length === 0) return;

    // Backtracking: if dragging back to previous tile, undo the tip
    if (
      path.length >= 2 &&
      path[path.length - 2].row === row &&
      path[path.length - 2].col === col
    ) {
      traceMoved.current = true;
      sound.playClick();
      const newPath = path.slice(0, -1);
      selectedPathRef.current = newPath;
      setSelectedPath(newPath);
      return;
    }

    // Prohibit reusing the same tile in a word
    if (path.some((p) => p.row === row && p.col === col)) {
      return;
    }

    // 8-direction adjacency check
    const last = path[path.length - 1];
    const dRow = Math.abs(last.row - row);
    const dCol = Math.abs(last.col - col);
    if (dRow <= 1 && dCol <= 1 && (dRow + dCol > 0)) {
      traceMoved.current = true;
      const newPath = [...path, { row, col, letter }];
      selectedPathRef.current = newPath;
      setSelectedPath(newPath);
      sound.playPop(440 + newPath.length * 40);
    }
  };

  // Finish pointer trace on mouse release or touch end
  const handleTraceEnd = () => {
    if (!isTracingRef.current) return;
    isTracingRef.current = false;

    // If traced word is 3+ letters, auto-submit directly to the engine
    if (traceMoved.current && selectedPathRef.current.length >= 3) {
      const word = selectedPathRef.current.map((p) => p.letter).join('');
      handleSubmitWord(word);
    }
  };

  // Read the latest handler instead of retaining the first render's empty game state.
  const traceEndHandler = useRef(handleTraceEnd);
  traceEndHandler.current = handleTraceEnd;
  useEffect(() => {
    const onEnd = () => traceEndHandler.current();
    const onCancel = () => { isTracingRef.current = false; };
    window.addEventListener('pointerup', onEnd);
    window.addEventListener('pointercancel', onCancel);
    return () => {
      window.removeEventListener('pointerup', onEnd);
      window.removeEventListener('pointercancel', onCancel);
    };
  }, []);

  // Pointer move handler on the grid container for smooth continuous dragging
  const handleContainerPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!isTracingRef.current) return;
    const target = document.elementFromPoint(e.clientX, e.clientY);
    const tileBtn = target?.closest('[data-boggle-tile]');
    if (tileBtn) {
      const r = parseInt(tileBtn.getAttribute('data-row') || '-1', 10);
      const c = parseInt(tileBtn.getAttribute('data-col') || '-1', 10);
      const l = tileBtn.getAttribute('data-letter') || '';
      if (r >= 0 && c >= 0 && l) {
        handleTraceEnter(r, c, l);
      }
    }
  };

  // Discrete click interaction (for tapping letter by letter without dragging)
  const handleTileClick = (row: number, col: number, letter: string) => {
    if (!gameState?.is_active || pending.current) return;
    setCurrentInput('');

    if (
      selectedPath.length > 0 &&
      selectedPath[selectedPath.length - 1].row === row &&
      selectedPath[selectedPath.length - 1].col === col
    ) {
      sound.playClick();
      const newPath = selectedPath.slice(0, -1);
      selectedPathRef.current = newPath;
      setSelectedPath(newPath);
      return;
    }

    if (selectedPath.some((p) => p.row === row && p.col === col)) {
      sound.playInvalid();
      setFeedback({ text: 'Cannot reuse the same tile in a word!', isError: true });
      setTimeout(() => setFeedback(null), 2000);
      return;
    }

    if (selectedPath.length > 0) {
      const last = selectedPath[selectedPath.length - 1];
      const dRow = Math.abs(last.row - row);
      const dCol = Math.abs(last.col - col);
      if (dRow > 1 || dCol > 1) {
        sound.playInvalid();
        setFeedback({ text: 'Must be adjacent in 8 directions!', isError: true });
        setTimeout(() => setFeedback(null), 2000);
        return;
      }
    }

    sound.playPop(440 + selectedPath.length * 50);
    const newPath = [...selectedPath, { row, col, letter }];
    selectedPathRef.current = newPath;
    setSelectedPath(newPath);
  };

  // Keyboard support for typing words directly
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (!active || loading || pending.current || e.ctrlKey || e.altKey || e.metaKey || e.isComposing) return;
      if (e.target instanceof HTMLElement && e.target.closest('input, textarea, select, [contenteditable="true"], [role="dialog"]')) return;
      if (document.querySelector('[role="dialog"]')) return;
      if (e.target instanceof HTMLElement && e.target.closest('button') && (e.key === 'Enter' || e.key === ' ')) return;
      if (e.key === 'Enter' || e.key === 'Backspace' || e.key === 'Delete' || /^[a-zA-Z]$/.test(e.key)) e.preventDefault();
      if (e.key === 'Enter') {
        handleSubmitWord();
      } else if (e.key === 'Backspace' || e.key === 'Delete') {
        sound.playClick();
        if (currentInput.length > 0) {
          setCurrentInput((prev) => prev.slice(0, -1));
        } else if (selectedPath.length > 0) {
          const newPath = selectedPath.slice(0, -1);
          selectedPathRef.current = newPath;
          setSelectedPath(newPath);
        }
      } else if (/^[a-zA-Z]$/.test(e.key)) {
        sound.playClick();
        setCurrentInput((prev) => prev + e.key.toUpperCase());
        selectedPathRef.current = [];
        setSelectedPath([]);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [currentInput, selectedPath, gameState, active, loading]);

  if (!gameState) {
    return (
      <div style={{ textAlign: 'center', padding: '60px' }}>
        <h2 style={{ fontSize: '1.4rem' }}>Opening Boggle tabletop spread...</h2>
        <p role="alert">{feedback?.text}</p>
        <CozyButton onClick={fetchState} disabled={loading}>Retry connection</CozyButton>
      </div>
    );
  }

  const formatTimer = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const currentDisplayWord =
    currentInput || selectedPath.map((p) => p.letter).join('');

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
      {/* LEFT DIARY PAGE: Boggle Board, Size Picker, Actions            */}
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
        {/* Header Ribbon & Size Selector */}
        <div
          style={{
            width: '100%',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <BoggleGameIcon size={26} />
            <h2 style={{ fontSize: '1.5rem', color: 'var(--text-main)', margin: 0 }}>
              BOGGLE
            </h2>
          </div>

          {/* Grid size picker buttons */}
          <div style={{ display: 'flex', gap: '6px' }}>
            {[4, 5, 6].map((size) => (
              <button
                key={size}
                disabled={loading}
                onClick={() => handleStartNewRound(size)}
                style={{
                  padding: '3px 10px',
                  borderRadius: 'var(--radius-pill)',
                  border: '2px solid var(--outline-cocoa)',
                  backgroundColor:
                    selectedBoardSize === size ? '#FFD56D' : '#FAF0DE',
                  fontWeight: 900,
                  fontSize: '0.85rem',
                  cursor: 'pointer',
                  fontFamily: 'var(--font-heading)',
                }}
              >
                {size}×{size}
              </button>
            ))}
          </div>
        </div>

        {/* Current Word Display Box */}
        <div
          className={isShaking ? 'animate-shake' : ''}
          style={{
            minWidth: '220px',
            height: '42px',
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
          {currentDisplayWord || (
            <span style={{ opacity: 0.35, fontSize: '1.2rem' }}>Trace tiles...</span>
          )}
        </div>

        {/* Feedback message */}
        {feedback && (
          <div
            className="animate-pop"
            style={{
              padding: '3px 12px',
              borderRadius: 'var(--radius-pill)',
              border: '1.5px solid var(--outline-cocoa)',
              backgroundColor: feedback.isError ? '#FFD3D1' : '#D4EDDA',
              color: feedback.isError ? '#721C24' : '#155724',
              fontWeight: 800,
              fontSize: '0.82rem',
            }}
          >
            {feedback.text}
          </div>
        )}

        {/* TABLETOP WOODEN BOGGLE GRID WITH INTERACTIVE TRACING */}
        {(() => {
          const tileSize = gameState.size === 6 ? 42 : gameState.size === 5 ? 48 : 56;
          const gap = gameState.size === 6 ? 5 : 8;
          const padding = 12;

          const getTileCenter = (r: number, c: number) => ({
            x: padding + c * (tileSize + gap) + tileSize / 2,
            y: padding + r * (tileSize + gap) + tileSize / 2,
          });

          const pathPoints = selectedPath
            .map((p) => {
              const pt = getTileCenter(p.row, p.col);
              return `${pt.x},${pt.y}`;
            })
            .join(' ');

          return (
            <div
              onPointerMove={handleContainerPointerMove}
              style={{
                position: 'relative',
                display: 'grid',
                gridTemplateColumns: `repeat(${gameState.size}, 1fr)`,
                gap: `${gap}px`,
                padding: `${padding}px`,
                backgroundColor: '#8B5E3C', // Warm wooden game box
                border: '3.5px solid #60433B',
                borderRadius: '16px',
                boxShadow: '0 5px 0 #60433B',
                touchAction: 'none',
                userSelect: 'none',
              }}
            >
              {/* Visual Polyline Ribbon connecting traced letters */}
              <svg
                style={{
                  position: 'absolute',
                  inset: 0,
                  width: '100%',
                  height: '100%',
                  pointerEvents: 'none',
                  zIndex: 8,
                }}
              >
                {selectedPath.length > 1 && (
                  <polyline
                    points={pathPoints}
                    fill="none"
                    stroke="#F48E88"
                    strokeWidth={gameState.size === 6 ? '6' : '7'}
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeOpacity="0.85"
                    style={{ filter: 'drop-shadow(0 2px 4px rgba(96, 67, 59, 0.4))' }}
                  />
                )}
                {selectedPath.map((p, idx) => {
                  const pt = getTileCenter(p.row, p.col);
                  const isHead = idx === selectedPath.length - 1;
                  return (
                    <circle
                      key={idx}
                      cx={pt.x}
                      cy={pt.y}
                      r={isHead ? 5 : 3.5}
                      fill={isHead ? '#FFD56D' : '#FFF0D9'}
                      stroke="#60433B"
                      strokeWidth="1.8"
                    />
                  );
                })}
              </svg>

              {gameState.grid.map((row, rIdx) =>
                row.map((letter, cIdx) => {
                  const pathIndex = selectedPath.findIndex(
                    (p) => p.row === rIdx && p.col === cIdx
                  );
                  const isSelected = pathIndex !== -1;
                  const isLatest =
                    selectedPath.length > 0 &&
                    selectedPath[selectedPath.length - 1].row === rIdx &&
                    selectedPath[selectedPath.length - 1].col === cIdx;

                  return (
                    <button
                      key={`${rIdx}-${cIdx}`}
                      data-boggle-tile="true"
                      data-row={rIdx}
                      data-col={cIdx}
                      data-letter={letter}
                      onPointerDown={(e) => {
                        e.preventDefault();
                        e.currentTarget.setPointerCapture(e.pointerId);
                        handleTraceStart(rIdx, cIdx, letter);
                      }}
                      onPointerEnter={() => handleTraceEnter(rIdx, cIdx, letter)}
                      onClick={(e) => { if (e.detail === 0) handleTileClick(rIdx, cIdx, letter); }}
                      style={{
                        width: `${tileSize}px`,
                        height: `${tileSize}px`,
                        backgroundColor: isLatest
                          ? '#FFD56D'
                          : isSelected
                          ? '#F5D5B9'
                          : '#FFF0D9',
                        border: isSelected
                          ? '2.5px solid var(--outline-cocoa)'
                          : '2px solid var(--outline-cocoa)',
                        borderRadius: '10px',
                        boxShadow: isSelected
                          ? '0 1px 0 var(--outline-cocoa)'
                          : '0 3.5px 0 var(--outline-cocoa)',
                        transform: isSelected ? 'translateY(2px)' : 'none',
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        justifyContent: 'center',
                        fontFamily: 'var(--font-heading)',
                        fontSize: gameState.size === 6 ? '1.25rem' : '1.5rem',
                        fontWeight: 900,
                        color: 'var(--text-main)',
                        cursor: 'pointer',
                        position: 'relative',
                        userSelect: 'none',
                        zIndex: 9,
                        touchAction: 'none',
                      }}
                    >
                      <span>{letter}</span>
                      {isSelected && (
                        <span
                          style={{
                            position: 'absolute',
                            top: '2px',
                            right: '4px',
                            fontSize: '0.65rem',
                            fontWeight: 900,
                            backgroundColor: 'var(--outline-cocoa)',
                            color: '#fff',
                            width: '14px',
                            height: '14px',
                            borderRadius: '50%',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                          }}
                        >
                          {pathIndex + 1}
                        </span>
                      )}
                    </button>
                  );
                })
              )}
            </div>
          );
        })()}

        {/* Action Buttons: Clear Path & Submit */}
        <div style={{ display: 'flex', gap: '10px', marginTop: '6px' }}>
          <CozyButton
            variant="cream"
            size="sm"
            onClick={() => {
              sound.playClick();
              selectedPathRef.current = [];
              setSelectedPath([]);
              setCurrentInput('');
            }}
            disabled={!currentDisplayWord || !gameState.is_active}
          >
            Clear
          </CozyButton>

          <CozyButton
            variant="coral"
            size="sm"
            onClick={() => handleSubmitWord()}
            disabled={!currentDisplayWord || !gameState.is_active}
          >
            Submit Word
          </CozyButton>
        </div>

        {/* Handwritten note at bottom of Left Page */}
        <div
          style={{
            fontFamily: 'var(--font-handwritten)',
            fontSize: '1.05rem',
            color: '#7A5E44',
            textAlign: 'center',
            marginTop: '4px',
          }}
        >
          ~ Drag across adjacent letters to trace words! ~
        </div>
      </div>

      {/* ============================================================== */}
      {/* RIGHT DIARY PAGE: Timer, Score, Accepted Words, Snowy Cat     */}
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
        {/* Top Header: Timer, Score, Finish Round */}
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
              title="Tap Snowy to hear her purr!"
            >
              <SnowyCat size={44} />
            </div>
            <div>
              <div style={{ fontWeight: 900, fontSize: '1.1rem', color: 'var(--text-main)' }}>
                Score: {gameState.score} pts
              </div>
              <div style={{ fontSize: '0.8rem', fontWeight: 800, color: 'var(--text-muted)' }}>
                Found: {gameState.accepted_words.length} / {gameState.total_possible_words}
              </div>
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <div
              style={{
                fontFamily: 'monospace',
                fontSize: '1.2rem',
                fontWeight: 900,
                color: timeLeft <= 30 ? '#c92a2a' : 'var(--text-main)',
                backgroundColor: '#FAF0DE',
                padding: '3px 8px',
                borderRadius: '6px',
                border: '1.5px solid var(--outline-cocoa)',
              }}
            >
              {formatTimer(timeLeft)}
            </div>

            {gameState.is_active && (
              <CozyButton variant="coral" size="sm" onClick={handleEndRound}>
                End
              </CozyButton>
            )}
          </div>
        </div>

        {/* Accepted Words Notebook Section */}
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
            margin: '8px 0',
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
            <span style={{ fontWeight: 800, fontSize: '0.85rem' }}>
              Accepted Words ({gameState.accepted_words.length})
            </span>
            <span style={{ fontSize: '0.75rem', fontWeight: 700, color: 'var(--text-muted)' }}>
              Max Possible: {gameState.max_possible_score} pts
            </span>
          </div>

          {gameState.accepted_words.length === 0 ? (
            <div
              style={{
                textAlign: 'center',
                padding: '28px 8px',
                fontFamily: 'var(--font-handwritten)',
                fontSize: '1.2rem',
                color: '#8C6E5A',
              }}
            >
              "Trace a path of tiles on the grid or type with your keyboard!"
            </div>
          ) : (
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(3, 1fr)',
                gap: '4px 6px',
                overflowY: 'auto',
                maxHeight: '180px',
                paddingRight: '4px',
              }}
            >
              {gameState.accepted_words.map((w) => (
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
        </div>

        {/* Snowy's Handwritten Scrapbook Note filling bottom blank space */}
        <div>
          <ScrawlyNote
            title="Snowy's Boggle Tip"
            text="Look for suffixes like -ED, -ING, -ER, and -S to quickly multiply your score!"
            author="Snowy Cat"
            tapeColor="mint"
            rotation={-1}
            width={240}
          />
        </div>
      </div>

      {/* End of Round Modal */}
      <CozyModal
        isOpen={active && showEndModal}
        onClose={() => setShowEndModal(false)}
        title="ROUND FINISHED!"
        isVictory={endResults ? endResults.score > 0 : false}
      >
        {endResults && (
          <div style={{ textAlign: 'center', padding: '10px 0' }}>
            <div
              style={{
                fontSize: '2rem',
                fontWeight: 900,
                color: 'var(--text-main)',
                marginBottom: '8px',
              }}
            >
              Final Score: {endResults.score} Points
            </div>
            <p style={{ fontSize: '1rem', color: 'var(--text-muted)', marginBottom: '14px' }}>
              Found {endResults.accepted_count} of {endResults.total_possible_words} words (Max:{' '}
              {endResults.max_possible_score} pts)
            </p>

            {endResults.missed_sample.length > 0 && (
              <div
                style={{
                  textAlign: 'left',
                  backgroundColor: '#FAF0DE',
                  border: '2px solid var(--outline-cocoa)',
                  borderRadius: '10px',
                  padding: '10px',
                  marginBottom: '14px',
                }}
              >
                <div style={{ fontSize: '0.85rem', fontWeight: 800, marginBottom: '6px' }}>
                  Sample of Missed High-Value Words:
                </div>
                <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                  {endResults.missed_sample.slice(0, 12).map((w) => (
                    <span
                      key={w}
                      style={{
                        backgroundColor: '#FFFBEA',
                        border: '1px solid var(--outline-cocoa)',
                        borderRadius: '4px',
                        padding: '1px 5px',
                        fontSize: '0.75rem',
                        fontWeight: 700,
                      }}
                    >
                      {w}
                    </span>
                  ))}
                </div>
              </div>
            )}

            <CozyButton variant="yellow" onClick={() => handleStartNewRound(selectedBoardSize)}>
              PLAY ANOTHER ROUND
            </CozyButton>
          </div>
        )}
      </CozyModal>
    </div>
  );
};
