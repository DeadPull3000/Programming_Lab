import React, { useState } from 'react';
import {
  ChevronLeft,
  ChevronRight,
  Sparkles,
} from 'lucide-react';
import { GameTab, GamesSummary } from '../types/arcade';
import { sound } from '../services/sound';
import {
  BeeGameIcon,
  BoggleGameIcon,
  CookieEasterEgg,
  DiaryBookIcon,
  PawPrintIcon,
  SettingsCogIcon,
  StatsRibbonIcon,
  WordleGameIcon,
} from './stickers/CustomIcons';
import { PostageStamp, PressedFern, WashiTape } from './stickers/DiaryStickers';

interface DiaryBookProps {
  currentTab: GameTab;
  onSelectTab: (tab: GameTab) => void;
  summary: GamesSummary | null;
  children: React.ReactNode;
}

const TABS: { id: GameTab; label: string; icon: React.ReactNode; color: string }[] = [
  { id: 'home', label: 'Gallery', icon: <DiaryBookIcon size={18} />, color: '#F48E88' },
  { id: 'wordle', label: 'Wordle', icon: <WordleGameIcon size={18} />, color: '#8CCB79' },
  { id: 'spelling-bee', label: 'Spelling Bee', icon: <BeeGameIcon size={18} />, color: '#FFD56D' },
  { id: 'boggle', label: 'Boggle', icon: <BoggleGameIcon size={18} />, color: '#93DFE8' },
  { id: 'stats', label: 'Scrapbook', icon: <StatsRibbonIcon size={18} />, color: '#EEDDF7' },
  { id: 'settings', label: 'Settings', icon: <SettingsCogIcon size={18} />, color: '#F5D5B9' },
];

