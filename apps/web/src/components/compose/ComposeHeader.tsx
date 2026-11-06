// The collapsible composer header: a summary line (button, aria-expanded) over
// the From/To/Cc/Bcc/Subject fields. Folded, the fields stay mounted (hidden),
// so refs, drafts and the recipient state never change; only their visibility
// does. It opens on a click on the summary, on focus inside, and when the host
// asks for attention (a problem clicked in the status bar).

import { createEffect, createMemo, createSignal, createUniqueId, on, onCleanup, type JSX } from 'solid-js';
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
  let body: HTMLDivElement | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  onCleanup(() => clearTimeout(timer));
  // A bump of `attention` opens the header; it stays open while focus is inside, then returns to the mode.
  createEffect(on(props.attention, (n) => {
    if (n === 0) return;
    setAttentionOn(true);
    clearTimeout(timer);
    timer = setTimeout(() => setAttentionOn(false), 4000);
  }, { defer: true }));
  const open = createMemo(() => headerOpen({
    mode: props.mode(),
    remembered: remembered(),
    focusWithin: focusWithin(),
    attention: attentionOn(),
    empty: props.empty(),
    peek: peek(),
  }));
  function toggle(): void {
    const next = !open();
    if (props.mode() === 'auto') {
      setPeek(next);
      return;
    }
    setRemembered(next);
    writeHeaderMemory(next);
  }
  return (
    <section
      class={`compose-head ${props.mode() === 'collapsed' ? densityCompact : densityComfortable}`}
      data-density={props.mode() === 'collapsed' ? 'compact' : 'comfortable'}
      data-testid="compose-header"
      data-collapsed={open() ? undefined : ''}
      aria-label={t('mail-compose-header')}
    >
      <div class="compose-head__bar">
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
        {props.extras}
      </div>
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
    </section>
  );
}
