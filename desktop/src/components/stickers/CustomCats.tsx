import React from 'react';

// Custom-drawn vector illustrations for the cat portraits in the diary (Image 5)

export const PrincessCat: React.FC<{ size?: number }> = ({ size = 110 }) => (
  <svg
    width={size}
    height={size * 1.15}
    viewBox="0 0 100 115"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Oval Backdrop Glow */}
    <ellipse cx="50" cy="58" rx="42" ry="50" fill="#E8D5BC" stroke="#60433B" strokeWidth="3" />
    <ellipse cx="50" cy="58" rx="38" ry="46" fill="#F4E6D2" />

    {/* Fluffy Cat Head & Body */}
    <path
      d="M32 95 C32 82 40 76 50 76 C60 76 68 82 68 95 Z"
      fill="#FFF9F0"
      stroke="#60433B"
      strokeWidth="2.5"
    />
    <path
      d="M24 38 L32 20 L44 32 Z"
      fill="#FFF9F0"
      stroke="#60433B"
      strokeWidth="2.5"
      strokeLinejoin="round"
    />
    <path d="M28 34 L33 24 L40 31 Z" fill="#F8B4BD" />

    <path
      d="M76 38 L68 20 L56 32 Z"
      fill="#FFF9F0"
      stroke="#60433B"
      strokeWidth="2.5"
      strokeLinejoin="round"
    />
    <path d="M72 34 L67 24 L60 31 Z" fill="#F8B4BD" />

    {/* Head */}
    <ellipse cx="50" cy="48" rx="26" ry="22" fill="#FFF9F0" stroke="#60433B" strokeWidth="2.5" />

    {/* Golden Royal Tiara */}
    <path
      d="M37 26 L42 32 L50 22 L58 32 L63 26 L61 35 L39 35 Z"
      fill="#FFD56D"
      stroke="#60433B"
      strokeWidth="2"
      strokeLinejoin="round"
    />
    <circle cx="50" cy="22" r="2.5" fill="#F48E88" stroke="#60433B" strokeWidth="1" />
    <circle cx="37" cy="26" r="2" fill="#93DFE8" stroke="#60433B" strokeWidth="1" />
    <circle cx="63" cy="26" r="2" fill="#93DFE8" stroke="#60433B" strokeWidth="1" />

    {/* Eyelashes & Sweet Eyes */}
    <ellipse cx="40" cy="48" rx="3.5" ry="4" fill="#60433B" />
    <ellipse cx="60" cy="48" rx="3.5" ry="4" fill="#60433B" />
    <circle cx="41.5" cy="46.5" r="1.5" fill="#FFFFFF" />
    <circle cx="61.5" cy="46.5" r="1.5" fill="#FFFFFF" />
    {/* Eyelashes */}
    <path d="M35 44 L32 42" stroke="#60433B" strokeWidth="1.5" strokeLinecap="round" />
    <path d="M65 44 L68 42" stroke="#60433B" strokeWidth="1.5" strokeLinecap="round" />

    {/* Pink Nose & Smile */}
    <polygon points="50,53 47.5,50 52.5,50" fill="#F48E88" />
    <path
      d="M47.5 54 C47.5 56 45 56 44 55 M52.5 54 C52.5 56 55 56 56 55"
      stroke="#60433B"
      strokeWidth="1.8"
      strokeLinecap="round"
    />

    {/* Blush Cheeks */}
    <ellipse cx="34" cy="52" rx="4" ry="2" fill="#F8B4BD" opacity="0.65" />
    <ellipse cx="66" cy="52" rx="4" ry="2" fill="#F8B4BD" opacity="0.65" />

    {/* Whiskers */}
    <line x1="26" y1="48" x2="16" y2="47" stroke="#60433B" strokeWidth="1.2" strokeLinecap="round" />
    <line x1="26" y1="52" x2="15" y2="54" stroke="#60433B" strokeWidth="1.2" strokeLinecap="round" />
    <line x1="74" y1="48" x2="84" y2="47" stroke="#60433B" strokeWidth="1.2" strokeLinecap="round" />
    <line x1="74" y1="52" x2="85" y2="54" stroke="#60433B" strokeWidth="1.2" strokeLinecap="round" />

    {/* Regal Velvet Ribbon Bow */}
    <ellipse cx="50" cy="74" rx="4" ry="4" fill="#60433B" />
    <path
      d="M50 74 C44 68 34 70 36 78 C38 82 46 76 50 74 Z"
      fill="#F48E88"
      stroke="#60433B"
      strokeWidth="2"
    />
    <path
      d="M50 74 C56 68 66 70 64 78 C62 82 54 76 50 74 Z"
      fill="#F48E88"
      stroke="#60433B"
      strokeWidth="2"
    />
    <circle cx="50" cy="74" r="3" fill="#FFD56D" stroke="#60433B" strokeWidth="1.5" />
  </svg>
);

