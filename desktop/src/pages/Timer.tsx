import React, { useState, useEffect, useRef } from 'react';
import { sound } from '../services/sound';
import { CozyButton } from '../components/CozyButton';
import { Play, Pause, RotateCcw, Plus, Minus, Clock, Gamepad2 } from 'lucide-react';
import { GameTab } from '../types/arcade';
import { getGameTimers, setGameTimer, GameTimerSettings } from '../services/gameTimers';
import { BoggleGameIcon, BeeGameIcon, WordleGameIcon } from '../components/stickers/CustomIcons';

interface ExerciseStep {
  name: string;
  workSecs: number;
  restSecs: number;
}

const DEFAULT_ROUTINE: ExerciseStep[] = [
  { name: 'Jumping Jacks', workSecs: 30, restSecs: 5 },
  { name: 'Wall Sit', workSecs: 30, restSecs: 5 },
  { name: 'Push-ups', workSecs: 45, restSecs: 15 },
  { name: 'Crunches', workSecs: 45, restSecs: 15 },
  { name: 'Squats', workSecs: 45, restSecs: 10 },
  { name: 'Dips', workSecs: 30, restSecs: 5 },
  { name: 'Plank', workSecs: 60, restSecs: 10 },
  { name: 'Spot running', workSecs: 60, restSecs: 30 },
  { name: 'Lunges', workSecs: 30, restSecs: 5 },
  { name: 'Side Plank Left', workSecs: 20, restSecs: 5 },
  { name: 'Side Plank Right', workSecs: 20, restSecs: 0 },
];

interface TimerPageProps {
  onSelectGame?: (tab: GameTab) => void;
}

