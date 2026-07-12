// Self-hosted @font-face layer (plan §3 e4, SPEC §17: `font-src 'self'`).
//
// NEVER a remote font URL. The binaries are produced by e10's
// `mailwoman fonts pull` (Google Fonts → unicode-range subset → woff2 under
// `/fonts/`); this module only declares the faces + `local()` fallbacks, and
// the token font stacks (tokens.ts) fall back to system fonts until the files
// exist, so the build/UI never break on a missing binary. The face list here
// mirrors `fonts/manifest.json` — keep them in sync.

import { globalFontFace } from '@vanilla-extract/css';

const UI = 'Inter';
const READING = 'Newsreader';
const MONO = 'JetBrains Mono';

interface Face {
  file: string;
  weight: string;
  style?: string;
  local?: string[];
}

function declare(family: string, faces: Face[]): void {
  for (const f of faces) {
    const sources = [
      ...(f.local ?? []).map((n) => `local('${n}')`),
      `url('/fonts/${f.file}') format('woff2')`,
    ].join(', ');
    globalFontFace(family, {
      src: sources,
      fontWeight: f.weight,
      fontStyle: f.style ?? 'normal',
      fontDisplay: 'swap',
    });
  }
}

// UI sans — Inter (variable-ish; ship the common weights subset).
declare(UI, [
  { file: 'inter-400.woff2', weight: '400', local: ['Inter', 'Inter Regular'] },
  { file: 'inter-500.woff2', weight: '500', local: ['Inter Medium'] },
  { file: 'inter-600.woff2', weight: '600', local: ['Inter SemiBold'] },
  { file: 'inter-700.woff2', weight: '700', local: ['Inter Bold'] },
]);

// Reading serif — Newsreader (message bodies / long-form).
declare(READING, [
  { file: 'newsreader-400.woff2', weight: '400', local: ['Newsreader'] },
  { file: 'newsreader-600.woff2', weight: '600', local: ['Newsreader SemiBold'] },
]);

// Mono — JetBrains Mono (code / raw headers / Sieve editor).
declare(MONO, [
  { file: 'jetbrains-mono-400.woff2', weight: '400', local: ['JetBrains Mono'] },
  { file: 'jetbrains-mono-500.woff2', weight: '500', local: ['JetBrains Mono Medium'] },
]);