export const BennyCat: React.FC<{ size?: number }> = ({ size = 95 }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 100 100"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Photo Card Border */}
    <rect
      x="8"
      y="8"
      width="84"
      height="84"
      rx="8"
      fill="#F5E8D3"
      stroke="#60433B"
      strokeWidth="3"
    />
    <rect x="12" y="12" width="76" height="76" rx="6" fill="#FFF2E0" />

    {/* Tape strip at top */}
    <rect
      x="35"
      y="4"
      width="30"
      height="10"
      rx="2"
      fill="rgba(255,255,255,0.75)"
      stroke="#b5a089"
      strokeWidth="1"
      transform="rotate(-2 50 9)"
    />

    {/* Ginger Kitten Body */}
    <path
      d="M40 76 C40 60 52 56 62 64 C70 72 68 80 62 82 Z"
      fill="#E69858"
      stroke="#60433B"
      strokeWidth="2.5"
    />
    {/* Tail */}
    <path
      d="M65 74 C75 70 82 60 78 52 C76 48 72 50 73 54 C74 58 70 66 62 72"
      fill="none"
      stroke="#E69858"
      strokeWidth="5"
      strokeLinecap="round"
    />
    <path
      d="M65 74 C75 70 82 60 78 52 C76 48 72 50 73 54 C74 58 70 66 62 72"
      fill="none"
      stroke="#60433B"
      strokeWidth="2"
      strokeLinecap="round"
    />

    {/* Paws */}
    <ellipse cx="44" cy="80" rx="5" ry="3.5" fill="#FFF2E0" stroke="#60433B" strokeWidth="2" />
    <ellipse cx="54" cy="80" rx="5" ry="3.5" fill="#FFF2E0" stroke="#60433B" strokeWidth="2" />

    {/* Ears */}
    <polygon points="34,42 38,24 50,36" fill="#E69858" stroke="#60433B" strokeWidth="2.5" />
    <polygon points="37,39 40,28 47,35" fill="#F8B4BD" />
    <polygon points="66,42 62,24 50,36" fill="#E69858" stroke="#60433B" strokeWidth="2.5" />
    <polygon points="63,39 60,28 53,35" fill="#F8B4BD" />

    {/* Head */}
    <circle cx="50" cy="44" r="18" fill="#E69858" stroke="#60433B" strokeWidth="2.5" />
    {/* White Muzzle */}
    <ellipse cx="50" cy="50" rx="9" ry="7" fill="#FFF2E0" />

    {/* Big Curious Eyes */}
    <ellipse cx="43" cy="42" rx="4" ry="4.5" fill="#60433B" />
    <ellipse cx="57" cy="42" rx="4" ry="4.5" fill="#60433B" />
    <circle cx="44.5" cy="40.5" r="1.5" fill="#FFFFFF" />
    <circle cx="58.5" cy="40.5" r="1.5" fill="#FFFFFF" />

    {/* Pink Nose & Mouth */}
    <polygon points="50,47 48,45 52,45" fill="#F48E88" />
    <path
      d="M48 48 C48 50 46 51 45 50 M52 48 C52 50 54 51 55 50"
      stroke="#60433B"
      strokeWidth="1.5"
      strokeLinecap="round"
    />

    {/* Tabby Forehead Stripes */}
    <path d="M50 30 L50 35 M45 32 L47 36 M55 32 L53 36" stroke="#9C5223" strokeWidth="1.8" strokeLinecap="round" />
  </svg>
);