export const DiaryBook: React.FC<DiaryBookProps> = ({
  currentTab,
  onSelectTab,
  summary,
  children,
}) => {
  const [flipDirection, setFlipDirection] = useState<'next' | 'prev' | null>(null);
  const [cookieEaten, setCookieEaten] = useState(false);
  const [catSpeech, setCatSpeech] = useState<string | null>(null);

  const currentIndex = TABS.findIndex((t) => t.id === currentTab);
  const prevTabRef = React.useRef<GameTab>(currentTab);

  React.useEffect(() => {
    if (prevTabRef.current !== currentTab) {
      const prevIdx = TABS.findIndex((t) => t.id === prevTabRef.current);
      const nextIdx = TABS.findIndex((t) => t.id === currentTab);
      setFlipDirection(nextIdx > prevIdx ? 'next' : 'prev');
      sound.playPageFlip();
      const timer = setTimeout(() => {
        setFlipDirection(null);
      }, 320);
      prevTabRef.current = currentTab;
      return () => clearTimeout(timer);
    }
  }, [currentTab]);

  const flipToTab = (tabId: GameTab) => {
    if (tabId === currentTab) return;
    onSelectTab(tabId);
  };

  const handlePrevPage = () => {
    if (currentIndex > 0) {
      flipToTab(TABS[currentIndex - 1].id);
    }
  };

  const handleNextPage = () => {
    if (currentIndex < TABS.length - 1) {
      flipToTab(TABS[currentIndex + 1].id);
    }
  };

  const handleCookieClick = () => {
    sound.playPop(540);
    setCookieEaten(true);
    setCatSpeech('Crunch! A warm chocolate chip cookie! Brain fuel for word games.');
    setTimeout(() => setCookieEaten(false), 8000);
    setTimeout(() => setCatSpeech(null), 4000);
  };

  return (
    <div
      style={{
        minHeight: 'calc(100vh - 68px)',
        backgroundColor: '#8B5E3C',
        backgroundImage: `
          repeating-linear-gradient(
            0deg,
            rgba(60, 35, 20, 0.08) 0px,
            rgba(60, 35, 20, 0.08) 1px,
            transparent 1px,
            transparent 14px
          ),
          radial-gradient(#9e6b46 15%, #7a4f30 90%)
        `,
        padding: '20px 16px 85px',
        position: 'relative',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        overflow: 'hidden',
      }}
    >
      {/* Wooden Desk Accessories */}
      {/* Pencils on the left */}
      <div
        style={{
          position: 'absolute',
          left: '24px',
          top: '70px',
          display: 'flex',
          flexDirection: 'column',
          gap: '6px',
          pointerEvents: 'none',
          opacity: 0.85,
          zIndex: 1,
        }}
      >
        <div
          style={{
            width: '10px',
            height: '140px',
            background: 'linear-gradient(to bottom, #4a3b32 18px, #c89d70 18px)',
            borderRadius: '4px',
            transform: 'rotate(16deg)',
            boxShadow: '2px 4px 6px rgba(0,0,0,0.3)',
          }}
        />
        <div
          style={{
            width: '8px',
            height: '160px',
            background: 'linear-gradient(to bottom, #f3d4a0 14px, #48bfe3 14px)',
            borderRadius: '3px',
            transform: 'rotate(22deg) translateY(-20px)',
            boxShadow: '2px 4px 6px rgba(0,0,0,0.3)',
          }}
        />
        <div
          style={{
            width: '8px',
            height: '170px',
            background: 'linear-gradient(to bottom, #f3d4a0 14px, #f77f00 14px)',
            borderRadius: '3px',
            transform: 'rotate(28deg) translateY(-40px)',
            boxShadow: '2px 4px 6px rgba(0,0,0,0.3)',
          }}
        />
      </div>

      {/* Decorative Stamp on desk */}
      <div style={{ position: 'absolute', left: '30px', top: '24px', zIndex: 1 }}>
        <PostageStamp label="15¢" illustration="flower" rotation={-14} size={50} />
      </div>

      {/* Chocolate Chip Cookie with Crumbs on desk bottom-right */}
      <div
        style={{
          position: 'absolute',
          right: '30px',
          bottom: '25px',
          zIndex: 20,
        }}
      >
        <CookieEasterEgg size={82} onClick={handleCookieClick} />
      </div>

      {/* Cat Speech Bubble */}
      {catSpeech && (
        <div
          className="animate-pop"
          style={{
            position: 'sticky',
            top: '72px',
            zIndex: 40,
            backgroundColor: 'var(--panel-cream)',
            border: '3px solid var(--outline-cocoa)',
            borderRadius: 'var(--radius-pill)',
            padding: '8px 22px',
            boxShadow: '0 6px 16px rgba(0,0,0,0.25)',
            fontWeight: 800,
            fontSize: '0.95rem',
            color: 'var(--text-main)',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            marginBottom: '10px',
          }}
        >
          <PawPrintIcon size={20} />
          <span>{catSpeech}</span>
        </div>
      )}

      {/* BOOKMARK TABS AT TOP OF DIARY */}
      <div
        style={{
          width: '100%',
          maxWidth: '960px',
          display: 'flex',
          justifyContent: 'flex-start',
          gap: '8px',
          paddingLeft: '32px',
          marginBottom: '-3px',
          zIndex: 10,
        }}
      >
        {TABS.map((tab, idx) => {
          const isCurrent = tab.id === currentTab;
          const hasActiveGame =
            (tab.id === 'wordle' && summary?.wordle_active) ||
            (tab.id === 'spelling-bee' && summary?.spelling_bee_active) ||
            (tab.id === 'boggle' && summary?.boggle_active);

          return (
            <button
              key={tab.id}
              onClick={() => flipToTab(tab.id)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: isCurrent ? '8px 18px 10px' : '6px 14px 6px',
                backgroundColor: tab.color,
                color: 'var(--text-main)',
                fontFamily: 'var(--font-heading)',
                fontSize: '0.92rem',
                fontWeight: 800,
                border: '3px solid var(--outline-cocoa)',
                borderBottom: isCurrent ? '3px solid var(--panel-cream)' : 'none',
                borderRadius: '12px 12px 0 0',
                cursor: 'pointer',
                transform: isCurrent ? 'translateY(-2px)' : 'none',
                boxShadow: isCurrent
                  ? '0 -3px 6px rgba(0,0,0,0.15)'
                  : '0 -1px 3px rgba(0,0,0,0.1)',
                transition: 'all 0.15s ease',
                position: 'relative',
              }}
            >
              {tab.icon}
              <span>{tab.label}</span>
              {hasActiveGame && (
                <span
                  title="Game active"
                  style={{
                    width: '8px',
                    height: '8px',
                    borderRadius: '50%',
                    backgroundColor: '#2b8a3e',
                    border: '1px solid var(--outline-cocoa)',
                  }}
                />
              )}
            </button>
          );
        })}
      </div>

      {/* THE OPEN DIARY JOURNAL CONTAINER */}
      <div
        style={{
          width: '100%',
          maxWidth: '980px',
          backgroundColor: '#4e5473', // Outer leather binding
          borderRadius: '26px',
          padding: '14px',
          boxShadow: '0 16px 36px rgba(0, 0, 0, 0.45), 0 4px 0 #2e334a',
          border: '4px solid #2e334a',
          position: 'relative',
          perspective: '2000px',
        }}
      >
        {/* Leather buckle strap on the right */}
        <div
          style={{
            position: 'absolute',
            right: '-18px',
            top: '46%',
            width: '38px',
            height: '70px',
            backgroundColor: '#4e5473',
            border: '3px solid #2e334a',
            borderRadius: '0 16px 16px 0',
            boxShadow: '4px 4px 8px rgba(0,0,0,0.3)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <div
            style={{
              width: '12px',
              height: '12px',
              borderRadius: '50%',
              backgroundColor: '#d8b068',
              border: '2px solid #554120',
            }}
          />
        </div>

        {/* Double-page open spread with center spine shadow */}
        <div
          style={{
            backgroundColor: '#FAF0DE', // Rich creamy paper
            borderRadius: '16px',
            border: '3.5px solid var(--outline-cocoa)',
            minHeight: '560px',
            position: 'relative',
            overflow: 'hidden',
            boxShadow: 'inset 0 0 35px rgba(184, 150, 110, 0.35)',
            transformStyle: 'preserve-3d',
            transition: 'transform 0.28s ease',
            transform:
              flipDirection === 'next'
                ? 'rotateY(-2deg) scale(0.99)'
                : flipDirection === 'prev'
                ? 'rotateY(2deg) scale(0.99)'
                : 'none',
          }}
        >
          {/* Center Book Spine Fold Shadow */}
          <div
            style={{
              position: 'absolute',
              top: 0,
              bottom: 0,
              left: '50%',
              width: '30px',
              transform: 'translateX(-50%)',
              background:
                'linear-gradient(to right, rgba(96,67,59,0.18), rgba(96,67,59,0.35), rgba(96,67,59,0.18))',
              pointerEvents: 'none',
              zIndex: 30,
            }}
          />

          {/* Book Stitching Down Spine */}
          <div
            style={{
              position: 'absolute',
              top: '10px',
              bottom: '10px',
              left: '50%',
              width: '2px',
              transform: 'translateX(-50%)',
              borderLeft: '2px dashed #8c6e5a',
              pointerEvents: 'none',
              zIndex: 31,
            }}
          />

          {/* PAGE CONTENT CONTAINER */}
          <div style={{ position: 'relative', zIndex: 10 }}>{children}</div>
        </div>

        {/* Dog-ear Page Flippers: Previous & Next Page controls at bottom */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            padding: '8px 20px 2px',
            fontSize: '0.85rem',
            fontWeight: 800,
            fontFamily: 'var(--font-heading)',
            color: '#FAF0DE',
          }}
        >
          {currentIndex > 0 ? (
            <button
              onClick={handlePrevPage}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
                color: '#FAF0DE',
                cursor: 'pointer',
                opacity: 0.9,
              }}
            >
              <ChevronLeft size={16} />
              <span>Flip back: {TABS[currentIndex - 1].label}</span>
            </button>
          ) : (
            <span style={{ opacity: 0.4 }}>First Page</span>
          )}

          <span style={{ fontSize: '0.8rem', opacity: 0.75 }}>
            Page {currentIndex + 1} of {TABS.length}
          </span>

          {currentIndex < TABS.length - 1 ? (
            <button
              onClick={handleNextPage}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
                color: '#FAF0DE',
                cursor: 'pointer',
                opacity: 0.9,
              }}
            >
              <span>Flip to: {TABS[currentIndex + 1].label}</span>
              <ChevronRight size={16} />
            </button>
          ) : (
            <span style={{ opacity: 0.4 }}>Back Cover</span>
          )}
        </div>
      </div>

      {/* TWO CHUBBY CAT PAWS RESTING AT THE BOTTOM OF THE DESK */}
      <div
        style={{
          position: 'fixed',
          bottom: '-14px',
          display: 'flex',
          gap: '240px',
          zIndex: 35,
          pointerEvents: 'none',
        }}
      >
        {/* Left Paw */}
        <div
          style={{
            width: '110px',
            height: '95px',
            backgroundColor: '#FAF0DE',
            border: '4px solid #60433B',
            borderRadius: '55px 55px 0 0',
            boxShadow: '0 -4px 10px rgba(0,0,0,0.25)',
            display: 'flex',
            alignItems: 'flex-end',
            justifyContent: 'center',
            paddingBottom: '20px',
            position: 'relative',
          }}
        >
          <div
            style={{
              width: '32px',
              height: '24px',
              borderRadius: '50%',
              backgroundColor: '#F8B4BD',
              border: '2px solid #60433B',
            }}
          />
          <div
            style={{
              position: 'absolute',
              top: '18px',
              left: '20px',
              width: '14px',
              height: '16px',
              borderRadius: '50%',
              backgroundColor: '#F8B4BD',
              border: '1.5px solid #60433B',
            }}
          />
          <div
            style={{
              position: 'absolute',
              top: '10px',
              left: '46px',
              width: '16px',
              height: '18px',
              borderRadius: '50%',
              backgroundColor: '#F8B4BD',
              border: '1.5px solid #60433B',
            }}
          />
          <div
            style={{
              position: 'absolute',
              top: '18px',
              right: '20px',
              width: '14px',
              height: '16px',
              borderRadius: '50%',
              backgroundColor: '#F8B4BD',
              border: '1.5px solid #60433B',
            }}
          />
        </div>

        {/* Right Paw */}
        <div
          style={{
            width: '110px',
            height: '95px',
            backgroundColor: '#FAF0DE',
            border: '4px solid #60433B',
            borderRadius: '55px 55px 0 0',
            boxShadow: '0 -4px 10px rgba(0,0,0,0.25)',
            display: 'flex',
            alignItems: 'flex-end',
            justifyContent: 'center',
            paddingBottom: '20px',
            position: 'relative',
          }}
        >
          <div
            style={{
              width: '32px',
              height: '24px',
              borderRadius: '50%',
              backgroundColor: '#F8B4BD',
              border: '2px solid #60433B',
            }}
          />
          <div
            style={{
              position: 'absolute',
              top: '18px',
              left: '20px',
              width: '14px',
              height: '16px',
              borderRadius: '50%',
              backgroundColor: '#F8B4BD',
              border: '1.5px solid #60433B',
            }}
          />
          <div
            style={{
              position: 'absolute',
              top: '10px',
              left: '46px',
              width: '16px',
              height: '18px',
              borderRadius: '50%',
              backgroundColor: '#F8B4BD',
              border: '1.5px solid #60433B',
            }}
          />
          <div
            style={{
              position: 'absolute',
              top: '18px',
              right: '20px',
              width: '14px',
              height: '16px',
              borderRadius: '50%',
              backgroundColor: '#F8B4BD',
              border: '1.5px solid #60433B',
            }}
          />
        </div>
      </div>
    </div>
  );
};
