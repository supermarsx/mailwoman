// Self-contained QR Code generator (byte mode) for the device-pairing QR (SPEC §9.1,
// plan §3 e8). Zero external dependencies (the web license floor + bundle discipline):
// this is a compact, standards-conformant (ISO/IEC 18004) encoder covering versions
// 1–10 at error-correction levels L and M, which comfortably holds the ~44–80-char
// base64 pairing public point. It renders to a boolean module matrix that `Qr.tsx`
// paints as inline SVG. It encodes ONLY the already-public ephemeral pairing key —
// no secret ever reaches this code.
//
// The codebase runs `noUncheckedIndexedAccess`; every table/matrix index below is a
// checked-in-range access, so the non-null assertions on reads are the intended,
// eslint-permitted way to express that (index math is bounded by construction).

/** Error-correction level (byte-mode subset used here). */
export type EcLevel = 'L' | 'M';

// ── Galois field GF(256), primitive polynomial 0x11D, generator 2 ───────────────
const GF_EXP = new Uint8Array(512);
const GF_LOG = new Uint8Array(256);
(function initGf(): void {
  let x = 1;
  for (let i = 0; i < 255; i += 1) {
    GF_EXP[i] = x;
    GF_LOG[x] = i;
    x <<= 1;
    if (x & 0x100) x ^= 0x11d;
  }
  for (let i = 255; i < 512; i += 1) GF_EXP[i] = GF_EXP[i - 255]!;
})();

function gfMul(a: number, b: number): number {
  if (a === 0 || b === 0) return 0;
  return GF_EXP[GF_LOG[a]! + GF_LOG[b]!]!;
}

/** Reed–Solomon: `data` codewords → `ecCount` error-correction codewords. */
function rsEncode(data: Uint8Array, ecCount: number): Uint8Array {
  const gen = new Uint8Array(ecCount + 1);
  gen[0] = 1;
  for (let i = 0; i < ecCount; i += 1) {
    for (let j = i + 1; j > 0; j -= 1) {
      gen[j] = gen[j - 1]! ^ gfMul(gen[j]!, GF_EXP[i]!);
    }
    gen[0] = gfMul(gen[0]!, GF_EXP[i]!);
  }
  const res = new Uint8Array(ecCount);
  for (const d of data) {
    const factor = d ^ res[0]!;
    res.copyWithin(0, 1);
    res[ecCount - 1] = 0;
    for (let j = 0; j < ecCount; j += 1) res[j] = res[j]! ^ gfMul(gen[ecCount - 1 - j]!, factor);
  }
  return res;
}

// ── Per-version tables (ISO 18004), versions 1–10, levels L & M ─────────────────
// [ecPerBlock, blocksGroup1, dataPerBlockG1, blocksGroup2, dataPerBlockG2]
type EcSpec = readonly [number, number, number, number, number];
const EC_TABLE: Record<EcLevel, readonly EcSpec[]> = {
  L: [
    [7, 1, 19, 0, 0],
    [10, 1, 34, 0, 0],
    [15, 1, 55, 0, 0],
    [20, 1, 80, 0, 0],
    [26, 1, 108, 0, 0],
    [18, 2, 68, 0, 0],
    [20, 2, 78, 0, 0],
    [24, 2, 97, 0, 0],
    [30, 2, 116, 0, 0],
    [18, 2, 68, 2, 69],
  ],
  M: [
    [10, 1, 16, 0, 0],
    [16, 1, 28, 0, 0],
    [26, 1, 44, 0, 0],
    [18, 2, 32, 0, 0],
    [24, 2, 43, 0, 0],
    [16, 4, 27, 0, 0],
    [18, 4, 31, 0, 0],
    [22, 2, 38, 2, 39],
    [22, 3, 36, 2, 37],
    [26, 4, 43, 1, 44],
  ],
};

/** Alignment-pattern centre coordinates per version (empty for v1). */
const ALIGN_POS: readonly (readonly number[])[] = [
  [],
  [6, 18],
  [6, 22],
  [6, 26],
  [6, 30],
  [6, 34],
  [6, 22, 38],
  [6, 24, 42],
  [6, 26, 46],
  [6, 28, 50],
];

function dataCapacity(spec: EcSpec): number {
  return spec[1] * spec[2] + spec[3] * spec[4];
}

