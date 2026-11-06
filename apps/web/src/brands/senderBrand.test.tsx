import {describe, expect, it, vi} from 'vitest';
import {render} from '@solidjs/testing-library';
import type {AuthVerdict} from '../api/security-types.ts';

const state = vi.hoisted(() => ({value: undefined as boolean | undefined}));
vi.mock('../api/settings.ts', () => ({settings: () => (state.value === undefined ? {} : {'mail.brandAvatars': {value: state.value}})}));

import {brandAvatarFor} from './senderBrand.ts';
import {SenderBrandMark} from './SenderBrandMark.tsx';

const auth = (over: Partial<{dmarc: string; aligned: boolean; dkim: string; dkimDomain: string | null}> = {}): AuthVerdict => ({
  dkim: {result: (over.dkim ?? 'pass') as 'pass', domain: over.dkimDomain === undefined ? 'paypal.com' : over.dkimDomain, selector: 's1'},
  spf: {result: 'pass', domain: 'paypal.com'},
  dmarc: {result: (over.dmarc ?? 'pass') as 'pass', policy: 'reject', aligned: over.aligned ?? true},
  arc: {result: 'none', chainLength: 0},
});

describe('brandAvatarFor', () => {
  it('shows the brand only for an aligned DMARC pass with DKIM for the same brand', () => {
    expect(brandAvatarFor('service@paypal.com', auth())?.id).toBe('paypal');
    expect(brandAvatarFor('service@mail.paypal.com', auth({dkimDomain: 'paypal-communication.com'}))?.id).toBe('paypal');
  });
  it('never shows it without authentication, with misalignment, or for another domain', () => {
    expect(brandAvatarFor('service@paypal.com', null)).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com', auth({dmarc: 'fail'}))).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com', auth({dmarc: 'none'}))).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com', auth({aligned: false}))).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com', auth({dkim: 'fail'}))).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com', auth({dkimDomain: null}))).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com', auth({dkimDomain: 'evil.io'}))).toBeUndefined();
    expect(brandAvatarFor('service@paypal.com.evil.io', auth({dkimDomain: 'paypal.com.evil.io'}))).toBeUndefined();
    expect(brandAvatarFor('service@evilpaypal.com', auth({dkimDomain: 'evilpaypal.com'}))).toBeUndefined();
    expect(brandAvatarFor('someone@example.org', auth({dkimDomain: 'example.org'}))).toBeUndefined();
  });
  it('shows no tile for a brand without a bundled mark', () => {
    expect(brandAvatarFor('x@cgd.pt', auth({dkimDomain: 'cgd.pt'}))).toBeUndefined();
  });
});

describe('SenderBrandMark', () => {
  it('renders when authenticated and the setting is on or unset, nothing when off or unauthenticated', () => {
    state.value = undefined;
    expect(render(() => <SenderBrandMark address="a@paypal.com" auth={auth()} />).container.querySelector('svg')).not.toBeNull();
    expect(render(() => <SenderBrandMark address="a@paypal.com" auth={auth({dmarc: 'fail'})} />).container.querySelector('svg')).toBeNull();
    state.value = false;
    expect(render(() => <SenderBrandMark address="a@paypal.com" auth={auth()} />).container.querySelector('svg')).toBeNull();
    state.value = true;
    expect(render(() => <SenderBrandMark address="a@paypal.com" auth={auth()} />).container.querySelector('svg')).not.toBeNull();
  });
});