export const TimerPage: React.FC<TimerPageProps> = ({ onSelectGame }) => {
  // Mode selection: 'timer' | 'game-timers' | 'routine' | 'concurrent'
  const [activeMode, setActiveMode] = useState<'timer' | 'game-timers' | 'routine' | 'concurrent'>('timer');

  // Single Timer State
  const [presetDuration, setPresetDuration] = useState<number>(180); // seconds
  const [remainingMillis, setRemainingMillis] = useState<number>(180 * 1000);
  const [timerState, setTimerState] = useState<'Ready' | 'Running' | 'Paused' | 'Expired'>('Ready');
  const [subsecondMode, setSubsecondMode] = useState<boolean>(true);
  const [customInput, setCustomInput] = useState<string>('');
  const lastTickRef = useRef<number>(Date.now());

  // Game Timers State
  const [gameTimers, setLocalGameTimers] = useState<GameTimerSettings>(() => getGameTimers());
  const [boggleCustomInput, setBoggleCustomInput] = useState<string>('');
  const [beeCustomInput, setBeeCustomInput] = useState<string>('');
  const [wordleCustomInput, setWordleCustomInput] = useState<string>('');

  // Single Timer Clock Tick Loop (Drift-free monotonic)
  useEffect(() => {
    if (timerState !== 'Running') return;

    lastTickRef.current = Date.now();
    const interval = setInterval(() => {
      const now = Date.now();
      const elapsed = now - lastTickRef.current;
      lastTickRef.current = now;

      setRemainingMillis((prev) => {
        if (prev <= elapsed) {
          clearInterval(interval);
          setTimerState('Expired');
          sound.playBell();
          return 0;
        }
        return prev - elapsed;
      });
    }, subsecondMode ? 50 : 250);

    return () => clearInterval(interval);
  }, [timerState, subsecondMode]);

  // Routine State
  const [routineIndex, setRoutineIndex] = useState<number>(0);
  const [routinePhase, setRoutinePhase] = useState<'Idle' | 'Work' | 'Rest' | 'Paused' | 'Complete'>('Idle');
  const [routineRemainingSecs, setRoutineRemainingSecs] = useState<number>(30);
  const routineLastTickRef = useRef<number>(Date.now());

  // Routine Clock Loop
  useEffect(() => {
    if (routinePhase !== 'Work' && routinePhase !== 'Rest') return;

    routineLastTickRef.current = Date.now();
    const interval = setInterval(() => {
      setRoutineRemainingSecs((prev) => {
        if (prev <= 1) {
          // Transition phase
          if (routinePhase === 'Work') {
            const currentStep = DEFAULT_ROUTINE[routineIndex];
            sound.playBell();
            if (currentStep.restSecs > 0) {
              setRoutinePhase('Rest');
              return currentStep.restSecs;
            } else {
              // Direct next exercise
              if (routineIndex + 1 < DEFAULT_ROUTINE.length) {
                setRoutineIndex((idx) => idx + 1);
                setRoutinePhase('Work');
                return DEFAULT_ROUTINE[routineIndex + 1].workSecs;
              } else {
                setRoutinePhase('Complete');
                sound.playSuccess();
                return 0;
              }
            }
          } else if (routinePhase === 'Rest') {
            sound.playBell();
            if (routineIndex + 1 < DEFAULT_ROUTINE.length) {
              const nextIdx = routineIndex + 1;
              setRoutineIndex(nextIdx);
              setRoutinePhase('Work');
              return DEFAULT_ROUTINE[nextIdx].workSecs;
            } else {
              setRoutinePhase('Complete');
              sound.playSuccess();
              return 0;
            }
          }
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(interval);
  }, [routinePhase, routineIndex]);

  // Concurrent Multi-Timers State
  const [concurrentTimers, setConcurrentTimers] = useState<
    { id: string; label: string; totalSecs: number; remainingSecs: number; running: boolean }[]
  >([
    { id: '1', label: 'Quick Sprint', totalSecs: 15, remainingSecs: 15, running: false },
    { id: '2', label: 'Boggle Blitz', totalSecs: 60, remainingSecs: 60, running: false },
    { id: '3', label: 'Match Round', totalSecs: 180, remainingSecs: 180, running: false },
  ]);

  useEffect(() => {
    const anyRunning = concurrentTimers.some((t) => t.running && t.remainingSecs > 0);
    if (!anyRunning) return;

    const interval = setInterval(() => {
      setConcurrentTimers((prev) =>
        prev.map((t) => {
          if (!t.running || t.remainingSecs <= 0) return t;
          const next = t.remainingSecs - 1;
          if (next === 0) sound.playBell();
          return { ...t, remainingSecs: Math.max(0, next), running: next > 0 };
        })
      );
    }, 1000);

    return () => clearInterval(interval);
  }, [concurrentTimers]);

  // Helpers
  const formatTime = (millis: number, showSub: boolean) => {
    const totalSecs = Math.floor(millis / 1000);
    const mins = Math.floor(totalSecs / 60);
    const secs = totalSecs % 60;
    const tenths = Math.floor((millis % 1000) / 100);

    if (showSub) {
      return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}.${tenths}`;
    }
    return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
  };

  const handleSetPreset = (secs: number) => {
    sound.playClick();
    setPresetDuration(secs);
    setRemainingMillis(secs * 1000);
    setTimerState('Ready');
  };

  const handleToggleTimer = () => {
    sound.playPop();
    if (timerState === 'Ready' || timerState === 'Paused') {
      setTimerState('Running');
    } else if (timerState === 'Running') {
      setTimerState('Paused');
    } else if (timerState === 'Expired') {
      setRemainingMillis(presetDuration * 1000);
      setTimerState('Running');
    }
  };

  const handleResetTimer = () => {
    sound.playClick();
    setRemainingMillis(presetDuration * 1000);
    setTimerState('Ready');
  };

  const handleAdjustSeconds = (deltaSecs: number) => {
    sound.playClick();
    setRemainingMillis((prev) => {
      const next = Math.max(0, prev + deltaSecs * 1000);
      if (next === 0) {
        setTimerState('Expired');
        sound.playBell();
      }
      return next;
    });
  };

  const handleApplyCustom = () => {
    sound.playClick();
    const raw = customInput.trim();
    if (!raw) return;

    let secs = 0;
    if (raw.includes(':')) {
      const parts = raw.split(':').map((p) => parseInt(p, 10));
      if (parts.length === 2 && !isNaN(parts[0]) && !isNaN(parts[1])) {
        secs = parts[0] * 60 + parts[1];
      }
    } else {
      const parsed = parseInt(raw, 10);
      if (!isNaN(parsed) && parsed > 0) secs = parsed;
    }

    if (secs > 0) {
      setPresetDuration(secs);
      setRemainingMillis(secs * 1000);
      setTimerState('Ready');
      setCustomInput('');
    }
  };

  const handleUpdateGameTimer = (game: 'boggle' | 'spelling-bee' | 'wordle', secs: number) => {
    sound.playClick();
    setGameTimer(game, secs);
    setLocalGameTimers(getGameTimers());
  };

  const handleParseCustomGameTimer = (game: 'boggle' | 'spelling-bee' | 'wordle', raw: string) => {
    const trimmed = raw.trim();
    if (!trimmed) return;
    let secs = 0;
    if (trimmed.includes(':')) {
      const parts = trimmed.split(':').map((p) => parseInt(p, 10));
      if (parts.length === 2 && !isNaN(parts[0]) && !isNaN(parts[1])) {
        secs = parts[0] * 60 + parts[1];
      }
    } else {
      const val = parseInt(trimmed, 10);
      if (!isNaN(val) && val >= 0) secs = val;
    }
    handleUpdateGameTimer(game, secs);
  };

  const timerProgress = presetDuration > 0 ? remainingMillis / (presetDuration * 1000) : 0;

  return (
    <div style={{ padding: '8px 16px', display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Top Mode Bar */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          borderBottom: '2px dashed var(--outline-cocoa)',
          paddingBottom: '8px',
          marginBottom: '12px',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Clock size={24} color="#60433B" />
          <h2 style={{ margin: 0, fontSize: '1.3rem', fontWeight: 900, color: 'var(--text-main)' }}>
            High-Precision Countdown Timer
          </h2>
        </div>

        <div style={{ display: 'flex', gap: '6px' }}>
          <button
            onClick={() => {
              sound.playClick();
              setActiveMode('timer');
            }}
            style={{
              padding: '4px 12px',
              borderRadius: '6px',
              fontSize: '0.85rem',
              fontWeight: 800,
              cursor: 'pointer',
              border: '1.5px solid var(--outline-cocoa)',
              backgroundColor: activeMode === 'timer' ? '#FFEAA7' : '#FAF0DE',
              color: 'var(--text-main)',
            }}
          >
            Countdown Clock
          </button>
          <button
            onClick={() => {
              sound.playClick();
              setActiveMode('game-timers');
            }}
            style={{
              padding: '4px 12px',
              borderRadius: '6px',
              fontSize: '0.85rem',
              fontWeight: 800,
              cursor: 'pointer',
              border: '1.5px solid var(--outline-cocoa)',
              backgroundColor: activeMode === 'game-timers' ? '#8CCB79' : '#FAF0DE',
              color: 'var(--text-main)',
            }}
          >
            Word Game Timers
          </button>
          <button
            onClick={() => {
              sound.playClick();
              setActiveMode('routine');
            }}
            style={{
              padding: '4px 12px',
              borderRadius: '6px',
              fontSize: '0.85rem',
              fontWeight: 800,
              cursor: 'pointer',
              border: '1.5px solid var(--outline-cocoa)',
              backgroundColor: activeMode === 'routine' ? '#FFD56D' : '#FAF0DE',
              color: 'var(--text-main)',
            }}
          >
            Exercise Routine
          </button>
          <button
            onClick={() => {
              sound.playClick();
              setActiveMode('concurrent');
            }}
            style={{
              padding: '4px 12px',
              borderRadius: '6px',
              fontSize: '0.85rem',
              fontWeight: 800,
              cursor: 'pointer',
              border: '1.5px solid var(--outline-cocoa)',
              backgroundColor: activeMode === 'concurrent' ? '#93DFE8' : '#FAF0DE',
              color: 'var(--text-main)',
            }}
          >
            Multi-Timer Concurrency
          </button>
        </div>
      </div>

      {/* Mode 1: Main Countdown Clock */}
      {activeMode === 'timer' && (
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px', flex: 1 }}>
          {/* Left Panel: Large Digital Clock & Controls */}
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              backgroundColor: '#FFFDF9',
              border: '2px solid var(--outline-cocoa)',
              borderRadius: '12px',
              padding: '20px',
              boxShadow: '3px 3px 0 var(--outline-cocoa)',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', width: '100%', marginBottom: '10px' }}>
              <span style={{ fontSize: '0.85rem', fontWeight: 800, color: 'var(--text-muted)' }}>
                PRESET: {formatTime(presetDuration * 1000, false)}
              </span>
              <span
                style={{
                  fontSize: '0.8rem',
                  fontWeight: 900,
                  padding: '2px 8px',
                  borderRadius: '6px',
                  backgroundColor:
                    timerState === 'Running'
                      ? '#8CCB79'
                      : timerState === 'Paused'
                      ? '#FFD56D'
                      : timerState === 'Expired'
                      ? '#F48E88'
                      : '#E0D0B8',
                  color: '#2B1E16',
                }}
              >
                [{timerState.toUpperCase()}]
              </span>
            </div>

            {/* Big Monospace Display */}
            <div
              style={{
                fontFamily: 'monospace',
                fontSize: '3.6rem',
                fontWeight: 900,
                color: timerState === 'Expired' ? '#c92a2a' : '#2B1E16',
                backgroundColor: '#FAF0DE',
                border: '3px solid var(--outline-cocoa)',
                borderRadius: '12px',
                padding: '12px 24px',
                letterSpacing: '2px',
                marginBottom: '16px',
                boxShadow: 'inset 2px 2px 4px rgba(0,0,0,0.06)',
              }}
            >
              {formatTime(remainingMillis, subsecondMode)}
            </div>

            {/* Visual Progress Bar */}
            <div
              style={{
                width: '100%',
                height: '14px',
                backgroundColor: '#EBE0D0',
                borderRadius: '7px',
                overflow: 'hidden',
                border: '1.5px solid var(--outline-cocoa)',
                marginBottom: '16px',
              }}
            >
              <div
                style={{
                  width: `${(timerProgress * 100).toFixed(1)}%`,
                  height: '100%',
                  backgroundColor: timerState === 'Expired' ? '#F48E88' : '#8CCB79',
                  transition: subsecondMode ? 'none' : 'width 0.25s linear',
                }}
              />
            </div>

            {/* Primary Action Buttons */}
            <div style={{ display: 'flex', gap: '10px', marginBottom: '14px' }}>
              <CozyButton
                variant={timerState === 'Running' ? 'yellow' : 'mint'}
                size="md"
                onClick={handleToggleTimer}
              >
                {timerState === 'Running' ? (
                  <>
                    <Pause size={16} /> Pause
                  </>
                ) : timerState === 'Paused' ? (
                  <>
                    <Play size={16} /> Resume
                  </>
                ) : (
                  <>
                    <Play size={16} /> Start
                  </>
                )}
              </CozyButton>

              <CozyButton variant="cream" size="md" onClick={handleResetTimer}>
                <RotateCcw size={16} /> Reset
              </CozyButton>

              <CozyButton variant="cream" size="md" onClick={() => handleAdjustSeconds(10)}>
                <Plus size={16} /> 10s
              </CozyButton>

              <CozyButton variant="cream" size="md" onClick={() => handleAdjustSeconds(-10)}>
                <Minus size={16} /> 10s
              </CozyButton>
            </div>

            {/* Subsecond resolution toggle */}
            <label
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                fontSize: '0.85rem',
                fontWeight: 800,
                color: 'var(--text-main)',
                cursor: 'pointer',
              }}
            >
              <input
                type="checkbox"
                checked={subsecondMode}
                onChange={(e) => setSubsecondMode(e.target.checked)}
              />
              Enable 100ms Sub-second Precision
            </label>
          </div>

          {/* Right Panel: Presets & Custom Configuration */}
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              backgroundColor: '#FFFDF9',
              border: '2px solid var(--outline-cocoa)',
              borderRadius: '12px',
              padding: '16px',
              boxShadow: '3px 3px 0 var(--outline-cocoa)',
            }}
          >
            <h3 style={{ margin: '0 0 10px 0', fontSize: '1.05rem', fontWeight: 900, color: 'var(--text-main)' }}>
              Preset Durations
            </h3>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px', marginBottom: '16px' }}>
              {[
                { label: '30s Blitz', secs: 30 },
                { label: '1m Quick', secs: 60 },
                { label: '3m Boggle Standard', secs: 180 },
                { label: '5m Study', secs: 300 },
                { label: '10m Focus', secs: 600 },
                { label: '15m Sprint', secs: 900 },
              ].map((p) => (
                <button
                  key={p.secs}
                  onClick={() => handleSetPreset(p.secs)}
                  style={{
                    padding: '8px',
                    borderRadius: '8px',
                    border: '1.5px solid var(--outline-cocoa)',
                    backgroundColor: presetDuration === p.secs ? '#FFEAA7' : '#FAF0DE',
                    fontWeight: 800,
                    fontSize: '0.82rem',
                    cursor: 'pointer',
                  }}
                >
                  {p.label}
                </button>
              ))}
            </div>

            <h3 style={{ margin: '0 0 8px 0', fontSize: '1.05rem', fontWeight: 900, color: 'var(--text-main)' }}>
              Custom Duration
            </h3>
            <div style={{ display: 'flex', gap: '8px', marginBottom: '16px' }}>
              <input
                type="text"
                placeholder="e.g. 03:00 or 90"
                value={customInput}
                onChange={(e) => setCustomInput(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleApplyCustom()}
                style={{
                  flex: 1,
                  padding: '6px 12px',
                  borderRadius: '6px',
                  border: '1.5px solid var(--outline-cocoa)',
                  backgroundColor: '#FAF0DE',
                  fontFamily: 'monospace',
                  fontSize: '0.95rem',
                  fontWeight: 800,
                }}
              />
              <CozyButton variant="yellow" size="sm" onClick={handleApplyCustom}>
                Set
              </CozyButton>
            </div>

            <div
              style={{
                marginTop: 'auto',
                padding: '10px',
                borderRadius: '8px',
                backgroundColor: '#F5ECE1',
                border: '1.5px solid var(--outline-cocoa)',
                fontSize: '0.8rem',
                color: 'var(--text-main)',
                lineHeight: 1.4,
              }}
            >
              <strong>Drift-Free Guarantee:</strong> Uses <code>Date.now()</code> system clock deltas rather than
              incremental interval additions, ensuring strict synchronization and zero accumulative drift.
            </div>
          </div>
        </div>
      )}

      {/* Mode 2: Word Game Custom Timers */}
      {activeMode === 'game-timers' && (
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(3, 1fr)',
            gap: '16px',
            flex: 1,
            overflowY: 'auto',
          }}
        >
          {/* Boggle Card */}
          <div
            style={{
              backgroundColor: '#FFFDF9',
              border: '2px solid var(--outline-cocoa)',
              borderRadius: '12px',
              padding: '16px',
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'space-between',
              boxShadow: '3px 3px 0 var(--outline-cocoa)',
            }}
          >
            <div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
                <BoggleGameIcon size={28} />
                <h3 style={{ margin: 0, fontSize: '1.15rem', fontWeight: 900, color: 'var(--text-main)' }}>
                  Boggle Round Timer
                </h3>
              </div>
              <p style={{ fontSize: '0.82rem', color: 'var(--text-muted)', marginBottom: '12px', lineHeight: 1.35 }}>
                Standard Boggle uses a 3-minute sand timer. Configure your desired round time or select untimed practice.
              </p>

              <div
                style={{
                  display: 'inline-block',
                  padding: '4px 10px',
                  borderRadius: '6px',
                  backgroundColor: '#FAF0DE',
                  border: '1.5px solid var(--outline-cocoa)',
                  fontFamily: 'monospace',
                  fontWeight: 900,
                  fontSize: '0.95rem',
                  marginBottom: '14px',
                }}
              >
                Current:{' '}
                {gameTimers.boggleSecs > 0
                  ? `${Math.floor(gameTimers.boggleSecs / 60)}m ${String(gameTimers.boggleSecs % 60).padStart(2, '0')}s (${gameTimers.boggleSecs}s)`
                  : 'Untimed (No Timer)'}
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '6px', marginBottom: '12px' }}>
                {[
                  { label: '60s (1m Blitz)', secs: 60 },
                  { label: '120s (2m Fast)', secs: 120 },
                  { label: '180s (3m Default)', secs: 180 },
                  { label: '300s (5m Relaxed)', secs: 300 },
                  { label: 'Untimed (Practice)', secs: 0 },
                ].map((p) => (
                  <button
                    key={p.secs}
                    onClick={() => handleUpdateGameTimer('boggle', p.secs)}
                    style={{
                      padding: '6px',
                      borderRadius: '6px',
                      border: '1.5px solid var(--outline-cocoa)',
                      backgroundColor: gameTimers.boggleSecs === p.secs ? '#93DFE8' : '#FAF0DE',
                      fontWeight: 800,
                      fontSize: '0.78rem',
                      cursor: 'pointer',
                    }}
                  >
                    {p.label}
                  </button>
                ))}
              </div>

              <div style={{ display: 'flex', gap: '6px', marginBottom: '12px' }}>
                <input
                  type="text"
                  placeholder="Custom secs or MM:SS"
                  value={boggleCustomInput}
                  onChange={(e) => setBoggleCustomInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      handleParseCustomGameTimer('boggle', boggleCustomInput);
                      setBoggleCustomInput('');
                    }
                  }}
                  style={{
                    flex: 1,
                    padding: '4px 8px',
                    borderRadius: '6px',
                    border: '1.5px solid var(--outline-cocoa)',
                    backgroundColor: '#FAF0DE',
                    fontFamily: 'monospace',
                    fontSize: '0.85rem',
                  }}
                />
                <CozyButton
                  variant="cream"
                  size="sm"
                  onClick={() => {
                    handleParseCustomGameTimer('boggle', boggleCustomInput);
                    setBoggleCustomInput('');
                  }}
                >
                  Set
                </CozyButton>
              </div>
            </div>

            <CozyButton
              variant="mint"
              size="sm"
              onClick={() => {
                sound.playPop();
                if (onSelectGame) onSelectGame('boggle');
              }}
            >
              <Gamepad2 size={16} /> Play Boggle
            </CozyButton>
          </div>

          {/* Spelling Bee Card */}
          <div
            style={{
              backgroundColor: '#FFFDF9',
              border: '2px solid var(--outline-cocoa)',
              borderRadius: '12px',
              padding: '16px',
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'space-between',
              boxShadow: '3px 3px 0 var(--outline-cocoa)',
            }}
          >
            <div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
                <BeeGameIcon size={28} />
                <h3 style={{ margin: 0, fontSize: '1.15rem', fontWeight: 900, color: 'var(--text-main)' }}>
                  Spelling Bee Blitz Timer
                </h3>
              </div>
              <p style={{ fontSize: '0.82rem', color: 'var(--text-muted)', marginBottom: '12px', lineHeight: 1.35 }}>
                Test how many words and pangrams you can discover within a time limit before the bell sounds.
              </p>

              <div
                style={{
                  display: 'inline-block',
                  padding: '4px 10px',
                  borderRadius: '6px',
                  backgroundColor: '#FAF0DE',
                  border: '1.5px solid var(--outline-cocoa)',
                  fontFamily: 'monospace',
                  fontWeight: 900,
                  fontSize: '0.95rem',
                  marginBottom: '14px',
                }}
              >
                Current:{' '}
                {gameTimers.spellingBeeSecs > 0
                  ? `${Math.floor(gameTimers.spellingBeeSecs / 60)}m ${String(gameTimers.spellingBeeSecs % 60).padStart(2, '0')}s (${gameTimers.spellingBeeSecs}s)`
                  : 'Untimed (Classic)'}
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '6px', marginBottom: '12px' }}>
                {[
                  { label: 'Untimed (Classic)', secs: 0 },
                  { label: '60s (1m Sprint)', secs: 60 },
                  { label: '120s (2m Fast)', secs: 120 },
                  { label: '180s (3m Blitz)', secs: 180 },
                  { label: '300s (5m Marathon)', secs: 300 },
                ].map((p) => (
                  <button
                    key={p.secs}
                    onClick={() => handleUpdateGameTimer('spelling-bee', p.secs)}
                    style={{
                      padding: '6px',
                      borderRadius: '6px',
                      border: '1.5px solid var(--outline-cocoa)',
                      backgroundColor: gameTimers.spellingBeeSecs === p.secs ? '#FFD56D' : '#FAF0DE',
                      fontWeight: 800,
                      fontSize: '0.78rem',
                      cursor: 'pointer',
                    }}
                  >
                    {p.label}
                  </button>
                ))}
              </div>

              <div style={{ display: 'flex', gap: '6px', marginBottom: '12px' }}>
                <input
                  type="text"
                  placeholder="Custom secs or MM:SS"
                  value={beeCustomInput}
                  onChange={(e) => setBeeCustomInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      handleParseCustomGameTimer('spelling-bee', beeCustomInput);
                      setBeeCustomInput('');
                    }
                  }}
                  style={{
                    flex: 1,
                    padding: '4px 8px',
                    borderRadius: '6px',
                    border: '1.5px solid var(--outline-cocoa)',
                    backgroundColor: '#FAF0DE',
                    fontFamily: 'monospace',
                    fontSize: '0.85rem',
                  }}
                />
                <CozyButton
                  variant="cream"
                  size="sm"
                  onClick={() => {
                    handleParseCustomGameTimer('spelling-bee', beeCustomInput);
                    setBeeCustomInput('');
                  }}
                >
                  Set
                </CozyButton>
              </div>
            </div>

            <CozyButton
              variant="yellow"
              size="sm"
              onClick={() => {
                sound.playPop();
                if (onSelectGame) onSelectGame('spelling-bee');
              }}
            >
              <Gamepad2 size={16} /> Play Spelling Bee
            </CozyButton>
          </div>

          {/* Wordle Card */}
          <div
            style={{
              backgroundColor: '#FFFDF9',
              border: '2px solid var(--outline-cocoa)',
              borderRadius: '12px',
              padding: '16px',
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'space-between',
              boxShadow: '3px 3px 0 var(--outline-cocoa)',
            }}
          >
            <div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
                <WordleGameIcon size={28} />
                <h3 style={{ margin: 0, fontSize: '1.15rem', fontWeight: 900, color: 'var(--text-main)' }}>
                  Wordle Speedrun Timer
                </h3>
              </div>
              <p style={{ fontSize: '0.82rem', color: 'var(--text-muted)', marginBottom: '12px', lineHeight: 1.35 }}>
                Deduce the secret 5-letter word before the speedrun clock runs out across your 6 allowed attempts.
              </p>

              <div
                style={{
                  display: 'inline-block',
                  padding: '4px 10px',
                  borderRadius: '6px',
                  backgroundColor: '#FAF0DE',
                  border: '1.5px solid var(--outline-cocoa)',
                  fontFamily: 'monospace',
                  fontWeight: 900,
                  fontSize: '0.95rem',
                  marginBottom: '14px',
                }}
              >
                Current:{' '}
                {gameTimers.wordleSecs > 0
                  ? `${Math.floor(gameTimers.wordleSecs / 60)}m ${String(gameTimers.wordleSecs % 60).padStart(2, '0')}s (${gameTimers.wordleSecs}s)`
                  : 'Untimed (Classic)'}
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '6px', marginBottom: '12px' }}>
                {[
                  { label: 'Untimed (Classic)', secs: 0 },
                  { label: '60s (1m Speedrun)', secs: 60 },
                  { label: '90s (1.5m Fast)', secs: 90 },
                  { label: '120s (2m Rapid)', secs: 120 },
                  { label: '180s (3m Standard)', secs: 180 },
                ].map((p) => (
                  <button
                    key={p.secs}
                    onClick={() => handleUpdateGameTimer('wordle', p.secs)}
                    style={{
                      padding: '6px',
                      borderRadius: '6px',
                      border: '1.5px solid var(--outline-cocoa)',
                      backgroundColor: gameTimers.wordleSecs === p.secs ? '#F48E88' : '#FAF0DE',
                      fontWeight: 800,
                      fontSize: '0.78rem',
                      cursor: 'pointer',
                    }}
                  >
                    {p.label}
                  </button>
                ))}
              </div>

              <div style={{ display: 'flex', gap: '6px', marginBottom: '12px' }}>
                <input
                  type="text"
                  placeholder="Custom secs or MM:SS"
                  value={wordleCustomInput}
                  onChange={(e) => setWordleCustomInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      handleParseCustomGameTimer('wordle', wordleCustomInput);
                      setWordleCustomInput('');
                    }
                  }}
                  style={{
                    flex: 1,
                    padding: '4px 8px',
                    borderRadius: '6px',
                    border: '1.5px solid var(--outline-cocoa)',
                    backgroundColor: '#FAF0DE',
                    fontFamily: 'monospace',
                    fontSize: '0.85rem',
                  }}
                />
                <CozyButton
                  variant="cream"
                  size="sm"
                  onClick={() => {
                    handleParseCustomGameTimer('wordle', wordleCustomInput);
                    setWordleCustomInput('');
                  }}
                >
                  Set
                </CozyButton>
              </div>
            </div>

            <CozyButton
              variant="coral"
              size="sm"
              onClick={() => {
                sound.playPop();
                if (onSelectGame) onSelectGame('wordle');
              }}
            >
              <Gamepad2 size={16} /> Play Wordle
            </CozyButton>
          </div>
        </div>
      )}

      {/* Mode 3: Routine Mode View */}
      {activeMode === 'routine' && (
        <div style={{ display: 'grid', gridTemplateColumns: '1.2fr 0.8fr', gap: '16px', flex: 1 }}>
          {/* Left Table */}
          <div
            style={{
              backgroundColor: '#FFFDF9',
              border: '2px solid var(--outline-cocoa)',
              borderRadius: '12px',
              padding: '12px',
              overflowY: 'auto',
              maxHeight: '420px',
              boxShadow: '3px 3px 0 var(--outline-cocoa)',
            }}
          >
            <h3 style={{ margin: '0 0 8px 0', fontSize: '1.05rem', fontWeight: 900, color: 'var(--text-main)' }}>
              Programmed 7-Minute Workout Table
            </h3>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.82rem' }}>
              <thead>
                <tr style={{ borderBottom: '2px solid var(--outline-cocoa)', textAlign: 'left' }}>
                  <th style={{ padding: '6px' }}>#</th>
                  <th style={{ padding: '6px' }}>Exercise</th>
                  <th style={{ padding: '6px' }}>Work</th>
                  <th style={{ padding: '6px' }}>Rest</th>
                  <th style={{ padding: '6px' }}>Status</th>
                </tr>
              </thead>
              <tbody>
                {DEFAULT_ROUTINE.map((step, idx) => {
                  const isCurrent = idx === routineIndex && (routinePhase === 'Work' || routinePhase === 'Rest');
                  const isDone = idx < routineIndex || routinePhase === 'Complete';
                  return (
                    <tr
                      key={step.name}
                      style={{
                        backgroundColor: isCurrent ? '#FFEAA7' : isDone ? '#EAE0D0' : 'transparent',
                        fontWeight: isCurrent ? 900 : 700,
                        borderBottom: '1px dashed #D0C0B0',
                      }}
                    >
                      <td style={{ padding: '6px' }}>{idx + 1}</td>
                      <td style={{ padding: '6px' }}>{step.name}</td>
                      <td style={{ padding: '6px' }}>{step.workSecs}s</td>
                      <td style={{ padding: '6px' }}>{step.restSecs}s</td>
                      <td style={{ padding: '6px' }}>
                        {isCurrent ? routinePhase.toUpperCase() : isDone ? 'DONE' : 'QUEUED'}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>

          {/* Right Routine Active HUD */}
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              backgroundColor: '#FFFDF9',
              border: '2px solid var(--outline-cocoa)',
              borderRadius: '12px',
              padding: '20px',
              boxShadow: '3px 3px 0 var(--outline-cocoa)',
              textAlign: 'center',
            }}
          >
            <span style={{ fontSize: '0.85rem', fontWeight: 800, color: 'var(--text-muted)', marginBottom: '4px' }}>
              STEP {routineIndex + 1} OF {DEFAULT_ROUTINE.length}
            </span>
            <h2 style={{ margin: '0 0 12px 0', fontSize: '1.4rem', fontWeight: 900, color: 'var(--text-main)' }}>
              {DEFAULT_ROUTINE[routineIndex].name}
            </h2>

            <div
              style={{
                fontFamily: 'monospace',
                fontSize: '3.2rem',
                fontWeight: 900,
                color: routinePhase === 'Rest' ? '#d97706' : '#2B1E16',
                backgroundColor: routinePhase === 'Rest' ? '#FEF3C7' : '#FAF0DE',
                border: '3px solid var(--outline-cocoa)',
                borderRadius: '12px',
                padding: '10px 24px',
                marginBottom: '16px',
              }}
            >
              {routineRemainingSecs}s
            </div>

            <div style={{ fontWeight: 800, fontSize: '0.9rem', marginBottom: '16px', color: 'var(--text-main)' }}>
              {routinePhase === 'Rest' ? (
                <span>
                  REST INTERVAL! Up next:{' '}
                  <strong>
                    {routineIndex + 1 < DEFAULT_ROUTINE.length ? DEFAULT_ROUTINE[routineIndex + 1].name : 'Finish!'}
                  </strong>
                </span>
              ) : routinePhase === 'Work' ? (
                <span>WORK INTERVAL IN PROGRESS!</span>
              ) : routinePhase === 'Complete' ? (
                <span>🏆 ROUTINE COMPLETE! Great job!</span>
              ) : (
                <span>Ready to start routine</span>
              )}
            </div>

            <div style={{ display: 'flex', gap: '10px' }}>
              {routinePhase === 'Idle' || routinePhase === 'Complete' ? (
                <CozyButton
                  variant="mint"
                  size="md"
                  onClick={() => {
                    sound.playPop();
                    setRoutineIndex(0);
                    setRoutinePhase('Work');
                    setRoutineRemainingSecs(DEFAULT_ROUTINE[0].workSecs);
                  }}
                >
                  <Play size={16} /> Start Routine
                </CozyButton>
              ) : (
                <CozyButton
                  variant={routinePhase === 'Paused' ? 'mint' : 'yellow'}
                  size="md"
                  onClick={() => {
                    sound.playPop();
                    if (routinePhase === 'Paused') {
                      setRoutinePhase('Work');
                    } else {
                      setRoutinePhase('Paused');
                    }
                  }}
                >
                  {routinePhase === 'Paused' ? <Play size={16} /> : <Pause size={16} />}
                  {routinePhase === 'Paused' ? 'Resume' : 'Pause'}
                </CozyButton>
              )}

              <CozyButton
                variant="cream"
                size="md"
                onClick={() => {
                  sound.playClick();
                  setRoutinePhase('Idle');
                  setRoutineIndex(0);
                  setRoutineRemainingSecs(DEFAULT_ROUTINE[0].workSecs);
                }}
              >
                <RotateCcw size={16} /> Reset
              </CozyButton>
            </div>
          </div>
        </div>
      )}

      {/* Mode 4: Concurrent Mode View */}
      {activeMode === 'concurrent' && (
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: '12px',
            flex: 1,
            backgroundColor: '#FFFDF9',
            border: '2px solid var(--outline-cocoa)',
            borderRadius: '12px',
            padding: '20px',
            boxShadow: '3px 3px 0 var(--outline-cocoa)',
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <h3 style={{ margin: 0, fontSize: '1.1rem', fontWeight: 900, color: 'var(--text-main)' }}>
              Parallel Concurrent Timers
            </h3>
            <div style={{ display: 'flex', gap: '8px' }}>
              <CozyButton
                variant="mint"
                size="sm"
                onClick={() => {
                  sound.playPop();
                  setConcurrentTimers((prev) => prev.map((t) => ({ ...t, running: t.remainingSecs > 0 })));
                }}
              >
                <Play size={14} /> Start All
              </CozyButton>
              <CozyButton
                variant="yellow"
                size="sm"
                onClick={() => {
                  sound.playPop();
                  setConcurrentTimers((prev) => prev.map((t) => ({ ...t, running: false })));
                }}
              >
                <Pause size={14} /> Pause All
              </CozyButton>
              <CozyButton
                variant="cream"
                size="sm"
                onClick={() => {
                  sound.playClick();
                  setConcurrentTimers((prev) =>
                    prev.map((t) => ({ ...t, remainingSecs: t.totalSecs, running: false }))
                  );
                }}
              >
                <RotateCcw size={14} /> Reset All
              </CozyButton>
            </div>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '14px', marginTop: '10px' }}>
            {concurrentTimers.map((t) => {
              const prog = t.totalSecs > 0 ? t.remainingSecs / t.totalSecs : 0;
              return (
                <div
                  key={t.id}
                  style={{
                    backgroundColor: '#FAF0DE',
                    border: '1.5px solid var(--outline-cocoa)',
                    borderRadius: '8px',
                    padding: '12px 16px',
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                    <span style={{ fontWeight: 900, fontSize: '1rem', color: 'var(--text-main)' }}>{t.label}</span>
                    <span
                      style={{
                        fontFamily: 'monospace',
                        fontWeight: 900,
                        fontSize: '1.2rem',
                        color: t.remainingSecs === 0 ? '#c92a2a' : 'var(--text-main)',
                      }}
                    >
                      {formatTime(t.remainingSecs * 1000, false)}
                    </span>
                  </div>

                  <div
                    style={{
                      height: '10px',
                      backgroundColor: '#EAE0D0',
                      borderRadius: '5px',
                      overflow: 'hidden',
                      border: '1px solid var(--outline-cocoa)',
                    }}
                  >
                    <div
                      style={{
                        width: `${(prog * 100).toFixed(1)}%`,
                        height: '100%',
                        backgroundColor: t.remainingSecs === 0 ? '#F48E88' : '#8CCB79',
                        transition: 'width 0.2s linear',
                      }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};
