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

/** The wire shape of a single invoke response (server-accumulated + redacted). */
interface WireInvokeResult {
  text: string;
  disclosure: { endpoint_host: string; sent: string[]; withheld: string[] };
  actions?: WireProposedAction[];
}

function disclosureFromWire(w: WireInvokeResult['disclosure']): Disclosure {
  return { endpointHost: w.endpoint_host, sent: [...w.sent], withheld: [...w.withheld] };
}

function actionFromWire(a: WireProposedAction): ProposedAction {
  return { id: a.id, tool: a.tool, summary: a.summary, wouldSend: a.would_send };
}

function requestToWire(req: InvokeRequest): unknown {
  return {
    capability: req.capability,
    prompt: req.prompt,
    context: req.context.map((c) => ({ account: c.account, folder: c.folder, text: c.text, kind: c.kind })),
  };
}

/**
 * The Assist gateway service backing the whole UI.
 * Endpoints (e9 fills, e14 mounts):
 *   GET  /api/assist/config              → WireAssistConfig
 *   POST /api/assist/invoke  (InvokeBody)→ WireInvokeResult   (server proxies + redacts)
 *   POST /api/assist/transcribe (audio)  → { text }           (Assist STT fallback)
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

  /** Invoke a read-only capability. Returns the model text + the honest disclosure. */
  async invoke(req: InvokeRequest): Promise<InvokeResult> {
    const res = await this.fetcher('/api/assist/invoke', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(requestToWire(req)),
    });
    if (!res.ok) throw new AssistError(res.status, `assist invoke failed (${res.status})`);
    const wire = (await res.json()) as WireInvokeResult;
    return {
      text: wire.text,
      disclosure: disclosureFromWire(wire.disclosure),
      actions: (wire.actions ?? []).map(actionFromWire),
    };
  }

  /** Transcribe dictation audio via the Assist STT slot (used when the browser has no SpeechRecognition). */
  async transcribe(audio: Blob): Promise<string> {
    const form = new FormData();
    form.append('audio', audio);
    const res = await this.fetcher('/api/assist/transcribe', { method: 'POST', body: form });
    if (!res.ok) throw new AssistError(res.status, `transcribe failed (${res.status})`);
    const out = (await res.json()) as { text: string };
    return out.text;
  }
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
