// Stream page template (t32-cfg UX3): log lines, job runs, live diagnostics.
//
// Entries arrive through `connect` (an SSE stream, a WebSocket or a poll; the
// caller decides) and are shown newest last, up to `maxEntries`. The toolbar has
// a filter, Pause / Resume (while paused new entries are held and counted, not
// shown, so the list does not move under the reader), Clear, and "Read new
// entries aloud", which turns the list into a polite live region; it is off by
// default so a busy stream does not talk over everything. The status line says
// whether the stream is connecting, live, paused, ended, lost (with Reconnect)
// or not available, in words with an icon.
//
// The list sits in a `role=log` element that takes focus, so it scrolls by keyboard.
// It stays scrolled to the end while the reader is at the end.

import { For, Match, Show, Switch, createMemo, createSignal, createUniqueId, onCleanup, onMount, type JSX } from 'solid-js';
import { t } from '../../../../i18n';
import { Icon } from '../icons.tsx';
import { AdminPage } from './PageFrame.tsx';
import * as css from '../shell.css.ts';
import * as ux from '../ux.css.ts';

export type StreamState = 'connecting' | 'live' | 'ended' | 'error' | 'unavailable';

export interface StreamHandlers<E> {
  onEntry: (entry: E) => void;
  onState: (state: StreamState) => void;
}

export interface StreamPageProps<E> {
  title: string;
  description?: string;
  actions?: JSX.Element;
  /** Opens the stream; returns the function that closes it. */
  connect: (handlers: StreamHandlers<E>) => () => void;
  renderEntry: (entry: E) => JSX.Element;
  entryKey: (entry: E) => string;
  /** Text the filter matches (default: none, the filter is hidden). */
  entryText?: (entry: E) => string;
  /** Oldest entries are dropped beyond this many. Default 500. */
  maxEntries?: number;
  /** Shown while no entry has arrived. */
  emptyText: string;
  /** Accessible name of the log. */
  logLabel: string;
  /** "Not available — <reason>" when the stream reports `unavailable`. */
  unavailableReason?: string;
  testId?: string;
}

export function StreamPage<E>(props: StreamPageProps<E>): JSX.Element {
  const id = createUniqueId();
  const [entries, setEntries] = createSignal<E[]>([]);
  const [held, setHeld] = createSignal<E[]>([]);
  const [paused, setPaused] = createSignal(false);
  const [announce, setAnnounce] = createSignal(false);
  const [filter, setFilter] = createSignal('');
  const [state, setState] = createSignal<StreamState>('connecting');
  let list: HTMLDivElement | undefined;
  let close: (() => void) | undefined;
  const max = (): number => props.maxEntries ?? 500;

  const atEnd = (): boolean => !list || list.scrollHeight - list.scrollTop - list.clientHeight < 8;

  function add(entry: E): void {
    if (paused()) {
      setHeld((h) => [...h, entry].slice(-max()));
      return;
    }
    const follow = atEnd();
    setEntries((all) => [...all, entry].slice(-max()));
    if (follow) queueMicrotask(() => list && (list.scrollTop = list.scrollHeight));
  }

  function open(): void {
    close?.();
    setState('connecting');
    close = props.connect({ onEntry: add, onState: setState });
  }
  onMount(open);
  onCleanup(() => close?.());

  function togglePause(): void {
    if (paused()) {
      setEntries((all) => [...all, ...held()].slice(-max()));
      setHeld([]);
      setPaused(false);
    } else setPaused(true);
  }

  const shown = createMemo(() => {
    const needle = filter().trim().toLowerCase();
    if (needle === '' || !props.entryText) return entries();
    return entries().filter((e) => props.entryText!(e).toLowerCase().includes(needle));
  });

  return (
    <AdminPage
      title={props.title}
      {...(props.description ? { description: props.description } : {})}
      actions={props.actions}
      testId={props.testId}
    >
      <div class={ux.streamToolbar}>
        <Show when={props.entryText}>
          <div class="field">
            <label for={`${id}-filter`}>{t('admin-ux-stream-filter')}</label>
            <input id={`${id}-filter`} type="search" value={filter()} onInput={(e) => setFilter(e.currentTarget.value)} />
          </div>
        </Show>
        <div class={css.rowActions}>
          <button type="button" class="btn" aria-pressed={paused()} onClick={togglePause}>
            {paused() ? t('admin-ux-stream-resume') : t('admin-ux-stream-pause')}
          </button>
          <button type="button" class="btn btn--ghost" onClick={() => setEntries([])}>
            {t('admin-ux-stream-clear')}
          </button>
        </div>
        <div class={ux.checkboxRow}>
          <input id={`${id}-announce`} type="checkbox" checked={announce()} onChange={(e) => setAnnounce(e.currentTarget.checked)} />
          <label for={`${id}-announce`}>{t('admin-ux-stream-announce')}</label>
        </div>
      </div>
      <p class={ux.streamStatus} data-testid="stream-status" data-state={paused() ? 'paused' : state()}>
        <Switch>
          <Match when={state() === 'unavailable'}>
            <Icon name="info" />
            <span>{t('admin-ux-state-unavailable', { reason: props.unavailableReason ?? t('admin-ux-unavailable-default') })}</span>
          </Match>
          <Match when={paused()}>
            <Icon name="info" />
            <span>{t('admin-ux-stream-paused', { count: held().length })}</span>
          </Match>
          <Match when={state() === 'connecting'}>
            <Icon name="info" />
            <span>{t('admin-ux-stream-connecting')}</span>
          </Match>
          <Match when={state() === 'live'}>
            <Icon name="check" />
            <span>{t('admin-ux-stream-live')}</span>
          </Match>
          <Match when={state() === 'ended'}>
            <Icon name="info" />
            <span>{t('admin-ux-stream-ended')}</span>
          </Match>
          <Match when={state() === 'error'}>
            <Icon name="alert" />
            <span>{t('admin-ux-stream-lost')}</span>
            <button type="button" class="btn btn--ghost" onClick={open}>
              {t('admin-ux-stream-reconnect')}
            </button>
          </Match>
        </Switch>
      </p>
      <Show when={state() !== 'unavailable'}>
        <Show when={shown().length > 0} fallback={<p class={css.help}>{entries().length > 0 ? t('admin-ux-stream-no-match') : props.emptyText}</p>}>
          <div
            ref={list}
            class={ux.streamLog}
            role="log"
            aria-label={props.logLabel}
            aria-live={announce() && !paused() ? 'polite' : 'off'}
            tabindex="0"
            data-testid="stream-log"
          >
            <ol class={ux.streamList}>
              <For each={shown()}>
                {(entry) => (
                  <li class={ux.streamEntry} data-entry={props.entryKey(entry)}>
                    {props.renderEntry(entry)}
                  </li>
                )}
              </For>
            </ol>
          </div>
        </Show>
      </Show>
    </AdminPage>
  );
}
