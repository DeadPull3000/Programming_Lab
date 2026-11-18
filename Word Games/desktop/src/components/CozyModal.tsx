import React, { useEffect, useId, useRef } from 'react';
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
  const titleId = useId();
  const dialogRef = useRef<HTMLDivElement>(null);
  const closeRef = useRef(onClose);
  closeRef.current = onClose;
  useEffect(() => {
    if (!isOpen) return;
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const dialog = dialogRef.current;
    dialog?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') { event.preventDefault(); closeRef.current(); }
      if (event.key !== 'Tab' || !dialog) return;
      const controls = Array.from(dialog.querySelectorAll<HTMLElement>('button:not(:disabled), a[href], input:not(:disabled), [tabindex="0"]'));
      const first = controls[0];
      const last = controls[controls.length - 1];
      if (!first) { event.preventDefault(); dialog.focus(); return; }
      if (event.shiftKey && (document.activeElement === first || document.activeElement === dialog)) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && (document.activeElement === last || document.activeElement === dialog)) { event.preventDefault(); first.focus(); }
    };
    document.addEventListener('keydown', onKey);
    return () => { document.removeEventListener('keydown', onKey); previous?.focus(); };
  }, [isOpen]);

  useEffect(() => {
    if (isOpen && isVictory) {
      sound.playSuccess();
      try {
        if (!document.documentElement.classList.contains('reduced-motion')) confetti({
          disableForReducedMotion: true,
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
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
        className="cozy-card animate-pop"
        style={{
          width: '100%',
          maxWidth: '520px',
          backgroundColor: 'var(--panel-cream)',
          padding: '24px',
          position: 'relative',
          maxHeight: '90vh',
          overflowY: 'auto',
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
            <h2 id={titleId} style={{ fontSize: '1.45rem', color: 'var(--text-main)' }}>{title}</h2>
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
