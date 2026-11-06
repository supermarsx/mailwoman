import {Show, type JSX} from 'solid-js';
import {settings} from '../api/settings.ts';
import type {AuthVerdict} from '../api/security-types.ts';
import {BrandLogo} from './BrandLogo.tsx';
import {brandAvatarFor} from './senderBrand.ts';

/** Verified-brand logo beside a sender (`mail.brandAvatars`, default on). Renders nothing unless the message authenticated for that brand's domain. */
export function SenderBrandMark(props: {address: string | null | undefined; auth: AuthVerdict | null | undefined}): JSX.Element {
  const brand = () => (settings()['mail.brandAvatars']?.value === false ? undefined : brandAvatarFor(props.address, props.auth));
  return <Show when={brand()}>{entry => <BrandLogo brand={entry().id} size={16} decorative />}</Show>;
}
