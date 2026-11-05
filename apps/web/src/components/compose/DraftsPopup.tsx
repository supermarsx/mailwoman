import { For, Show, createEffect, createMemo, createSignal, createUniqueId, type JSX } from 'solid-js';
import type { AccountDraftDocument } from '../../api/accountDrafts.ts';
import { t, isolate } from '../../i18n/index.ts';
import { formatRelative } from '../../i18n/format.ts';
import { createFocusTrap } from '../a11y/focusTrap.ts';
import { Icon } from '../Icon.tsx';
import { ConfirmDialog } from '../../screens/Admin/shell/ConfirmDialog.tsx';
import type { StoredDraft } from './drafts-store.ts';
import './DraftsPopup.css';

// The composer's Drafts popup. Prop-first: Compose owns the data and the actions
// (open / discard keep their existing semantics); this component only presents
// them, filters, selects and reports progress.

type Kind = 'account' | 'local';
interface Row {
  id: string;
  kind: Kind;
  subject: string;
  summary: string;
  savedAt: number;
  protectedRow: boolean;
  doc?: AccountDraftDocument;
  draft?: StoredDraft;
}
type RowState = { phase: 'busy' } | { phase: 'error' };

export interface DraftsPopupProps {
  open: boolean;
  accountDrafts: () => AccountDraftDocument[];
  localDrafts: () => StoredDraft[];
  accountSupported: boolean;
  loading: boolean;
  loadFailed: boolean;
  currentId: string | undefined;
  mount?: Node | undefined;
  onRetryLoad: () => void;
  onOpenAccount: (id: string) => void;
  onOpenLocal: (draft: StoredDraft) => void;
  /** Resolves true once the confirmed upstream copy is gone. */
  onDiscardAccount: (doc: AccountDraftDocument) => Promise<boolean>;
  onDiscardLocal: (id: string) => void;
  onClose: () => void;
}

export function DraftsPopup(props: DraftsPopupProps): JSX.Element {
  return (
    <Show when={props.open}>
      <DraftsPopupOpen {...props} />
    </Show>
  );
}