function encodeData(bytes: Uint8Array, version: number, capacity: number): Uint8Array {
  const bits: number[] = [];
  const put = (value: number, len: number): void => {
    for (let i = len - 1; i >= 0; i -= 1) bits.push((value >>> i) & 1);
  };
  put(0b0100, 4); // byte mode
  put(bytes.length, version <= 9 ? 8 : 16);
  for (const b of bytes) put(b, 8);
  // Terminator (up to 4 zero bits) + pad to byte boundary.
  const cap = capacity * 8;
  const term = Math.min(4, cap - bits.length);
  for (let i = 0; i < term; i += 1) bits.push(0);
  while (bits.length % 8 !== 0) bits.push(0);
  const out = new Uint8Array(capacity);
  for (let i = 0; i < bits.length; i += 8) {
    let v = 0;
    for (let j = 0; j < 8; j += 1) v = (v << 1) | bits[i + j]!;
    out[i / 8] = v;
  }
  // Pad codewords 0xEC / 0x11.
  const pads = [0xec, 0x11];
  for (let i = bits.length / 8, k = 0; i < capacity; i += 1, k += 1) out[i] = pads[k % 2]!;
  return out;
}

/** Interleave data + EC codewords across blocks per the standard. */
function buildCodewords(data: Uint8Array, spec: EcSpec): Uint8Array {
  const [ecPer, g1, d1, g2, d2] = spec;
  const blocks: { data: Uint8Array; ec: Uint8Array }[] = [];
  let offset = 0;
  for (let i = 0; i < g1; i += 1) {
    const d = data.slice(offset, offset + d1);
    offset += d1;
    blocks.push({ data: d, ec: rsEncode(d, ecPer) });
  }
  for (let i = 0; i < g2; i += 1) {
    const d = data.slice(offset, offset + d2);
    offset += d2;
    blocks.push({ data: d, ec: rsEncode(d, ecPer) });
  }
  const result: number[] = [];
  const maxData = Math.max(d1, d2);
  for (let i = 0; i < maxData; i += 1) {
    for (const blk of blocks) if (i < blk.data.length) result.push(blk.data[i]!);
  }
  for (let i = 0; i < ecPer; i += 1) {
    for (const blk of blocks) result.push(blk.ec[i]!);
  }
  return Uint8Array.from(result);
}

// ── Matrix (flat arrays: on-bit + reserved-bit) ───────────────────────────────────
class Matrix {
  readonly size: number;
  private readonly on: Uint8Array;
  private readonly res: Uint8Array;
  constructor(size: number) {
    this.size = size;
    this.on = new Uint8Array(size * size);
    this.res = new Uint8Array(size * size);
  }
  get(r: number, c: number): boolean {
    return this.on[r * this.size + c] === 1;
  }
  reserved(r: number, c: number): boolean {
    return this.res[r * this.size + c] === 1;
  }
  set(r: number, c: number, on: boolean, reserve = false): void {
    this.on[r * this.size + c] = on ? 1 : 0;
    if (reserve) this.res[r * this.size + c] = 1;
  }
  clone(): Matrix {
    const m = new Matrix(this.size);
    m.on.set(this.on);
    m.res.set(this.res);
    return m;
  }
}

function placeFinder(m: Matrix, row: number, col: number): void {
  for (let r = -1; r <= 7; r += 1) {
    for (let c = -1; c <= 7; c += 1) {
      const rr = row + r;
      const cc = col + c;
      if (rr < 0 || rr >= m.size || cc < 0 || cc >= m.size) continue;
      const on =
        (r >= 0 && r <= 6 && (c === 0 || c === 6)) ||
        (c >= 0 && c <= 6 && (r === 0 || r === 6)) ||
        (r >= 2 && r <= 4 && c >= 2 && c <= 4);
      m.set(rr, cc, on, true);
    }
  }
}

function placeAlignment(m: Matrix, version: number): void {
  const pos = ALIGN_POS[version - 1]!;
  const last = pos[pos.length - 1];
  for (const r of pos) {
    for (const c of pos) {
      if ((r === 6 && c === 6) || (r === 6 && c === last) || (r === last && c === 6)) continue;
      if (m.reserved(r, c)) continue;
      for (let dr = -2; dr <= 2; dr += 1) {
        for (let dc = -2; dc <= 2; dc += 1) {
          const on = Math.max(Math.abs(dr), Math.abs(dc)) !== 1;
          m.set(r + dr, c + dc, on, true);
        }
      }
    }
  }
}

