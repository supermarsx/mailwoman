// The collapsible composer header: a summary line (button, aria-expanded) over
// the From/To/Cc/Bcc/Subject fields. Folded, the fields stay mounted (hidden),
// so refs, drafts and the recipient state never change; only their visibility
// does. It opens on a click on the summary, on focus inside, and when the host
// asks for attention (a problem clicked in the status bar).

import { createEffect, createMemo, createSignal, createUniqueId, on, onCleanup, Show, type JSX } from 'solid-js';
import { Icon } from '../Icon.tsx';
import { t } from '../../i18n/index.ts';
import { headerOpen, readHeaderMemory, writeHeaderMemory, type HeaderCollapseMode } from './headerCollapse.ts';
import { densityComfortable, densityCompact } from './composeHeader.css.ts';
import './compose-chrome.css';

export interface ComposeHeaderProps {
  mode: () => HeaderCollapseMode;
  summary: () => string;
  /** No recipient and no subject yet: nothing to fold. */
  empty: () => boolean;
  /** Bumped by the host to open the header for a field that needs attention. */
  attention: () => number;
  /** The header has a recipient and a subject: it may fold itself after `autoCollapseSeconds`. */
  valid?: () => boolean;
  /** Seconds the body must hold focus before a valid header folds on its own; 0 or absent: never. */
  autoCollapseSeconds?: () => number;
  /** The layout.headerCollapse flag: false keeps the header open and shows no toggle. */
  collapsible?: () => boolean;
  /** Rendered in the summary row, after the toggle (the Cc/Bcc buttons and the From chip). */
  extras?: JSX.Element;
  children: JSX.Element;
}

