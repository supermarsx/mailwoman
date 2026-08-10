// t19-e7: the `/api/assist/*` transport itself (SPEC §14.3).
//
// `AssistService.invoke` consumes Server-Sent Events — a leading `disclosure` frame,
// `{delta}` token frames, then a terminal `done` frame carrying the actions the
// assistant PROPOSED. The component tests in `assist.test.tsx` stub `invoke` at the
// method level, so nothing there exercises the framing; these tests do, against the
// real parser.
//
// The frame boundaries are the point. A record split across transport chunks is the
// failure that only appears against a real network — a decoder that assumes one
// chunk is one frame passes every single-chunk test and then drops or mangles frames
// in production — so it is asserted explicitly here, byte-split mid-line and
// mid-JSON.
//
// Responses are hand-built rather than `new Response(...)`: these tests need to
// choose the chunk boundaries, which a Response built from a string does not let us
// do. The shape used is exactly the part of `Response` the service touches.

import { describe, it, expect, vi } from 'vitest';
import { AssistError, AssistService, type Fetcher } from './service.ts';
import type { InvokeRequest } from './types.ts';

/** A `Response` that streams `chunks` verbatim, so the test picks the boundaries. */
function sseResponse(chunks: readonly string[]): Response {
  const encoder = new TextEncoder();
  let i = 0;
  return {
    ok: true,
    status: 200,
    headers: { get: (name: string) => (name.toLowerCase() === 'content-type' ? 'text/event-stream' : null) },
    body: {
      getReader: () => ({
        read: () =>
          Promise.resolve(
            i < chunks.length ? { done: false, value: encoder.encode(chunks[i++]) } : { done: true, value: undefined },
          ),
      }),
    },
  } as unknown as Response;
}

/** A `Response` with no streaming body — the `res.text()` fallback path. */
function bodylessResponse(body: string, contentType = 'text/event-stream'): Response {
  return {
    ok: true,
    status: 200,
    headers: { get: (name: string) => (name.toLowerCase() === 'content-type' ? contentType : null) },
    body: null,
    text: () => Promise.resolve(body),
  } as unknown as Response;
}

function jsonResponse(value: unknown): Response {
  return {
    ok: true,
    status: 200,
    headers: { get: (name: string) => (name.toLowerCase() === 'content-type' ? 'application/json' : null) },
    body: null,
    json: () => Promise.resolve(value),
  } as unknown as Response;
}

function errorResponse(status: number): Response {
  return {
    ok: false,
    status,
    headers: { get: () => null },
  } as unknown as Response;
}

const DISCLOSURE_FRAME =
  'event: disclosure\ndata: {"endpoint_host":"ai.example.com","sent":["your prompt"],"withheld":["1 attachment(s)"]}\n\n';

function deltaFrame(text: string): string {
  return `data: ${JSON.stringify({ delta: text, done: false })}\n\n`;
}

const req: InvokeRequest = {
  capability: 'assistant',
  prompt: 'what changed?',
  context: [{ account: 'acct-1', folder: 'Inbox', text: 'body text', kind: 'plain' }],
};

