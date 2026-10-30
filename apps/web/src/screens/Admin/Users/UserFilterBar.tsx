// The Users table's filter chips (t107), after the mail search chips (t88,
// components/search/SearchFilterBar.tsx): each active filter is a chip that opens
// a small editor and has its own remove button; "Add filter" lists the filters,
// "Clear all" drops every filter and the search text. The chips are a view of
// the filters the URL holds: nothing here keeps its own copy.

import { createSignal, For, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { t, formatDate } from '../../../i18n/index.ts';
import { Icon, type IconName } from '../../../components/Icon.tsx';
import { createContextMenu, type ContextAction } from '../../../components/ContextMenu.tsx';
import { FACTORS, STATES, activeFilterCount, type UserFilters, type UserTablePage } from '../../../api/admin/usersTable.ts';
import * as css from './usersTable.css.ts';
import { srOnly } from '../shell/shell.css.ts';

export type FilterKind = 'domain' | 'status' | 'twoFactor' | 'role' | 'profile' | 'signin' | 'created';
const KINDS: readonly FilterKind[] = ['domain', 'status', 'twoFactor', 'role', 'profile', 'signin', 'created'];
const ICON: Record<FilterKind, IconName> = {
  domain: 'domains', status: 'check', twoFactor: 'twofactor', role: 'roles', profile: 'policy', signin: 'clock', created: 'calendar',
};
/** The string-id spelling of a filter kind. */
const slug = (kind: FilterKind): string => (kind === 'twoFactor' ? 'tfa' : kind);
const SIGNIN_PRESETS = ['never', 'older:30', 'older:90', 'within:7', 'within:30'] as const;

export interface Preset { name: string; query: Record<string, string> }

export interface UserFilterBarProps {
  filters: UserFilters;
  facets: UserTablePage['facets'] | null;
  onChange: (next: UserFilters) => void;
  presets: readonly Preset[];
  onApplyPreset: (preset: Preset) => void;
  onSavePreset: (name: string) => void;
  onDeletePreset: (name: string) => void;
}

/** The label of one filter value. */
export function valueLabel(kind: FilterKind, value: string, facets: UserTablePage['facets'] | null): string {
  switch (kind) {
    case 'status': return t(`admin-userlist-state-${value}`);
    case 'twoFactor': return t(`admin-userlist-2fa-${value}`);
    case 'role': return value === 'admin' ? t('admin-userlist-role-any') : value === 'none' ? t('admin-userlist-role-none') : value;
    case 'profile': return value === 'none' ? t('admin-userlist-profile-none') : facets?.profiles.find((p) => p.id === value)?.name ?? value;
    case 'signin': {
      if (value === 'never') return t('admin-userlist-signin-never');
      const [mode, days = '0'] = value.split(':');
      return t(mode === 'older' ? 'admin-userlist-signin-older' : 'admin-userlist-signin-within', { days: Number(days) });
    }
    default: return value;
  }
}

function isSet(f: UserFilters, kind: FilterKind): boolean {
  if (kind === 'signin') return f.signin !== null;
  if (kind === 'created') return f.createdFrom !== null || f.createdTo !== null;
  return f[kind].length > 0;
}

function cleared(f: UserFilters, kind: FilterKind): UserFilters {
  if (kind === 'signin') return { ...f, signin: null };
  if (kind === 'created') return { ...f, createdFrom: null, createdTo: null };
  return { ...f, [kind]: [] };
}

function dateLabel(value: string): string {
  const [y, m, d] = value.split('-').map(Number) as [number, number, number];
  return formatDate(new Date(y, m - 1, d));
}

export function UserFilterBar(props: UserFilterBarProps): JSX.Element {
  const [editing, setEditing] = createSignal<{ kind: FilterKind; anchor: HTMLElement } | null>(null);
  const [saving, setSaving] = createSignal(false);
  let addButton!: HTMLButtonElement;
  let viewsButton!: HTMLButtonElement;

  const chipValue = (kind: FilterKind): string => {
    const f = props.filters;
    if (kind === 'signin') return valueLabel('signin', f.signin ?? '', props.facets);
    if (kind === 'created') {
      if (f.createdFrom && f.createdTo) return t('admin-userlist-created-between', { from: dateLabel(f.createdFrom), to: dateLabel(f.createdTo) });
      return f.createdFrom ? t('admin-userlist-created-since', { date: dateLabel(f.createdFrom) }) : t('admin-userlist-created-until', { date: dateLabel(f.createdTo ?? '') });
    }
    return f[kind].map((v) => valueLabel(kind, v, props.facets)).join(', ');
  };
  const chipLabel = (kind: FilterKind): string => `${t(`admin-userlist-filter-${slug(kind)}`)}: ${chipValue(kind)}`;

  const addMenu = createContextMenu({
    label: () => t('admin-userlist-add-filter'),
    scope: () => 'users-add-filter',
    restoreFocus: () => addButton?.focus(),
    actions: (): ContextAction[] => KINDS.map((kind) => ({
      id: kind, label: t(`admin-userlist-filter-${slug(kind)}`), icon: ICON[kind], checked: isSet(props.filters, kind),
      run: () => { setSaving(false); setEditing({ kind, anchor: addButton }); },
    })),
  });

  const viewsMenu = createContextMenu({
    label: () => t('admin-userlist-views'),
    scope: () => 'users-views',
    restoreFocus: () => viewsButton?.focus(),
    actions: (): ContextAction[] => [
      ...props.presets.map((p, i): ContextAction => ({ id: `apply-${i}`, label: p.name, icon: 'filter', run: () => props.onApplyPreset(p) })),
      { id: 'save', label: t('admin-userlist-view-save'), icon: 'plus', separatorBefore: props.presets.length > 0, run: () => { setEditing(null); setSaving(true); } },
      ...props.presets.map((p, i): ContextAction => ({ id: `delete-${i}`, label: t('admin-userlist-view-delete', { name: p.name }), icon: 'delete', separatorBefore: i === 0, run: () => props.onDeletePreset(p.name) })),
    ],
  });

  const close = (anchor?: HTMLElement): void => {
    setEditing(null);
    setSaving(false);
    queueMicrotask(() => (anchor?.isConnected ? anchor : addButton)?.focus());
  };

  const anyActive = (): boolean => activeFilterCount(props.filters) > 0 || props.filters.q.trim() !== '';

  return (
    <div class={css.filterBar} role="toolbar" aria-label={t('admin-userlist-filters')} data-testid="users-filter-bar">
      <For each={KINDS.filter((kind) => isSet(props.filters, kind))}>
        {(kind) => (
          <span class={css.chip} data-kind={kind}>
            <button
              type="button" class={css.chipLabel} aria-haspopup="dialog" aria-expanded={editing()?.kind === kind}
              title={t('admin-userlist-chip-edit', { label: chipLabel(kind) })}
              onClick={(e) => { setSaving(false); setEditing({ kind, anchor: e.currentTarget }); }}
            >
              <Icon name={ICON[kind]} />
              <span class={css.chipText}><span class={css.chipField}>{t(`admin-userlist-filter-${slug(kind)}`)}:</span> {chipValue(kind)}</span>
            </button>
            <button
              type="button" class={css.chipRemove}
              aria-label={t('admin-userlist-chip-remove', { label: chipLabel(kind) })}
              title={t('admin-userlist-chip-remove', { label: chipLabel(kind) })}
              onClick={() => { props.onChange(cleared(props.filters, kind)); queueMicrotask(() => addButton?.focus()); }}
            ><Icon name="close" /></button>
          </span>
        )}
      </For>
      <button
        ref={addButton} type="button" class={css.addFilter} aria-haspopup="menu" aria-expanded={addMenu.isOpen()}
        data-testid="users-add-filter" onClick={(e) => addMenu.open(e)}
      ><Icon name="plus" />{t('admin-userlist-add-filter')}</button>
      <Show when={anyActive()}>
        <button type="button" class={css.textButton} onClick={() => props.onChange({ ...cleared(cleared(cleared(cleared(cleared(cleared(cleared(props.filters, 'domain'), 'status'), 'twoFactor'), 'role'), 'profile'), 'signin'), 'created'), q: '' })}>
          {t('admin-userlist-clear-all')}
        </button>
      </Show>
      <button
        ref={viewsButton} type="button" class={css.textButton} aria-haspopup="menu" aria-expanded={viewsMenu.isOpen()}
        data-testid="users-views" onClick={(e) => viewsMenu.open(e)}
      >{t('admin-userlist-views')}</button>
      {addMenu.menu()}
      {viewsMenu.menu()}
      <Show when={editing()}>
        {(state) => (
          <FilterEditor
            kind={state().kind} filters={props.filters} facets={props.facets}
            onApply={(next) => { props.onChange(next); close(state().anchor); }}
            onClose={() => close(state().anchor)}
          />
        )}
      </Show>
      <Show when={saving()}>
        <SavePreset onSave={(name) => { props.onSavePreset(name); close(viewsButton); }} onClose={() => close(viewsButton)} />
      </Show>
    </div>
  );
}

function Popover(props: { label: string; testId: string; onClose: () => void; children: JSX.Element }): JSX.Element {
  let panel!: HTMLDivElement;
  onMount(() => {
    panel.querySelector<HTMLElement>('input,select,button')?.focus({ preventScroll: true });
    // A press outside the editor (and outside the chip row that opened it) closes it.
    const outside = (e: PointerEvent): void => {
      if (e.target instanceof Element && !panel.contains(e.target) && !e.target.closest('[data-testid="users-filter-bar"]')) props.onClose();
    };
    document.addEventListener('pointerdown', outside, true);
    onCleanup(() => document.removeEventListener('pointerdown', outside, true));
  });
  return (
    <div
      ref={panel} role="dialog" aria-label={props.label} class={css.popover} data-testid={props.testId}
      onKeyDown={(e) => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); props.onClose(); } }}
    >{props.children}</div>
  );
}

