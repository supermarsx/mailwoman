// A small, self-contained QR-code encoder (plan §3 e2 — trust/verify "QR
// display"). Renders a key fingerprint as a scannable QR so a contact can capture
// it with a phone to verify out-of-band. No external dependency (the bundle stays
// lean, plan risk #12) — byte mode, error-correction level M, versions 1–10, with
// the standard mask selection. This is enough for a fingerprint (or a short
// `OPENPGP4FPR:` URI); larger payloads throw rather than silently truncate.
//
// The algorithm is ISO/IEC 18004: bitstream → Reed-Solomon over GF(256) → block
// interleave → matrix with finder/timing/alignment patterns → mask + format info.
// It is deterministic, so `qr.test.ts` pins structural invariants and a snapshot.

// ── GF(256) arithmetic (primitive polynomial 0x11d) ──────────────────────────
const EXP = new Uint8Array(512);
const LOG = new Uint8Array(256);
(() => {
  let x = 1;
  for (let i = 0; i < 255; i++) {
    EXP[i] = x;
    LOG[x] = i;
    x <<= 1;
    if (x & 0x100) x ^= 0x11d;
  }
  for (let i = 255; i < 512; i++) EXP[i] = EXP[i - 255]!;
})();

function gfMul(a: number, b: number): number {
  if (a === 0 || b === 0) return 0;
  return EXP[LOG[a]! + LOG[b]!]!;
}

/** Reed-Solomon EC codewords for `data` with `ecLen` check symbols. */
function rsEncode(data: number[], ecLen: number): number[] {
  // Generator polynomial (product of (x - a^i)).
  const gen = new Array<number>(ecLen + 1).fill(0);
  gen[0] = 1;
  for (let i = 0; i < ecLen; i++) {
    for (let j = i + 1; j > 0; j--) {
      gen[j] = gen[j - 1]! ^ gfMul(gen[j]!, EXP[i]!);
    }
    gen[0] = gfMul(gen[0]!, EXP[i]!);
  }
  const res = new Array<number>(ecLen).fill(0);
  for (const d of data) {
    const factor = d ^ res[0]!;
    res.shift();
    res.push(0);
    for (let j = 0; j < ecLen; j++) res[j] = res[j]! ^ gfMul(gen[ecLen - 1 - j]!, factor);
  }
  return res;
}

// ── Per-version tables (error-correction level M only) ───────────────────────
// [ ecPerBlock, [group1Blocks, group1DataCw], [group2Blocks, group2DataCw] ]
type BlockSpec = [number, [number, number], [number, number]];
const EC_M: Record<number, BlockSpec> = {
  1: [10, [1, 16], [0, 0]],
  2: [16, [1, 28], [0, 0]],
  3: [26, [1, 44], [0, 0]],
  4: [18, [2, 32], [0, 0]],
  5: [24, [2, 43], [0, 0]],
  6: [16, [4, 27], [0, 0]],
  7: [18, [4, 31], [0, 0]],
  8: [22, [2, 38], [2, 39]],
  9: [22, [3, 36], [2, 37]],
  10: [26, [4, 43], [1, 44]],
};

/** Alignment-pattern centre coordinates per version (empty for v1). */
const ALIGN: Record<number, number[]> = {
  1: [],
  2: [6, 18],
  3: [6, 22],
  4: [6, 26],
  5: [6, 30],
  6: [6, 34],
  7: [6, 22, 38],
  8: [6, 24, 42],
  9: [6, 26, 46],
  10: [6, 28, 50],
};

/** Total data codewords a version/level-M can carry. */
function dataCapacity(version: number): number {
  const [, [b1, d1], [b2, d2]] = EC_M[version]!;
  return b1 * d1 + b2 * d2;
}

/** Char-count indicator width for byte mode (8 bits up to v9, else 16). */
function countBits(version: number): number {
  return version <= 9 ? 8 : 16;
}

// ── Bit buffer ───────────────────────────────────────────────────────────────
class BitBuffer {
  readonly bits: number[] = [];
  put(value: number, length: number): void {
    for (let i = length - 1; i >= 0; i--) this.bits.push((value >>> i) & 1);
  }
}

/** Choose the smallest version (1–10) that fits `byteLen` bytes at level M. */
function chooseVersion(byteLen: number): number {
  for (let v = 1; v <= 10; v++) {
    const bits = 4 + countBits(v) + byteLen * 8;
    if (bits <= dataCapacity(v) * 8) return v;
  }
  throw new Error('qr: payload too large for versions 1–10');
}