function DraftsPopupOpen(props: DraftsPopupProps): JSX.Element {
  const uid = createUniqueId();
  const [el, setEl] = createSignal<HTMLElement>();
  // Until the user picks a tab, follow what the account supports (its list arrives after the popup opens).
  const [picked, setTab] = createSignal<Kind | null>(null);
  const tab = (): Kind => picked() ?? (props.accountSupported || props.accountDrafts().length > 0 || props.loading ? 'account' : 'local');
  const [query, setQuery] = createSignal('');
  const [selecting, setSelecting] = createSignal(false);
  const [selected, setSelected] = createSignal<ReadonlySet<string>>(new Set<string>());
  const [states, setStates] = createSignal<Readonly<Record<string, RowState>>>({});
  const [activeId, setActiveId] = createSignal<string | null>(null);
  const [menuId, setMenuId] = createSignal<string | null>(null);
  const [confirmIds, setConfirmIds] = createSignal<string[] | null>(null);
  const [announce, setAnnounce] = createSignal('');
  let searchInput: HTMLInputElement | undefined;

  createFocusTrap(el, {
    onEscape: () => {
      if (confirmIds() !== null) return;
      if (menuId() !== null) setMenuId(null);
      else props.onClose();
    },
    initialFocus: () => searchInput,
  });

  const accountRows = createMemo<Row[]>(() =>
    props.accountDrafts().map((doc) => ({
      id: doc.id, kind: 'account' as const, subject: doc.summary.subject ?? '', summary: doc.state?.to ?? '',
      savedAt: doc.savedAt * 1000, protectedRow: doc.status === 'uncertain', doc,
    })),
  );
  const localRows = createMemo<Row[]>(() => {
    const remote = new Set(props.accountDrafts().map((d) => d.id));
    return props.localDrafts().filter((d) => !remote.has(d.id)).map((draft) => ({
      id: draft.id, kind: 'local' as const, subject: draft.subject, summary: draft.to, savedAt: draft.savedAt, protectedRow: false, draft,
    }));
  });
  const rows = createMemo<Row[]>(() => {
    const q = query().trim().toLowerCase();
    return [...(tab() === 'account' ? accountRows() : localRows())]
      .filter((r) => q === '' || r.subject.toLowerCase().includes(q) || r.summary.toLowerCase().includes(q))
      .sort((a, b) => b.savedAt - a.savedAt);
  });
  const total = (): number => accountRows().length + localRows().length;
  const rowById = (id: string): Row | undefined => [...accountRows(), ...localRows()].find((r) => r.id === id);
  const selectable = (): Row[] => rows().filter((r) => !r.protectedRow);
  const selectedRows = (): Row[] => [...selected()].map(rowById).filter((r): r is Row => r !== undefined);

  // Keep the selection to rows that still exist.
  createEffect(() => {
    const live = new Set([...accountRows(), ...localRows()].map((r) => r.id));
    const next = [...selected()].filter((id) => live.has(id));
    if (next.length !== selected().size) setSelected(new Set(next));
  });

  const label = (r: Row): string => (r.subject.trim() !== '' ? r.subject : t('mail-no-subject'));
  const toggle = (id: string): void => { setSelected((s) => { const n = new Set<string>(s); if (n.has(id)) n.delete(id); else n.add(id); return n; }); };
  const setState = (id: string, s: RowState | null): void => { setStates((m) => { const n = { ...m }; if (s) n[id] = s; else delete n[id]; return n; }); };

  function open(r: Row): void {
    setMenuId(null);
    if (r.kind === 'account') props.onOpenAccount(r.id);
    else props.onOpenLocal(r.draft!);
  }

  async function discardOne(r: Row): Promise<boolean> {
    setState(r.id, { phase: 'busy' });
    let ok = false;
    try {
      if (r.kind === 'account') ok = await props.onDiscardAccount(r.doc!);
      else { props.onDiscardLocal(r.id); ok = true; }
    } catch { ok = false; }
    setState(r.id, ok ? null : { phase: 'error' });
    return ok;
  }
  async function discardMany(ids: string[]): Promise<void> {
    const list = ids.map(rowById).filter((r): r is Row => r !== undefined)
      .sort((a, b) => Number(a.id === props.currentId) - Number(b.id === props.currentId)); // the open draft last
    let done = 0;
    let failed = 0;
    for (const r of list) { if (await discardOne(r)) done += 1; else failed += 1; }
    setAnnounce(failed === 0 ? t('mail-drafts-popup-announce-done', { count: done }) : t('mail-drafts-popup-announce-failed', { done, failed }));
  }

  const confirmRows = (): Row[] => (confirmIds() ?? []).map(rowById).filter((r): r is Row => r !== undefined);
  const askDiscard = (ids: string[]): void => { setMenuId(null); if (ids.length > 0) setConfirmIds(ids); };

  function focusRow(id: string): void {
    setActiveId(id);
    queueMicrotask(() => el()?.querySelector<HTMLElement>(`[data-row-main="${CSS.escape(id)}"]`)?.focus());
  }
  function onRowKey(e: KeyboardEvent, r: Row): void {
    const list = rows();
    const i = list.findIndex((x) => x.id === r.id);
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      const next = list[Math.max(0, Math.min(list.length - 1, i + (e.key === 'ArrowDown' ? 1 : -1)))];
      if (next) focusRow(next.id);
    } else if (e.key === 'Home' || e.key === 'End') {
      e.preventDefault();
      const next = e.key === 'Home' ? list[0] : list[list.length - 1];
      if (next) focusRow(next.id);
    } else if (e.key === ' ') {
      e.preventDefault();
      if (!r.protectedRow) { setSelecting(true); toggle(r.id); }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      open(r);
    } else if (e.key === 'Delete') {
      e.preventDefault();
      if (r.protectedRow) return;
      askDiscard(selected().has(r.id) ? [...selected()] : [r.id]);
    }
  }
  const tabbable = (r: Row): boolean => {
    const active = activeId();
    return active !== null && rows().some((x) => x.id === active) ? active === r.id : rows()[0]?.id === r.id;
  };

  const confirmBody = (): string =>
    [
      confirmRows().some((r) => r.kind === 'account') ? t('mail-drafts-popup-confirm-account') : '',
      confirmRows().some((r) => r.kind === 'local') ? t('mail-drafts-popup-confirm-local') : '',
    ].filter(Boolean).join(' ');
  const showLoadError = (): boolean => tab() === 'account' && props.loadFailed && accountRows().length === 0;
  const showLoading = (): boolean => tab() === 'account' && props.loading && accountRows().length === 0 && !props.loadFailed;

  return (
    <>
      <div class="drafts-popup__scrim" data-testid="compose-drafts-scrim" onClick={() => props.onClose()} />
      <section
        ref={setEl}
        class="drafts-popup"
        role="dialog"
        aria-modal="true"
        aria-labelledby={`${uid}-title`}
        tabindex="-1"
        data-testid="compose-drafts"
      >
        <header class="drafts-popup__head">
          <h3 class="drafts-popup__title" id={`${uid}-title`}>{t('mail-compose-drafts')}</h3>
          <span class="drafts-popup__count" data-testid="drafts-popup-count">{t('mail-drafts-popup-count', { count: total() })}</span>
          <button type="button" class="btn btn--ghost" aria-label={t('mail-compose-close')} onClick={() => props.onClose()}><Icon name="close" /></button>
        </header>
        <div class="drafts-popup__tools">
          <input
            ref={searchInput}
            type="search"
            class="drafts-popup__search"
            aria-label={t('mail-drafts-popup-search')}
            placeholder={t('mail-drafts-popup-search')}
            value={query()}
            onInput={(e) => setQuery(e.currentTarget.value)}
          />
          <div class="drafts-popup__tabs" role="tablist" aria-label={t('mail-compose-drafts')}>
            <button type="button" role="tab" class="drafts-popup__tab" aria-selected={tab() === 'account'} id={`${uid}-tab-account`} aria-controls={`${uid}-panel`}
              onClick={() => { setTab('account'); setSelected(new Set<string>()); }}>{t('mail-draft-account-list')} ({accountRows().length})</button>
            <button type="button" role="tab" class="drafts-popup__tab" aria-selected={tab() === 'local'} id={`${uid}-tab-local`} aria-controls={`${uid}-panel`}
              onClick={() => { setTab('local'); setSelected(new Set<string>()); }}>{t('mail-drafts-popup-tab-local')} ({localRows().length})</button>
          </div>
          <div class="drafts-popup__bar">
            <Show
              when={selecting()}
              fallback={<button type="button" class="btn btn--ghost" disabled={selectable().length === 0} onClick={() => setSelecting(true)}>{t('mail-drafts-popup-select')}</button>}
            >
              <button type="button" class="btn btn--ghost" disabled={selectable().length === 0} onClick={() => setSelected(new Set(selectable().map((r) => r.id)))}>{t('mail-drafts-popup-select-all')}</button>
              <button type="button" class="btn btn--ghost" disabled={selected().size === 0} onClick={() => setSelected(new Set<string>())}>{t('mail-drafts-popup-clear')}</button>
              <span class="drafts-popup__spacer" />
              <span data-testid="drafts-popup-selected">{t('mail-drafts-popup-selected', { count: selected().size })}</span>
              <button type="button" class="btn btn--ghost" disabled={selected().size !== 1} onClick={() => { const r = selectedRows()[0]; if (r) open(r); }}>{t('mail-drafts-popup-open')}</button>
              <button type="button" class="btn btn--ghost" style={{ color: 'var(--danger)' }} disabled={selected().size === 0} aria-haspopup="dialog" onClick={() => askDiscard([...selected()])}>{t('mail-drafts-popup-discard-selected')}</button>
              <button type="button" class="btn btn--ghost" onClick={() => { setSelecting(false); setSelected(new Set<string>()); }}>{t('mail-drafts-popup-done')}</button>
            </Show>
          </div>
        </div>
        <div class="drafts-popup__body" role="tabpanel" id={`${uid}-panel`} aria-labelledby={`${uid}-tab-${tab()}`}>
          <Show when={showLoadError()}>
            <p class="drafts-popup__state" role="alert">
              {t('mail-drafts-popup-load-failed')}
              <button type="button" class="btn btn--ghost" onClick={() => props.onRetryLoad()}>{t('mail-drafts-popup-retry')}</button>
            </p>
          </Show>
          <Show when={showLoading()}>
            <p class="drafts-popup__state" role="status">{t('mail-draft-loading')}</p>
          </Show>
          <Show when={rows().length === 0 && !showLoadError() && !showLoading()}>
            <p class="drafts-popup__state" data-testid="drafts-popup-empty">
              {query().trim() !== ''
                ? t('mail-drafts-popup-no-match', { query: query().trim() })
                : tab() === 'account'
                  ? props.accountSupported ? t('mail-drafts-popup-empty-account') : t('mail-draft-status-unsupported')
                  : t('mail-compose-drafts-empty')}
            </p>
          </Show>
          <Show when={rows().length > 0}>
            <ul class="drafts-popup__list">
              <For each={rows()}>
                {(r) => {
                  const state = (): RowState | undefined => states()[r.id];
                  const current = (): boolean => r.id === props.currentId;
                  return (
                    <li class="drafts-popup__row" data-testid={`drafts-popup-row-${r.id}`} data-selected={selected().has(r.id)} data-current={current()}>
                      <Show when={selecting()}>
                        <input
                          type="checkbox"
                          tabindex="-1"
                          aria-label={t('mail-drafts-popup-select-row', { subject: label(r) })}
                          checked={selected().has(r.id)}
                          disabled={r.protectedRow || state()?.phase === 'busy'}
                          onChange={() => toggle(r.id)}
                        />
                      </Show>
                      <button
                        type="button"
                        class="drafts-popup__main"
                        data-row-main={r.id}
                        data-testid={r.kind === 'account' ? `draft-resume-account-${r.id}` : `draft-resume-${r.id}`}
                        tabindex={tabbable(r) ? 0 : -1}
                        aria-current={current() ? 'true' : undefined}
                        disabled={props.loading || state()?.phase === 'busy'}
                        onFocus={() => setActiveId(r.id)}
                        onKeyDown={(e) => onRowKey(e, r)}
                        onClick={() => (selecting() && !r.protectedRow ? toggle(r.id) : open(r))}
                      >
                        <span class="drafts-popup__line1">
                          <span class="drafts-popup__subject" classList={{ 'drafts-popup__subject--empty': r.subject.trim() === '' }}>
                            {r.subject.trim() === '' ? t('mail-no-subject') : isolate(r.subject)}
                          </span>
                          <Show when={r.savedAt > 0}><span class="drafts-popup__when">{formatRelative(r.savedAt)}</span></Show>
                        </span>
                        <span class="drafts-popup__line2">
                          <span class="drafts-popup__summary">{r.summary.trim() !== '' ? isolate(r.summary) : t('mail-compose-drafts-no-recipient')}</span>
                          <Show when={current()}><span class="drafts-popup__badge drafts-popup__badge--current">{t('mail-drafts-popup-current')}</span></Show>
                          <span class="drafts-popup__badge">{t(r.kind === 'account' ? 'mail-drafts-popup-badge-account' : 'mail-drafts-popup-badge-local')}</span>
                        </span>
                        <Show when={state()?.phase === 'busy'}><span class="drafts-popup__line2">{t('mail-drafts-popup-discarding')}</span></Show>
                        <Show when={state()?.phase === 'error'}><span class="drafts-popup__error" role="alert">{t('mail-drafts-popup-row-failed')}</span></Show>
                      </button>
                      <Show when={state()?.phase === 'error'}>
                        <button
                          type="button"
                          class="btn btn--ghost"
                          onClick={() => void discardOne(r).then((ok) => setAnnounce(ok ? t('mail-drafts-popup-announce-done', { count: 1 }) : t('mail-drafts-popup-announce-failed', { done: 0, failed: 1 })))}
                        >{t('mail-drafts-popup-retry')}</button>
                      </Show>
                      <button
                        type="button"
                        class="drafts-popup__more"
                        aria-haspopup="menu"
                        aria-expanded={menuId() === r.id}
                        aria-label={t('mail-drafts-popup-more', { subject: label(r) })}
                        disabled={state()?.phase === 'busy'}
                        onClick={() => setMenuId(menuId() === r.id ? null : r.id)}
                      ><Icon name="more" /></button>
                      <Show when={menuId() === r.id}>
                        <div class="drafts-popup__menu" role="menu">
                          <button type="button" role="menuitem" onClick={() => open(r)}>{t('mail-drafts-popup-open')}</button>
                          <button type="button" role="menuitem" class="drafts-popup__danger" disabled={r.protectedRow} onClick={() => askDiscard([r.id])}>{t('mail-drafts-popup-discard')}</button>
                        </div>
                      </Show>
                    </li>
                  );
                }}
              </For>
            </ul>
          </Show>
        </div>
        <footer class="drafts-popup__foot">{tab() === 'account' ? t('mail-draft-quota-help') : t('mail-drafts-popup-local-hint')}</footer>
        <p class="drafts-popup__sr" role="status" aria-live="polite" data-testid="drafts-popup-live">{announce()}</p>
        <ConfirmDialog
          open={confirmIds() !== null}
          testId="drafts-popup-confirm"
          mount={props.mount}
          title={t('mail-drafts-popup-confirm-title', { count: confirmRows().length })}
          body={confirmBody()}
          confirmLabel={t('mail-draft-discard-action')}
          cancelLabel={t('mail-draft-keep-editing')}
          onCancel={() => setConfirmIds(null)}
          onConfirm={() => {
            const ids = confirmIds() ?? [];
            setConfirmIds(null);
            void discardMany(ids);
          }}
        >
          <ul class="drafts-popup__confirm-list">
            <For each={confirmRows()}>{(r) => <li>{r.subject.trim() === '' ? t('mail-no-subject') : isolate(r.subject)}</li>}</For>
          </ul>
        </ConfirmDialog>
      </section>
    </>
  );
}
