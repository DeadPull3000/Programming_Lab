import React from 'react';

// Custom hand-drawn vector icons replacing all emojis and generic 3D icons

export const WordleGameIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* 3 mini letter tiles */}
    <rect x="2" y="5" width="8" height="9" rx="2.5" fill="#8CCB79" stroke="#60433B" strokeWidth="1.8" />
    <text x="6" y="12" fontSize="6" fontWeight="900" fill="#FFFFFF" textAnchor="middle" fontFamily="sans-serif">W</text>

    <rect x="12" y="5" width="8" height="9" rx="2.5" fill="#FFD56D" stroke="#60433B" strokeWidth="1.8" />
    <text x="16" y="12" fontSize="6" fontWeight="900" fill="#60433B" textAnchor="middle" fontFamily="sans-serif">O</text>

    <rect x="22" y="5" width="8" height="9" rx="2.5" fill="#C6BCB5" stroke="#60433B" strokeWidth="1.8" />
    <text x="26" y="12" fontSize="6" fontWeight="900" fill="#FFFFFF" textAnchor="middle" fontFamily="sans-serif">R</text>

    <rect x="7" y="17" width="8" height="9" rx="2.5" fill="#FFF0D9" stroke="#60433B" strokeWidth="1.8" />
    <text x="11" y="24" fontSize="6" fontWeight="900" fill="#60433B" textAnchor="middle" fontFamily="sans-serif">D</text>

    <rect x="17" y="17" width="8" height="9" rx="2.5" fill="#8CCB79" stroke="#60433B" strokeWidth="1.8" />
    <text x="21" y="24" fontSize="6" fontWeight="900" fill="#FFFFFF" textAnchor="middle" fontFamily="sans-serif">L</text>
  </svg>
);

export const BeeGameIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Honeycomb hexagon */}
    <polygon
      points="16,3 27,9.5 27,22.5 16,29 5,22.5 5,9.5"
      fill="#FFD56D"
      stroke="#60433B"
      strokeWidth="2"
      strokeLinejoin="round"
    />
    {/* Cute Bee inside */}
    {/* Wings */}
    <ellipse cx="12" cy="11" rx="4" ry="2.5" fill="#E8F4F8" stroke="#60433B" strokeWidth="1.2" transform="rotate(-30 12 11)" />
    <ellipse cx="20" cy="11" rx="4" ry="2.5" fill="#E8F4F8" stroke="#60433B" strokeWidth="1.2" transform="rotate(30 20 11)" />
    {/* Body */}
    <ellipse cx="16" cy="16" rx="6" ry="7" fill="#FDB813" stroke="#60433B" strokeWidth="1.8" />
    {/* Stripes */}
    <path d="M11 14 Q16 16 21 14" stroke="#60433B" strokeWidth="1.8" fill="none" />
    <path d="M11 18 Q16 20 21 18" stroke="#60433B" strokeWidth="1.8" fill="none" />
    {/* Eyes */}
    <circle cx="14" cy="12" r="1" fill="#60433B" />
    <circle cx="18" cy="12" r="1" fill="#60433B" />
  </svg>
);

export const BoggleGameIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Isometric Wooden Letter Die */}
    <polygon points="16,4 28,10 16,16 4,10" fill="#FAF0DE" stroke="#60433B" strokeWidth="2" strokeLinejoin="round" />
    <polygon points="4,10 16,16 16,28 4,22" fill="#EBD6B8" stroke="#60433B" strokeWidth="2" strokeLinejoin="round" />
    <polygon points="28,10 16,16 16,28 28,22" fill="#D6BA96" stroke="#60433B" strokeWidth="2" strokeLinejoin="round" />
    {/* Letter B on top face */}
    <text x="16" y="13.5" fontSize="8" fontWeight="900" fill="#60433B" textAnchor="middle" fontFamily="sans-serif">B</text>
    {/* Pips on side faces */}
    <circle cx="10" cy="20" r="1.5" fill="#60433B" />
    <circle cx="22" cy="18" r="1.5" fill="#60433B" />
    <circle cx="22" cy="24" r="1.5" fill="#60433B" />
  </svg>
);

export const DiaryBookIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Leather Book Cover */}
    <rect x="4" y="5" width="23" height="23" rx="3.5" fill="#4E5473" stroke="#60433B" strokeWidth="2" />
    {/* Pages inside */}
    <rect x="7" y="6" width="18" height="21" rx="2" fill="#FAF0DE" />
    {/* Gold Bookmark Ribbon */}
    <path d="M12 4 L12 16 L15 13 L18 16 L18 4 Z" fill="#F48E88" stroke="#60433B" strokeWidth="1.2" />
    {/* Buckle strap */}
    <rect x="23" y="14" width="7" height="6" rx="2" fill="#4E5473" stroke="#60433B" strokeWidth="1.5" />
    <circle cx="27" cy="17" r="1.5" fill="#FFD56D" />
  </svg>
);