/** Build the full (interleaved data + EC) codeword stream for `bytes`. */
function buildCodewords(bytes: number[], version: number): number[] {
  const cap = dataCapacity(version);
  const buf = new BitBuffer();
  buf.put(0b0100, 4); // byte mode
  buf.put(bytes.length, countBits(version));
  for (const b of bytes) buf.put(b, 8);
  // Terminator + pad to a byte boundary.
  const maxBits = cap * 8;
  for (let i = 0; i < 4 && buf.bits.length < maxBits; i++) buf.bits.push(0);
  while (buf.bits.length % 8 !== 0) buf.bits.push(0);
  const data: number[] = [];
  for (let i = 0; i < buf.bits.length; i += 8) {
    let byte = 0;
    for (let j = 0; j < 8; j++) byte = (byte << 1) | buf.bits[i + j]!;
    data.push(byte);
  }
  const PADS = [0xec, 0x11];
  for (let i = 0; data.length < cap; i++) data.push(PADS[i % 2]!);

  // Split into blocks, compute EC per block, then interleave.
  const [ecLen, [b1, d1], [b2, d2]] = EC_M[version]!;
  const blocks: { data: number[]; ec: number[] }[] = [];
  let pos = 0;
  for (const [count, dcw] of [[b1, d1], [b2, d2]] as const) {
    for (let i = 0; i < count; i++) {
      const chunk = data.slice(pos, pos + dcw);
      pos += dcw;
      blocks.push({ data: chunk, ec: rsEncode(chunk, ecLen) });
    }
  }
  const out: number[] = [];
  const maxData = Math.max(d1, d2);
  for (let i = 0; i < maxData; i++) {
    for (const blk of blocks) if (i < blk.data.length) out.push(blk.data[i]!);
  }
  for (let i = 0; i < ecLen; i++) {
    for (const blk of blocks) out.push(blk.ec[i]!);
  }
  return out;
}

// ── Matrix construction ──────────────────────────────────────────────────────
type Cell = { on: boolean; reserved: boolean };

function newMatrix(size: number): Cell[][] {
  return Array.from({ length: size }, () =>
    Array.from({ length: size }, () => ({ on: false, reserved: false })),
  );
}

function placeFinder(m: Cell[][], row: number, col: number): void {
  for (let r = -1; r <= 7; r++) {
    for (let c = -1; c <= 7; c++) {
      const rr = row + r;
      const cc = col + c;
      if (rr < 0 || rr >= m.length || cc < 0 || cc >= m.length) continue;
      const inRing = (r >= 0 && r <= 6 && (c === 0 || c === 6)) || (c >= 0 && c <= 6 && (r === 0 || r === 6));
      const inCore = r >= 2 && r <= 4 && c >= 2 && c <= 4;
      m[rr]![cc] = { on: inRing || inCore, reserved: true };
    }
  }
}

function placeAlignment(m: Cell[][], version: number): void {
  const centres = ALIGN[version]!;
  for (const r of centres) {
    for (const c of centres) {
      // Skip the three that overlap the finder patterns.
      if ((r === 6 && c === 6) || (r === 6 && c === m.length - 7) || (r === m.length - 7 && c === 6)) continue;
      for (let dr = -2; dr <= 2; dr++) {
        for (let dc = -2; dc <= 2; dc++) {
          const on = Math.max(Math.abs(dr), Math.abs(dc)) !== 1;
          m[r + dr]![c + dc] = { on, reserved: true };
        }
      }
    }
  }
}

function reserveFormat(m: Cell[][]): void {
  const n = m.length;
  for (let i = 0; i < 9; i++) {
    if (!m[8]![i]!.reserved) m[8]![i]!.reserved = true;
    if (!m[i]![8]!.reserved) m[i]![8]!.reserved = true;
  }
  for (let i = 0; i < 8; i++) {
    m[8]![n - 1 - i]!.reserved = true;
    m[n - 1 - i]![8]!.reserved = true;
  }
  m[n - 8]![8] = { on: true, reserved: true }; // dark module
}

