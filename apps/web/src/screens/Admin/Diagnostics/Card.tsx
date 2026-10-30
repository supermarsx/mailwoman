// t105: one check or tool as a card: title and purpose, a status chip, labelled
// result rows (monospace and copyable where technical), "How to fix" on Warn or
// Fail, an expandable Details area with the raw result, and a Run button that
// runs in place.

import { For, Show, createUniqueId, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { Icon, type IconName } from '../../../components/Icon.tsx';
import { toast } from '../../../components/toast/index.ts';
import type { CardStatus } from './model.ts';
import * as css from './diagnostics.css.ts';

const CHIP_ICON: Record<CardStatus, IconName> = { ok: 'check', warn: 'alert', fail: 'alert', skipped: 'minus', notRun: 'clock', running: 'sync' };

export function StatusChip(props: { status: CardStatus }): JSX.Element {
  return <span class={css.chip} data-status={props.status} data-testid="diag-status"><Icon name={CHIP_ICON[props.status]} /><span>{t(`admin-diagx-status-${props.status === 'notRun' ? 'not-run' : props.status}`)}</span></span>;
}

/** Copy `text` and say so in a corner toast. */
export async function copyText(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
    toast.success(t('admin-diagx-copied'), { id: 'admin-diagnostics-copied' });
  } catch {
    toast.error(t('admin-diagx-copy-failed'), { id: 'admin-diagnostics-copied' });
  }
}

export function CopyButton(props: { text: string; label: string }): JSX.Element {
  return <button type="button" class={css.copy} aria-label={t('admin-diagx-copy-value', { label: props.label })} title={t('admin-diagx-copy-value', { label: props.label })} onClick={() => void copyText(props.text)}><Icon name="clone" /></button>;
}

export interface CardRow {
  label: string;
  value: JSX.Element | string;
  /** Technical value: monospace, with a copy button for this text. */
  mono?: string;
}

export interface CheckCardProps {
  title: string;
  purpose: string;
  status: CardStatus;
  rows?: readonly CardRow[];
  /** Shown on Warn and Fail. */
  fix?: string | undefined;
  /** Raw result for the Details area (pretty-printed). */
  raw?: unknown;
  onRun?: (() => void) | undefined;
  runLabel?: string;
  runDisabled?: boolean;
  runTitle?: string | undefined;
  /** Stable id for tests and deep links (`data-check`). */
  check?: string;
  children?: JSX.Element;
}

export function CheckCard(props: CheckCardProps): JSX.Element {
  const heading = createUniqueId();
  const showFix = (): boolean => !!props.fix && (props.status === 'warn' || props.status === 'fail');
  const failing = (): string | undefined => props.status === 'fail' ? 'fail' : props.status === 'warn' ? 'warn' : undefined;
  return (
    <article class={css.card} aria-labelledby={heading} data-check={props.check} data-status={failing()} aria-busy={props.status === 'running' ? 'true' : undefined}>
      <header class={css.cardHead}>
        <div class={css.cardTitles}>
          <h4 id={heading} class={css.cardTitle}>{props.title}</h4>
          <p class={css.purpose}>{props.purpose}</p>
        </div>
        <StatusChip status={props.status} />
      </header>
      <Show when={props.rows?.length}>
        <dl class={css.rows}>
          <For each={props.rows}>{(row) => <>
            <dt>{row.label}</dt>
            <dd><span class={row.mono !== undefined ? `${css.value} ${css.mono}` : css.value}>{row.value}</span><Show when={row.mono}>{(text) => <CopyButton text={text()} label={row.label} />}</Show></dd>
          </>}</For>
        </dl>
      </Show>
      {props.children}
      <Show when={showFix()}>
        <p class={css.fix} data-status={failing()}><strong class={css.fixTitle}>{t('admin-diagx-how-to-fix')}</strong>{props.fix}</p>
      </Show>
      <div class={css.cardActions}>
        <Show when={props.onRun}>
          <button type="button" class="btn" disabled={props.runDisabled || props.status === 'running'} title={props.runTitle} onClick={() => props.onRun?.()}>
            <Icon name={props.status === 'running' ? 'sync' : 'refresh'} />
            <span>{props.status === 'running' ? t('admin-diagx-running') : props.runLabel ?? t('admin-diagx-run')}</span>
          </button>
        </Show>
      </div>
      <Show when={props.raw !== undefined}>
        <details class={css.details}>
          <summary>{t('admin-diagx-details')}</summary>
          <pre class={css.raw}>{JSON.stringify(props.raw, null, 2)}</pre>
          <button type="button" class="btn btn--ghost" onClick={() => void copyText(JSON.stringify(props.raw, null, 2))}><Icon name="clone" /><span>{t('admin-diagx-copy-details')}</span></button>
        </details>
      </Show>
    </article>
  );
}
