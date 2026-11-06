import React from 'react';

// Hand-drawn botanical stickers, washi tape, postage stamps, and handwritten scrawly notes

export const WashiTape: React.FC<{
  color?: 'mint' | 'coral' | 'yellow' | 'lavender';
  width?: number;
  rotation?: number;
  style?: React.CSSProperties;
}> = ({ color = 'mint', width = 75, rotation = -3, style }) => {
  const getFill = () => {
    switch (color) {
      case 'coral':
        return '#F8B4BD';
      case 'yellow':
        return '#FFE399';
      case 'lavender':
        return '#EEDDF7';
      case 'mint':
      default:
        return '#C7E4BF';
    }
  };

  return (
    <div
      style={{
        display: 'inline-block',
        width: `${width}px`,
        height: '18px',
        backgroundColor: getFill(),
        border: '1.5px dashed rgba(96, 67, 59, 0.45)',
        boxShadow: '0 2px 3px rgba(96, 67, 59, 0.15)',
        transform: `rotate(${rotation}deg)`,
        opacity: 0.9,
        pointerEvents: 'none',
        ...style,
      }}
    />
  );
};

export const PressedFlower: React.FC<{ size?: number; rotation?: number }> = ({
  size = 50,
  rotation = 8,
}) => (
  <svg
    width={size}
    height={size * 1.2}
    viewBox="0 0 60 72"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    style={{ transform: `rotate(${rotation}deg)`, pointerEvents: 'none' }}
  >
    {/* Stem */}
    <path d="M30 32 Q26 50 28 68" stroke="#5E7D52" strokeWidth="2.5" strokeLinecap="round" />
    {/* Leaves */}
    <path d="M28 48 Q38 42 34 54 Q28 52 28 48 Z" fill="#7FA971" stroke="#5E7D52" strokeWidth="1.5" />
    <path d="M28 56 Q18 52 22 62 Q28 60 28 56 Z" fill="#7FA971" stroke="#5E7D52" strokeWidth="1.5" />
    {/* 5 Petals */}
    {[0, 72, 144, 216, 288].map((deg) => (
      <ellipse
        key={deg}
        cx="30"
        cy="20"
        rx="7"
        ry="10"
        fill="#FFD56D"
        stroke="#60433B"
        strokeWidth="1.8"
        transform={`rotate(${deg} 30 20)`}
      />
    ))}
    {/* Flower center */}
    <circle cx="30" cy="20" r="5" fill="#E88265" stroke="#60433B" strokeWidth="1.5" />
  </svg>
);

export const PressedFern: React.FC<{ size?: number; rotation?: number }> = ({
  size = 60,
  rotation = -12,
}) => (
  <svg
    width={size}
    height={size * 1.5}
    viewBox="0 0 50 75"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    style={{ transform: `rotate(${rotation}deg)`, pointerEvents: 'none' }}
  >
    {/* Main stalk */}
    <path d="M25 70 Q24 35 25 5" stroke="#486B3E" strokeWidth="2" strokeLinecap="round" />
    {/* Leaflets pairs */}
    {[12, 22, 32, 42, 52, 60].map((y, idx) => {
      const len = 14 - idx * 1.8;
      return (
        <React.Fragment key={y}>
          <ellipse
            cx={25 - len / 2}
            cy={y - 2}
            rx={len / 2}
            ry="2.5"
            fill="#6A975A"
            stroke="#486B3E"
            strokeWidth="1.2"
            transform={`rotate(-25 ${25 - len / 2} ${y})`}
          />
          <ellipse
            cx={25 + len / 2}
            cy={y - 2}
            rx={len / 2}
            ry="2.5"
            fill="#6A975A"
            stroke="#486B3E"
            strokeWidth="1.2"
            transform={`rotate(25 ${25 + len / 2} ${y})`}
          />
        </React.Fragment>
      );
    })}
  </svg>
);

