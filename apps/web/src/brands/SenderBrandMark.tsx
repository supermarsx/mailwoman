import {Show, type JSX} from 'solid-js';
import {settings} from '../api/settings.ts';
import type {AuthVerdict} from '../api/security-types.ts';
import {BrandLogo} from './BrandLogo.tsx';
import {brandAvatarFor, DEFAULT_REQUIRE_AUTH, isRequireAuth} from './senderBrand.ts';

/** Verified-brand logo beside a sender (`mail.brandAvatars`, default on). Shown for the brand's domain and its subdomains unless the verdict says the message failed authentication (`mail.brandAvatars.requireAuth`). */
export function SenderBrandMark(props: {address: string | null | undefined; auth: AuthVerdict | null | undefined}): JSX.Element {
  const mode = () => { const value = settings()['mail.brandAvatars.requireAuth']?.value; return isRequireAuth(value) ? value : DEFAULT_REQUIRE_AUTH; };
  const brand = () => (settings()['mail.brandAvatars']?.value === false ? undefined : brandAvatarFor(props.address, props.auth, mode()));
  return <Show when={brand()}>{entry => <BrandLogo brand={entry().id} size={16} decorative />}</Show>;
}