function SavePreset(props: { onSave: (name: string) => void; onClose: () => void }): JSX.Element {
  const [name, setName] = createSignal('');
  return (
    <Popover label={t('admin-userlist-view-save')} testId="users-view-save" onClose={props.onClose}>
      <form class={css.popoverForm} onSubmit={(e) => { e.preventDefault(); const value = name().trim(); if (value) props.onSave(value.slice(0, 40)); }}>
        <label class="field"><span>{t('admin-userlist-view-name')}</span>
          <input type="text" maxLength={40} value={name()} onInput={(e) => setName(e.currentTarget.value)} />
        </label>
        <div class={css.popoverActions}>
          <button type="button" class="btn btn--ghost" onClick={() => props.onClose()}>{t('admin-userlist-cancel')}</button>
          <button type="submit" class="btn btn--primary" disabled={name().trim() === ''}>{t('admin-userlist-view-save-confirm')}</button>
        </div>
      </form>
    </Popover>
  );
}

function FilterEditor(props: { kind: FilterKind; filters: UserFilters; facets: UserTablePage['facets'] | null; onApply: (next: UserFilters) => void; onClose: () => void }): JSX.Element {
  const kind = props.kind;
  const title = t(`admin-userlist-filter-${slug(kind)}`);
  const choices = (): string[] => {
    const f = props.facets;
    switch (kind) {
      case 'domain': return f?.domains ?? props.filters.domain;
      case 'status': return [...STATES];
      case 'twoFactor': return [...FACTORS];
      case 'role': return ['admin', 'none', ...(f?.roles ?? [])];
      case 'profile': return [...(f?.profiles.map((p) => p.id) ?? []), 'none'];
      default: return [];
    }
  };
  const initial = kind === 'signin' || kind === 'created' ? [] : [...props.filters[kind]];
  const [picked, setPicked] = createSignal<string[]>(initial);
  const presetSignin = props.filters.signin === null || (SIGNIN_PRESETS as readonly string[]).includes(props.filters.signin);
  const [signin, setSignin] = createSignal<string>(props.filters.signin === null ? 'older:30' : presetSignin ? props.filters.signin : 'custom');
  const [days, setDays] = createSignal(presetSignin ? '60' : (props.filters.signin ?? '').split(':')[1] ?? '60');
  const [from, setFrom] = createSignal(props.filters.createdFrom ?? '');
  const [to, setTo] = createSignal(props.filters.createdTo ?? '');
  const [error, setError] = createSignal('');

  const apply = (e: Event): void => {
    e.preventDefault();
    if (kind === 'signin') {
      let value = signin();
      if (value === 'custom') {
        const n = Number(days());
        if (!Number.isInteger(n) || n < 1 || n > 36500) { setError(t('admin-userlist-days-invalid')); return; }
        value = `older:${n}`;
      }
      props.onApply({ ...props.filters, signin: value });
      return;
    }
    if (kind === 'created') {
      if (from() && to() && from() > to()) { setError(t('admin-userlist-dates-invalid')); return; }
      props.onApply({ ...props.filters, createdFrom: from() || null, createdTo: to() || null });
      return;
    }
    props.onApply({ ...props.filters, [kind]: picked() } as UserFilters);
  };
  const remove = (): void => props.onApply(cleared(props.filters, kind));

  return (
    <Popover label={t('admin-userlist-chip-edit', { label: title })} testId="users-filter-editor" onClose={props.onClose}>
      <form class={css.popoverForm} onSubmit={apply}>
        <p class={css.popoverTitle}>{title}</p>
        <Show when={kind !== 'signin' && kind !== 'created'}>
          <fieldset class={css.options}>
            <legend class={srOnly}>{title}</legend>
            <Show when={choices().length > 0} fallback={<p class={css.dim}>{t('admin-userlist-filter-no-choices')}</p>}>
              <For each={choices()}>
                {(value) => (
                  <label class={css.option}>
                    <input
                      type="checkbox" checked={picked().includes(value)}
                      onChange={(e) => setPicked((now) => e.currentTarget.checked ? [...now, value] : now.filter((v) => v !== value))}
                    />
                    <span dir="auto">{valueLabel(kind, value, props.facets)}</span>
                  </label>
                )}
              </For>
            </Show>
          </fieldset>
        </Show>
        <Show when={kind === 'signin'}>
          <fieldset class={css.options}>
            <legend class={srOnly}>{title}</legend>
            <For each={[...SIGNIN_PRESETS, 'custom']}>
              {(value) => (
                <label class={css.option}>
                  <input type="radio" name="users-signin" value={value} checked={signin() === value} onChange={() => { setSignin(value); setError(''); }} />
                  <Show when={value === 'custom'} fallback={<span>{valueLabel('signin', value, null)}</span>}>
                    <span>{t('admin-userlist-signin-custom')}</span>
                    <input
                      class={css.inlineNumber} type="number" min="1" max="36500" inputMode="numeric" value={days()}
                      aria-label={t('admin-userlist-signin-days')}
                      onInput={(e) => { setDays(e.currentTarget.value); setSignin('custom'); setError(''); }}
                    />
                  </Show>
                </label>
              )}
            </For>
          </fieldset>
        </Show>
        <Show when={kind === 'created'}>
          <label class="field"><span>{t('admin-userlist-created-from')}</span>
            <input type="date" value={from()} onInput={(e) => { setFrom(e.currentTarget.value); setError(''); }} />
          </label>
          <label class="field"><span>{t('admin-userlist-created-to')}</span>
            <input type="date" value={to()} onInput={(e) => { setTo(e.currentTarget.value); setError(''); }} />
          </label>
        </Show>
        <Show when={error()}><p class={css.dim} role="alert">{error()}</p></Show>
        <div class={css.popoverActions}>
          <Show when={isSet(props.filters, kind)}>
            <button type="button" class={`btn btn--ghost ${css.popoverRemove}`} onClick={remove}>{t('admin-userlist-filter-remove')}</button>
          </Show>
          <button type="button" class="btn btn--ghost" onClick={() => props.onClose()}>{t('admin-userlist-cancel')}</button>
          <button type="submit" class="btn btn--primary">{t('admin-userlist-apply')}</button>
        </div>
      </form>
    </Popover>
  );
}