function placeTiming(m: Matrix): void {
  for (let i = 8; i < m.size - 8; i += 1) {
    const on = i % 2 === 0;
    if (!m.reserved(6, i)) m.set(6, i, on, true);
    if (!m.reserved(i, 6)) m.set(i, 6, on, true);
  }
}

function reserveFormat(m: Matrix, version: number): void {
  const size = m.size;
  for (let i = 0; i < 9; i += 1) {
    m.set(8, i, m.get(8, i), true);
    m.set(i, 8, m.get(i, 8), true);
  }
  for (let i = 0; i < 8; i += 1) {
    m.set(8, size - 1 - i, m.get(8, size - 1 - i), true);
    m.set(size - 1 - i, 8, m.get(size - 1 - i, 8), true);
  }
  m.set(size - 8, 8, true, true); // dark module
  if (version >= 7) {
    for (let i = 0; i < 6; i += 1) {
      for (let j = 0; j < 3; j += 1) {
        m.set(size - 11 + j, i, m.get(size - 11 + j, i), true);
        m.set(i, size - 11 + j, m.get(i, size - 11 + j), true);
      }
    }
  }
}

function placeData(m: Matrix, codewords: Uint8Array): void {
  const size = m.size;
  const bits: number[] = [];
  for (const cw of codewords) for (let i = 7; i >= 0; i -= 1) bits.push((cw >>> i) & 1);
  let idx = 0;
  let upward = true;
  for (let col = size - 1; col > 0; col -= 2) {
    if (col === 6) col -= 1; // skip vertical timing column
    for (let i = 0; i < size; i += 1) {
      const row = upward ? size - 1 - i : i;
      for (let c = 0; c < 2; c += 1) {
        const cc = col - c;
        if (m.reserved(row, cc)) continue;
        m.set(row, cc, idx < bits.length ? bits[idx] === 1 : false);
        idx += 1;
      }
    }
    upward = !upward;
  }
}

function maskFn(mask: number, r: number, c: number): boolean {
  switch (mask) {
    case 0:
      return (r + c) % 2 === 0;
    case 1:
      return r % 2 === 0;
    case 2:
      return c % 3 === 0;
    case 3:
      return (r + c) % 3 === 0;
    case 4:
      return (Math.floor(r / 2) + Math.floor(c / 3)) % 2 === 0;
    case 5:
      return ((r * c) % 2) + ((r * c) % 3) === 0;
    case 6:
      return (((r * c) % 2) + ((r * c) % 3)) % 2 === 0;
    default:
      return (((r + c) % 2) + ((r * c) % 3)) % 2 === 0;
  }
}

const EC_BITS: Record<EcLevel, number> = { L: 0b01, M: 0b00 };

function formatBits(level: EcLevel, mask: number): number {
  const data = (EC_BITS[level] << 3) | mask; // 5 bits
  let rem = data << 10;
  for (let i = 14; i >= 10; i -= 1) if ((rem >>> i) & 1) rem ^= 0x537 << (i - 10);
  return ((data << 10) | rem) ^ 0x5412;
}

function versionBits(version: number): number {
  let rem = version << 12;
  for (let i = 17; i >= 12; i -= 1) if ((rem >>> i) & 1) rem ^= 0x1f25 << (i - 12);
  return (version << 12) | rem;
}

function applyFormatAndVersion(m: Matrix, level: EcLevel, mask: number, version: number): void {
  const size = m.size;
  const fmt = formatBits(level, mask);
  for (let i = 0; i < 15; i += 1) {
    const bit = ((fmt >>> i) & 1) === 1;
    if (i < 6) m.set(8, i, bit);
    else if (i === 6) m.set(8, 7, bit);
    else if (i === 7) m.set(8, 8, bit);
    else if (i === 8) m.set(7, 8, bit);
    else m.set(14 - i, 8, bit);
    if (i < 8) m.set(size - 1 - i, 8, bit);
    else m.set(8, size - 15 + i, bit);
  }
  if (version >= 7) {
    const vb = versionBits(version);
    for (let i = 0; i < 18; i += 1) {
      const bit = ((vb >>> i) & 1) === 1;
      const a = Math.floor(i / 3);
      const b = i % 3;
      m.set(size - 11 + b, a, bit);
      m.set(a, size - 11 + b, bit);
    }
  }
}