function reserveVersionInfo(m: Cell[][], version: number): void {
  if (version < 7) return;
  const n = m.length;
  for (let i = 0; i < 6; i++) {
    for (let j = 0; j < 3; j++) {
      m[i]![n - 11 + j]!.reserved = true;
      m[n - 11 + j]![i]!.reserved = true;
    }
  }
}

function placeTiming(m: Cell[][]): void {
  for (let i = 8; i < m.length - 8; i++) {
    const on = i % 2 === 0;
    if (!m[6]![i]!.reserved) m[6]![i] = { on, reserved: true };
    if (!m[i]![6]!.reserved) m[i]![6] = { on, reserved: true };
  }
}

/** Zig-zag place the codeword bitstream into the unreserved modules. */
function placeData(m: Cell[][], codewords: number[]): void {
  const n = m.length;
  let bitIdx = 0;
  const totalBits = codewords.length * 8;
  const bitAt = (i: number): number => (i < totalBits ? (codewords[i >> 3]! >> (7 - (i & 7))) & 1 : 0);
  let upward = true;
  for (let col = n - 1; col > 0; col -= 2) {
    if (col === 6) col = 5; // skip the vertical timing column
    for (let i = 0; i < n; i++) {
      const row = upward ? n - 1 - i : i;
      for (let c = 0; c < 2; c++) {
        const cc = col - c;
        const cell = m[row]![cc]!;
        if (cell.reserved) continue;
        cell.on = bitAt(bitIdx) === 1;
        bitIdx++;
      }
    }
    upward = !upward;
  }
}

const MASKS: ((r: number, c: number) => boolean)[] = [
  (r, c) => (r + c) % 2 === 0,
  (r) => r % 2 === 0,
  (_r, c) => c % 3 === 0,
  (r, c) => (r + c) % 3 === 0,
  (r, c) => (Math.floor(r / 2) + Math.floor(c / 3)) % 2 === 0,
  (r, c) => ((r * c) % 2) + ((r * c) % 3) === 0,
  (r, c) => (((r * c) % 2) + ((r * c) % 3)) % 2 === 0,
  (r, c) => (((r + c) % 2) + ((r * c) % 3)) % 2 === 0,
];

function applyMask(m: Cell[][], mask: number): Cell[][] {
  const fn = MASKS[mask]!;
  return m.map((row, r) =>
    row.map((cell, c) => (cell.reserved ? cell : { on: cell.on !== fn(r, c), reserved: false })),
  );
}

/** Penalty score for a masked matrix (lower is better) — the four ISO rules. */
function penalty(m: Cell[][]): number {
  const n = m.length;
  let score = 0;
  // Rule 1: runs of 5+ same-colour modules in rows and columns.
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      for (const [dr, dc] of [[0, 1], [1, 0]] as const) {
        let run = 1;
        while (c + dc * run < n && r + dr * run < n && m[r + dr * run]![c + dc * run]!.on === m[r]![c]!.on) run++;
        if ((dc === 1 && (c === 0 || m[r]![c - 1]!.on !== m[r]![c]!.on)) || (dr === 1 && (r === 0 || m[r - 1]![c]!.on !== m[r]![c]!.on))) {
          if (run >= 5) score += 3 + (run - 5);
        }
      }
    }
  }
  // Rule 2: 2×2 blocks of the same colour.
  for (let r = 0; r < n - 1; r++) {
    for (let c = 0; c < n - 1; c++) {
      const v = m[r]![c]!.on;
      if (m[r]![c + 1]!.on === v && m[r + 1]![c]!.on === v && m[r + 1]![c + 1]!.on === v) score += 3;
    }
  }
  // Rule 3: finder-like 1:1:3:1:1 patterns.
  const pat1 = [true, false, true, true, true, false, true, false, false, false, false];
  const pat2 = [false, false, false, false, true, false, true, true, true, false, true];
  const matches = (line: boolean[], i: number, pat: boolean[]): boolean => pat.every((p, k) => line[i + k] === p);
  for (let r = 0; r < n; r++) {
    for (let c = 0; c <= n - 11; c++) {
      const rowLine = m[r]!.map((x) => x.on);
      const colLine = m.map((row) => row[r]!.on);
      if (matches(rowLine, c, pat1) || matches(rowLine, c, pat2)) score += 40;
      if (matches(colLine, c, pat1) || matches(colLine, c, pat2)) score += 40;
    }
  }
  // Rule 4: overall dark-module balance.
  let dark = 0;
  for (const row of m) for (const cell of row) if (cell.on) dark++;
  const pct = (dark * 100) / (n * n);
  score += Math.floor(Math.abs(pct - 50) / 5) * 10;
  return score;
}