describe('AssistService.invoke — SSE transport', () => {
  it('assembles the text, the disclosure and the proposed actions from the frames', async () => {
    const service = new AssistService(async () =>
      sseResponse([
        DISCLOSURE_FRAME,
        deltaFrame('I can look '),
        deltaFrame('that up.'),
        'event: done\ndata: {"actions":[{"id":"act-1","tool":"mail.search","summary":"Search for Bob","would_send":false}]}\n\n',
      ]),
    );

    const result = await service.invoke(req);

    expect(result.text).toBe('I can look that up.');
    expect(result.disclosure).toEqual({
      endpointHost: 'ai.example.com',
      sent: ['your prompt'],
      withheld: ['1 attachment(s)'],
    });
    expect(result.actions).toEqual([
      { id: 'act-1', tool: 'mail.search', summary: 'Search for Bob', wouldSend: false },
    ]);
  });

  // The case that only reproduces against a real network: transport chunks do not
  // respect frame boundaries. Every split below lands mid-record — inside the event
  // name, inside a `data:` line, and inside the JSON payload.
  it('decodes a frame that arrives split across transport chunks, exactly once', async () => {
    const service = new AssistService(async () =>
      sseResponse([
        'event: disclo',
        'sure\ndata: {"endpoint_host":"ai.exa',
        'mple.com","sent":[],"withheld":[]}\n',
        '\ndata: {"delta":"Hel',
        'lo","done":false}\n\ndata: {"delta":" world","done":false}\n\nevent: do',
        'ne\ndata: {"actions":[{"id":"act-1","tool":"mail.send","summary":"Reply","would_send":true}]}\n\n',
      ]),
    );

    const result = await service.invoke(req);

    expect(result.text).toBe('Hello world');
    expect(result.disclosure.endpointHost).toBe('ai.example.com');
    expect(result.actions).toHaveLength(1);
    expect(result.actions[0]?.wouldSend).toBe(true);
  });

  it('tolerates keep-alive comments, CRLF framing and a record with no terminator', async () => {
    const service = new AssistService(async () =>
      sseResponse([
        ':keep-alive\r\n\r\n',
        'event: disclosure\r\ndata: {"endpoint_host":"h","sent":[],"withheld":[]}\r\n\r\n',
        ':keep-alive\r\n\r\n',
        'data: {"delta":"tail","done":false}\r\n\r\n',
        // A terminal frame the server never got to terminate with a blank line.
        'event: done\ndata: {"actions":[]}',
      ]),
    );

    const result = await service.invoke(req);

    expect(result.text).toBe('tail');
    expect(result.disclosure.endpointHost).toBe('h');
    expect(result.actions).toEqual([]);
  });

  it('reports each token run through onDelta, in order, as it arrives', async () => {
    const seen: string[] = [];
    const service = new AssistService(async () =>
      sseResponse([DISCLOSURE_FRAME, deltaFrame('one '), deltaFrame('two '), deltaFrame('three'), 'event: done\ndata: {"actions":[]}\n\n']),
    );

    const result = await service.invoke(req, (delta) => seen.push(delta));

    expect(seen).toEqual(['one ', 'two ', 'three']);
    expect(result.text).toBe(seen.join(''));
  });

  it('rejects with an AssistError when the stream ends in an error frame', async () => {
    const service = new AssistService(async () =>
      sseResponse([DISCLOSURE_FRAME, deltaFrame('partial'), 'event: error\ndata: {"error":"assist stream failed"}\n\n']),
    );

    await expect(service.invoke(req)).rejects.toBeInstanceOf(AssistError);
  });

  it('claims nothing left the device when the server sent no disclosure frame', async () => {
    const service = new AssistService(async () => sseResponse([deltaFrame('hi'), 'event: done\ndata: {"actions":[]}\n\n']));

    const result = await service.invoke(req);

    expect(result.disclosure).toEqual({ endpointHost: '', sent: [], withheld: [] });
  });

  it('parses the whole body in one pass when the response has no streaming body', async () => {
    const service = new AssistService(async () =>
      bodylessResponse(`${DISCLOSURE_FRAME}${deltaFrame('buffered')}event: done\ndata: {"actions":[]}\n\n`),
    );

    const result = await service.invoke(req);

    expect(result.text).toBe('buffered');
    expect(result.disclosure.endpointHost).toBe('ai.example.com');
  });

  it('still understands a non-streaming JSON response', async () => {
    const service = new AssistService(async () =>
      jsonResponse({
        text: 'whole answer',
        disclosure: { endpoint_host: 'json.example', sent: ['your prompt'], withheld: [] },
        actions: [{ id: 'a1', tool: 'mail.send', summary: 'Send it', would_send: true }],
      }),
    );

    const seen: string[] = [];
    const result = await service.invoke(req, (delta) => seen.push(delta));

    expect(result.text).toBe('whole answer');
    expect(result.disclosure.endpointHost).toBe('json.example');
    expect(result.actions[0]?.wouldSend).toBe(true);
    expect(seen).toEqual(['whole answer']);
  });

  it('throws with the status when the gateway refuses the call', async () => {
    const service = new AssistService(async () => errorResponse(403));

    await expect(service.invoke(req)).rejects.toMatchObject({ status: 403 });
  });
});

describe('AssistService request bodies', () => {
  it('sends capability, the scope derived from the context accounts, and the input', async () => {
    const fetcher = vi.fn<Fetcher>(async () => sseResponse(['event: done\ndata: {"actions":[]}\n\n']));
    const service = new AssistService(fetcher);

    await service.invoke({
      capability: 'summarize',
      prompt: 'summarize this',
      context: [
        { account: 'acct-1', folder: 'Inbox', text: 'a', kind: 'plain' },
        { account: 'acct-2', folder: 'Inbox', text: 'b', kind: 'plain' },
        // A repeat account must not be listed twice, and an unattributed item
        // must not put an empty account in the scope.
        { account: 'acct-1', folder: 'Archive', text: 'c', kind: 'plain' },
        { account: '', folder: '', text: 'd', kind: 'plain' },
      ],
    });

    const [url, init] = fetcher.mock.calls[0] ?? [];
    expect(url).toBe('/api/assist/invoke');
    const body = JSON.parse(String(init?.body)) as {
      capability: string;
      scope: { accounts: string[]; include_e2ee: boolean; include_attachments: boolean };
      input: { prompt: string; context: unknown[] };
    };
    expect(body.capability).toBe('summarize');
    expect(body.scope.accounts).toEqual(['acct-1', 'acct-2']);
    // The client never opts in to either ceiling; only an admin can.
    expect(body.scope.include_e2ee).toBe(false);
    expect(body.scope.include_attachments).toBe(false);
    expect(body.input.prompt).toBe('summarize this');
    expect(body.input.context).toHaveLength(4);
  });

  it('posts dictation audio as base64 JSON carrying the blob mime type', async () => {
    const fetcher = vi.fn<Fetcher>(async () => jsonResponse({ text: 'transcribed words' }));
    const service = new AssistService(fetcher);

    const audio = new Blob([new Uint8Array([1, 2, 3, 250])], { type: 'audio/ogg' });
    const text = await service.transcribe(audio);

    expect(text).toBe('transcribed words');
    const [url, init] = fetcher.mock.calls[0] ?? [];
    expect(url).toBe('/api/assist/transcribe');
    const body = JSON.parse(String(init?.body)) as { audioBase64: string; mime: string };
    expect(body.mime).toBe('audio/ogg');
    expect(body.audioBase64).toBe(btoa(String.fromCharCode(1, 2, 3, 250)));
  });
});

describe('AssistService.getConfig', () => {
  it('falls back to the disabled config when the gateway is unreachable', async () => {
    const service = new AssistService(async () => {
      throw new Error('network down');
    });

    const config = await service.getConfig();

    expect(config.availability).toBe('disabled');
    expect(config.capabilities).toEqual([]);
  });
});
