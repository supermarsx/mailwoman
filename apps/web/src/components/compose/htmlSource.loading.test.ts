import { expect, it, vi } from 'vitest';
import { normalizeHtmlSource } from './htmlSource.ts';

const initialize = vi.hoisted(() => vi.fn());

vi.mock('../../wasm/mw-sanitize/mw_sanitize.js', () => ({
  default: initialize,
  sanitizeEmailHtml: vi.fn(() => '<p>Safe</p>'),
  sanitizeSignatureHtml: vi.fn(() => '<p>Safe</p>'),
}));

it('rejects instead of publishing unchecked markup when the sanitizer cannot load', async () => {
  initialize.mockRejectedValueOnce(new Error('WASM initialization unavailable'));
  await expect(normalizeHtmlSource('<script>unsafe()</script>', undefined, true))
    .rejects.toThrow('WASM initialization unavailable');
  initialize.mockResolvedValueOnce(undefined);
  await expect(normalizeHtmlSource('<p>Safe</p>')).resolves.toEqual({ html: '<p>Safe</p>', text: 'Safe' });
  expect(initialize).toHaveBeenCalledTimes(2);
});