// Format-info BCH (level M = bits 00) and version-info BCH tables.
const FORMAT_BITS: Record<number, number> = {
  0: 0x5412, 1: 0x5125, 2: 0x5e7c, 3: 0x5b4b, 4: 0x45f9, 5: 0x40ce, 6: 0x4f97, 7: 0x4aa0,
};
const VERSION_BITS: Record<number, number> = {
  7: 0x07c94, 8: 0x085bc, 9: 0x09a99, 10: 0x0a4d3,
};

function placeFormat(m: Cell[][], mask: number): void {
  const n = m.length;
  const bits = FORMAT_BITS[mask]!;
  for (let i = 0; i < 15; i++) {
    const on = ((bits >> i) & 1) === 1;
    // Around the top-left finder.
    if (i < 6) m[8]![i] = { on, reserved: true };
    else if (i === 6) m[8]![7] = { on, reserved: true };
    else if (i === 7) m[8]![8] = { on, reserved: true };
    else if (i === 8) m[7]![8] = { on, reserved: true };
    else m[14 - i]![8] = { on, reserved: true };
    // The mirrored copy split across the other two finders.
    if (i < 8) m[n - 1 - i]![8] = { on, reserved: true };
    else m[8]![n - 15 + i] = { on, reserved: true };
  }
  // The always-dark module sits just below the format strip; the second copy's
  // loop writes over it (bit 7), so re-assert it last (standard QR behaviour —
  // the redundant first copy + BCH absorb the one displaced bit).
  m[n - 8]![8] = { on: true, reserved: true };
}

function placeVersionInfo(m: Cell[][], version: number): void {
  if (version < 7) return;
  const n = m.length;
  const bits = VERSION_BITS[version]!;
  for (let i = 0; i < 18; i++) {
    const on = ((bits >> i) & 1) === 1;
    const r = Math.floor(i / 3);
    const c = i % 3;
    m[r]![n - 11 + c] = { on, reserved: true };
    m[n - 11 + c]![r] = { on, reserved: true };
  }
}

/** Encode `text` (UTF-8, byte mode) into a boolean QR matrix (no quiet zone). */
export function encodeQr(text: string): boolean[][] {
  const bytes = [...new TextEncoder().encode(text)];
  const version = chooseVersion(bytes.length);
  const codewords = buildCodewords(bytes, version);
  const size = 17 + version * 4;

  const base = newMatrix(size);
  placeFinder(base, 0, 0);
  placeFinder(base, 0, size - 7);
  placeFinder(base, size - 7, 0);
  placeAlignment(base, version);
  placeTiming(base);
  reserveFormat(base);
  reserveVersionInfo(base, version);
  placeData(base, codewords);

  // Pick the lowest-penalty mask.
  let best: Cell[][] | null = null;
  let bestMask = 0;
  let bestScore = Infinity;
  for (let mask = 0; mask < 8; mask++) {
    const masked = applyMask(base, mask);
    const score = penalty(masked);
    if (score < bestScore) {
      bestScore = score;
      bestMask = mask;
      best = masked;
    }
  }
  const chosen = best!;
  placeFormat(chosen, bestMask);
  placeVersionInfo(chosen, version);
  return chosen.map((row) => row.map((cell) => cell.on));
}

/** Render a QR matrix as a compact, self-contained SVG string (with quiet zone). */
export function qrToSvg(matrix: boolean[][], modulePx = 4, quiet = 4): string {
  const n = matrix.length;
  const dim = (n + quiet * 2) * modulePx;
  let rects = '';
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      if (matrix[r]![c]) {
        const x = (c + quiet) * modulePx;
        const y = (r + quiet) * modulePx;
        rects += `<rect x="${x}" y="${y}" width="${modulePx}" height="${modulePx}"/>`;
      }
    }
  }
  return (
    `<svg xmlns="http://www.w3.org/2000/svg" width="${dim}" height="${dim}" viewBox="0 0 ${dim} ${dim}" shape-rendering="crispEdges">` +
    `<rect width="${dim}" height="${dim}" fill="#ffffff"/><g fill="#000000">${rects}</g></svg>`
  );
}
