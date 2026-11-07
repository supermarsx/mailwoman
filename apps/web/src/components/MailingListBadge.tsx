import {resourceValue} from './resourceValue.ts';
import {createMemo, createResource, createSignal, createUniqueId, For, onMount, Show, type JSX} from 'solid-js';
import {listMailDraft, mailingListCommands, mailingListId, mailingListName, safeListHref, unsubscribeMethods, unsubscribeOneClick, type ListLink} from '../api/mailingList.ts';
import {settings} from '../api/settings.ts';
import {inspectMessageLink, type MessageLink} from '../viewers/messageLinks.ts';
import {formatDateTime} from '../i18n/format.ts';
import {isolate, loadCatalog, t} from '../i18n';
import {badgeVisible} from '../state/readerBadges.ts';
import {clearUnsubscribed, listKey, markUnsubscribed, senderKept, setSenderKept, stillReceiving, unsubscribedList, type UnsubscribeMethod} from '../state/listSubscription.ts';
import {BadgeContextMenu} from './BadgeContextMenu.tsx';
import type {ComposeInitial} from './compose/reply.ts';
import {Icon, type IconName} from './Icon.tsx';
import {LinkReview} from './MessageLinks.tsx';
import {SecurityPopup} from './SecurityPanel.tsx';
import {toast} from './toast/toast.ts';
import {listRuleDraft, queueRuleDraft} from '../state/ruleDraft.ts';
import type {ListCount} from '../api/listCount.ts';
import './readerBadges.css';

export type ListSenderAction = 'block' | 'silence' | 'report-junk';
export type UnsubscribeMode = 'ask' | 'one-click-direct' | 'open-link';
export const unsubscribeMode = (): UnsubscribeMode => {
  const value = settings()['mail.list.unsubscribeMode']?.value;
  return value === 'one-click-direct' || value === 'open-link' ? value : 'ask';
};

const INFO_LINKS = [['archive', 'mailing-list-archive'], ['help', 'mailing-list-help'], ['post', 'mailing-list-post'], ['owner', 'mailing-list-owner']] as const;

function ActionButton(a: {testid: string; label: string; icon: IconName; onClick: () => void; disabled?: boolean | undefined; title?: string | undefined}): JSX.Element {
  return <button type="button" class="btn btn--ghost list-detail__action" data-testid={a.testid} disabled={a.disabled} title={a.title} onClick={() => a.onClick()}><Icon name={a.icon}/>{a.label}</button>;
}

/**
 * The mailing list of the open message, as a badge in the reader's header row (after the spam score). It replaces the
 * old notice bar. The badge opens a dialog: the list's identity, its links as plain text, how the message offers to
 * unsubscribe, how much mail from this sender is here, and the actions. Header data is the sender's claim: nothing is
 * fetched when it renders, every web link opens only on a click (new tab, no referrer), and the one-click request is
 * sent by the server so the list host never sees the reader's address.
 */
