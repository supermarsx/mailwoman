// Request protection > Overview: the HTTP protocol guard's counters, grouped by
// the same classes as the Protocol guard subtab. A tile row (detected, refused,
// classes enforcing, what enforcing would have refused), then one compact table
// per group: class, action in effect, counts (right-aligned, zero dimmed, non-zero
// with an icon and bold), last seen as a relative time with the exact time in a
// title. Counters are in-memory since the server started (the status API has no
// time window). Under 560px each row stacks into a labelled card.

import { For, Show, createSignal, type JSX } from 'solid-js';
import type { ProtocolGuardRow } from '../../api/admin/requestProtection.ts';
import { formatDateTime, formatNumber, formatRelative, t } from '../../i18n';
import { Icon } from './shell/icons.tsx';
import { StatusBadge, type StatusTone } from './shell/StatusBadge.tsx';
import { adminDocsHref } from './shell/docsLink.ts';
import { GUARD_GROUPS, byCount, totalsOf, wouldRefuse } from './protocolGuard.ts';
import * as css from './protocolGuard.css.ts';
import * as admin from './admin.css.ts';

const TONE: Record<string, StatusTone> = { off: 'off', detect: 'neutral', enforce: 'ok' };

/** The action in effect as a chip: icon and text, never colour alone. */
export function ModeChip(props: { mode: string }): JSX.Element {
  return (
    <StatusBadge tone={TONE[props.mode] ?? 'neutral'} status={props.mode} testId="protocol-mode-chip">
      {t(`admin-protocol-mode-${props.mode}`)}
    </StatusBadge>
  );
}

const seconds = (ts: number): string => new Date(ts * 1000).toISOString();

export function ProtocolGuardStats(props: { rows: readonly ProtocolGuardRow[]; onOpenEditor?: (() => void) | undefined }): JSX.Element {
  const [showZero, setShowZero] = createSignal(true);
  const all = (): readonly ProtocolGuardRow[] => props.rows;
  const total = () => totalsOf(all());
  const enforcing = (): number => all().filter((r) => r.mode === 'enforce').length;
  const max = (): number => Math.max(1, ...all().map((r) => r.detected + r.enforced));
  const tile = (value: string, label: string, hint?: string): JSX.Element => (
    <div class={css.tile} title={hint}>
      <span class={css.tileValue}>{value}</span>
      <span class={css.tileLabel}>{label}</span>
    </div>
  );
  const count = (n: number, label: string): JSX.Element => (
    <td class="num" data-label={label}>
      <Show when={n > 0} fallback={<span class={css.zero}>{formatNumber(0)}</span>}>
        <span class={css.hit}><Icon name="alert" />{formatNumber(n)}</span>
      </Show>
    </td>
  );
  return (
    <div class={admin.card} data-testid="protocol-guard">
      <h3 class={admin.heading}>{t('admin-protocol-title')}</h3>
      <p class={admin.note}>{t('admin-protocol-note')}</p>
      <div class={css.tiles}>
        {tile(formatNumber(total().detected), t('admin-protocol-tile-detected'), t('admin-protocol-tile-detected-help'))}
        {tile(formatNumber(total().enforced), t('admin-protocol-tile-refused'), t('admin-protocol-tile-refused-help'))}
        {tile(t('admin-protocol-tile-enforcing-value', { on: enforcing(), total: all().length }), t('admin-protocol-tile-enforcing'))}
        {tile(formatNumber(wouldRefuse(all())), t('admin-protocol-tile-would'), t('admin-protocol-tile-would-help'))}
      </div>
      <div class={css.toolbar}>
        <span class={admin.note}>{t('admin-protocol-since-start')}</span>
        <span>
          <button class="btn btn--ghost" aria-pressed={!showZero()} onClick={() => setShowZero(!showZero())}>
            {showZero() ? t('admin-protocol-hide-zero') : t('admin-protocol-show-all')}
          </button>{' '}
          <Show when={props.onOpenEditor}>
            <button class="btn btn--ghost" onClick={() => props.onOpenEditor?.()}>{t('admin-protocol-edit-policies')}</button>
          </Show>{' '}
          <a href={adminDocsHref('protocol-guard')} target="_blank" rel="noopener noreferrer">{t('admin-ux-learn-more')}</a>
        </span>
      </div>
      <Show when={all().length > 0} fallback={<p class={admin.note}>{t('admin-protocol-empty')}</p>}>
        <For each={GUARD_GROUPS}>
          {(group) => {
            const rows = (): ProtocolGuardRow[] => byCount(all().filter((r) => group.classes.includes(r.class)));
            const shown = (): ProtocolGuardRow[] => rows().filter((r) => showZero() || r.detected + r.enforced > 0);
            return (
              <Show when={rows().length > 0}>
                <section aria-label={t(`admin-protocol-group-${group.id}`)} data-testid={`protocol-group-${group.id}`}>
                  <div class={css.groupHead}>
                    <h4>{t(`admin-protocol-group-${group.id}`)}</h4>
                    <span class={admin.note}>{t('admin-protocol-group-total', { detected: formatNumber(totalsOf(rows()).detected), refused: formatNumber(totalsOf(rows()).enforced) })}</span>
                  </div>
                  <Show when={shown().length > 0} fallback={<p class={admin.note}>{t('admin-protocol-group-quiet')}</p>}>
                    <table class={css.table}>
                      <thead>
                        <tr>
                          <th scope="col">{t('admin-protocol-class')}</th>
                          <th scope="col">{t('admin-protocol-action')}</th>
                          <th scope="col" class="num" title={t('admin-protocol-tile-detected-help')}>{t('admin-protocol-detected')}</th>
                          <th scope="col" class="num" title={t('admin-protocol-tile-refused-help')}>{t('admin-protocol-enforced')}</th>
                          <th scope="col">{t('admin-protocol-last-seen')}</th>
                        </tr>
                      </thead>
                      <tbody>
                        <For each={shown()}>
                          {(row) => (
                            <tr data-class={row.class}>
                              <td class={css.label} title={t(`admin-protocol-class-help-${row.class}`)}>
                                <span>{t(`admin-protocol-class-${row.class}`)}</span>
                                <progress class={css.bar} max={max()} value={row.detected + row.enforced} aria-hidden="true" tabindex="-1" />
                              </td>
                              <td data-label={t('admin-protocol-action')}><ModeChip mode={row.mode} /></td>
                              {count(row.detected, t('admin-protocol-detected'))}
                              {count(row.enforced, t('admin-protocol-enforced'))}
                              <td data-label={t('admin-protocol-last-seen')}>
                                <Show when={row.lastSeen !== null} fallback={<span class={css.zero}>{t('admin-protocol-never')}</span>}>
                                  <time datetime={seconds(row.lastSeen!)} title={formatDateTime(seconds(row.lastSeen!))}>{formatRelative(seconds(row.lastSeen!))}</time>
                                </Show>
                              </td>
                            </tr>
                          )}
                        </For>
                      </tbody>
                    </table>
                  </Show>
                </section>
              </Show>
            );
          }}
        </For>
      </Show>
    </div>
  );
}
