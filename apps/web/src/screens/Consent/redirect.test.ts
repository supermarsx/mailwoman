// t32-h10: the consent page never navigates to an address that runs code or
// renders content in its place, whatever the decision endpoint returned.

import { describe, it, expect } from 'vitest';
import { ConsentService, isNavigableRedirect, type AuthorizeParams } from './service.ts';

const PARAMS: AuthorizeParams = {
  responseType: 'code',
  clientId: 'c1',
  redirectUri: 'https://app.example/cb',
  codeChallenge: 'x',
  codeChallengeMethod: 'S256',
  resource: 'https://mail.example',
};

describe('consent redirect target', () => {
  it('allows https, http and a native app scheme', () => {
    for (const ok of ['https://app.example/cb?code=1', 'http://127.0.0.1:8123/cb', 'com.example.app:/cb?code=1']) {
      expect(isNavigableRedirect(ok)).toBe(true);
    }
  });

  it('refuses script, data and other in-place schemes, and garbage', () => {
    for (const bad of [
      'javascript:alert(1)//?error=access_denied',
      'JavaScript:alert(1)',
      ' javascript:alert(1)',
      'data:text/html,<p>x</p>',
      'vbscript:msgbox(1)',
      'blob:https://app.example/1',
      'file:///etc/passwd',
      'about:blank',
      '/relative/path',
      '',
    ]) {
      expect(isNavigableRedirect(bad), bad).toBe(false);
    }
  });

  it('decide() rejects a decision that names a refused address', async () => {
    const svc = new ConsentService(
      async () => new Response(JSON.stringify({ redirectUri: 'javascript:alert(1)?error=access_denied' }), { status: 200 }),
    );
    await expect(svc.decide(PARAMS, false)).rejects.toThrow('unusable address');
  });
});