export function ComposeHeader(props: ComposeHeaderProps): JSX.Element {
  const bodyId = `compose-head-${createUniqueId()}`;
  const [remembered, setRemembered] = createSignal<boolean | null>(readHeaderMemory());
  const [focusWithin, setFocusWithin] = createSignal(false);
  const [attentionOn, setAttentionOn] = createSignal(false);
  const [peek, setPeek] = createSignal(false);
  const [autoFolded, setAutoFolded] = createSignal(false);
  const collapsible = (): boolean => props.collapsible?.() ?? true;
  let corner: HTMLButtonElement | undefined;
  let foldTimer: ReturnType<typeof setTimeout> | undefined;
  onCleanup(() => clearTimeout(foldTimer));
  let body: HTMLDivElement | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  onCleanup(() => clearTimeout(timer));
  // A bump of `attention` opens the header; it stays open while focus is inside, then returns to the mode.
  createEffect(on(props.attention, (n) => {
    if (n === 0) return;
    setAttentionOn(true);
    setAutoFolded(false);
    clearTimeout(timer);
    timer = setTimeout(() => setAttentionOn(false), 4000);
  }, { defer: true }));
  const open = createMemo(() => !collapsible() || headerOpen({
    mode: props.mode(),
    remembered: remembered(),
    focusWithin: focusWithin(),
    attention: attentionOn(),
    empty: props.empty(),
    peek: peek(),
    autoFolded: autoFolded(),
  }));
  // The body (an editor surface outside the header) holding focus starts the auto-fold clock; focus coming
  // back into the header, a change that makes the header incomplete, or a toggle stops it.
  const [bodyFocused, setBodyFocused] = createSignal(false);
  function onDocumentFocusIn(event: FocusEvent): void {
    const target = event.target;
    if (!(target instanceof Element) || body?.contains(target)) return;
    setBodyFocused(target.matches('[contenteditable], textarea, [role="textbox"]') && !target.closest('.compose-head'));
  }
  if (typeof document !== 'undefined') {
    document.addEventListener('focusin', onDocumentFocusIn);
    onCleanup(() => document.removeEventListener('focusin', onDocumentFocusIn));
  }
  createEffect(() => {
    clearTimeout(foldTimer);
    const seconds = props.autoCollapseSeconds?.() ?? 0;
    if (!collapsible() || seconds <= 0 || !bodyFocused() || focusWithin() || attentionOn() || !(props.valid?.() ?? false) || autoFolded()) return;
    if (!open()) return;
    foldTimer = setTimeout(() => setAutoFolded(true), seconds * 1000);
  });
  function toggle(): void {
    if (!collapsible()) return;
    clearTimeout(foldTimer);
    const next = !open();
    setAutoFolded(false);
    if (props.mode() === 'auto') {
      setPeek(next);
      return;
    }
    setRemembered(next);
    writeHeaderMemory(next);
  }
  // Alt+H folds or unfolds the header from anywhere in the composer; moving focus out of a folding header
  // keeps it from being lost inside hidden fields.
  function onShortcut(event: KeyboardEvent): void {
    if (!collapsible() || !event.altKey || event.ctrlKey || event.metaKey || event.shiftKey || event.key.toLowerCase() !== 'h') return;
    if (section && !section.closest('.compose')?.contains(document.activeElement) && document.activeElement !== document.body) return;
    event.preventDefault();
    // Focus leaves the fields first, otherwise focus-within would hold the header open.
    if (open() && body?.contains(document.activeElement)) corner?.focus();
    toggle();
  }
  if (typeof document !== 'undefined') {
    document.addEventListener('keydown', onShortcut);
    onCleanup(() => document.removeEventListener('keydown', onShortcut));
  }
  let section: HTMLElement | undefined;
  return (
    <section
      ref={section}
      class={`compose-head ${props.mode() === 'collapsed' ? densityCompact : densityComfortable}`}
      data-density={props.mode() === 'collapsed' ? 'compact' : 'comfortable'}
      data-testid="compose-header"
      data-collapsed={open() ? undefined : ''}
      aria-label={t('mail-compose-header')}
    >
      <div class="compose-head__bar">
        <Show when={collapsible()}>
        <button
          type="button"
          class="compose-head__toggle"
          data-testid="compose-header-toggle"
          aria-expanded={open() ? 'true' : 'false'}
          aria-controls={bodyId}
          aria-label={open() ? t('mail-header-collapse') : t('mail-header-expand', { summary: props.summary() })}
          onClick={toggle}
        >
          <span class="compose-head__chev"><Icon name="chevronDown" /></span>
          <span class="compose-head__summary" data-testid="compose-header-summary">{open() ? t('mail-header-label') : props.summary()}</span>
        </button>
        </Show>
        {props.extras}
        <Show when={collapsible()}>
          <button
            ref={corner}
            type="button"
            class="compose-head__corner"
            data-testid="compose-header-corner"
            aria-expanded={open() ? 'true' : 'false'}
            aria-controls={bodyId}
            aria-keyshortcuts="Alt+H"
            aria-label={open() ? t('mail-header-collapse') : t('mail-header-expand-short')}
            data-tooltip={open() ? t('mail-header-collapse') : t('mail-header-expand-short')}
            onClick={toggle}
          >
            <span class="compose-head__chev"><Icon name="chevronDown" /></span>
          </button>
        </Show>
      </div>
      <div class="compose-head__wrap" data-open={open() ? 'true' : 'false'}>
      <div class="compose-head__clip">
      <div
        ref={body}
        id={bodyId}
        class="compose-head__body"
        data-testid="compose-header-body"
        hidden={!open()}
        onFocusIn={() => setFocusWithin(true)}
        onFocusOut={(event) => {
          const next = event.relatedTarget;
          // Focus moving to a suggestion list or the contact card portalled elsewhere still counts as inside.
          if (next instanceof Element && (body?.contains(next) || next.closest('[role="listbox"], .address-card-backdrop'))) return;
          setFocusWithin(false);
          // The peek ends when focus leaves, so `auto` folds again once the user is writing.
          setPeek(false);
        }}
      >
        {props.children}
      </div>
      </div>
      </div>
    </section>
  );
}
