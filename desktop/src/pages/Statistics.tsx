import React, { useEffect, useState } from 'react';
import {
  RotateCcw,
} from 'lucide-react';
import { UnifiedStats } from '../types/arcade';
import { arcadeApi } from '../services/api';
import { CozyButton } from '../components/CozyButton';
import { CozyModal } from '../components/CozyModal';
import { sound } from '../services/sound';
import {
  ArcadeCabinetIcon,
  BeeGameIcon,
  BoggleGameIcon,
  CrownQueenIcon,
  HoneyJarIcon,
  PawPrintIcon,
  StatsRibbonIcon,
  StreakFlameIcon,
  WordleGameIcon,
} from '../components/stickers/CustomIcons';
import {
  DoodleStars,
  PostageStamp,
  PressedFern,
  PressedFlower,
  ScrawlyNote,
  WashiTape,
} from '../components/stickers/DiaryStickers';

export const StatisticsPage: React.FC<{ active?: boolean; onStatsChange?: () => void }> = ({ active = true, onStatsChange }) => {
  const [stats, setStats] = useState<UnifiedStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showResetModal, setShowResetModal] = useState(false);

  const fetchStats = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await arcadeApi.getStats();
      setStats(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Could not load statistics');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (active) void fetchStats();
  }, [active]);

  const handleConfirmReset = async () => {
    try {
      sound.playSuccess();
      await arcadeApi.resetStats();
      setShowResetModal(false);
      await fetchStats();
      onStatsChange?.();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Could not reset statistics');
    }
  };

  if (loading || !stats) {
    return (
      <div style={{ textAlign: 'center', padding: '60px' }}>
        <h2 style={{ fontSize: '1.4rem' }}>Opening Arcade Scrapbook...</h2>
        <p role="alert">{error}</p>
        <CozyButton onClick={fetchStats} disabled={loading}>Retry connection</CozyButton>
      </div>
    );
  }

  // Max value for Wordle distribution bars
  const maxGuessCount = Math.max(...stats.wordle.guess_distribution, 1);
  const totalGamesAll =
    stats.wordle.games_played + stats.spelling_bee.games_played + stats.boggle.games_played;

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
      {/* LEFT PAGE: Lifetime Overview, Wordle Records & Guess Chart    */}
      {/* ============================================================== */}
      <div
        style={{
          padding: '24px 22px 24px 28px',
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'space-between',
          borderRight: '1px solid rgba(96,67,59,0.15)',
          position: 'relative',
        }}
      >
        {/* Top Header & Scrapbook Title */}
        <div>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              marginBottom: '12px',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <StatsRibbonIcon size={28} />
              <div>
                <h2 style={{ fontSize: '1.45rem', color: 'var(--text-main)', margin: 0 }}>
                  ARCADE SCRAPBOOK
                </h2>
                <div
                  style={{
                    fontFamily: 'var(--font-handwritten)',
                    fontSize: '1.15rem',
                    color: '#6E4E37',
                    fontWeight: 700,
                  }}
                >
                  ~ Volume I: Solvers & High Scores ~
                </div>
              </div>
            </div>

            <PostageStamp label="25¢" illustration="cat" rotation={-4} size={42} />
          </div>

          <div style={{ marginBottom: '14px' }}>
            <WashiTape color="lavender" width={110} rotation={-1} />
          </div>

          {/* Quick Summary Pill Row */}
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '10px',
              marginBottom: '16px',
            }}
          >
            {/* Total Games */}
            <div
              style={{
                backgroundColor: 'var(--panel-cream)',
                borderRadius: '12px',
                border: '2px solid var(--outline-cocoa)',
                padding: '10px 12px',
                display: 'flex',
                alignItems: 'center',
                gap: '10px',
                boxShadow: '0 2px 0 var(--outline-cocoa)',
              }}
            >
              <ArcadeCabinetIcon size={32} />
              <div>
                <div style={{ fontSize: '1.35rem', fontWeight: 900, color: 'var(--text-main)', lineHeight: 1 }}>
                  {totalGamesAll}
                </div>
                <div style={{ fontSize: '0.72rem', fontWeight: 800, color: 'var(--text-muted)' }}>
                  Total Played
                </div>
              </div>
            </div>

            {/* Wordle Streak */}
            <div
              style={{
                backgroundColor: 'var(--panel-cream)',
                borderRadius: '12px',
                border: '2px solid var(--outline-cocoa)',
                padding: '10px 12px',
                display: 'flex',
                alignItems: 'center',
                gap: '10px',
                boxShadow: '0 2px 0 var(--outline-cocoa)',
              }}
            >
              <StreakFlameIcon size={32} />
              <div>
                <div style={{ fontSize: '1.35rem', fontWeight: 900, color: '#e8590c', lineHeight: 1 }}>
                  {stats.wordle.current_streak}
                </div>
                <div style={{ fontSize: '0.72rem', fontWeight: 800, color: 'var(--text-muted)' }}>
                  Streak (Max {stats.wordle.max_streak})
                </div>
              </div>
            </div>
          </div>

          {/* Wordle Records Card */}
          <div
            style={{
              backgroundColor: '#FAF5EE',
              borderRadius: '14px',
              border: '2.5px solid var(--outline-cocoa)',
              padding: '14px 16px',
              boxShadow: '0 3px 0 var(--outline-cocoa)',
              marginBottom: '14px',
            }}
          >
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                borderBottom: '2px dashed var(--outline-cocoa)',
                paddingBottom: '6px',
                marginBottom: '10px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <WordleGameIcon size={22} />
                <span style={{ fontWeight: 900, fontSize: '1.05rem', color: 'var(--text-main)' }}>
                  Wordle Records
                </span>
              </div>
              <span
                style={{
                  fontFamily: 'var(--font-handwritten)',
                  fontSize: '1rem',
                  color: '#6E4E37',
                  fontWeight: 700,
                }}
              >
                Princess's picks
              </span>
            </div>

            {/* Stats Metric Grid */}
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(4, 1fr)',
                gap: '6px',
                textAlign: 'center',
                marginBottom: '12px',
              }}
            >
              <div style={{ padding: '6px', backgroundColor: 'var(--panel-cream)', borderRadius: '8px' }}>
                <div style={{ fontSize: '1.15rem', fontWeight: 900 }}>{stats.wordle.games_played}</div>
                <div style={{ fontSize: '0.68rem', fontWeight: 700, color: 'var(--text-muted)' }}>Played</div>
              </div>
              <div style={{ padding: '6px', backgroundColor: 'var(--panel-cream)', borderRadius: '8px' }}>
                <div style={{ fontSize: '1.15rem', fontWeight: 900 }}>{stats.wordle.win_rate.toFixed(0)}%</div>
                <div style={{ fontSize: '0.68rem', fontWeight: 700, color: 'var(--text-muted)' }}>Win %</div>
              </div>
              <div style={{ padding: '6px', backgroundColor: 'var(--panel-cream)', borderRadius: '8px' }}>
                <div style={{ fontSize: '1.15rem', fontWeight: 900 }}>{stats.wordle.current_streak}</div>
                <div style={{ fontSize: '0.68rem', fontWeight: 700, color: 'var(--text-muted)' }}>Streak</div>
              </div>
              <div style={{ padding: '6px', backgroundColor: 'var(--panel-cream)', borderRadius: '8px' }}>
                <div style={{ fontSize: '1.15rem', fontWeight: 900 }}>
                  {stats.wordle.average_guesses > 0 ? stats.wordle.average_guesses.toFixed(1) : '—'}
                </div>
                <div style={{ fontSize: '0.68rem', fontWeight: 700, color: 'var(--text-muted)' }}>Avg Guess</div>
              </div>
            </div>

            {/* Guess Distribution Chart */}
            <div>
              <div
                style={{
                  fontSize: '0.82rem',
                  fontWeight: 800,
                  color: 'var(--text-muted)',
                  marginBottom: '6px',
                }}
              >
                Guess Distribution:
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                {stats.wordle.guess_distribution.map((count, idx) => {
                  const pct = (count / maxGuessCount) * 100;
                  return (
                    <div key={idx} style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                      <span
                        style={{
                          width: '14px',
                          fontWeight: 900,
                          fontSize: '0.8rem',
                          color: 'var(--text-main)',
                        }}
                      >
                        {idx + 1}
                      </span>
                      <div
                        style={{
                          flex: 1,
                          backgroundColor: 'var(--panel-peach)',
                          borderRadius: '5px',
                          overflow: 'hidden',
                          height: '18px',
                          border: '1px solid rgba(96,67,59,0.2)',
                        }}
                      >
                        <div
                          style={{
                            width: count > 0 ? `${Math.max(pct, 12)}%` : '0%',
                            backgroundColor: 'var(--color-success)',
                            height: '100%',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'flex-end',
                            paddingRight: '6px',
                            color: '#ffffff',
                            fontWeight: 900,
                            fontSize: '0.72rem',
                            transition: 'width 0.4s ease',
                          }}
                        >
                          {count > 0 ? count : ''}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        </div>

        {/* Bottom Left Note & Botanical Decor */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '10px' }}>
          <ScrawlyNote
            text="Princess says: Keep average guesses below 4.0 or no extra cat treats!"
            author="Princess"
            rotation={-2}
          />
          <PressedFlower rotation={14} size={36} />
        </div>
      </div>

      {/* ============================================================== */}
      {/* RIGHT PAGE: Spelling Bee & Boggle Records, Reset Records       */}
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
        <div>
          {/* Header & Reset Action */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              marginBottom: '10px',
            }}
          >
            <div>
              <span
                style={{
                  fontFamily: 'var(--font-handwritten)',
                  fontSize: '1.35rem',
                  color: '#6E4E37',
                  fontWeight: 700,
                }}
              >
                ~ Honey & Tabletop Records ~
              </span>
              <div style={{ display: 'flex', gap: '4px', marginTop: '2px' }}>
                <WashiTape color="mint" width={80} rotation={1} />
              </div>
            </div>

            <CozyButton
              variant="cream"
              size="sm"
              onClick={() => {
                sound.playClick();
                setShowResetModal(true);
              }}
              title="Reset player statistics"
            >
              <RotateCcw size={14} />
              <span style={{ fontSize: '0.8rem' }}>Reset</span>
            </CozyButton>
          </div>

          {/* SPELLING BEE RECORDS CARD */}
          <div
            style={{
              backgroundColor: '#FAF5EE',
              borderRadius: '14px',
              border: '2.5px solid var(--outline-cocoa)',
              padding: '12px 14px',
              boxShadow: '0 3px 0 var(--outline-cocoa)',
              marginBottom: '12px',
            }}
          >
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                borderBottom: '2px dashed var(--outline-cocoa)',
                paddingBottom: '5px',
                marginBottom: '8px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <BeeGameIcon size={22} />
                <span style={{ fontWeight: 900, fontSize: '1rem', color: 'var(--text-main)' }}>
                  Spelling Bee Hive
                </span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                <HoneyJarIcon size={20} />
                <span style={{ fontWeight: 900, color: '#b8860b', fontSize: '0.9rem' }}>
                  {stats.spelling_bee.highest_score} pts
                </span>
              </div>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px', textAlign: 'center' }}>
              <div style={{ padding: '6px', backgroundColor: 'var(--panel-cream)', borderRadius: '8px' }}>
                <div style={{ fontSize: '1.1rem', fontWeight: 900 }}>{stats.spelling_bee.games_played}</div>
                <div style={{ fontSize: '0.68rem', fontWeight: 700, color: 'var(--text-muted)' }}>Puzzles</div>
              </div>
              <div style={{ padding: '6px', backgroundColor: 'var(--panel-cream)', borderRadius: '8px' }}>
                <div style={{ fontSize: '1.1rem', fontWeight: 900 }}>{stats.spelling_bee.total_words_found}</div>
                <div style={{ fontSize: '0.68rem', fontWeight: 700, color: 'var(--text-muted)' }}>Words Found</div>
              </div>
              <div style={{ padding: '6px', backgroundColor: 'var(--panel-cream)', borderRadius: '8px' }}>
                <div style={{ fontSize: '1.1rem', fontWeight: 900, color: '#e8590c' }}>
                  {stats.spelling_bee.pangrams_found}
                </div>
                <div style={{ fontSize: '0.68rem', fontWeight: 700, color: 'var(--text-muted)' }}>Pangrams</div>
              </div>
            </div>

            <div
              style={{
                marginTop: '6px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '4px 8px',
                backgroundColor: 'var(--panel-cream)',
                borderRadius: '6px',
                fontSize: '0.78rem',
              }}
            >
              <span style={{ fontWeight: 700, display: 'flex', alignItems: 'center', gap: '4px' }}>
                <CrownQueenIcon size={16} /> Queen Bee Completions:
              </span>
              <span style={{ fontWeight: 900, color: '#b8860b' }}>
                {stats.spelling_bee.queen_bees}
              </span>
            </div>
          </div>

          {/* BOGGLE RECORDS CARD */}
          <div
            style={{
              backgroundColor: '#FAF5EE',
              borderRadius: '14px',
              border: '2.5px solid var(--outline-cocoa)',
              padding: '12px 14px',
              boxShadow: '0 3px 0 var(--outline-cocoa)',
            }}
          >
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                borderBottom: '2px dashed var(--outline-cocoa)',
                paddingBottom: '5px',
                marginBottom: '8px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <BoggleGameIcon size={22} />
                <span style={{ fontWeight: 900, fontSize: '1rem', color: 'var(--text-main)' }}>
                  Boggle Tabletop
                </span>
              </div>
              <span style={{ fontWeight: 900, color: '#2b8a3e', fontSize: '0.9rem' }}>
                Best: {stats.boggle.highest_score} pts
              </span>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px', textAlign: 'center' }}>
              <div style={{ padding: '6px', backgroundColor: 'var(--panel-cream)', borderRadius: '8px' }}>
                <div style={{ fontSize: '1.1rem', fontWeight: 900 }}>{stats.boggle.games_played}</div>
                <div style={{ fontSize: '0.68rem', fontWeight: 700, color: 'var(--text-muted)' }}>Rounds</div>
              </div>
              <div style={{ padding: '6px', backgroundColor: 'var(--panel-cream)', borderRadius: '8px' }}>
                <div style={{ fontSize: '1.1rem', fontWeight: 900 }}>{stats.boggle.total_words_found}</div>
                <div style={{ fontSize: '0.68rem', fontWeight: 700, color: 'var(--text-muted)' }}>Words Found</div>
              </div>
              <div style={{ padding: '6px', backgroundColor: 'var(--panel-cream)', borderRadius: '8px' }}>
                <div style={{ fontSize: '1.1rem', fontWeight: 900 }}>
                  {stats.boggle.average_score.toFixed(1)}
                </div>
                <div style={{ fontSize: '0.68rem', fontWeight: 700, color: 'var(--text-muted)' }}>Avg Score</div>
              </div>
            </div>

            {stats.boggle.best_words && stats.boggle.best_words.length > 0 && (
              <div style={{ marginTop: '8px' }}>
                <div style={{ fontSize: '0.72rem', fontWeight: 800, color: 'var(--text-muted)', marginBottom: '3px' }}>
                  Longest Discovered Words:
                </div>
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
                  {stats.boggle.best_words.slice(0, 6).map((word, i) => (
                    <span
                      key={i}
                      style={{
                        padding: '2px 6px',
                        backgroundColor: 'var(--panel-cream)',
                        border: '1px solid var(--outline-cocoa)',
                        borderRadius: '4px',
                        fontSize: '0.72rem',
                        fontWeight: 800,
                      }}
                    >
                      {word}
                    </span>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Bottom Right Note & Botanical Decor */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '10px', marginTop: '10px' }}>
          <PressedFern rotation={-12} size={36} />
          <ScrawlyNote
            text="Benny loves honey pangrams! Snowy is the speed demon of 4x4 boggle boards."
            author="Benny & Snowy"
            rotation={1.5}
          />
          <DoodleStars size={32} />
        </div>
      </div>

      {/* Confirmation Modal for Resetting Records */}
      <CozyModal
        isOpen={showResetModal}
        onClose={() => setShowResetModal(false)}
        title="Reset Player Records?"
      >
        <div style={{ textAlign: 'center', padding: '10px 0' }}>
          <p style={{ fontWeight: 700, fontSize: '0.95rem', color: 'var(--text-main)', marginBottom: '16px' }}>
            Are you sure you want to clear your lifetime puzzle statistics, guess distribution, and streaks?
            This cannot be undone.
          </p>

          <div style={{ display: 'flex', justifyContent: 'center', gap: '12px' }}>
            <CozyButton
              variant="cream"
              size="md"
              onClick={() => {
                sound.playClick();
                setShowResetModal(false);
              }}
            >
              Keep Records
            </CozyButton>

            <CozyButton
              variant="coral"
              size="md"
              onClick={handleConfirmReset}
            >
              Reset Everything
            </CozyButton>
          </div>
        </div>
      </CozyModal>
    </div>
  );
};