export function MailingListBadge(props: {
  headers: string;
  accountId: string | null;
  sender: string;
  /** The open message's received time, for the "still receiving" check. */
  receivedAt?: string | null | undefined;
  /** Messages from this sender in the mailbox, and the newest one's time. */
  stats?: {count: number; last: string | null} | undefined;
  onCompose?: ((draft: ComposeInitial) => void) | undefined;
  onCleanup?: (() => void) | undefined;
  onSenderControl?: ((action: ListSenderAction) => Promise<boolean>) | undefined;
  onOpenRules?: (() => void) | undefined;
  /** Messages from this list by List-Id (the engine's `list:` search); `null` falls back to the sender count. */
  listCount?: ((listId: string) => Promise<ListCount | null>) | undefined;
  /** Save the rule that mutes a list: mark read and archive, matched by List-Id. */
  onMuteList?: ((listId: string) => Promise<boolean>) | undefined;
}): JSX.Element {
  onMount(() => void loadCatalog('mailing-list'));
  const id = `mw-list-${createUniqueId()}`;
  let trigger: HTMLButtonElement | undefined;
  const [open, setOpen] = createSignal(false);
  const [confirming, setConfirming] = createSignal<'unsubscribe' | 'clean' | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [review, setReview] = createSignal<{links: MessageLink[]; method: UnsubscribeMethod; thenClean: boolean} | null>(null);
  const [pending, setPending] = createSignal<{method: UnsubscribeMethod; thenClean: boolean} | null>(null);

  const listId = createMemo(() => mailingListId(props.headers));
  const name = createMemo(() => mailingListName(props.headers));
  const methods = createMemo(() => unsubscribeMethods(props.headers));
  const present = (): boolean => listId() !== null || methods().web !== null || methods().mail !== null;
  const key = (): string => listKey(props.accountId, listId() ?? props.sender);
  const record = (): ReturnType<typeof unsubscribedList> => unsubscribedList(key());
  const latest = (): string | null => {
    const times = [props.receivedAt, props.stats?.last].filter((v): v is string => !!v && Number.isFinite(Date.parse(v)));
    return times.sort((a, b) => Date.parse(b) - Date.parse(a))[0] ?? null;
  };
  const ignored = (): boolean => stillReceiving(record(), latest());
  const kept = (): boolean => senderKept(props.sender, listId());
  const [counted] = createResource(() => (open() && listId() && props.listCount ? listId() : null), (id) => props.listCount!(id!));
  const date = (ms: number): string => formatDateTime(new Date(ms).toISOString());

  const display = (): string => name() ?? listId() ?? props.sender;
  const label = (): string => record() ? t('mailing-list-badge-unsubscribed') : name() ? t('mailing-list-badge-named', {name: isolate(name()!)}) : t('mailing-list-bar');
  const tone = (): string => ignored() ? 'warning' : record() ? 'good' : 'neutral';
  const state = (): string => record() ? (ignored() ? 'ignored' : 'unsubscribed') : 'subscribed';
  const tooltip = (): string => {
    const rec = record();
    if (rec) return t(ignored() ? 'mailing-list-badge-tooltip-ignored' : 'mailing-list-badge-tooltip-unsubscribed', {name: isolate(display()), date: date(rec.at)});
    return t('mailing-list-badge-tooltip', {name: isolate(display())});
  };
  const visible = (): boolean => present() && badgeVisible('list', ignored());

  const close = (): void => { setOpen(false); setConfirming(null); trigger?.focus(); };
  const links = createMemo(() => {
    const actions = mailingListCommands(props.headers).actions;
    return INFO_LINKS.flatMap(([action, text]) => (actions[action] ?? []).map(link => ({text, link})));
  });
  const choice = (): UnsubscribeMethod | null => {
    const m = methods();
    if (m.oneClick && unsubscribeMode() !== 'open-link') return 'one-click';
    return m.web ? 'link' : m.mail ? 'mail' : null;
  };

  async function copy(text: string): Promise<void> {
    try { await navigator.clipboard.writeText(text); toast.success(t('mailing-list-copied')); } catch { toast.error(t('mailing-list-copy-failed')); }
  }
  function openReview(method: UnsubscribeMethod, thenClean: boolean): void {
    const link = method === 'mail' ? methods().mail : methods().web;
    const checked = link ? inspectMessageLink(link.href, '') : null;
    if (checked) setReview({links: [checked], method, thenClean});
  }
  async function sendOneClick(thenClean: boolean): Promise<void> {
    const link = methods().oneClick;
    if (!link) return;
    setBusy(true); setConfirming(null);
    const result = await unsubscribeOneClick(link.href);
    setBusy(false);
    if (result.ok) {
      markUnsubscribed(key(), 'one-click');
      toast.success(t('mailing-list-done', {host: link.host}));
      if (thenClean) { close(); props.onCleanup?.(); }
    } else {
      toast.error(t('mailing-list-failed', {reason: result.reason}));
    }
  }
  function start(thenClean: boolean): void {
    const method = choice();
    if (method === null) return;
    if (method === 'one-click') {
      if (unsubscribeMode() === 'one-click-direct') void sendOneClick(thenClean);
      else setConfirming(thenClean ? 'clean' : 'unsubscribe');
    } else openReview(method, thenClean);
  }
  function reviewClosed(): void {
    const value = review();
    setReview(null);
    if (value) setPending({method: value.method, thenClean: value.thenClean});
  }
  function confirmDone(): void {
    const value = pending();
    if (!value) return;
    markUnsubscribed(key(), value.method);
    setPending(null);
    toast.success(t('mailing-list-remembered'));
    if (value.thenClean) { close(); props.onCleanup?.(); }
  }
  async function senderAction(action: ListSenderAction, done: string): Promise<void> {
    if (!props.onSenderControl) return;
    setBusy(true);
    const ok = await props.onSenderControl(action).catch(() => false);
    setBusy(false);
    if (ok) toast.success(t(done)); else toast.error(t('mailing-list-action-failed'));
  }
  function compose(link: ListLink): void {
    const draft = listMailDraft(link);
    if (!draft) return;
    close();
    props.onCompose?.({...draft, mode: 'new', cc: '', inReplyTo: [], references: [], attachments: [], quotesDecrypted: false});
  }
  function toRules(action: 'move' | 'archive'): void {
    // With a List-Id the editor opens with that condition filled in and the chosen action; without one the user starts from scratch.
    const id = listId();
    if (id) queueRuleDraft(listRuleDraft(id, action));
    toast.info(t('mailing-list-rule-hint', {id: id ?? props.sender}));
    close();
    props.onOpenRules?.();
  }
  async function mute(): Promise<void> {
    const id = listId();
    if (!id || !props.onMuteList) { await senderAction('silence', 'mailing-list-toast-muted'); return; }
    setBusy(true);
    const ok = await props.onMuteList(id).catch(() => false);
    setBusy(false);
    if (ok) toast.success(t('mailing-list-toast-muted')); else toast.error(t('mailing-list-action-failed'));
  }

  return <Show when={visible()}>
    <BadgeContextMenu badge="list">
      <button ref={trigger} type="button" class="mw-badge mw-badge--list" data-testid="reader-list-badge" data-tone={tone()} data-state={state()}
        aria-haspopup="dialog" aria-expanded={open()} aria-controls={`${id}-dialog`} aria-label={tooltip()} data-tooltip={tooltip()} onClick={() => setOpen(v => !v)}>
        <Icon name="mail"/><span class="mw-badge__label">{label()}</span>
      </button>
      <Show when={open()}>
        <SecurityPopup id={`${id}-dialog`} title={t('mailing-list-bar')} onClose={close}>
          <div class="list-detail" data-testid="reader-list-detail">
            <dl class="list-detail__ident">
              <div><dt>{t('mailing-list-name')}</dt><dd><bdi>{display()}</bdi></dd></div>
              <Show when={listId()}>{value => <div><dt>{t('mailing-list-id')}</dt><dd dir="ltr">{value()}</dd></div>}</Show>
              <div><dt>{t('mailing-list-address')}</dt><dd dir="ltr">{props.sender}</dd></div>
            </dl>
            <p class="list-detail__status" role="status" data-testid="reader-list-status" data-state={state()}>
              <Show when={record()} fallback={t('mailing-list-status-subscribed')}>
                {rec => <>{t('mailing-list-status-unsubscribed', {date: date(rec().at)})}<Show when={ignored()}><strong class="list-detail__warn">{' '}{t('mailing-list-status-ignored')}</strong></Show>{' '}<button type="button" class="btn btn--ghost list-detail__small" onClick={() => clearUnsubscribed(key())}>{t('mailing-list-forget')}</button></>}
              </Show>
            </p>
            <Show when={resourceValue(counted)}>{c => <p class="list-detail__stats" data-testid="reader-list-count">{c().count > 0 ? t('mailing-list-list-count', {count: c().count, date: c().last ? formatDateTime(c().last!) : '-'}) : t('mailing-list-list-count-none')}</p>}</Show>
            <Show when={props.stats}>{stats => <p class="list-detail__stats" data-testid="reader-list-stats">{stats().count > 0 ? t('mailing-list-stats', {count: stats().count, date: stats().last ? formatDateTime(stats().last!) : '-'}) : t('mailing-list-stats-none')}</p>}</Show>
            <Show when={links().length > 0}>
              <section aria-label={t('mailing-list-links')}>
                <h4 class="list-detail__heading">{t('mailing-list-links')}</h4>
                <ul class="list-detail__links" data-testid="reader-list-links">
                  <For each={links()}>{item => <li>
                    <span class="list-detail__kind">{t(item.text)}</span>
                    <span class="list-detail__href" dir="ltr">{item.link.kind === 'mail' ? item.link.host : item.link.href}</span>
                    <Show when={safeListHref(item.link.href)?.kind === 'web'}><a class="btn btn--ghost list-detail__small" href={item.link.href} target="_blank" rel="noopener noreferrer" referrerpolicy="no-referrer">{t('mailing-list-open-new-tab')}</a></Show>
                    <Show when={item.link.kind === 'mail' && props.onCompose}><button type="button" class="btn btn--ghost list-detail__small" onClick={() => compose(item.link)}>{t('mailing-list-write')}</button></Show>
                    <button type="button" class="btn btn--ghost list-detail__small" onClick={() => void copy(item.link.kind === 'mail' ? item.link.host : item.link.href)}>{t('mailing-list-copy')}</button>
                  </li>}</For>
                </ul>
              </section>
            </Show>
            <Show when={mailingListCommands(props.headers).postingForbidden}><p class="list-detail__stats" data-testid="reader-list-no-post">{t('mailing-list-no-post')}</p></Show>
            <section aria-label={t('mailing-list-methods')}>
              <h4 class="list-detail__heading">{t('mailing-list-methods')}</h4>
              <ul class="list-detail__methods" data-testid="reader-list-methods">
                <Show when={methods().oneClick}>{m => <li data-method="one-click">{t('mailing-list-method-one-click', {host: m().host})}</li>}</Show>
                <Show when={methods().web}>{m => <li data-method="link">{t('mailing-list-method-link', {host: m().host})}</li>}</Show>
                <Show when={methods().mail}>{m => <li data-method="mail">{t('mailing-list-method-mail', {address: m().host})}</li>}</Show>
                <Show when={choice() === null}><li data-method="none">{t('mailing-list-method-none')}</li></Show>
              </ul>
            </section>
            <Show when={confirming()}>{mode => <div class="list-detail__confirm" role="group" aria-label={t('mailing-list-confirm-title')} data-testid="reader-list-confirm">
              <p>{t('mailing-list-confirm', {host: methods().oneClick?.host ?? ''})}</p>
              <button type="button" class="btn" data-testid="reader-list-confirm-send" disabled={busy()} onClick={() => void sendOneClick(mode() === 'clean')}>{t('mailing-list-confirm-send')}</button>
              <button type="button" class="btn btn--ghost" onClick={() => setConfirming(null)}>{t('mailing-list-confirm-cancel')}</button>
            </div>}</Show>
            <Show when={pending()}><div class="list-detail__confirm" role="group" aria-label={t('mailing-list-mark-done')} data-testid="reader-list-pending">
              <p>{t('mailing-list-leaves-app')}</p>
              <button type="button" class="btn" onClick={confirmDone}>{t('mailing-list-mark-done')}</button>
              <button type="button" class="btn btn--ghost" onClick={() => setPending(null)}>{t('mailing-list-confirm-cancel')}</button>
            </div></Show>
            <div class="list-detail__actions" role="group" aria-label={t('mailing-list-title')}>
              <ActionButton testid="reader-list-unsubscribe" icon="disable" label={t('mailing-list-unsubscribe')} disabled={busy() || choice() === null} onClick={() => start(false)}/>
              <ActionButton testid="reader-list-unsubscribe-clean" icon="sweep" label={t('mailing-list-action-unsubscribe-clean')} disabled={busy() || choice() === null || kept() || !props.onCleanup} onClick={() => start(true)}/>
              <ActionButton testid="reader-list-folder" icon="folder" label={t('mailing-list-action-folder')} disabled={!props.onOpenRules} onClick={() => toRules('move')}/>
              <ActionButton testid="reader-list-mute" icon="mute" label={t('mailing-list-action-mute')} disabled={busy() || (!props.onSenderControl && !(listId() && props.onMuteList))} onClick={() => void mute()}/>
              <ActionButton testid="reader-list-keep" icon={kept() ? 'close' : 'check'} label={kept() ? t('mailing-list-action-unkeep') : t('mailing-list-action-keep')} title={t('mailing-list-keep-help')} onClick={() => { const keep = !kept(); void setSenderKept(props.sender, keep, listId()).then((saved) => { if (!saved) toast.error(t('mailing-list-keep-not-saved')); }); toast.success(t(keep ? 'mailing-list-toast-kept' : 'mailing-list-toast-unkept')); }}/>
              <ActionButton testid="reader-list-block" icon="block" label={t('mailing-list-action-block')} disabled={busy() || !props.onSenderControl} onClick={() => void senderAction('block', 'mailing-list-toast-blocked')}/>
              <ActionButton testid="reader-list-filter" icon="filter" label={t('mailing-list-action-filter')} disabled={!props.onOpenRules} onClick={() => toRules('archive')}/>
              <ActionButton testid="reader-list-spam" icon="spam" label={t('mailing-list-action-spam')} disabled={busy() || !props.onSenderControl} onClick={() => void senderAction('report-junk', 'mailing-list-toast-reported')}/>
              <ActionButton testid="reader-list-sweep" icon="sweep" label={t('mailing-list-action-clean')} title={kept() ? t('mailing-list-kept-note') : t('mailing-list-action-clean-help')} disabled={kept() || !props.onCleanup} onClick={() => { close(); props.onCleanup?.(); }}/>
            </div>
          </div>
        </SecurityPopup>
      </Show>
      <Show when={review()}>{value => <LinkReview links={value().links} truncated={false} title={t('mailing-list-unsubscribe')} help={`${t('mailing-list-unverified')} ${t('mailing-list-leaves-app')}`} onClose={reviewClosed} {...(props.onCompose ? {onCompose: props.onCompose} : {})}/>}</Show>
    </BadgeContextMenu>
  </Show>;
}
