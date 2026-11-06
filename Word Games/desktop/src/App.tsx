import React, { useEffect, useState } from 'react';
import { GameTab, GamesSummary, SystemStatus, UnifiedStats, ArcadeSettings } from './types/arcade';
import { arcadeApi } from './services/api';
import { Home } from './pages/Home';
import { WordlePage } from './pages/Wordle';
import { SpellingBeePage } from './pages/SpellingBee';
import { BogglePage } from './pages/Boggle';
import { StatisticsPage } from './pages/Statistics';
import { SettingsPage } from './pages/Settings';
import { sound } from './services/sound';
import { DiaryBook } from './components/DiaryBook';

export const App: React.FC = () => {
  const [currentTab, setCurrentTab] = useState<GameTab>('home');
  const [summary, setSummary] = useState<GamesSummary | null>(null);
  const [stats, setStats] = useState<UnifiedStats | null>(null);
  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
  const [settings, setSettings] = useState<ArcadeSettings>({
    sound_enabled: true,
    music_enabled: false,
    reduced_motion: false,
  });

  // Initial load
  const loadInitialData = async () => {
    try {
      const [statusData, summaryData, statsData, settingsData] = await Promise.all([
        arcadeApi.getStatus().catch(() => null),
        arcadeApi.getSummary().catch(() => null),
        arcadeApi.getStats().catch(() => null),
        arcadeApi.getSettings().catch(() => null),
      ]);

      if (statusData) setSystemStatus(statusData);
      if (summaryData) setSummary(summaryData);
      if (statsData) setStats(statsData);
      if (settingsData) {
        setSettings(settingsData);
        sound.setEnabled(settingsData.sound_enabled);
        if (settingsData.reduced_motion) {
          document.documentElement.classList.add('reduced-motion');
        }
      }
    } catch (e) {
      console.warn('Initial data load notice:', e);
    }
  };

  useEffect(() => {
    loadInitialData();
  }, []);

  const refreshSummaryAndStats = async () => {
    try {
      const [summaryData, statsData] = await Promise.all([
        arcadeApi.getSummary(),
        arcadeApi.getStats(),
      ]);
      setSummary(summaryData);
      setStats(statsData);
    } catch (e) {
      // quiet refresh
    }
  };

  const handleToggleSound = () => {
    const nextVal = !settings.sound_enabled;
    const updated = { ...settings, sound_enabled: nextVal };
    setSettings(updated);
    sound.setEnabled(nextVal);
    if (nextVal) {
      sound.playPop(520);
    }
    arcadeApi.updateSettings({ sound_enabled: nextVal }).catch(() => {});
  };

  const handleUpdateSettings = (newSettings: Partial<ArcadeSettings>) => {
    const merged = { ...settings, ...newSettings };
    setSettings(merged);
    arcadeApi.updateSettings(newSettings).catch(() => {});
  };

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        minHeight: '100vh',
      }}
    >
      <main style={{ flex: 1 }}>
        <DiaryBook
          currentTab={currentTab}
          onSelectTab={(tab) => setCurrentTab(tab)}
          summary={summary}
          soundEnabled={settings.sound_enabled}
          onToggleSound={handleToggleSound}
        >
          {/* All active pages remain alive in memory for instant flipping without reset */}
          <div style={{ display: currentTab === 'home' ? 'block' : 'none' }}>
            <Home
              onSelectGame={(gameTab) => setCurrentTab(gameTab)}
              summary={summary}
              stats={stats}
            />
          </div>

          <div style={{ display: currentTab === 'wordle' ? 'block' : 'none' }}>
            <WordlePage onGameComplete={refreshSummaryAndStats} />
          </div>

          <div style={{ display: currentTab === 'spelling-bee' ? 'block' : 'none' }}>
            <SpellingBeePage onGameComplete={refreshSummaryAndStats} />
          </div>

          <div style={{ display: currentTab === 'boggle' ? 'block' : 'none' }}>
            <BogglePage onGameComplete={refreshSummaryAndStats} />
          </div>

          <div style={{ display: currentTab === 'stats' ? 'block' : 'none' }}>
            <StatisticsPage />
          </div>

          <div style={{ display: currentTab === 'settings' ? 'block' : 'none' }}>
            <SettingsPage
              settings={settings}
              onUpdateSettings={handleUpdateSettings}
              systemStatus={systemStatus}
            />
          </div>
        </DiaryBook>
      </main>
    </div>
  );
};
export default App;
