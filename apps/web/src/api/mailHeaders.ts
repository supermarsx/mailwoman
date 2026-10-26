import { apiFetch } from './http.ts';

const LIMIT = 256 * 1024;
/** Stop at the header/body boundary even when the server ignores Range. */
export async function fetchMailHeaders(url: string, signal: AbortSignal): Promise<string> {
  const response = await apiFetch(url, { signal, headers: { range: `bytes=0-${LIMIT - 1}` } });
  if (!response.ok || !response.body) throw new Error('Message headers unavailable');
  const reader = response.body.getReader();
  const decoder = new TextDecoder('utf-8');
  let bytes = 0; let text = '';
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (value) { bytes += value.length; text += decoder.decode(value.subarray(0, Math.max(0, LIMIT - (bytes - value.length))), { stream: true }); }
      const boundary = /\r?\n\r?\n/.exec(text);
      if (boundary) return text.slice(0, boundary.index);
      if (bytes >= LIMIT) throw new Error('Message headers exceed limit');
      if (done) return text;
    }
  } finally { await reader.cancel().catch(() => undefined); }
}

export function mailHeaderValues(raw: string): Map<string, string[]> {
  const head = raw.slice(0, LIMIT).split(/\r?\n\r?\n/, 1)[0]!.replace(/\r?\n[ \t]+/g, ' ');
  const fields = new Map<string, string[]>();
  for (const line of head.split(/\r?\n/)) {
    const match = /^([a-zA-Z0-9-]+):[ \t]*(.*)$/.exec(line);
    if (!match) continue;
    const key = match[1]!.toLowerCase(); const values = fields.get(key) ?? [];
    if (values.length < 8) values.push(match[2]!.slice(0, 8192));
    fields.set(key, values);
  }
  return fields;
}
