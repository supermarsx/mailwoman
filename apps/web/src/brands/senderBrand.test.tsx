import {describe, expect, it, vi} from 'vitest';
import {render} from '@solidjs/testing-library';
import type {AuthVerdict} from '../api/security-types.ts';

const state = vi.hoisted(() => ({value: undefined as boolean | undefined}));
vi.mock('../api/settings.ts', () => ({settings: () => (state.value === undefined ? {} : {'mail.brandAvatars': {value: state.value}})}));

import {authFailed, brandAvatarDecision, brandAvatarFor} from './senderBrand.ts';
import {SenderBrandMark} from './SenderBrandMark.tsx';

const auth = (over: Partial<{dmarc: string; aligned: boolean; dkim: string; dkimDomain: string | null}> = {}): AuthVerdict => ({
  dkim: {result: (over.dkim ?? 'pass') as 'pass', domain: over.dkimDomain === undefined ? 'paypal.com' : over.dkimDomain, selector: 's1'},
  spf: {result: 'pass', domain: 'paypal.com'},
  dmarc: {result: (over.dmarc ?? 'pass') as 'pass', policy: 'reject', aligned: over.aligned ?? true},
  arc: {result: 'none', chainLength: 0},
});

describe('brandAvatarFor', () => {
  it('shows the brand for the domain and its subdomains when the message passed', () => {
    expect(brandAvatarFor('service@paypal.com', auth())?.id).toBe('paypal');
    expect(brandAvatarFor('service@mail.paypal.com', auth({dkimDomain: 'paypal-communication.com'}))?.id).toBe('paypal');
  });
  it('keeps the brand when there is no verdict or the checks are merely absent or unaligned (known-failure, the default)', () => {
    expect(brandAvatarFor('service@em.paypal.com', null)?.id).toBe('paypal');
    expect(brandAvatarFor('service@paypal.com', undefined)?.id).toBe('paypal');
    expect(brandAvatarFor('service@paypal.com', auth({dmarc: 'none'}))?.id).toBe('paypal');
    expect(brandAvatarFor('service@paypal.com', auth({aligned: false}))?.id).toBe('paypal');
    expect(brandAvatarFor('service@paypal.com', auth({dkimDomain: null}))?.id).toBe('paypal');
  });
  it('hides the brand only on a known failure', () => {
    expect(authFailed(auth({dmarc: 'fail'}))).toBe(true);
    expect(authFailed(auth({dkim: 'fail', dmarc: 'none'}))).toBe(true);
    expect(authFailed(auth({dkim: 'fail', dmarc: 'pass'}))).toBe(false);
    expect(authFailed({...auth({dkim: 'fail', dmarc: 'none'}), spf: {result: 'fail'}})).toBe(true);
    expect(authFailed(null)).toBe(false);
    expect(brandAvatarFor('service@paypal.com', auth({dmarc: 'fail'}))).toBeUndefined();
    expect(brandAvatarDecision('service@paypal.com', auth({dmarc: 'fail'}))).toMatchObject({brand: undefined, unverified: true});
    expect(brandAvatarDecision('someone@example.org', auth({dmarc: 'fail'}))).toMatchObject({brand: undefined, unverified: false});
  });
  it('never mode ignores authentication, always mode needs an aligned pass for the same brand', () => {
    expect(brandAvatarFor('service@paypal.com', auth({dmarc: 'fail'}), 'never')?.id).toBe('paypal');
    expect(brandAvatarFor('service@paypal.com', null, 'always')).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com', auth({dmarc: 'none'}), 'always')).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com', auth({aligned: false}), 'always')).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com', auth({dkim: 'fail'}), 'always')).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com', auth({dkimDomain: 'evil.io'}), 'always')).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com', auth(), 'always')?.id).toBe('paypal');
  });
  it('never matches a lookalike, an unknown domain or a public mailbox provider', () => {
    for (const mode of ['never', 'known-failure', 'always'] as const) {
      expect(brandAvatarFor('service@paypal.com.evil.io', auth({dkimDomain: 'paypal.com.evil.io'}), mode)).toBeUndefined();
      expect(brandAvatarFor('service@evilpaypal.com', auth({dkimDomain: 'evilpaypal.com'}), mode)).toBeUndefined();
      expect(brandAvatarFor('someone@example.org', auth({dkimDomain: 'example.org'}), mode)).toBeUndefined();
      expect(brandAvatarFor('friend@gmail.com', auth({dkimDomain: 'gmail.com'}), mode)).toBeUndefined();
    }
  });
  it('shows no tile for a brand without a bundled mark', () => {
    expect(brandAvatarFor('x@cgd.pt', auth({dkimDomain: 'cgd.pt'}))).toBeUndefined();
  });
});

describe('SenderBrandMark', () => {
  it('renders when authenticated and the setting is on or unset, nothing when off or unauthenticated', () => {
    state.value = undefined;
    expect(render(() => <SenderBrandMark address="a@paypal.com" auth={auth()} />).container.querySelector('svg')).not.toBeNull();
    expect(render(() => <SenderBrandMark address="a@em.paypal.com" auth={null} />).container.querySelector('svg')).not.toBeNull();
    expect(render(() => <SenderBrandMark address="a@paypal.com" auth={auth({dmarc: 'fail'})} />).container.querySelector('svg')).toBeNull();
    state.value = false;
    expect(render(() => <SenderBrandMark address="a@paypal.com" auth={auth()} />).container.querySelector('svg')).toBeNull();
    state.value = true;
    expect(render(() => <SenderBrandMark address="a@paypal.com" auth={auth()} />).container.querySelector('svg')).not.toBeNull();
  });
});