export const SnowyCat: React.FC<{ size?: number }> = ({ size = 100 }) => (
  <svg
    width={size}
    height={size * 1.15}
    viewBox="0 0 100 115"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Oval Backdrop */}
    <ellipse cx="50" cy="58" rx="40" ry="50" fill="#E8D5BC" stroke="#60433B" strokeWidth="3" />
    <ellipse cx="50" cy="58" rx="36" ry="46" fill="#FDF8F0" />

    {/* White Fluffy Body */}
    <path
      d="M32 96 C32 80 40 72 50 72 C60 72 68 80 68 96 Z"
      fill="#FFFFFF"
      stroke="#60433B"
      strokeWidth="2.5"
    />

    {/* Raised Paw Reaching Up */}
    <path
      d="M58 72 C64 64 68 50 64 46 C60 42 56 50 54 58 Z"
      fill="#FFFFFF"
      stroke="#60433B"
      strokeWidth="2.5"
    />
    <circle cx="64" cy="46" r="3.5" fill="#F8B4BD" />

    {/* Ears */}
    <polygon points="32,46 32,28 46,38" fill="#FFFFFF" stroke="#60433B" strokeWidth="2.5" />
    <polygon points="34,42 35,32 42,37" fill="#F8B4BD" />
    <polygon points="68,46 68,28 54,38" fill="#FFFFFF" stroke="#60433B" strokeWidth="2.5" />
    <polygon points="66,42 65,32 58,37" fill="#F8B4BD" />

    {/* Head */}
    <circle cx="50" cy="50" r="18" fill="#FFFFFF" stroke="#60433B" strokeWidth="2.5" />

    {/* Happy Eyes (Smiling Arcs) */}
    <path d="M40 48 C41 45 45 45 46 48" stroke="#60433B" strokeWidth="2.2" strokeLinecap="round" fill="none" />
    <path d="M54 48 C55 45 59 45 60 48" stroke="#60433B" strokeWidth="2.2" strokeLinecap="round" fill="none" />

    {/* Pink Nose & Smile */}
    <polygon points="50,53 48,51 52,51" fill="#F48E88" />
    <path
      d="M48 54 C48 56 46 57 45 56 M52 54 C52 56 54 57 55 56"
      stroke="#60433B"
      strokeWidth="1.8"
      strokeLinecap="round"
    />
    {/* Cheeks */}
    <ellipse cx="36" cy="54" rx="3.5" ry="2" fill="#F8B4BD" opacity="0.7" />
    <ellipse cx="64" cy="54" rx="3.5" ry="2" fill="#F8B4BD" opacity="0.7" />

    {/* Playful Butterfly Friend */}
    <g transform="translate(62, 22) rotate(-15)">
      {/* Wings */}
      <ellipse cx="6" cy="6" rx="6" ry="8" fill="#93DFE8" stroke="#60433B" strokeWidth="1.5" />
      <ellipse cx="14" cy="6" rx="6" ry="8" fill="#F48E88" stroke="#60433B" strokeWidth="1.5" />
      <ellipse cx="7" cy="13" rx="4" ry="5" fill="#FFD56D" stroke="#60433B" strokeWidth="1.2" />
      <ellipse cx="13" cy="13" rx="4" ry="5" fill="#B8D8AE" stroke="#60433B" strokeWidth="1.2" />
      <line x1="10" y1="2" x2="10" y2="16" stroke="#60433B" strokeWidth="2" strokeLinecap="round" />
    </g>
  </svg>
);

export const MaggieStrip: React.FC<{ size?: number }> = ({ size = 52 }) => (
  <svg
    width={size}
    height={size * 2.6}
    viewBox="0 0 50 130"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* Dark Wood Photo Filmstrip Container */}
    <rect
      x="3"
      y="3"
      width="44"
      height="124"
      rx="4"
      fill="#6E4A2F"
      stroke="#4A2F1B"
      strokeWidth="2.5"
    />

    {/* Film sprocket holes on side */}
    {[8, 20, 32, 44, 56, 68, 80, 92, 104, 116].map((y) => (
      <rect key={y} x="4" y={y} width="3" height="4" rx="1" fill="#FFF0D9" />
    ))}

    {/* Frame 1: Sleeping Tabby */}
    <rect x="10" y="8" width="34" height="34" rx="3" fill="#FFF2E0" stroke="#4A2F1B" strokeWidth="1.5" />
    <circle cx="27" cy="25" r="9" fill="#E69858" />
    <path d="M23 23 C24 21 27 21 28 23 M26 23 C27 21 30 21 31 23" stroke="#60433B" strokeWidth="1.2" strokeLinecap="round" fill="none" />
    <path d="M20 28 C22 34 32 34 34 28" fill="none" stroke="#E69858" strokeWidth="3" strokeLinecap="round" />

    {/* Frame 2: Cat Paw Stamp */}
    <rect x="10" y="48" width="34" height="34" rx="3" fill="#FFF2E0" stroke="#4A2F1B" strokeWidth="1.5" />
    <ellipse cx="27" cy="67" rx="6" ry="5" fill="#F48E88" />
    <circle cx="22" cy="58" r="2.5" fill="#F48E88" />
    <circle cx="27" cy="56" r="2.8" fill="#F48E88" />
    <circle cx="32" cy="58" r="2.5" fill="#F48E88" />

    {/* Frame 3: Smiling Tabby Face */}
    <rect x="10" y="88" width="34" height="34" rx="3" fill="#FFF2E0" stroke="#4A2F1B" strokeWidth="1.5" />
    <polygon points="17,96 19,89 25,94" fill="#E69858" stroke="#60433B" strokeWidth="1" />
    <polygon points="37,96 35,89 29,94" fill="#E69858" stroke="#60433B" strokeWidth="1" />
    <circle cx="27" cy="104" r="10" fill="#E69858" stroke="#60433B" strokeWidth="1.5" />
    <ellipse cx="27" cy="107" rx="5" ry="4" fill="#FFF2E0" />
    <ellipse cx="23" cy="102" rx="1.8" ry="2" fill="#60433B" />
    <ellipse cx="31" cy="102" rx="1.8" ry="2" fill="#60433B" />
    <polygon points="27,105 26,104 28,104" fill="#F48E88" />
  </svg>
);