function penalty(m: Matrix): number {
  const size = m.size;
  let score = 0;
  // Rule 1: runs of 5+ same-colour in row/col.
  for (let r = 0; r < size; r += 1) {
    for (let dir = 0; dir < 2; dir += 1) {
      let run = 1;
      for (let c = 1; c < size; c += 1) {
        const cur = dir === 0 ? m.get(r, c) : m.get(c, r);
        const prev = dir === 0 ? m.get(r, c - 1) : m.get(c - 1, r);
        if (cur === prev) {
          run += 1;
        } else {
          if (run >= 5) score += 3 + (run - 5);
          run = 1;
        }
      }
      if (run >= 5) score += 3 + (run - 5);
    }
  }
  // Rule 2: 2x2 blocks.
  for (let r = 0; r < size - 1; r += 1) {
    for (let c = 0; c < size - 1; c += 1) {
      const v = m.get(r, c);
      if (v === m.get(r, c + 1) && v === m.get(r + 1, c) && v === m.get(r + 1, c + 1)) score += 3;
    }
  }
  // Rule 3: finder-like patterns.
  const pat = [true, false, true, true, true, false, true];
  for (let r = 0; r < size; r += 1) {
    for (let c = 0; c < size - 6; c += 1) {
      let hMatch = true;
      let vMatch = true;
      for (let k = 0; k < 7; k += 1) {
        if (m.get(r, c + k) !== pat[k]) hMatch = false;
        if (m.get(c + k, r) !== pat[k]) vMatch = false;
      }
      if (hMatch) score += 40;
      if (vMatch) score += 40;
    }
  }
  // Rule 4: dark-module ratio.
  let dark = 0;
  for (let r = 0; r < size; r += 1) for (let c = 0; c < size; c += 1) if (m.get(r, c)) dark += 1;
  const ratio = (dark * 100) / (size * size);
  score += Math.floor(Math.abs(ratio - 50) / 5) * 10;
  return score;
}

/**
 * Encode `text` (UTF-8, byte mode) into a QR module matrix at EC level `level`
 * (default `M`). Auto-selects the smallest fitting version (1–10) and the
 * lowest-penalty mask. Returns a square boolean matrix (`true` = dark module).
 */
export function encodeQr(text: string, level: EcLevel = 'M'): boolean[][] {
  const bytes = new TextEncoder().encode(text);
  const table = EC_TABLE[level];
  let version = -1;
  for (let v = 1; v <= 10; v += 1) {
    const spec = table[v - 1]!;
    const countBits = v <= 9 ? 8 : 16;
    const capacityBytes = dataCapacity(spec) - Math.ceil((4 + countBits) / 8);
    if (bytes.length <= capacityBytes) {
      version = v;
      break;
    }
  }
  if (version === -1) throw new Error('QR payload too large for versions 1–10');

  const spec = table[version - 1]!;
  const data = encodeData(bytes, version, dataCapacity(spec));
  const codewords = buildCodewords(data, spec);

  const size = 21 + (version - 1) * 4;
  const base = new Matrix(size);
  placeFinder(base, 0, 0);
  placeFinder(base, 0, size - 7);
  placeFinder(base, size - 7, 0);
  placeAlignment(base, version);
  placeTiming(base);
  reserveFormat(base, version);
  placeData(base, codewords);

  let best: Matrix | null = null;
  let bestScore = Infinity;
  for (let mask = 0; mask < 8; mask += 1) {
    const trial = base.clone();
    for (let r = 0; r < size; r += 1) {
      for (let c = 0; c < size; c += 1) {
        if (!trial.reserved(r, c) && maskFn(mask, r, c)) trial.set(r, c, !trial.get(r, c));
      }
    }
    applyFormatAndVersion(trial, level, mask, version);
    const s = penalty(trial);
    if (s < bestScore) {
      bestScore = s;
      best = trial;
    }
  }
  const chosen = best!;
  const out: boolean[][] = [];
  for (let r = 0; r < size; r += 1) {
    const row: boolean[] = [];
    for (let c = 0; c < size; c += 1) row.push(chosen.get(r, c));
    out.push(row);
  }
  return out;
}
