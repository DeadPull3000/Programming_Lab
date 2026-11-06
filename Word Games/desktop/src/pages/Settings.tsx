import React, { useEffect, useState } from 'react';
import {
  Volume2,
  VolumeX,
  Zap,
} from 'lucide-react';
import { ArcadeSettings, SystemStatus } from '../types/arcade';
import { CozyButton } from '../components/CozyButton';
import { sound } from '../services/sound';
import {
  PawPrintIcon,
  SettingsCogIcon,
} from '../components/stickers/CustomIcons';
import {
  DoodleStars,
  PostageStamp,
  PressedFern,
  PressedFlower,
  ScrawlyNote,
  WashiTape,
} from '../components/stickers/DiaryStickers';

interface SettingsPageProps {
  settings: ArcadeSettings | null;
  onUpdateSettings: (settings: Partial<ArcadeSettings>) => void;
  systemStatus: SystemStatus | null;
}

export const SettingsPage: React.FC<SettingsPageProps> = ({
  settings,
  onUpdateSettings,
  systemStatus,
}) => {
  const [localSettings, setLocalSettings] = useState<ArcadeSettings>({
    sound_enabled: true,
    music_enabled: false,
    reduced_motion: false,
  });

  useEffect(() => {
    if (settings) {
      setLocalSettings(settings);
    }
  }, [settings]);

  const toggleSound = () => {
    const newVal = !localSettings.sound_enabled;
    setLocalSettings((prev) => ({ ...prev, sound_enabled: newVal }));
    sound.setEnabled(newVal);
    if (newVal) {
      sound.playPop(550);
    }
    onUpdateSettings({ sound_enabled: newVal });
  };

  const toggleReducedMotion = () => {
    const newVal = !localSettings.reduced_motion;
    setLocalSettings((prev) => ({ ...prev, reduced_motion: newVal }));
    if (newVal) {
      document.documentElement.classList.add('reduced-motion');
    } else {
      document.documentElement.classList.remove('reduced-motion');
    }
    onUpdateSettings({ reduced_motion: newVal });
    sound.playClick();
  };

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
      {/* LEFT PAGE: Sound, Animation & Tactile Preferences              */}
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
        <div>
          {/* Header */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              marginBottom: '12px',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <SettingsCogIcon size={28} />
              <div>
                <h2 style={{ fontSize: '1.45rem', color: 'var(--text-main)', margin: 0 }}>
                  DIARY PREFERENCES
                </h2>
                <div
                  style={{
                    fontFamily: 'var(--font-handwritten)',
                    fontSize: '1.15rem',
                    color: '#6E4E37',
                    fontWeight: 700,
                  }}
                >
                  ~ Audio, animations & comfort ~
                </div>
              </div>
            </div>

            <PostageStamp label="5¢" illustration="flower" rotation={6} size={42} />
          </div>

          <div style={{ marginBottom: '16px' }}>
            <WashiTape color="coral" width={100} rotation={-1.5} />
          </div>

          {/* Sound Toggle Box */}
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
                marginBottom: '10px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                <div
                  style={{
                    width: '36px',
                    height: '36px',
                    borderRadius: '50%',
                    backgroundColor: localSettings.sound_enabled ? 'var(--highlight-yellow)' : '#E7D8D5',
                    border: '2px solid var(--outline-cocoa)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  {localSettings.sound_enabled ? <Volume2 size={20} /> : <VolumeX size={20} />}
                </div>
                <div>
                  <div style={{ fontWeight: 900, fontSize: '1.05rem', color: 'var(--text-main)' }}>
                    Tactile Audio
                  </div>
                  <div style={{ fontSize: '0.75rem', fontWeight: 700, color: 'var(--text-muted)' }}>
                    Paper page rustles, wooden clicks & victory tunes
                  </div>
                </div>
              </div>

              {/* Cozy Toggle Button */}
              <button
                onClick={toggleSound}
                style={{
                  width: '54px',
                  height: '30px',
                  backgroundColor: localSettings.sound_enabled ? 'var(--color-success)' : '#d0c4b8',
                  borderRadius: 'var(--radius-pill)',
                  border: '2.5px solid var(--outline-cocoa)',
                  position: 'relative',
                  cursor: 'pointer',
                  transition: 'background-color 0.2s ease',
                }}
              >
                <div
                  style={{
                    width: '18px',
                    height: '18px',
                    borderRadius: '50%',
                    backgroundColor: '#ffffff',
                    border: '2px solid var(--outline-cocoa)',
                    position: 'absolute',
                    top: '3px',
                    left: localSettings.sound_enabled ? '27px' : '4px',
                    transition: 'left 0.2s ease',
                  }}
                />
              </button>
            </div>

            {/* Test Audio Button */}
            <div style={{ display: 'flex', gap: '8px', marginTop: '6px' }}>
              <CozyButton
                variant="cream"
                size="sm"
                onClick={() => {
                  sound.playPageFlip();
                }}
              >
                <span>Rustle Paper</span>
              </CozyButton>

              <CozyButton
                variant="cream"
                size="sm"
                onClick={() => {
                  sound.playSuccess();
                }}
              >
                <span>Victory Chime</span>
              </CozyButton>

              <CozyButton
                variant="cream"
                size="sm"
                onClick={() => {
                  sound.playMeow();
                }}
              >
                <span>Cat Purr</span>
              </CozyButton>
            </div>
          </div>

          {/* Reduced Motion Toggle Box */}
          <div
            style={{
              backgroundColor: '#FAF5EE',
              borderRadius: '14px',
              border: '2.5px solid var(--outline-cocoa)',
              padding: '14px 16px',
              boxShadow: '0 3px 0 var(--outline-cocoa)',
            }}
          >
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                <div
                  style={{
                    width: '36px',
                    height: '36px',
                    borderRadius: '50%',
                    backgroundColor: 'var(--bg-mint)',
                    border: '2px solid var(--outline-cocoa)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <Zap size={20} />
                </div>
                <div>
                  <div style={{ fontWeight: 900, fontSize: '1.05rem', color: 'var(--text-main)' }}>
                    Calm Animations
                  </div>
                  <div style={{ fontSize: '0.75rem', fontWeight: 700, color: 'var(--text-muted)' }}>
                    Gentle page flips without rotational perspective
                  </div>
                </div>
              </div>

              {/* Cozy Toggle Button */}
              <button
                onClick={toggleReducedMotion}
                style={{
                  width: '54px',
                  height: '30px',
                  backgroundColor: localSettings.reduced_motion ? 'var(--color-success)' : '#d0c4b8',
                  borderRadius: 'var(--radius-pill)',
                  border: '2.5px solid var(--outline-cocoa)',
                  position: 'relative',
                  cursor: 'pointer',
                  transition: 'background-color 0.2s ease',
                }}
              >
                <div
                  style={{
                    width: '18px',
                    height: '18px',
                    borderRadius: '50%',
                    backgroundColor: '#ffffff',
                    border: '2px solid var(--outline-cocoa)',
                    position: 'absolute',
                    top: '3px',
                    left: localSettings.reduced_motion ? '27px' : '4px',
                    transition: 'left 0.2s ease',
                  }}
                />
              </button>
            </div>
          </div>
        </div>

        {/* Bottom Left Note & Botanical Decor */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '10px', marginTop: '16px' }}>
          <ScrawlyNote
            text="Paper rustles and soft purrs make for the coziest word-solving sessions."
            author="Notebook Care"
            rotation={-2}
          />
          <PressedFlower rotation={10} size={36} />
        </div>
      </div>

      {/* ============================================================== */}
      {/* RIGHT PAGE: Rust Engine Diagnostics & Scrapbook Credits       */}
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
          {/* Header */}
          <div style={{ marginBottom: '10px' }}>
            <span
              style={{
                fontFamily: 'var(--font-handwritten)',
                fontSize: '1.35rem',
                color: '#6E4E37',
                fontWeight: 700,
              }}
            >
              ~ Rust Engine Diagnostics & Credits ~
            </span>
            <div style={{ display: 'flex', gap: '4px', marginTop: '2px' }}>
              <WashiTape color="lavender" width={85} rotation={1} />
            </div>
          </div>

          {/* Engine Status Card */}
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
              <span style={{ fontWeight: 900, fontSize: '0.98rem', color: 'var(--text-main)' }}>
                Authoritative Rust Backend
              </span>
              <span
                style={{
                  fontSize: '0.72rem',
                  fontWeight: 900,
                  color: '#2b8a3e',
                  backgroundColor: '#e6fcf5',
                  padding: '2px 8px',
                  borderRadius: 'var(--radius-pill)',
                  border: '1.5px solid #2b8a3e',
                }}
              >
                ● 127.0.0.1:4567
              </span>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', fontSize: '0.78rem' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 8px', backgroundColor: 'var(--panel-cream)', borderRadius: '6px' }}>
                <span style={{ fontWeight: 700 }}>Trie Dictionary Status:</span>
                <span style={{ fontWeight: 900, color: 'var(--color-success)' }}>
                  Loaded ({systemStatus?.legal_guess_count?.toLocaleString() || '114,499'} words)
                </span>
              </div>

              <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 8px', backgroundColor: 'var(--panel-cream)', borderRadius: '6px' }}>
                <span style={{ fontWeight: 700 }}>Wordle Answer Pool:</span>
                <span style={{ fontWeight: 900 }}>
                  {systemStatus?.answers_count || 2309} target words
                </span>
              </div>

              <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 8px', backgroundColor: 'var(--panel-cream)', borderRadius: '6px' }}>
                <span style={{ fontWeight: 700 }}>Boggle Die Tray:</span>
                <span style={{ fontWeight: 900 }}>Standard 16 / 25 Tabletop</span>
              </div>
            </div>
          </div>

          {/* Scrapbook Credits Card */}
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
                fontWeight: 900,
                fontSize: '0.95rem',
                color: 'var(--text-main)',
                borderBottom: '2px dashed var(--outline-cocoa)',
                paddingBottom: '5px',
                marginBottom: '8px',
              }}
            >
              Scrapbook Colophon
            </div>

            <div style={{ fontSize: '0.76rem', color: 'var(--text-main)', lineHeight: 1.5 }}>
              <div>
                <strong>Design:</strong> The Word Garden — Cozy Indie Desktop Arcade
              </div>
              <div>
                <strong>Typography:</strong> Google Fonts (Fredoka, Patrick Hand, Caveat)
              </div>
              <div>
                <strong>Portraits:</strong> Princess (The Wordle Queen), Benny (The Bee), Snowy & Maggie
              </div>
            </div>
          </div>
        </div>

        {/* Bottom Right Note & Decor */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '10px', marginTop: '10px' }}>
          <PressedFern rotation={10} size={34} />
          <ScrawlyNote
            text="Maggie developed all the portrait prints using vintage 35mm film."
            author="Maggie's Camera"
            rotation={2}
          />
          <DoodleStars size={30} />
        </div>
      </div>
    </div>
  );
};