export const StatsRibbonIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Ribbon tails */}
    <polygon points="12,18 8,30 15,25" fill="#F48E88" stroke="#60433B" strokeWidth="1.8" />
    <polygon points="20,18 24,30 17,25" fill="#F48E88" stroke="#60433B" strokeWidth="1.8" />
    {/* Medal badge */}
    <circle cx="16" cy="14" r="10" fill="#FFD56D" stroke="#60433B" strokeWidth="2" />
    <circle cx="16" cy="14" r="7.5" fill="#FFF0D9" stroke="#60433B" strokeWidth="1" strokeDasharray="2 2" />
    {/* Star in center */}
    <polygon points="16,8.5 17.8,12 21.5,12.5 18.8,15 19.5,18.5 16,16.8 12.5,18.5 13.2,15 10.5,12.5 14.2,12" fill="#F48E88" stroke="#60433B" strokeWidth="1" strokeLinejoin="round" />
  </svg>
);

export const SettingsCogIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Floral Brass Cog */}
    <circle cx="16" cy="16" r="9" fill="#93DFE8" stroke="#60433B" strokeWidth="2" />
    {[0, 45, 90, 135, 180, 225, 270, 315].map((deg) => (
      <rect
        key={deg}
        x="14"
        y="3"
        width="4"
        height="5"
        rx="1.5"
        fill="#93DFE8"
        stroke="#60433B"
        strokeWidth="1.5"
        transform={`rotate(${deg} 16 16)`}
      />
    ))}
    <circle cx="16" cy="16" r="4.5" fill="#FFF0D9" stroke="#60433B" strokeWidth="1.8" />
  </svg>
);

export const PawPrintIcon: React.FC<{ size?: number; color?: string }> = ({ size = 28, color = '#60433B' }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Main Pad */}
    <ellipse cx="16" cy="20" rx="7" ry="5.5" fill="#F8B4BD" stroke={color} strokeWidth="2" />
    {/* 4 Toe Pads */}
    <circle cx="9" cy="12" r="3" fill="#F8B4BD" stroke={color} strokeWidth="1.6" />
    <circle cx="14" cy="8" r="3.2" fill="#F8B4BD" stroke={color} strokeWidth="1.6" />
    <circle cx="19" cy="8" r="3.2" fill="#F8B4BD" stroke={color} strokeWidth="1.6" />
    <circle cx="24" cy="12" r="3" fill="#F8B4BD" stroke={color} strokeWidth="1.6" />
  </svg>
);

export const CookieEasterEgg: React.FC<{ size?: number; onClick?: () => void }> = ({ size = 80, onClick }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 100 100"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    style={{ cursor: 'pointer', filter: 'drop-shadow(2px 5px 6px rgba(0,0,0,0.3))' }}
    onClick={onClick}
  >
    {/* Cookie Base */}
    <ellipse cx="48" cy="48" rx="42" ry="40" fill="#C99256" stroke="#60433B" strokeWidth="3.5" />
    {/* Textured cookie edge roughness */}
    <path d="M12 36 Q10 48 16 62 Q28 82 48 85 Q72 84 84 66 Q90 48 84 32 Q74 14 50 12 Q24 12 12 36 Z" fill="#D8A265" stroke="#60433B" strokeWidth="3" />
    {/* Chocolate Chunks */}
    <rect x="26" y="24" width="12" height="10" rx="3" fill="#422817" stroke="#60433B" strokeWidth="1.5" transform="rotate(15 32 29)" />
    <rect x="56" y="28" width="14" height="12" rx="3" fill="#422817" stroke="#60433B" strokeWidth="1.5" transform="rotate(-10 63 34)" />
    <rect x="42" y="52" width="13" height="11" rx="3" fill="#422817" stroke="#60433B" strokeWidth="1.5" transform="rotate(25 48 57)" />
    <rect x="22" y="58" width="11" height="9" rx="3" fill="#422817" stroke="#60433B" strokeWidth="1.5" transform="rotate(-20 27 62)" />
    <rect x="66" y="60" width="10" height="9" rx="2.5" fill="#422817" stroke="#60433B" strokeWidth="1.5" transform="rotate(5 71 64)" />
    {/* Crumb 1 */}
    <circle cx="8" cy="74" r="3.5" fill="#422817" />
    {/* Crumb 2 */}
    <circle cx="16" cy="84" r="2.5" fill="#C99256" stroke="#60433B" strokeWidth="1" />
  </svg>
);

