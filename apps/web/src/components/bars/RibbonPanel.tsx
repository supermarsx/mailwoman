// `<RibbonPanel>`: one ribbon tab's row of groups, kept to its width (t38 §2.2,
// user decision D1).
//
// While the groups are wider than the panel, the lowest-priority group still
// open folds into a single dropdown button (on a tie, the one furthest to the
// inline end goes first). When the panel grows again, groups open in the
// reverse order, each only once the panel is at least 1 px wider than it was
// when that group folded, so a group never folds and opens in a loop. Only
// when every group has folded and the row still does not fit does the panel
// scroll sideways, and then the mouse wheel scrolls it (`bindHorizontalWheel`).
//
// Layout runs one step per animation frame from a ResizeObserver on the panel
// and its track. A run of more than `4 × groups + 8` steps that change
// something without settling stops until the panel's width changes.

import {
  createContext,
  createSignal,
  onCleanup,
  onMount,
  useContext,
  type Accessor,
  type JSX,
} from 'solid-js';
import { bindHorizontalWheel } from '../horizontalWheel.ts';
import * as css from './ribbonGroup.css.ts';

/** Re-open a folded group only once the panel is this much wider than when it folded. */
export const EXPAND_HYSTERESIS_PX = 1;

export interface RibbonGroupRegistration {
  /** Higher stays open longer. */
  priority: () => number;
  element: () => HTMLElement | undefined;
}

interface RibbonPanelApi {
  register(group: RibbonGroupRegistration): { folded: Accessor<boolean>; dispose: () => void };
}

const PanelContext = createContext<RibbonPanelApi>();

export function useRibbonPanel(): RibbonPanelApi | undefined {
  return useContext(PanelContext);
}

export interface RibbonPanelProps {
  /** Accessible name (the ribbon tab's title); the panel is a `role=group`. */
  label?: string;
  id?: string;
  class?: string;
  children?: JSX.Element;
  /** Called after each layout step with the number of folded groups (tests, diagnostics). */
  onLayout?: (state: { folded: number; scrolling: boolean }) => void;
}

interface Entry extends RibbonGroupRegistration {
  readonly folded: Accessor<boolean>;
  readonly setFolded: (value: boolean) => void;
}

interface FoldRecord {
  entry: Entry;
  /** The panel's scrollWidth just before this group folded. */
  needed: number;
}

const schedule: (cb: () => void) => number =
  typeof requestAnimationFrame === 'function'
    ? (cb) => requestAnimationFrame(cb)
    : (cb) => setTimeout(cb, 16) as unknown as number;
const cancel: (handle: number) => void =
  typeof cancelAnimationFrame === 'function' ? (h) => cancelAnimationFrame(h) : (h) => clearTimeout(h);

function documentOrder(a: Entry, b: Entry): number {
  const ea = a.element();
  const eb = b.element();
  if (!ea || !eb || ea === eb) return 0;
  return ea.compareDocumentPosition(eb) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1;
}

export function RibbonPanel(props: RibbonPanelProps): JSX.Element {
  let panelEl: HTMLDivElement | undefined;
  let trackEl: HTMLDivElement | undefined;
  const entries: Entry[] = [];
  const stack: FoldRecord[] = [];
  const [scrolling, setScrolling] = createSignal(false);
  let frame = 0;
  let steps = 0;
  let trippedAtWidth: number | null = null;
  let unbindWheel: (() => void) | null = null;

  function setScroll(on: boolean): void {
    if (on === scrolling()) return;
    setScrolling(on);
    if (on && panelEl) unbindWheel = bindHorizontalWheel(panelEl);
    else {
      unbindWheel?.();
      unbindWheel = null;
      if (panelEl) panelEl.scrollLeft = 0;
    }
  }

  function requestLayout(): void {
    if (frame !== 0) return;
    frame = schedule(() => {
      frame = 0;
      step();
    });
  }

  /** One layout decision; schedules the next frame when it changed something. */
  function step(): void {
    const el = panelEl;
    if (!el) return;
    const available = el.clientWidth;
    if (trippedAtWidth !== null) {
      if (available === trippedAtWidth) return;
      trippedAtWidth = null;
      steps = 0;
    }
    const needed = el.scrollWidth;
    let changed = false;

    if (needed > available) {
      const open = entries.filter((e) => !e.folded());
      if (open.length > 0) {
        // Lowest priority first; on a tie the one nearest the inline end.
        open.sort((a, b) => a.priority() - b.priority() || -documentOrder(a, b));
        const victim = open[0]!;
        stack.push({ entry: victim, needed });
        victim.setFolded(true);
        changed = true;
      } else if (!scrolling()) {
        setScroll(true);
        changed = true;
      }
    } else {
      if (scrolling()) {
        setScroll(false);
        changed = true;
      }
      const last = stack.at(-1);
      if (last && !entries.includes(last.entry)) {
        stack.pop();
        changed = true;
      } else if (last && available >= last.needed + EXPAND_HYSTERESIS_PX) {
        stack.pop();
        last.entry.setFolded(false);
        changed = true;
      }
    }

    props.onLayout?.({ folded: stack.length, scrolling: scrolling() });
    if (!changed) {
      steps = 0;
      return;
    }
    if (++steps > 4 * entries.length + 8) {
      trippedAtWidth = available;
      steps = 0;
      if (import.meta.env.DEV) console.error('RibbonPanel: layout did not settle; stopped until the width changes');
      return;
    }
    requestLayout();
  }

  const api: RibbonPanelApi = {
    register(group) {
      const [folded, setFolded] = createSignal(false);
      const entry: Entry = { ...group, folded, setFolded };
      entries.push(entry);
      requestLayout();
      return {
        folded,
        dispose: () => {
          const i = entries.indexOf(entry);
          if (i >= 0) entries.splice(i, 1);
          const s = stack.findIndex((r) => r.entry === entry);
          if (s >= 0) stack.splice(s, 1);
          requestLayout();
        },
      };
    },
  };

  onMount(() => {
    if (typeof ResizeObserver === 'function') {
      const observer = new ResizeObserver(() => requestLayout());
      if (panelEl) observer.observe(panelEl);
      if (trackEl) observer.observe(trackEl);
      onCleanup(() => observer.disconnect());
    }
    requestLayout();
  });
  onCleanup(() => {
    if (frame !== 0) cancel(frame);
    unbindWheel?.();
  });

  return (
    <PanelContext.Provider value={api}>
      <div
        ref={panelEl}
        id={props.id}
        class={[css.panel, props.class ?? ''].join(' ').trim()}
        role="group"
        aria-label={props.label}
        data-ribbon-panel=""
        data-scroll={scrolling() ? 'true' : 'false'}
      >
        <div ref={trackEl} class={css.track} data-ribbon-track="">
          {props.children}
        </div>
      </div>
    </PanelContext.Provider>
  );
}
