import { withBase } from '../api/basePath.ts';

export type AppearanceUpdate = Partial<Record<'accent' | 'density' | 'font' | 'mode', string>>;
export interface AppearanceScriptOptions {
  code: string;
  context: Readonly<{dark: boolean; reducedMotion: boolean}>;
  signal: AbortSignal;
  onUpdate: (update: AppearanceUpdate) => void;
  onError: (message: string) => void;
}
/** No owner identifiers or application objects cross this channel. */
export function runAppearanceScript(options: AppearanceScriptOptions): () => void {
  if (options.signal.aborted || !options.code.trim()) return () => {};
  const frame = document.createElement('iframe');
  frame.hidden = true;
  frame.title = 'Isolated appearance script';
  frame.setAttribute('sandbox', 'allow-scripts');
  frame.referrerPolicy = 'no-referrer';
  frame.src = withBase('/api/appearance/sandbox');
  const channel = new MessageChannel();
  let finished = false;
  let ready = false;
  let updates = 0;
  const cleanup = () => {
    if (finished) return;
    finished = true;
    clearTimeout(timer);
    options.signal.removeEventListener('abort', cleanup);
    channel.port1.close();
    channel.port2.close();
    frame.remove();
  };
  const fail = (message: string) => { cleanup(); options.onError(message); };
  const timer = window.setTimeout(() => fail('The isolated appearance script timed out.'), 2000);
  options.signal.addEventListener('abort', cleanup, {once: true});
  channel.port1.onmessage = ({data}: MessageEvent<unknown>) => {
    if (finished || !data || typeof data !== 'object') return;
    const message = data as Record<string, unknown>;
    if (message.kind === 'ready' && !ready) {
      ready = true;
      channel.port1.postMessage({kind: 'run', code: options.code, context: options.context});
    } else if (message.kind === 'done') cleanup();
    else if (message.kind === 'error') fail('The isolated appearance script could not complete.');
    else if (ready && message.kind === 'appearance-set') {
      const value = message.value;
      if (++updates > 32 || !value || typeof value !== 'object' || Array.isArray(value)) {
        fail('The script requested an unsupported appearance change.'); return;
      }
      const entries = Object.entries(value);
      if (entries.length > 4 || entries.some(([key, item]) => !['accent', 'density', 'font', 'mode'].includes(key) || typeof item !== 'string' || item.length > 32)) {
        fail('The script requested an unsupported appearance change.'); return;
      }
      // The owner controller additionally validates enum values and current locks.
      options.onUpdate(Object.fromEntries(entries) as AppearanceUpdate);
    }
  };
  channel.port1.start();
  frame.addEventListener('load', () => {
    if (!finished) frame.contentWindow?.postMessage({kind: 'appearance-connect'}, '*', [channel.port2]);
  }, {once: true});
  frame.addEventListener('error', () => fail('Isolated appearance scripts are unavailable.'), {once: true});
  document.body.append(frame);
  return cleanup;
}

/** Server-compiled CSS only; no style-src exception or user HTML is needed. */
export function mountAppearanceCss(compiled: string): () => void {
  if (!compiled) return () => {};
  const sheet = new CSSStyleSheet();
  sheet.replaceSync(compiled);
  document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
  return () => { document.adoptedStyleSheets = document.adoptedStyleSheets.filter(current => current !== sheet); };
}
