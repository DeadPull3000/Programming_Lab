import React, { useEffect, useState } from 'react';
import { GameTab, GamesSummary, SystemStatus, UnifiedStats, ArcadeSettings } from './types/arcade';
import { arcadeApi } from './services/api';
import { Home } from './pages/Home';
import { WordlePage } from './pages/Wordle';
import { SpellingBeePage } from './pages/SpellingBee';
import { BogglePage } from './pages/Boggle';
import { TimerPage } from './pages/Timer';
import { StatisticsPage } from './pages/Statistics';
import { SettingsPage } from './pages/Settings';
import { sound } from './services/sound';
import { DiaryBook } from './components/DiaryBook';

export const App: React.FC = () => {
  const [currentTab, setCurrentTab] = useState<GameTab>('home');
  const [visitedTabs, setVisitedTabs] = useState<Set<GameTab>>(new Set(['home']));
  const selectTab = (tab: GameTab) => {
    setVisitedTabs((previous) => new Set(previous).add(tab));
    setCurrentTab(tab);
  };
  const [summary, setSummary] = useState<GamesSummary | null>(null);
  const [stats, setStats] = useState<UnifiedStats | null>(null);
  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
  const [settingsError, setSettingsError] = useState<string | null>(null);
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

  const handleToggleSound = () => handleUpdateSettings({ sound_enabled: !settings.sound_enabled });

  const handleUpdateSettings = async (newSettings: Partial<ArcadeSettings>) => {
    const merged = { ...settings, ...newSettings };
    setSettings(merged);
    sound.setEnabled(merged.sound_enabled);
    document.documentElement.classList.toggle('reduced-motion', merged.reduced_motion);
    setSettingsError(null);
    try {
      await arcadeApi.updateSettings(newSettings);
    } catch (error) {
      setSettingsError(error instanceof Error ? error.message : 'Could not save settings');
      const saved = await arcadeApi.getSettings().catch(() => settings);
      setSettings(saved);
      sound.setEnabled(saved.sound_enabled);
      document.documentElement.classList.toggle('reduced-motion', saved.reduced_motion);
    }
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
        {settingsError && <p role="alert" style={{ textAlign: 'center' }}>{settingsError}</p>}
        <DiaryBook
          currentTab={currentTab}
          onSelectTab={(tab) => { selectTab(tab); void refreshSummaryAndStats(); }}
          summary={summary}
          soundEnabled={settings.sound_enabled}
          onToggleSound={handleToggleSound}
        >
          {/* Open games stay mounted; only the visible page handles keyboard input. */}
          <div style={{ display: currentTab === 'home' ? 'block' : 'none' }}>
            <Home
              onSelectGame={selectTab}
              summary={summary}
              stats={stats}
            />
          </div>

          <div style={{ display: currentTab === 'wordle' ? 'block' : 'none' }}>
            {visitedTabs.has('wordle') && <WordlePage active={currentTab === 'wordle'} onGameComplete={refreshSummaryAndStats} />}
          </div>

          <div style={{ display: currentTab === 'spelling-bee' ? 'block' : 'none' }}>
            {visitedTabs.has('spelling-bee') && <SpellingBeePage active={currentTab === 'spelling-bee'} onGameComplete={refreshSummaryAndStats} />}
          </div>

          <div style={{ display: currentTab === 'boggle' ? 'block' : 'none' }}>
            {visitedTabs.has('boggle') && <BogglePage active={currentTab === 'boggle'} onGameComplete={refreshSummaryAndStats} />}
          </div>

          <div style={{ display: currentTab === 'timer' ? 'block' : 'none' }}>
            {visitedTabs.has('timer') && <TimerPage onSelectGame={selectTab} />}
          </div>

          <div style={{ display: currentTab === 'stats' ? 'block' : 'none' }}>
            {visitedTabs.has('stats') && <StatisticsPage active={currentTab === 'stats'} onStatsChange={refreshSummaryAndStats} />}
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
