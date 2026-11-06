# Asset Credits and Licences

This non-commercial indie desktop project, **A Cozy Word Game Arcade**, incorporates curated designs, open-source typography, iconography, and procedural audio synthesis.

---

## 1. Visual Direction and Design Inspiration
- **Primary Visual Direction**: Priority reference image 5 — *The Cat Scrapbook & Cozy Wooden Desk* featuring open journal layout, cat portraits (Princess, Benny, Snowy, Maggie), scalloped PLAY badge, washi-tape buttons, dried botanical elements, and resting cat paws.
- **Secondary UI Inspiration**:
  - *Cozy UI Pack* by Cup Nooble (peach/cream cards, soft rounded buttons, pastel tones).
  - *Sprout Lands UI Expansion Pack* by Cup Nooble (wooden signs, heart badges, level complete stars).
  - *Clipboard Rewards UI* (claim buttons, status pills, pastel tags).

---

## 2. Typography
- **Fredoka**: Designed by Hafontia. Available under the **SIL Open Font License (OFL)**.
  - Usage: Display titles, section headers, tile letters, and buttons.
- **Nunito**: Designed by Vernon Adams, Cyreal, Jacques Le Bailly. Available under the **SIL Open Font License (OFL)**.
  - Usage: Body text, dialogs, hint explanations, and statistics numbers.

---

## 3. Iconography
- **Lucide Icons** (`lucide-react`): Open-source icon set licensed under the **ISC Licence**.
  - Usage: Top navigation tabs, audio controls, action indicators, and diagnostic dialogs.

---

## 4. Celebrations and Animation
- **Canvas Confetti**: Created by Kiril Vatev. Licensed under the **ISC Licence**.
  - Usage: Star & pastel victory bursts on game completion and Queen Bee achievements.

---

## 5. Audio Synthesizer
- **Native Web Audio API Procedural Synthesizer**: Built custom for this application with zero external audio assets.
  - Soft woodblock click on keypress (`triangle` wave with exponential decay).
  - Bubble pop on tile reveal (`sine` sweep).
  - Marimba chord on valid word submission (`C5-E5-G5-C6` progression).
  - Fanfare on pangram discovery (`G4-C5-E5-G5-C6-E6` arpeggio).
  - Soft low thud on invalid submissions.
  - Cat chirrup/purr on clicking scrapbook cat portraits.
  - 100% user-toggleable with persistent sound setting.

---

## 6. Authoritative Rust Game Engines
- Reused directly without duplication from the course workspace:
  - `trie` (26-ary prefix search and cursor traversal).
  - `wordle` (two-pass letter feedback, CandidateSet filtering, Shannon entropy solver, persistent stats).
  - `spelling-bee` (7-letter puzzle validation, bitmask operations, Queen Bee detection, hint matrix).
  - `boggle` (NxN grid representation, 8-directional neighbor discovery, Trie board solver, path validation).
