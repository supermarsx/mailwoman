// V7 Assist gateway I/O (plan §3 e6/e9/e14). Talks to `/api/assist/*`, which the
// SERVER proxies to the configured endpoint — the browser never contacts the AI
// host (CSP `connect-src 'self'`, mirroring the `/errors` tunnel). The transport is
// injectable so components unit-test without a live server.
//
// SAFETY-CRITICAL (R4): this service exposes NO method that transmits / deletes /
// accepts mail. It reads config, invokes read-only capabilities, and transcribes
// dictation audio. Proposed tool actions are returned to the caller for HUMAN
// confirmation via the Outbox — never executed here. Do not add a send method.

import {
  configFromWire,
  DISABLED_CONFIG,
  type AssistConfig,
  type Disclosure,
  type InvokeRequest,
  type InvokeResult,
  type ProposedAction,
  type WireAssistConfig,
} from './types.ts';

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

/** A proposed tool action as the server reports it. */
interface WireProposedAction {
  id: string;
  tool: string;
  summary: string;
  would_send: boolean;
}

interface WireDisclosure {
  endpoint_host: string;
  sent: string[];
  withheld: string[];
}

/** The non-streaming invoke shape (a mocked or non-SSE endpoint may still return it). */
interface WireInvokeResult {
  text: string;
  disclosure: WireDisclosure;
  actions?: WireProposedAction[];
}

/** Nothing is claimed to have left the device until the server says what did. */
const NO_DISCLOSURE: Disclosure = { endpointHost: '', sent: [], withheld: [] };

function disclosureFromWire(w: WireDisclosure): Disclosure {
  return { endpointHost: w.endpoint_host ?? '', sent: [...(w.sent ?? [])], withheld: [...(w.withheld ?? [])] };
}

function actionFromWire(a: WireProposedAction): ProposedAction {
  return { id: a.id, tool: a.tool, summary: a.summary, wouldSend: a.would_send === true };
}

/**
 * The `POST /api/assist/invoke` body. `scope` names the accounts the context was
 * drawn from — the server CLAMPS it to the admin data-class ceiling, and context from
 * an account outside the effective scope is dropped rather than forwarded. E2EE and
 * attachments are never opted into from here.
 */
function requestToWire(req: InvokeRequest): unknown {
  const accounts = [...new Set(req.context.map((c) => c.account).filter((a) => a.length > 0))];
  return {
    capability: req.capability,
    scope: { accounts, folders: [], include_e2ee: false, include_attachments: false },
    input: {
      prompt: req.prompt,
      context: req.context.map((c) => ({ account: c.account, folder: c.folder, text: c.text, kind: c.kind })),
    },
  };
}

/** One decoded Server-Sent Event. */
interface SseFrame {
  readonly event: string;
  readonly data: string;
}

/**
 * Incremental SSE decoder: feed transport text, get back whole frames. Records are
 * separated by a blank line, so a record split across chunks is buffered until its
 * terminator arrives.
 */
class SseReader {
  private buf = '';

  push(text: string): SseFrame[] {
    this.buf += text.replace(/\r\n/g, '\n');
    const frames: SseFrame[] = [];
    let idx = this.buf.indexOf('\n\n');
    while (idx >= 0) {
      const record = this.buf.slice(0, idx);
      this.buf = this.buf.slice(idx + 2);
      const frame = parseRecord(record);
      if (frame !== null) frames.push(frame);
      idx = this.buf.indexOf('\n\n');
    }
    return frames;
  }

  /** Flush a trailing record that arrived without its blank-line terminator. */
  end(): SseFrame[] {
    const rest = this.buf;
    this.buf = '';
    const frame = rest.length > 0 ? parseRecord(rest) : null;
    return frame === null ? [] : [frame];
  }
}

function parseRecord(record: string): SseFrame | null {
  let event = 'message';
  const data: string[] = [];
  for (const line of record.split('\n')) {
    if (line.startsWith(':')) continue; // keep-alive comment
    if (line.startsWith('event:')) event = line.slice(6).trim();
    else if (line.startsWith('data:')) data.push(line.slice(5).replace(/^ /, ''));
  }
  return data.length === 0 ? null : { event, data: data.join('\n') };
}

/**
 * The Assist gateway service backing the whole UI.
 * Endpoints:
 *   GET  /api/assist/config                 → WireAssistConfig
 *   POST /api/assist/invoke  {capability, scope, input}
 *        → text/event-stream: `disclosure` frame, `{delta}` frames, terminal `done`
 *          frame with the PROPOSED actions   (server proxies + redacts)
 *   POST /api/assist/transcribe {audioBase64, mime} → { text }  (Assist STT fallback)
 *
 * There is deliberately NO send/delete/accept endpoint on this client.
 */
export class AssistService {
  constructor(private readonly fetcher: Fetcher = defaultFetcher) {}

