import { createEffect, createMemo, createSignal, For, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { preferenceAccount } from '../../../api/accountPreferenceLifecycle.ts';
import { useSettingsNavigation } from '../navigation.tsx';
import { SETTINGS_CATEGORIES } from '../settingsCategories.ts';
import { field as legacyField } from '../styles.css.ts';
import * as css from './layout.css.ts';
import { cardMatches, normalise, tokens } from './searchModel.ts';
import { searchRegistry } from './registry.ts';

interface CardRef { id: string; title: string }

const STORE = 'mw.settings.cards.collapsed';
const storageKey = (): string => `${STORE}:${String(preferenceAccount() ?? '')}`;
function loadCollapsed(): Set<string> {
  try { const raw = localStorage.getItem(storageKey()); const list = raw ? JSON.parse(raw) as unknown : []; return new Set(Array.isArray(list) ? list.filter((x): x is string => typeof x === 'string') : []); } catch { return new Set(); }
}
function saveCollapsed(set: Set<string>): void {
  try { localStorage.setItem(storageKey(), JSON.stringify([...set])); } catch { /* storage unavailable: state is per page only */ }
}
const slug = (s: string): string => normalise(s).replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'card';
const HEAD = ':scope > h2, :scope > h3, :scope > h4, :scope > header h2, :scope > header h3, :scope > header h4';

/** Sticky search, section sub-navigation (scroll-spy chips) and collapse
 * controls for the cards inside one settings section. It reads the rendered
 * DOM (outermost `section` elements that have a heading), so every section and
 * its existing rows join the grid without per-component edits, and the search
 * covers the rendered Fluent labels and hints at runtime. */
export function SettingsToolbar(props: { name: string }): JSX.Element {
  let bar!: HTMLDivElement;
  let input!: HTMLInputElement;
  const nav = useSettingsNavigation();
  const [cards, setCards] = createSignal<CardRef[]>([], { equals: (a, b) => a.length === b.length && a.every((c, i) => c.id === b[i]!.id && c.title === b[i]!.title) });
  const [query, setQuery] = createSignal('');
  const [active, setActive] = createSignal('');
  const [collapsedCount, setCollapsedCount] = createSignal(0);
  const [noResults, setNoResults] = createSignal(false);
  const collapsed = loadCollapsed();
  let frame: number | undefined;
  let observer: MutationObserver | undefined;
  let io: IntersectionObserver | undefined;

  const root = (): HTMLElement | null => bar.parentElement;
  const cardEls = (): HTMLElement[] => {
    const r = root(); if (!r) return [];
    return [...r.querySelectorAll<HTMLElement>('section')].filter(s => {
      if (s.parentElement?.closest('section') && r.contains(s.parentElement.closest('section'))) return false;
      // Cards in a hidden tab panel are not part of the page until their tab opens.
      if (s.closest('[hidden]:not([data-settings-section])')) return false;
      return s.querySelector(HEAD) !== null;
    });
  };
  const setCollapsed = (el: HTMLElement, value: boolean): void => {
    el.dataset.kitCollapsed = String(value);
    el.querySelector<HTMLElement>(':scope [data-kit-toggle]')?.setAttribute('aria-expanded', String(!value));
  };
  const scan = (): void => {
    frame = undefined;
    const els = cardEls();
    const r0 = root()!;
    const refs: CardRef[] = [];
    const parents = new Map<HTMLElement, number>();
    for (const el of els) {
      const heading = el.querySelector<HTMLElement>(HEAD)!;
      const title = (heading.textContent ?? '').trim();
      if (!el.id) el.id = `kit-${props.name}-${slug(title)}-${refs.length}`;
      el.dataset.kitCard = '';
      const head = (heading.parentElement?.tagName === 'HEADER' && heading.parentElement.parentElement === el ? heading.parentElement : heading) as HTMLElement;
      head.dataset.kitHead = '';
      if (!head.querySelector(':scope > [data-kit-toggle]')) {
        if (!heading.id) heading.id = `${el.id}-title`;
        const button = document.createElement('button');
        button.type = 'button'; button.dataset.kitToggle = '';
        button.setAttribute('aria-labelledby', heading.id);
        head.appendChild(button);
        button.addEventListener('click', () => toggle(el));
        setCollapsed(el, collapsed.has(`${props.name}/${el.id}`));
      }
      refs.push({ id: el.id, title });
      const parent = el.parentElement!; parents.set(parent, (parents.get(parent) ?? 0) + 1);
    }
    for (const [parent, n] of parents) { if (n >= 2) parent.dataset.kitGrid = ''; else delete parent.dataset.kitGrid; }
    setCards(refs);
    if (refs.length > 1) r0.dataset.kitRail = ''; else delete r0.dataset.kitRail;
    setCollapsedCount(els.filter(e => e.dataset.kitCollapsed === 'true').length);
    if (bar.hidden !== (refs.length === 0)) bar.hidden = refs.length === 0;
    observe(els);
    applyFilter();
  };
  const schedule = (): void => { if (frame === undefined) frame = requestAnimationFrame(scan); };
  const toggle = (el: HTMLElement): void => {
    const next = el.dataset.kitCollapsed !== 'true';
    setCollapsed(el, next);
    const key = `${props.name}/${el.id}`;
    if (next) collapsed.add(key); else collapsed.delete(key);
    saveCollapsed(collapsed);
    setCollapsedCount(cardEls().filter(e => e.dataset.kitCollapsed === 'true').length);
  };
  const setAll = (value: boolean): void => {
    for (const el of cardEls()) {
      setCollapsed(el, value);
      const key = `${props.name}/${el.id}`;
      if (value) collapsed.add(key); else collapsed.delete(key);
    }
    saveCollapsed(collapsed);
    setCollapsedCount(value ? cardEls().length : 0);
  };
  const observe = (els: HTMLElement[]): void => {
    if (typeof IntersectionObserver === 'undefined') return;
    io?.disconnect();
    io = new IntersectionObserver(entries => {
      const hit = entries.filter(e => e.isIntersecting).sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top)[0];
      if (hit) setActive(hit.target.id);
    }, { rootMargin: '-15% 0px -65% 0px' });
    for (const el of els) io.observe(el);
  };
  const jump = (id: string): void => {
    const el = document.getElementById(id); if (!el) return;
    if (el.dataset.kitCollapsed === 'true') toggle(el);
    const reduce = typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
    el.scrollIntoView?.({ block: 'start', behavior: reduce ? 'auto' : 'smooth' });
    el.querySelector<HTMLElement>('[data-kit-toggle]')?.focus({ preventScroll: true });
    setActive(id);
  };

  // Rows that carry text to search: the kit's rows and the legacy field blocks.
  const rowSelector = `[data-setting-row], .${CSS.escape(legacyField)}`;
  function applyFilter(): void {
    const r = root(); if (!r) return;
    const words = tokens(query());
    const els = cardEls();
    delete r.dataset.kitSearching;
    for (const el of r.querySelectorAll<HTMLElement>('[data-kit-hidden],[data-kit-match]')) { delete el.dataset.kitHidden; delete el.dataset.kitMatch; }
    if (!words.length) { setNoResults(false); return; }
    r.dataset.kitSearching = '';
    let any = false;
    for (const el of els) {
      const title = el.querySelector(HEAD)?.textContent ?? '';
      const rows = [...el.querySelectorAll<HTMLElement>(rowSelector)];
      if (cardMatches(title, words)) { any = true; continue; }
      if (!rows.length) { if (cardMatches(el.textContent ?? '', words)) { any = true; el.dataset.kitMatch = ''; } else el.dataset.kitHidden = ''; continue; }
      let shown = 0;
      for (const row of rows) {
        if (cardMatches(`${title} ${row.textContent ?? ''}`, words)) { shown++; row.dataset.kitMatch = ''; } else row.dataset.kitHidden = '';
      }
      if (shown) any = true; else el.dataset.kitHidden = '';
    }
    setNoResults(!any);
  }
  const onQuery = (value: string): void => { setQuery(value); applyFilter(); };
  // Rows of other (possibly never opened) sections, from the static registry.
  const remote = createMemo(() => searchRegistry(query(), props.name));
  const remoteGroups = (): { section: string; label: string; hits: ReturnType<typeof searchRegistry> }[] => {
    const by = new Map<string, ReturnType<typeof searchRegistry>>();
    for (const hit of remote()) by.set(hit.section, [...(by.get(hit.section) ?? []), hit]);
    const entries = SETTINGS_CATEGORIES.flatMap(c => c.sections);
    return [...by].map(([section, hits]) => ({ section, label: t(entries.find(e => e.id === section)?.label ?? section), hits }));
  };
  const open = (section: string, card: string): void => { onQuery(''); void nav?.navigate({ section, card }); };
  // Deep link: the route names a card of this section.
  let jumped = '';
  createEffect(() => {
    const want = nav?.route().card;
    const have = cards();
    if (!want || jumped === `${props.name}/${want}` || !have.some(c => c.id === want)) return;
    jumped = `${props.name}/${want}`;
    queueMicrotask(() => {
      jump(want);
      const el = document.getElementById(want);
      if (el) { el.dataset.kitFlash = ''; setTimeout(() => delete el.dataset.kitFlash, 1600); }
    });
  });
  const suggestions = (): { id: string; label: string }[] => {
    const words = tokens(query());
    return SETTINGS_CATEGORIES.flatMap(c => c.sections).filter(s => s.id !== props.name && cardMatches(`${t(s.label)} ${s.terms ?? ''}`, words)).map(s => ({ id: s.id, label: t(s.label) }));
  };

  const onKey = (event: KeyboardEvent): void => {
    if (event.key !== '/' || event.ctrlKey || event.metaKey || event.altKey) return;
    const target = event.target as HTMLElement | null;
    if (target && (target.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName))) return;
    if (!bar.isConnected || bar.hidden) return;
    event.preventDefault(); input.focus(); input.select();
  };
  onMount(() => {
    const r = root(); if (!r) return;
    scan();
    observer = new MutationObserver(schedule);
    observer.observe(r, { childList: true, subtree: true, attributes: true, attributeFilter: ['hidden'] });
    document.addEventListener('keydown', onKey);
    // Heading text is reactive and arrives with lazily loaded catalogs: re-read chip titles shortly after mount.
    for (const ms of [250, 1000]) { const id = setTimeout(schedule, ms); onCleanup(() => clearTimeout(id)); }
  });
  onCleanup(() => {
    observer?.disconnect(); io?.disconnect(); document.removeEventListener('keydown', onKey);
    if (frame !== undefined) cancelAnimationFrame(frame);
  });

  return <div ref={bar} class={css.toolbar} role="search" aria-label={t('settings-kit-toolbar')} hidden>
    <input ref={input} type="text" class={css.searchBox} value={query()} enterkeyhint="search" autocomplete="off" spellcheck={false}
      aria-label={t('settings-kit-search')} placeholder={t('settings-kit-search-placeholder')} onInput={e => onQuery(e.currentTarget.value)}
      onKeyDown={e => { if (e.key === 'Escape' && query()) { e.stopPropagation(); onQuery(''); } }} />
    <Show when={cards().length > 1}>
      <button type="button" class={css.toolButton} onClick={() => setAll(collapsedCount() < cards().length)}>
        {collapsedCount() < cards().length ? t('settings-kit-collapse-all') : t('settings-kit-expand-all')}
      </button>
      <ul class={css.chips} aria-label={t('settings-kit-jump')}>
        <For each={cards()}>{card => <li><button type="button" class={css.chip} aria-current={active() === card.id ? 'true' : undefined} onClick={() => jump(card.id)}>{card.title}</button></li>}</For>
      </ul>
    </Show>
    <Show when={remote().length > 0}>
      <div class={css.remote} role="region" aria-label={t('settings-kit-other-sections')}>
        <For each={remoteGroups()}>{group => <section><h3 class={css.remoteHead}>{group.label}</h3><ul class={css.remoteList}>
          <For each={group.hits}>{hit => <li><button type="button" class={css.remoteHit} onClick={() => open(hit.section, hit.card)}>{hit.label}<span class={css.remoteCard}>{hit.cardTitle}</span></button></li>}</For>
        </ul></section>}</For>
      </div>
    </Show>
    <Show when={noResults() && remote().length === 0}>
      <p class={css.empty} role="status">{t('settings-kit-no-results')}
        <For each={suggestions()}>{s => <button type="button" class={css.chip} onClick={() => { onQuery(''); void nav?.navigate({ section: s.id }); }}>{s.label}</button>}</For>
      </p>
    </Show>
  </div>;
}
