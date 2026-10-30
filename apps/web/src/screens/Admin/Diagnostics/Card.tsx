// t105: one check or tool as a card: title and purpose, a status chip, labelled
// result rows (monospace and copyable where technical), "How to fix" on Warn or
// Fail, an expandable Details area with the raw result, and a Run button that
// runs in place.

import { For, Show, createContext, useContext, createUniqueId, type JSX } from 'solid-js';
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
  kind?: 'duration';
  label: string;
  value: JSX.Element | string;
  /** Technical value: monospace, with a copy button for this text. */
  mono?: string;
}

const TableContext=createContext(false);
/** All diagnostics use the same cells and actions; only the presentation changes. */
export function CheckTable(props:{caption:string;children:JSX.Element}):JSX.Element {
  return <div class={css.checkTableScroll} role="region" aria-label={props.caption} tabIndex={0}>
    <table class={css.checkTable}><caption>{props.caption}</caption><thead><tr>
      <th scope="col">{t('admin-diag-check')}</th><th scope="col">{t('admin-diag-status')}</th><th scope="col">{t('admin-diagx-row-result')}</th><th scope="col">{t('admin-diag-duration')}</th><th scope="col">{t('admin-diagx-table-actions')}</th>
    </tr></thead><tbody><TableContext.Provider value={true}>{props.children}</TableContext.Provider></tbody></table>
  </div>;
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
  const table=useContext(TableContext);
  const heading = createUniqueId();
  const showFix = (): boolean => !!props.fix && (props.status === 'warn' || props.status === 'fail');
  const failing = (): string | undefined => props.status === 'fail' ? 'fail' : props.status === 'warn' ? 'warn' : undefined;
  const Rows=():JSX.Element=><Show when={props.rows?.length}><dl class={css.rows}><For each={props.rows?.filter(row=>!table||row.kind!=='duration')}>{row=><><dt>{row.label}</dt><dd><span class={row.mono!==undefined?`${css.value} ${css.mono}`:css.value}>{row.value}</span><Show when={row.mono}>{text=><CopyButton text={text()} label={row.label}/>}</Show></dd></>}</For></dl></Show>;
  const Fix=():JSX.Element=><Show when={showFix()}><p class={css.fix} data-status={failing()}><strong class={css.fixTitle}>{t('admin-diagx-how-to-fix')}</strong>{props.fix}</p></Show>;
  const Run=():JSX.Element=><Show when={props.onRun}><button type="button" class="btn" disabled={props.runDisabled||props.status==='running'} title={props.runTitle} onClick={()=>props.onRun?.()}><Icon name={props.status==='running'?'sync':'refresh'}/><span>{props.status==='running'?t('admin-diagx-running'):props.runLabel??t('admin-diagx-run')}</span></button></Show>;
  const Details=():JSX.Element=><Show when={props.raw!==undefined}><details class={css.details}><summary>{t('admin-diagx-details')}</summary><pre class={css.raw}>{JSON.stringify(props.raw,null,2)}</pre><button type="button" class="btn btn--ghost" onClick={()=>void copyText(JSON.stringify(props.raw,null,2))}><Icon name="clone"/><span>{t('admin-diagx-copy-details')}</span></button></details></Show>;
  if(table)return <tr aria-labelledby={heading} data-check={props.check} data-status={failing()} aria-busy={props.status==='running'?'true':undefined}>
    <th scope="row"><span id={heading} class={css.cardTitle}>{props.title}</span><p class={css.purpose}>{props.purpose}</p></th>
    <td><StatusChip status={props.status}/></td><td><Rows/>{props.children}<Fix/></td><td data-diag-duration="">{props.rows?.find(row=>row.kind==='duration')?.value??'—'}</td><td><div class={css.cardActions}><Run/></div><Details/></td>
  </tr>;
  return (
    <article class={css.card} aria-labelledby={heading} data-check={props.check} data-status={failing()} aria-busy={props.status === 'running' ? 'true' : undefined}>
      <header class={css.cardHead}>
        <div class={css.cardTitles}>
          <h4 id={heading} class={css.cardTitle}>{props.title}</h4>
          <p class={css.purpose}>{props.purpose}</p>
        </div>
        <StatusChip status={props.status} />
      </header>
      <Rows/>{props.children}<Fix/>
      <div class={css.cardActions}><Run/></div><Details/>
    </article>
  );
}
