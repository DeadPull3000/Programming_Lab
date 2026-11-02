import React from 'react';
import {
  Volume2,
  VolumeX,
} from 'lucide-react';
import { GameTab, GamesSummary } from '../types/arcade';
import { sound } from '../services/sound';
import {
  BeeGameIcon,
  BoggleGameIcon,
  DiaryBookIcon,
  PawPrintIcon,
  SettingsCogIcon,
  StatsRibbonIcon,
  WordleGameIcon,
} from './stickers/CustomIcons';

interface TopNavigationProps {
  currentTab: GameTab;
  onSelectTab: (tab: GameTab) => void;
  summary: GamesSummary | null;
  soundEnabled: boolean;
  onToggleSound: () => void;
}

export const TopNavigation: React.FC<TopNavigationProps> = ({
  currentTab,
  onSelectTab,
  summary,
  soundEnabled,
  onToggleSound,
}) => {
  const tabs: { id: GameTab; label: string; icon: React.ReactNode; isActiveGame?: boolean }[] = [
    { id: 'home', label: 'Home', icon: <DiaryBookIcon size={18} /> },
    {
      id: 'wordle',
      label: 'Wordle',
      icon: <WordleGameIcon size={18} />,
      isActiveGame: summary?.wordle_active,
    },
    {
      id: 'spelling-bee',
      label: 'Spelling Bee',
      icon: <BeeGameIcon size={18} />,
      isActiveGame: summary?.spelling_bee_active,
    },
    {
      id: 'boggle',
      label: 'Boggle',
      icon: <BoggleGameIcon size={18} />,
      isActiveGame: summary?.boggle_active,
    },
    { id: 'stats', label: 'Stats', icon: <StatsRibbonIcon size={18} /> },
    { id: 'settings', label: 'Settings', icon: <SettingsCogIcon size={18} /> },
  ];

  return (
    <header
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '12px 24px',
        backgroundColor: 'var(--panel-cream)',
        borderBottom: '3.5px solid var(--outline-cocoa)',
        boxShadow: '0 4px 12px rgba(96, 67, 59, 0.1)',
        position: 'sticky',
        top: 0,
        zIndex: 50,
      }}
    >
      {/* Brand logo & title */}
      <div
        onClick={() => {
          sound.playClick();
          onSelectTab('home');
        }}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '10px',
          cursor: 'pointer',
        }}
      >
        <div
          style={{
            width: '40px',
            height: '40px',
            borderRadius: '50%',
            backgroundColor: 'var(--highlight-yellow)',
            border: '2.5px solid var(--outline-cocoa)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            boxShadow: '0 3px 0 var(--outline-cocoa)',
          }}
        >
          <PawPrintIcon size={24} />
        </div>
        <div>
          <h1
            style={{
              fontSize: '1.35rem',
              lineHeight: 1.1,
              color: 'var(--text-main)',
              letterSpacing: '0.5px',
            }}
          >
            THE WORD GARDEN
          </h1>
          <p
            style={{
              fontSize: '0.75rem',
              color: 'var(--text-muted)',
              fontWeight: 700,
            }}
          >
            A cozy word-game arcade
          </p>
        </div>
      </div>

      {/* Navigation tabs strip */}
      <nav
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          backgroundColor: 'var(--panel-peach)',
          padding: '6px 8px',
          borderRadius: 'var(--radius-pill)',
          border: '2.5px solid var(--outline-cocoa)',
        }}
      >
        {tabs.map((tab) => {
          const isSelected = currentTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => {
                sound.playClick();
                onSelectTab(tab.id);
              }}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '6px 14px',
                borderRadius: 'var(--radius-pill)',
                border: isSelected ? '2px solid var(--outline-cocoa)' : '2px solid transparent',
                backgroundColor: isSelected ? 'var(--panel-cream)' : 'transparent',
                boxShadow: isSelected ? '0 3px 0 var(--outline-cocoa)' : 'none',
                color: isSelected ? 'var(--text-main)' : 'var(--text-muted)',
                fontWeight: 700,
                fontSize: '0.92rem',
                transform: isSelected ? 'translateY(-1px)' : 'none',
                position: 'relative',
                transition: 'all 0.15s ease',
              }}
            >
              {tab.icon}
              <span>{tab.label}</span>
              {tab.isActiveGame && (
                <span
                  title="Game session active in memory"
                  style={{
                    width: '8px',
                    height: '8px',
                    borderRadius: '50%',
                    backgroundColor: 'var(--color-success)',
                    border: '1.5px solid var(--outline-cocoa)',
                    marginLeft: '2px',
                    animation: 'tilePop 1.5s infinite',
                  }}
                />
              )}
            </button>
          );
        })}
      </nav>

      {/* Quick audio toggle */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <button
          onClick={onToggleSound}
          title={soundEnabled ? 'Mute Sounds' : 'Unmute Sounds'}
          style={{
            width: '38px',
            height: '38px',
            borderRadius: '50%',
            backgroundColor: soundEnabled ? 'var(--bg-mint)' : '#E7D8D5',
            border: '2.5px solid var(--outline-cocoa)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            boxShadow: '0 3px 0 var(--outline-cocoa)',
            color: 'var(--text-main)',
            cursor: 'pointer',
          }}
        >
          {soundEnabled ? <Volume2 size={18} /> : <VolumeX size={18} />}
        </button>
      </div>
    </header>
  );
};
