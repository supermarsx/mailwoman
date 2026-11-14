import {badgeVisible} from '../state/readerBadges.ts';
import {formatDateTime as regionalDateTime} from '../i18n/format.ts';
// Reader badges for the optional message headers: Sensitivity, Reply-By, Expires
// (with the expired state and the user's "expired messages" choice) and
// Supersedes. The values come with the opened message (Email/get header:<Name>:asText).
import { Show, createResource, type JSX } from 'solid-js';
import { resourceValue } from './resourceValue.ts';
import { t, loadCatalog } from '../i18n/index.ts';
import { settings } from '../api/settings.ts';
import { factsFromEmail, hasFacts, isExpired, isExpiryMode, type HeaderFacts } from './readerHeaders.ts';
import './headerFacts.css';

export function HeaderFactsBadges(props: {
  email: Parameters<typeof factsFromEmail>[0];
  now?: () => Date;
  onSearch?: (messageId: string) => void;
  /** This message's own Message-ID, and the lookup that says whether a newer message replaces it. */
  messageId?: string | undefined;
  supersededLookup?: ((messageId: string) => Promise<boolean>) | undefined;
  /** Show the newer message (a search for it). */
  onFindSupersedingMessage?: ((messageId: string) => void) | undefined;
}): JSX.Element {
  void loadCatalog('reader-headers');
  const visible = (): boolean => settings()['mail.badge.headers']?.value !== false;
  const mode = () => { const v = settings()['mail.expiry.hideExpired']?.value; return isExpiryMode(v) ? v : 'off'; };
  const f = (): HeaderFacts => factsFromEmail(props.email);
  const date = (d: Date): string => regionalDateTime(d);
  const expired = (): boolean => isExpired(f(), props.now?.() ?? new Date());
  const [superseded] = createResource(() => (visible() && badgeVisible('supersedes',true) && props.messageId && props.supersededLookup ? props.messageId : null), (id) => props.supersededLookup!(id));
  const supersededBadge = (): JSX.Element => <Show when={badgeVisible('supersedes',true)&&resourceValue(superseded) === true}>
    <span class="hf-badge hf-badge--warn" data-testid="hf-superseded-by" title={t('hf-superseded-by-help')}>
      {t('hf-superseded-by')}
      <Show when={props.onFindSupersedingMessage}> <button type="button" class="hf-link" onClick={() => props.onFindSupersedingMessage?.(props.messageId!)}>{t('hf-find')}</button></Show>
    </span>
  </Show>;
  return <><Show when={visible() && hasFacts(f())}>
    <span class="hf" data-testid="header-facts">
      <Show when={badgeVisible('sensitivity',!!f().sensitivity)?f().sensitivity:undefined}>{(s) => <span class={`hf-badge hf-badge--${s() === 'company-confidential' ? 'warn' : 'info'}`} data-testid="hf-sensitivity" title={t('hf-sensitivity-help')}>{t(`hf-sensitivity-${s()}`)}</span>}</Show>
      <Show when={badgeVisible('reply-by',true)?f().replyBy:undefined}>{(d) => <span class="hf-badge hf-badge--info" data-testid="hf-reply-by" title={t('hf-reply-by-help')}>{t('hf-reply-by', { date: date(d()) })}</span>}</Show>
      <Show when={badgeVisible('expires',expired()||!!f().expires&&f().expires!.getTime()-(props.now?.()??new Date()).getTime()<86400000)?f().expires:undefined}>{(d) => <>
        <span class={`hf-badge hf-badge--${expired() ? 'warn' : 'info'}`} data-testid="hf-expires" data-expired={expired() ? '' : undefined} data-mode={expired() ? mode() : undefined} title={t('hf-expires-help')}>
          {expired() ? t('hf-expired', { date: date(d()) }) : t('hf-expires', { date: date(d()) })}
        </span>
        <Show when={expired() && mode() === 'archive-suggestion'}><span class="hf-note" data-testid="hf-archive-suggestion">{t('hf-archive-suggestion')}</span></Show>
      </>}</Show>
      <Show when={badgeVisible('supersedes',true)?f().supersedes:undefined}>{(id) => <span class="hf-badge hf-badge--info" data-testid="hf-supersedes" title={t('hf-supersedes-help')}>
        {t('hf-supersedes')}
        <Show when={props.onSearch}> <button type="button" class="hf-link" onClick={() => props.onSearch?.(id())}>{t('hf-find')}</button></Show>
      </span>}</Show>
    </span>
  </Show>{supersededBadge()}</>;
}
