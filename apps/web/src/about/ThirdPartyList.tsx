// Third-party libraries, fonts and icons with their licences (t32-e75; t32-cfg.md
// §4.11, DC9 with full licence texts).
//
// Search (name, version, licence), one licence filter with a count per licence,
// and per entry: the declared SPDX expression, links to the repository and home
// page (new tab, `rel="noopener noreferrer"`), and the licence texts behind a
// native <details>, so the disclosure is keyboard and screen-reader operable
// without extra script. An entry's texts are loaded when it is first opened
// (loadLicenceTexts, one shard chunk per first id character).
//
// Mount (e72 / e74a own the About screens):
//   import { ThirdPartyList, webEntries, crateEntries } from '../../about';
//   onMount(() => void loadCatalog('about'));
//   <ThirdPartyList entries={[...webEntries(), ...crateEntries(cratesDoc)]} />
// `cratesDoc` is crates/mw-server/src/about/third-party.crates.json as the server
// serves it. The list renders 50 entries and a "Show more" button; it has no
// heading of its own (the screen gives one; pass `label` for the region name).

import { For, Show, createMemo, createSignal, createUniqueId, type JSX } from 'solid-js';
import { t } from '../i18n';
import { formatNumber } from '../i18n/format.ts';
import { Feedback } from '../screens/Admin/shell/Feedback.tsx';
import * as shell from '../screens/Admin/shell/shell.css.ts';
import * as css from './thirdParty.css.ts';
import { licenceIdsOf, loadLicenceTexts, type ThirdPartyEntry } from './thirdParty.ts';

export const PAGE_SIZE = 50;

export interface ThirdPartyListProps {
  entries: readonly ThirdPartyEntry[];
  /** Accessible name of the list region. */
  label?: string;
  /** Replaces the lazy text shards (tests, or texts fetched elsewhere). */
  loadTexts?: (ids: readonly string[]) => Promise<Record<string, string>>;
}

const ALL = '';

function kindLabel(e: ThirdPartyEntry): string {
  switch (e.kind) {
    case 'crate':
      return t('about-tp-kind-crate');
    case 'npm':
      return t('about-tp-kind-npm');
    case 'font':
      return t('about-tp-kind-font');
    default:
      return t('about-tp-kind-icons');
  }
}

function artifactLabel(a: string): string {
  switch (a) {
    case 'server':
      return t('about-tp-artifact-server');
    case 'web':
      return t('about-tp-artifact-web');
    case 'plugins':
      return t('about-tp-artifact-plugins');
    case 'desktop':
      return t('about-tp-artifact-desktop');
    case 'mobile':
      return t('about-tp-artifact-mobile');
    default:
      return a;
  }
}

function noteLabel(note: string | undefined): string | null {
  if (note === 'bundled') return t('about-tp-note-bundled');
  if (note === 'fonts-pull') return t('about-tp-note-fonts-pull');
  if (note === 'first-party') return t('about-tp-note-first-party');
  return null;
}

function matches(e: ThirdPartyEntry, q: string): boolean {
  if (!q) return true;
  return [e.name, e.version ?? '', e.licence].some((s) => s.toLowerCase().includes(q));
}

export function ThirdPartyList(props: ThirdPartyListProps): JSX.Element {
  const id = createUniqueId();
  const [query, setQuery] = createSignal('');
  const [licence, setLicence] = createSignal<string>(ALL);
  const [shown, setShown] = createSignal(PAGE_SIZE);

  // Count per licence id over the whole list, most used first; an entry under
  // "MIT OR Apache-2.0" counts for both, so the counts can add up to more than the total.
  const counts = createMemo(() => {
    const m = new Map<string, number>();
    for (const e of props.entries) for (const l of licenceIdsOf(e.licence)) m.set(l, (m.get(l) ?? 0) + 1);
    return [...m.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0], 'en'));
  });

  const filtered = createMemo(() => {
    const q = query().trim().toLowerCase();
    const l = licence();
    return props.entries.filter((e) => matches(e, q) && (l === ALL || licenceIdsOf(e.licence).includes(l)));
  });

  const visible = createMemo(() => filtered().slice(0, shown()));

  function onSearch(q: string): void {
    setQuery(q);
    setShown(PAGE_SIZE);
  }
  function onLicence(l: string): void {
    setLicence(l);
    setShown(PAGE_SIZE);
  }
  function clear(): void {
    setQuery('');
    setLicence(ALL);
    setShown(PAGE_SIZE);
  }

  return (
    <section class={css.root} aria-label={props.label ?? t('about-tp-label')} data-testid="third-party-list">
      <div class={css.toolbar}>
        <form role="search" class={['field', css.search].join(' ')} onSubmit={(e) => e.preventDefault()}>
          <label for={`${id}-q`}>{t('about-tp-search')}</label>
          <input
            id={`${id}-q`}
            type="search"
            autocomplete="off"
            value={query()}
            onInput={(e) => onSearch(e.currentTarget.value)}
          />
        </form>
        <div class={shell.chips} role="group" aria-label={t('about-tp-filter')}>
          <button type="button" class={shell.chip} aria-pressed={licence() === ALL} onClick={() => onLicence(ALL)}>
            {t('about-tp-filter-all', { count: props.entries.length, n: formatNumber(props.entries.length) })}
          </button>
          <For each={counts()}>
            {([l, n]) => (
              <button
                type="button"
                class={shell.chip}
                aria-pressed={licence() === l}
                data-licence={l}
                onClick={() => onLicence(licence() === l ? ALL : l)}
              >
                {t('about-tp-filter-licence', { licence: l, count: n, n: formatNumber(n) })}
              </button>
            )}
          </For>
        </div>
        <p class={css.summary} role="status" aria-live="polite" data-testid="third-party-summary">
          {t('about-tp-showing', {
            count: filtered().length,
            shown: formatNumber(visible().length),
            total: formatNumber(filtered().length),
          })}
        </p>
      </div>

      <Show
        when={filtered().length > 0}
        fallback={
          <div class={shell.state}>
            <p>{t('about-tp-empty')}</p>
            <button type="button" class="btn" onClick={clear}>
              {t('about-tp-clear')}
            </button>
          </div>
        }
      >
        <ul class={css.list}>
          <For each={visible()}>{(e) => <Entry entry={e} loadTexts={props.loadTexts ?? loadLicenceTexts} />}</For>
        </ul>
      </Show>

      <Show when={visible().length < filtered().length}>
        <button type="button" class={['btn', css.more].join(' ')} onClick={() => setShown((n) => n + PAGE_SIZE)}>
          {t('about-tp-more', { count: filtered().length - visible().length, n: formatNumber(filtered().length - visible().length) })}
        </button>
      </Show>
    </section>
  );
}

