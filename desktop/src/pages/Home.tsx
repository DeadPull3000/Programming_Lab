import React, { useState } from 'react';
import { GameTab, GamesSummary, UnifiedStats } from '../types/arcade';
import { CozyButton } from '../components/CozyButton';
import { sound } from '../services/sound';
import {
  BennyCat,
  MaggieStrip,
  PrincessCat,
  SnowyCat,
} from '../components/stickers/CustomCats';
import {
  BeeGameIcon,
  BoggleGameIcon,
  CookieEasterEgg,
  CountdownTimerIcon,
  DiaryBookIcon,
  PawPrintIcon,
  SettingsCogIcon,
  StatsRibbonIcon,
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

interface HomeProps {
  onSelectGame: (tab: GameTab) => void;
  summary: GamesSummary | null;
  stats: UnifiedStats | null;
}

export const Home: React.FC<HomeProps> = ({ onSelectGame, summary, stats }) => {
  const [activeSpeech, setActiveSpeech] = useState<string | null>(null);
  const [showGameDrawer, setShowGameDrawer] = useState(false);

  const handleCatClick = (name: string, speech: string) => {
    sound.playPurr();
    setActiveSpeech(`${name}: "${speech}"`);
    setTimeout(() => {
      setActiveSpeech((prev) => (prev?.startsWith(name) ? null : prev));
    }, 4500);
  };

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        minHeight: '560px',
        position: 'relative',
      }}
    >
      {/* Speech bubble overlay */}
      {activeSpeech && (
        <div
          className="animate-pop"
          style={{
            position: 'absolute',
            top: '16px',
            left: '50%',
            transform: 'translateX(-50%)',
            zIndex: 40,
            backgroundColor: '#FFFBEA',
            border: '2.5px solid var(--outline-cocoa)',
            borderRadius: 'var(--radius-pill)',
            padding: '8px 20px',
            boxShadow: '0 4px 12px rgba(96, 67, 59, 0.2)',
            fontWeight: 800,
            fontSize: '0.92rem',
            color: 'var(--text-main)',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
          }}
        >
          <PawPrintIcon size={18} />
          <span>{activeSpeech}</span>
        </div>
      )}

      {/* ============================================================== */}
      {/* LEFT PAGE: Interactive Menu Buttons, Notes & Botanical Stickers */}
      {/* ============================================================== */}
      <div
        style={{
          padding: '30px 24px 30px 32px',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'space-between',
          position: 'relative',
          borderRight: '1px solid rgba(96,67,59,0.15)',
        }}
      >
        {/* Top left handwritten scrapbook title */}
        <div
          style={{
            alignSelf: 'flex-start',
            transform: 'rotate(-2deg)',
            marginBottom: '6px',
          }}
        >
          <span
            style={{
              fontFamily: 'var(--font-handwritten)',
              fontSize: '1.45rem',
              color: '#6E4E37',
              fontWeight: 700,
            }}
          >
            ~ Word Arcade Scrapbook ~
          </span>
          <div style={{ display: 'flex', gap: '4px', marginTop: '2px' }}>
            <WashiTape color="lavender" width={60} rotation={-1} />
          </div>
        </div>

        {/* Scalloped Pink PLAY Badge */}
        <div style={{ textAlign: 'center', margin: '4px 0 10px' }}>
          <button
            className="scalloped-play-badge"
            onClick={() => {
              sound.playSuccess();
              if (summary?.spelling_bee_active) {
                onSelectGame('spelling-bee');
              } else if (summary?.boggle_active) {
                onSelectGame('boggle');
              } else {
                onSelectGame('wordle');
              }
            }}
            title="Start Playing Word Games"
          >
            <div
              style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                gap: '2px',
              }}
            >
              <span
                style={{
                  fontFamily: 'var(--font-heading)',
                  fontSize: '2rem',
                  fontWeight: 900,
                  color: '#4e2a33',
                  letterSpacing: '1px',
                }}
              >
                PLAY
              </span>
            </div>
          </button>
        </div>

        {/* Washi-Tape Menu Buttons */}
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: '12px',
            width: '100%',
            maxWidth: '220px',
            alignItems: 'center',
          }}
        >
          <button
            onClick={() => {
              sound.playClick();
              onSelectGame('settings');
            }}
            className="washi-tape washi-tape-mint"
            style={{
              width: '100%',
              textAlign: 'center',
              fontSize: '1.15rem',
              fontWeight: 800,
              cursor: 'pointer',
              transform: 'rotate(-1.5deg)',
            }}
          >
            SETTING
          </button>

          <button
            onClick={() => {
              sound.playClick();
              onSelectGame('stats');
            }}
            className="washi-tape washi-tape-coral"
            style={{
              width: '100%',
              textAlign: 'center',
              fontSize: '1.15rem',
              fontWeight: 800,
              cursor: 'pointer',
              transform: 'rotate(1.5deg)',
            }}
          >
            STATISTICS
          </button>

          <button
            onClick={() => {
              sound.playClick();
              setShowGameDrawer(!showGameDrawer);
            }}
            className="washi-tape washi-tape-yellow"
            style={{
              width: '100%',
              textAlign: 'center',
              fontSize: '1.1rem',
              fontWeight: 800,
              cursor: 'pointer',
              transform: 'rotate(-0.8deg)',
            }}
          >
            {showGameDrawer ? 'CLOSE DRAWER ▲' : 'CHOOSE GAME ▼'}
          </button>
        </div>

        {/* Scrawly Handwritten Memo Note in Left Page Margin */}
        <div style={{ marginTop: '8px' }}>
          <ScrawlyNote
            title="Archivist's Diary Note"
            text="Flip through the diary tabs to play Wordle, Spelling Bee, or Boggle anytime!"
            author="The Garden Keeper"
            tapeColor="coral"
            rotation={-2}
            width={210}
          />
        </div>

        {/* Pressed Botanical Elements at Page Bottom */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            width: '100%',
            alignItems: 'flex-end',
            paddingTop: '6px',
          }}
        >
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
            <WashiTape color="mint" width={48} rotation={-6} />
            <PressedFern size={52} rotation={-14} />
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
            <WashiTape color="yellow" width={48} rotation={4} />
            <PressedFlower size={44} rotation={8} />
          </div>
        </div>
      </div>

      {/* ============================================================== */}
      {/* RIGHT PAGE: Custom Vector Cat Scrapbook & Handwritten Notes    */}
      {/* ============================================================== */}
      <div
        style={{
          padding: '24px 28px 24px 24px',
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'space-between',
          position: 'relative',
        }}
      >
        {/* Top Stamp & Doodle Stars */}
        <div
          style={{
            position: 'absolute',
            top: '14px',
            right: '24px',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
          }}
        >
          <PostageStamp label="8¢" illustration="bee" rotation={6} size={48} />
          <DoodleStars size={22} />
        </div>

        {/* Top Row: Princess Oval Portrait & Maggie Filmstrip */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
          {/* PRINCESS (Custom Vector Illustration) */}
          <div
            onClick={() =>
              handleCatClick(
                'Princess',
                'A royal welcome! Wordle has 629 curated 5-letter words. Shall we test your vocabulary?'
              )
            }
            style={{
              cursor: 'pointer',
              textAlign: 'center',
              transform: 'rotate(-2deg)',
            }}
            className="animate-float"
          >
            <PrincessCat size={115} />
            <div
              style={{
                backgroundColor: 'var(--panel-cream)',
                border: '2px solid var(--outline-cocoa)',
                borderRadius: 'var(--radius-pill)',
                padding: '2px 12px',
                fontSize: '0.85rem',
                fontWeight: 900,
                marginTop: '-6px',
                display: 'inline-block',
                boxShadow: '0 2px 0 var(--outline-cocoa)',
                fontFamily: 'var(--font-heading)',
              }}
            >
              PRINCESS
            </div>
            {/* Scrawly note under Princess */}
            <div
              style={{
                fontFamily: 'var(--font-handwritten)',
                fontSize: '0.95rem',
                color: '#8C6E5A',
                marginTop: '2px',
              }}
            >
              "Her royal highness loves Wordle"
            </div>
          </div>

          {/* MAGGIE (3-Frame Filmstrip) */}
          <div
            onClick={() =>
              handleCatClick(
                'Maggie',
                'Purrr... Check your guess distribution chart in the Scrapbook tab!'
              )
            }
            style={{
              cursor: 'pointer',
              textAlign: 'center',
              transform: 'rotate(2.5deg)',
              marginRight: '12px',
            }}
          >
            <MaggieStrip size={50} />
            <span
              style={{
                fontSize: '0.8rem',
                fontWeight: 900,
                color: 'var(--text-main)',
                marginTop: '3px',
                display: 'block',
                fontFamily: 'var(--font-heading)',
              }}
            >
              MAGGIE
            </span>
          </div>
        </div>

        {/* Center handwritten annotation filling blank space */}
        <div
          style={{
            fontFamily: 'var(--font-handwritten)',
            fontSize: '1.15rem',
            color: '#6B4F39',
            textAlign: 'center',
            margin: '4px 0',
            lineHeight: '1.2',
          }}
        >
          ✧ Tap any portrait to hear their purr! ✧
        </div>

        {/* Bottom Row: Benny & Snowy */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-around',
            alignItems: 'flex-end',
          }}
        >
          {/* BENNY (Ginger Kitten) */}
          <div
            onClick={() =>
              handleCatClick(
                'Benny',
                'Mew! In Spelling Bee, pangrams use all seven letters for huge points!'
              )
            }
            style={{
              cursor: 'pointer',
              textAlign: 'center',
              transform: 'rotate(2deg)',
            }}
          >
            <BennyCat size={92} />
            <span
              style={{
                fontSize: '0.85rem',
                fontWeight: 900,
                color: 'var(--text-main)',
                marginTop: '2px',
                display: 'block',
                fontFamily: 'var(--font-heading)',
              }}
            >
              BENNY
            </span>
            <div
              style={{
                fontFamily: 'var(--font-handwritten)',
                fontSize: '0.9rem',
                color: '#8C6E5A',
              }}
            >
              honeycomb lover
            </div>
          </div>

          {/* SNOWY (Fluffy White Cat with Butterfly) */}
          <div
            onClick={() =>
              handleCatClick(
                'Snowy',
                'Paws ready! In Boggle, connect letters horizontally, vertically, or diagonally!'
              )
            }
            style={{
              cursor: 'pointer',
              textAlign: 'center',
              transform: 'rotate(-3deg)',
            }}
          >
            <SnowyCat size={102} />
            <span
              style={{
                fontSize: '0.85rem',
                fontWeight: 900,
                color: 'var(--text-main)',
                marginTop: '-4px',
                display: 'block',
                fontFamily: 'var(--font-heading)',
              }}
            >
              SNOWY
            </span>
            <div
              style={{
                fontFamily: 'var(--font-handwritten)',
                fontSize: '0.9rem',
                color: '#8C6E5A',
              }}
            >
              boggle champion
            </div>
          </div>
        </div>
      </div>

      {/* GAME SELECTOR OVERLAY DRAWER */}
      {showGameDrawer && (
        <div
          className="animate-pop"
          style={{
            position: 'absolute',
            inset: '16px',
            backgroundColor: 'rgba(255, 251, 240, 0.98)',
            border: '3.5px solid var(--outline-cocoa)',
            borderRadius: '20px',
            padding: '24px',
            boxShadow: '0 10px 24px rgba(0,0,0,0.3)',
            zIndex: 45,
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-between',
          }}
        >
          <div style={{ textAlign: 'center', borderBottom: '2px dashed var(--outline-cocoa)', paddingBottom: '10px' }}>
            <h2 style={{ fontSize: '1.6rem', color: 'var(--text-main)' }}>CHOOSE A WORD GAME</h2>
            <p style={{ color: 'var(--text-muted)', fontWeight: 600 }}>
              Flip to any game page or select below:
            </p>
          </div>

          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(4, 1fr)',
              gap: '12px',
              margin: '16px 0',
            }}
          >
            {/* Wordle Card */}
            <div
              className="cozy-card"
              style={{
                padding: '14px',
                backgroundColor: '#FAF5EE',
                display: 'flex',
                flexDirection: 'column',
                justifyContent: 'space-between',
              }}
            >
              <div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <WordleGameIcon size={30} />
                  {summary?.wordle_active && (
                    <span style={{ fontSize: '0.7rem', fontWeight: 800, color: '#2b8a3e' }}>● ACTIVE</span>
                  )}
                </div>
                <h3 style={{ fontSize: '1.15rem', marginTop: '6px' }}>Wordle</h3>
                <p style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                  5-letter word puzzle with two-pass color feedback & Shannon entropy solver.
                </p>
              </div>
              <CozyButton variant="coral" size="sm" onClick={() => onSelectGame('wordle')}>
                PLAY WORDLE
              </CozyButton>
            </div>

            {/* Spelling Bee Card */}
            <div
              className="cozy-card"
              style={{
                padding: '14px',
                backgroundColor: '#FAF5EE',
                display: 'flex',
                flexDirection: 'column',
                justifyContent: 'space-between',
              }}
            >
              <div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <BeeGameIcon size={30} />
                  {summary?.spelling_bee_active && (
                    <span style={{ fontSize: '0.7rem', fontWeight: 800, color: '#2b8a3e' }}>● ACTIVE</span>
                  )}
                </div>
                <h3 style={{ fontSize: '1.15rem', marginTop: '6px' }}>Spelling Bee</h3>
                <p style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                  7-letter honeycomb puzzle with mandatory center letter & pangrams.
                </p>
              </div>
              <CozyButton variant="yellow" size="sm" onClick={() => onSelectGame('spelling-bee')}>
                PLAY SPELLING BEE
              </CozyButton>
            </div>

            {/* Boggle Card */}
            <div
              className="cozy-card"
              style={{
                padding: '14px',
                backgroundColor: '#FAF5EE',
                display: 'flex',
                flexDirection: 'column',
                justifyContent: 'space-between',
              }}
            >
              <div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <BoggleGameIcon size={30} />
                  {summary?.boggle_active && (
                    <span style={{ fontSize: '0.7rem', fontWeight: 800, color: '#2b8a3e' }}>● ACTIVE</span>
                  )}
                </div>
                <h3 style={{ fontSize: '1.15rem', marginTop: '6px' }}>Boggle</h3>
                <p style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                  Tabletop word grid with interactive path tracing & timed round countdown.
                </p>
              </div>
              <CozyButton variant="mint" size="sm" onClick={() => onSelectGame('boggle')}>
                PLAY BOGGLE
              </CozyButton>
            </div>

            {/* Countdown Timer Card */}
            <div
              className="cozy-card"
              style={{
                padding: '14px',
                backgroundColor: '#FAF5EE',
                display: 'flex',
                flexDirection: 'column',
                justifyContent: 'space-between',
              }}
            >
              <div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <CountdownTimerIcon size={30} />
                  <span style={{ fontSize: '0.7rem', fontWeight: 800, color: '#d97706' }}>● ASSIGN 5</span>
                </div>
                <h3 style={{ fontSize: '1.15rem', marginTop: '6px' }}>Timer & Routines</h3>
                <p style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                  Drift-free clock, 100ms precision, 11-step exercise circuit, and concurrent timers.
                </p>
              </div>
              <CozyButton variant="blue" size="sm" onClick={() => onSelectGame('timer')}>
                OPEN TIMER
              </CozyButton>
            </div>
          </div>

          <div style={{ textAlign: 'center' }}>
            <CozyButton variant="cream" size="sm" onClick={() => setShowGameDrawer(false)}>
              Close Drawer
            </CozyButton>
          </div>
        </div>
      )}
    </div>
  );
};
