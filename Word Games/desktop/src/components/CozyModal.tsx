import React, { useEffect } from 'react';
import { X } from 'lucide-react';
import confetti from 'canvas-confetti';
import { sound } from '../services/sound';

interface CozyModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  isVictory?: boolean;
  children: React.ReactNode;
}

export const CozyModal: React.FC<CozyModalProps> = ({
  isOpen,
  onClose,
  title,
  isVictory,
  children,
}) => {
  useEffect(() => {
    if (isOpen && isVictory) {
      sound.playSuccess();
      try {
        confetti({
          particleCount: 50,
          spread: 70,
          origin: { y: 0.6 },
          colors: ['#F48E88', '#FFD56D', '#B8D8AE', '#93DFE8'],
        });
      } catch (e) {
        // Fallback gracefully if canvas context unavailable
      }
    }
  }, [isOpen, isVictory]);

  if (!isOpen) return null;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        backgroundColor: 'rgba(96, 67, 59, 0.45)',
        backdropFilter: 'blur(3px)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 100,
        padding: '16px',
      }}
      onClick={onClose}
    >
      <div
        className="cozy-card animate-pop"
        style={{
          width: '100%',
          maxWidth: '520px',
          backgroundColor: 'var(--panel-cream)',
          padding: '24px',
          position: 'relative',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header Ribbon */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            marginBottom: '16px',
            borderBottom: '2.5px dashed var(--outline-cocoa)',
            paddingBottom: '12px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            {isVictory && (
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
                <polygon
                  points="12,2 15,9 22,9 17,14 19,21 12,17 5,21 7,14 2,9 9,9"
                  fill="#FFD56D"
                  stroke="#60433B"
                  strokeWidth="2"
                  strokeLinejoin="round"
                />
              </svg>
            )}
            <h2 style={{ fontSize: '1.45rem', color: 'var(--text-main)' }}>{title}</h2>
          </div>
          <button
            onClick={() => {
              sound.playClick();
              onClose();
            }}
            title="Close dialog"
            style={{
              width: '34px',
              height: '34px',
              borderRadius: '50%',
              backgroundColor: 'var(--panel-peach)',
              border: '2.5px solid var(--outline-cocoa)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              cursor: 'pointer',
              color: 'var(--text-main)',
              boxShadow: '0 2px 0 var(--outline-cocoa)',
            }}
          >
            <X size={18} />
          </button>
        </div>

        {/* Modal Content */}
        <div>{children}</div>
      </div>
    </div>
  );
};