export const PostageStamp: React.FC<{
  label?: string;
  illustration?: 'cat' | 'flower' | 'bee';
  size?: number;
  rotation?: number;
}> = ({ label = '12¢', illustration = 'cat', size = 56, rotation = 4 }) => (
  <svg
    width={size}
    height={size * 1.15}
    viewBox="0 0 50 58"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    style={{ transform: `rotate(${rotation}deg)`, filter: 'drop-shadow(2px 3px 3px rgba(0,0,0,0.2))' }}
  >
    {/* Perforated stamp background */}
    <rect x="3" y="3" width="44" height="52" fill="#FAF2E3" stroke="#60433B" strokeWidth="1.5" strokeDasharray="3 2" />
    <rect x="7" y="7" width="36" height="44" fill="#EBD6B8" stroke="#60433B" strokeWidth="1" />

    {/* Illustration */}
    {illustration === 'cat' && (
      <g transform="translate(14, 15)">
        <polygon points="4,10 7,2 12,8" fill="#60433B" />
        <polygon points="18,10 15,2 10,8" fill="#60433B" />
        <circle cx="11" cy="14" r="9" fill="#60433B" />
      </g>
    )}
    {illustration === 'flower' && (
      <g transform="translate(18, 16)">
        <circle cx="7" cy="7" r="5" fill="#F48E88" stroke="#60433B" strokeWidth="1" />
        <path d="M7 12 L7 24" stroke="#5E7D52" strokeWidth="1.5" />
      </g>
    )}
    {illustration === 'bee' && (
      <g transform="translate(15, 16)">
        <ellipse cx="10" cy="10" rx="6" ry="7" fill="#FFD56D" stroke="#60433B" strokeWidth="1.2" />
        <line x1="5" y1="9" x2="15" y2="9" stroke="#60433B" strokeWidth="1.5" />
        <line x1="5" y1="12" x2="15" y2="12" stroke="#60433B" strokeWidth="1.5" />
      </g>
    )}

    {/* Postmark Canceled Wavy Lines */}
    <path d="M2 18 Q12 14 24 18 Q36 22 48 18" stroke="#8C6E5A" strokeWidth="1" fill="none" opacity="0.6" />
    <path d="M2 24 Q12 20 24 24 Q36 28 48 24" stroke="#8C6E5A" strokeWidth="1" fill="none" opacity="0.6" />

    {/* Denomination Text */}
    <text x="25" y="46" fontSize="6.5" fontWeight="900" fill="#60433B" textAnchor="middle" fontFamily="sans-serif">
      {label}
    </text>
  </svg>
);

export const ScrawlyNote: React.FC<{
  title?: string;
  text: string;
  author?: string;
  tapeColor?: 'coral' | 'mint' | 'yellow' | 'lavender';
  rotation?: number;
  width?: number;
}> = ({
  title,
  text,
  author,
  tapeColor = 'yellow',
  rotation = -2,
  width = 180,
}) => (
  <div
    style={{
      width: `${width}px`,
      backgroundColor: '#FFFBEA',
      border: '2px solid var(--outline-cocoa)',
      borderRadius: '8px',
      padding: '12px 14px 10px',
      boxShadow: '0 4px 8px rgba(96, 67, 59, 0.16)',
      transform: `rotate(${rotation}deg)`,
      position: 'relative',
      margin: '6px',
    }}
  >
    {/* Washi tape on top center */}
    <div
      style={{
        position: 'absolute',
        top: '-9px',
        left: '50%',
        transform: 'translateX(-50%) rotate(1deg)',
      }}
    >
      <WashiTape color={tapeColor} width={50} />
    </div>

    {title && (
      <div
        style={{
          fontFamily: 'var(--font-heading)',
          fontSize: '0.85rem',
          fontWeight: 800,
          color: 'var(--text-main)',
          marginBottom: '4px',
          borderBottom: '1px dashed #d8c3a5',
          paddingBottom: '2px',
        }}
      >
        {title}
      </div>
    )}

    {/* Scrawly handwritten body */}
    <p
      style={{
        fontFamily: 'var(--font-handwritten)',
        fontSize: '1.15rem',
        lineHeight: '1.25',
        color: '#4A3423',
        margin: 0,
        whiteSpace: 'pre-line',
      }}
    >
      {text}
    </p>

    {author && (
      <div
        style={{
          fontFamily: 'var(--font-handwritten)',
          fontSize: '0.95rem',
          color: '#8C6E5A',
          textAlign: 'right',
          marginTop: '4px',
        }}
      >
        — {author}
      </div>
    )}
  </div>
);

export const DoodleStars: React.FC<{ size?: number }> = ({ size = 24 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* 4-point doodle sparkle star */}
    <path
      d="M12 2 Q12 12 2 12 Q12 12 12 22 Q12 12 22 12 Q12 12 12 2 Z"
      fill="#FFD56D"
      stroke="#60433B"
      strokeWidth="1.5"
      strokeLinejoin="round"
    />
    <circle cx="18" cy="6" r="1.5" fill="#F48E88" />
    <circle cx="6" cy="18" r="1.5" fill="#93DFE8" />
  </svg>
);
