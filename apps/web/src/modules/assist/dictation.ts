// V7 dictation helper (SPEC §14.3, plan §3 e6). Minimal typings for the browser
// Web Speech API (not in lib.dom for all targets) + a capability probe so the
// Dictation component can pick the browser recogniser first and fall back to the
// Assist STT endpoint only when it is absent. Keeps `any` out of strict TS.

export interface SpeechRecognitionAlternativeLike {
  readonly transcript: string;
}
export interface SpeechRecognitionResultLike {
  readonly length: number;
  readonly isFinal: boolean;
  item(index: number): SpeechRecognitionAlternativeLike;
  readonly [index: number]: SpeechRecognitionAlternativeLike;
}
export interface SpeechRecognitionResultListLike {
  readonly length: number;
  item(index: number): SpeechRecognitionResultLike;
  readonly [index: number]: SpeechRecognitionResultLike;
}
export interface SpeechRecognitionEventLike {
  readonly resultIndex: number;
  readonly results: SpeechRecognitionResultListLike;
}
export interface SpeechRecognitionLike {
  lang: string;
  continuous: boolean;
  interimResults: boolean;
  onresult: ((event: SpeechRecognitionEventLike) => void) | null;
  onerror: ((event: unknown) => void) | null;
  onend: (() => void) | null;
  start(): void;
  stop(): void;
}

type SpeechRecognitionCtor = new () => SpeechRecognitionLike;

interface SpeechWindow {
  SpeechRecognition?: SpeechRecognitionCtor;
  webkitSpeechRecognition?: SpeechRecognitionCtor;
}

/** Resolve the browser SpeechRecognition constructor, or null if unsupported. */
export function browserRecognitionCtor(): SpeechRecognitionCtor | null {
  if (typeof globalThis === 'undefined') return null;
  const w = globalThis as unknown as SpeechWindow;
  return w.SpeechRecognition ?? w.webkitSpeechRecognition ?? null;
}

/** Is MediaRecorder available (needed for the Assist-STT fallback path)? */
export function mediaRecorderSupported(): boolean {
  return typeof globalThis !== 'undefined' && 'MediaRecorder' in globalThis;
}

/** Concatenate the final transcript out of a recognition event. */
export function transcriptFromEvent(event: SpeechRecognitionEventLike): string {
  let out = '';
  for (let i = event.resultIndex; i < event.results.length; i += 1) {
    const result = event.results.item(i);
    if (result.length > 0) out += result.item(0).transcript;
  }
  return out;
}