export const CrownQueenIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Crown base */}
    <path
      d="M4 22 L6 10 L12 16 L16 8 L20 16 L26 10 L28 22 Z"
      fill="#FFD56D"
      stroke="#60433B"
      strokeWidth="2"
      strokeLinejoin="round"
    />
    <rect x="4" y="22" width="24" height="4" rx="2" fill="#F48E88" stroke="#60433B" strokeWidth="1.8" />
    {/* Jewels */}
    <circle cx="6" cy="9" r="2" fill="#E84855" stroke="#60433B" strokeWidth="1" />
    <circle cx="16" cy="7" r="2.4" fill="#93DFE8" stroke="#60433B" strokeWidth="1" />
    <circle cx="26" cy="9" r="2" fill="#E84855" stroke="#60433B" strokeWidth="1" />
    <circle cx="11" cy="24" r="1.2" fill="#FFFFFF" />
    <circle cx="16" cy="24" r="1.2" fill="#FFFFFF" />
    <circle cx="21" cy="24" r="1.2" fill="#FFFFFF" />
  </svg>
);

export const StreakFlameIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Outer Orange Flame */}
    <path
      d="M16 2 C16 2 22 9 22 17 C22 23 18.5 27 16 28 C13.5 27 10 23 10 17 C10 12 13 8 16 2 Z"
      fill="#FF7A59"
      stroke="#60433B"
      strokeWidth="2"
      strokeLinejoin="round"
    />
    {/* Inner Yellow Core */}
    <path
      d="M16 11 C16 11 19.5 15.5 19.5 19.5 C19.5 23 17.5 25 16 25.5 C14.5 25 12.5 23 12.5 19.5 C12.5 16 14.5 14 16 11 Z"
      fill="#FFD56D"
      stroke="#60433B"
      strokeWidth="1.4"
    />
  </svg>
);

export const HoneyJarIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Pot Lid */}
    <rect x="9" y="5" width="14" height="4" rx="2" fill="#F48E88" stroke="#60433B" strokeWidth="1.8" />
    {/* Honey Pot Jar */}
    <path
      d="M8 9 C5 14 5 24 9 27 C12 29 20 29 23 27 C27 24 27 14 24 9 Z"
      fill="#EBD6B8"
      stroke="#60433B"
      strokeWidth="2"
      strokeLinejoin="round"
    />
    {/* Honey Drip */}
    <path
      d="M11 9 C11 12 13 14 15 14 C17 14 19 12 19 9 Z"
      fill="#FFD56D"
      stroke="#60433B"
      strokeWidth="1.4"
    />
    {/* Label */}
    <rect x="9.5" y="16" width="13" height="7" rx="2" fill="#FFFBEA" stroke="#60433B" strokeWidth="1.2" />
    <text x="16" y="21.5" fontSize="4.5" fontWeight="900" fill="#60433B" textAnchor="middle" fontFamily="sans-serif">HONEY</text>
  </svg>
);

export const ArcadeCabinetIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Cabinet body */}
    <path
      d="M7 4 L25 4 L23 15 L26 21 L24 29 L8 29 L6 21 L9 15 Z"
      fill="#93DFE8"
      stroke="#60433B"
      strokeWidth="2"
      strokeLinejoin="round"
    />
    {/* Marquee header */}
    <rect x="8.5" y="5.5" width="15" height="4" rx="1.5" fill="#F48E88" stroke="#60433B" strokeWidth="1.2" />
    {/* Screen */}
    <rect x="9" y="11" width="14" height="8" rx="1.5" fill="#4E5473" stroke="#60433B" strokeWidth="1.2" />
    {/* Pixel heart or smile on screen */}
    <circle cx="16" cy="15" r="2" fill="#FFD56D" />
    {/* Joystick */}
    <circle cx="13" cy="22" r="1.8" fill="#F48E88" stroke="#60433B" strokeWidth="1" />
    <line x1="13" y1="22" x2="13" y2="24" stroke="#60433B" strokeWidth="1.5" />
    {/* Buttons */}
    <circle cx="19" cy="23" r="1.2" fill="#FFD56D" stroke="#60433B" strokeWidth="0.8" />
    <circle cx="21" cy="22" r="1.2" fill="#8CCB79" stroke="#60433B" strokeWidth="0.8" />
  </svg>
);

export const GoldStarIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <polygon
      points="12,2 15,9 22,9 17,14 19,21 12,17 5,21 7,14 2,9 9,9"
      fill="#FFD56D"
      stroke="#60433B"
      strokeWidth="2"
      strokeLinejoin="round"
    />
  </svg>
);


export const CountdownTimerIcon: React.FC<{ size?: number }> = ({ size = 28 }) => (
  <svg width={size} height={size} viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="16" cy="17" r="11" fill="#FFEAA7" stroke="#60433B" strokeWidth="2" />
    <path d="M16 6V3M13 3H19M23 8L25 6" stroke="#60433B" strokeWidth="2" strokeLinecap="round" />
    <path d="M16 17L16 12M16 17L20 19" stroke="#60433B" strokeWidth="2" strokeLinecap="round" />
  </svg>
);