  /** Read the gateway config. A gateway that is off / unreachable ⇒ DISABLED_CONFIG (hide all UI). */
  async getConfig(): Promise<AssistConfig> {
    try {
      const res = await this.fetcher('/api/assist/config');
      if (!res.ok) return DISABLED_CONFIG;
      const wire = (await res.json()) as WireAssistConfig;
      return configFromWire(wire);
    } catch {
      return DISABLED_CONFIG;
    }
  }

  /**
   * Invoke a read-only capability. The server streams the reply as Server-Sent
   * Events: a leading `disclosure` frame, then `{delta}` token frames, then a
   * terminal `done` frame carrying any PROPOSED actions. `onDelta` (optional) sees
   * each token run as it arrives; the resolved result is the accumulated text plus
   * the honest disclosure.
   *
   * Proposed actions are returned for human review — this method never executes one.
   */
  async invoke(req: InvokeRequest, onDelta?: (delta: string) => void): Promise<InvokeResult> {
    const res = await this.fetcher('/api/assist/invoke', {
      method: 'POST',
      headers: { 'content-type': 'application/json', accept: 'text/event-stream' },
      body: JSON.stringify(requestToWire(req)),
    });
    if (!res.ok) throw new AssistError(res.status, `assist invoke failed (${res.status})`);

    // A non-streaming endpoint (or a test double) may answer with the whole result.
    if (res.headers.get('content-type')?.includes('application/json') === true) {
      const wire = (await res.json()) as WireInvokeResult;
      const text = wire.text ?? '';
      if (text.length > 0) onDelta?.(text);
      return {
        text,
        disclosure: wire.disclosure === undefined ? NO_DISCLOSURE : disclosureFromWire(wire.disclosure),
        actions: (wire.actions ?? []).map(actionFromWire),
      };
    }

    let text = '';
    let disclosure = NO_DISCLOSURE;
    let actions: ProposedAction[] = [];
    let failed = false;

    const handle = (frame: SseFrame): void => {
      let payload: unknown;
      try {
        payload = JSON.parse(frame.data);
      } catch {
        return; // a frame we cannot read is dropped, never rendered raw
      }
      switch (frame.event) {
        case 'disclosure':
          disclosure = disclosureFromWire(payload as WireDisclosure);
          break;
        case 'done':
          actions = ((payload as { actions?: WireProposedAction[] }).actions ?? []).map(actionFromWire);
          break;
        case 'error':
          failed = true;
          break;
        default: {
          const delta = (payload as { delta?: string }).delta ?? '';
          if (delta.length > 0) {
            text += delta;
            onDelta?.(delta);
          }
        }
      }
    };

    const reader = new SseReader();
    const body = res.body;
    if (body !== null && body !== undefined && typeof body.getReader === 'function') {
      const stream = body.getReader();
      const decoder = new TextDecoder();
      for (;;) {
        const { done, value } = await stream.read();
        if (done === true) {
          // Flush any bytes the decoder held back mid-character.
          for (const frame of reader.push(decoder.decode())) handle(frame);
          break;
        }
        for (const frame of reader.push(decoder.decode(value, { stream: true }))) handle(frame);
      }
    } else {
      // No streaming body available (jsdom, a polyfilled fetch): the whole SSE body
      // is still parseable in one pass.
      for (const frame of reader.push(await res.text())) handle(frame);
    }
    for (const frame of reader.end()) handle(frame);

    if (failed) throw new AssistError(502, 'assist stream failed');
    return { text, disclosure, actions };
  }

  /**
   * Transcribe dictation audio via the Assist STT slot (used when the browser has no
   * SpeechRecognition). The audio is base64-encoded into the JSON body the gateway's
   * transcribe route accepts; the server proxies it, so the browser never contacts
   * the AI host.
   */
  async transcribe(audio: Blob): Promise<string> {
    const res = await this.fetcher('/api/assist/transcribe', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ audioBase64: await blobToBase64(audio), mime: audio.type || 'audio/webm' }),
    });
    if (!res.ok) throw new AssistError(res.status, `transcribe failed (${res.status})`);
    const out = (await res.json()) as { text: string };
    return out.text;
  }
}

/**
 * Base64-encode a blob without pulling in a dependency.
 *
 * `Blob.arrayBuffer()` is not universal — it is missing on Safari below 14 and on
 * some WebViews (and on jsdom, which is how this was caught) — so fall back to
 * `FileReader`, which every target has. `readAsDataURL` hands back base64 already,
 * so the fallback needs no byte loop of its own.
 */
async function blobToBase64(blob: Blob): Promise<string> {
  if (typeof blob.arrayBuffer === 'function') {
    const bytes = new Uint8Array(await blob.arrayBuffer());
    let binary = '';
    for (let i = 0; i < bytes.length; i += 1) binary += String.fromCharCode(bytes[i] ?? 0);
    return btoa(binary);
  }
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(new AssistError(0, 'could not read the recording'));
    reader.onload = () => {
      const url = String(reader.result);
      resolve(url.slice(url.indexOf(',') + 1));
    };
    reader.readAsDataURL(blob);
  });
}

/** Raised when an `/api/assist/*` request fails. */
export class AssistError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'AssistError';
    this.status = status;
  }
}