function Entry(props: {
  entry: ThirdPartyEntry;
  loadTexts: (ids: readonly string[]) => Promise<Record<string, string>>;
}): JSX.Element {
  const e = (): ThirdPartyEntry => props.entry;
  // Fetched when the disclosure is first opened. Plain signals rather than a
  // resource, so a loading text never suspends the screen around the list.
  const [texts, setTexts] = createSignal<Record<string, string> | null>(null);
  const [failed, setFailed] = createSignal(false);
  let loading = false;
  function load(): void {
    if (loading || texts()) return;
    loading = true;
    setFailed(false);
    props.loadTexts(e().texts.map((r) => r.id)).then(
      (all) => setTexts(all),
      () => setFailed(true),
    ).finally(() => {
      loading = false;
    });
  }

  return (
    <li class={css.item} data-testid="third-party-entry">
      <h3 class={css.head}>
        <span>{e().name}</span>
        <Show when={e().version}>{(v) => <span class={css.version}>{v()}</span>}</Show>
        <span class={css.kind}>{kindLabel(e())}</span>
      </h3>
      <p class={css.meta}>
        {t('about-tp-licence')} <span class={css.licence}>{e().licence || t('about-tp-licence-none')}</span>
      </p>
      <Show when={e().artifacts && e().artifacts!.length > 0}>
        <p class={css.meta}>{t('about-tp-used-in', { parts: e().artifacts!.map(artifactLabel).join(', ') })}</p>
      </Show>
      <Show when={noteLabel(e().note)}>{(n) => <p class={css.meta}>{n()}</p>}</Show>
      <Show when={e().repository || e().homepage}>
        <ul class={css.links}>
          <Show when={e().repository}>
            {(href) => (
              <li>
                <a class={css.link} href={href()} target="_blank" rel="noopener noreferrer">
                  {t('about-tp-repository')}
                  <span class={shell.srOnly}> {t('about-tp-new-tab', { name: e().name })}</span>
                </a>
              </li>
            )}
          </Show>
          <Show when={e().homepage}>
            {(href) => (
              <li>
                <a class={css.link} href={href()} target="_blank" rel="noopener noreferrer">
                  {t('about-tp-homepage')}
                  <span class={shell.srOnly}> {t('about-tp-new-tab', { name: e().name })}</span>
                </a>
              </li>
            )}
          </Show>
        </ul>
      </Show>
      <Show when={e().texts.length > 0} fallback={<p class={css.meta}>{t('about-tp-no-text')}</p>}>
        <details class={css.details} onToggle={(ev) => {
            if (ev.currentTarget.open) load();
          }}>
          <summary class={css.detailsSummary}>{t('about-tp-text-toggle', { count: e().texts.length })}</summary>
          <Show when={!failed()} fallback={
            <div>
              <Feedback kind="error" message={t('about-tp-text-error')} />
              <button type="button" class="btn" onClick={load}>
                {t('about-tp-retry')}
              </button>
            </div>
          }>
            <Show when={texts()} fallback={<p class={css.meta}>{t('about-tp-text-loading')}</p>}>
              {(all) => (
                <For each={e().texts}>
                  {(ref) => (
                    <div class={css.textBlock}>
                      <h4 class={css.textTitle}>
                        {ref.file ?? t('about-tp-standard', { licence: ref.standard ?? '' })}
                        <Show when={ref.standard}>
                          {' '}
                          <span class={css.textNote}>{t('about-tp-standard-note')}</span>
                        </Show>
                      </h4>
                      <pre
                        class={css.pre}
                        tabIndex={0}
                        aria-label={t('about-tp-text-label', { name: e().name, file: ref.file ?? ref.standard ?? '' })}
                      >
                        {all()[ref.id] ?? t('about-tp-text-missing')}
                      </pre>
                    </div>
                  )}
                </For>
              )}
            </Show>
          </Show>
        </details>
      </Show>
    </li>
  );
}
