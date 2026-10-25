// Third-party libraries, fonts, icons and image files with their licences (t32-e75;
// t32-cfg.md §4.11, DC9 with full licence texts).
//
// Grouped by origin (shipped Rust crates, crates only in the desktop/mobile apps,
// bundled npm packages, assets, server image files, build tools), with one search
// (name, version, licence) and one licence filter with a count per licence across all
// groups. Per entry: the declared licence, links to the repository and home page (new
// tab, `rel="noopener noreferrer"`) and a button that opens the licence texts in a
// modal dialog (focus moves in and is trapped, Escape or Close ends it, focus returns
// to the button). An entry's texts are loaded when its dialog first opens
// (loadLicenceTexts, one shard chunk per first id character).
//
// Mount (e74a admin About, e74b webmail About): import the lazy wrapper from
// `about/index.ts` and give it the screen's own h2 above it (groups are h3, entries h4):
//   import { ThirdPartyList } from '../../about';
//   onMount(() => void loadCatalog('about'));
//   <Suspense fallback={…}><ThirdPartyList label={t('…')} /></Suspense>
// With no `entries` it shows the committed third-party.json; all data is in the web
// bundle. Each group renders 50 entries and a "Show more" button.

import { For, Show, createMemo, createSignal, createUniqueId, onMount, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import { t } from '../i18n';
import { formatNumber } from '../i18n/format.ts';
import { createFocusTrap } from '../components/a11y';
import { Feedback } from '../screens/Admin/shell/Feedback.tsx';
import * as shell from '../screens/Admin/shell/shell.css.ts';
import * as css from './thirdParty.css.ts';
import {
  GROUPS,
  aboutEntries,
  licenceIdsOf,
  loadLicenceTexts,
  type ThirdPartyEntry,
  type ThirdPartyGroup,
} from './thirdParty.ts';

export const PAGE_SIZE = 50;

type LoadTexts = (ids: readonly string[]) => Promise<Record<string, string>>;

export interface ThirdPartyListProps {
  /** Defaults to every entry of the committed third-party.json. */
  entries?: readonly ThirdPartyEntry[];
  /** Accessible name of the list region. */
  label?: string;
  /** Replaces the lazy text shards (tests, or texts fetched elsewhere). */
  loadTexts?: LoadTexts;
}

const ALL = '';

function groupTitle(g: ThirdPartyGroup): string {
  switch (g) {
    case 'rust':
      return t('about-tp-group-rust');
    case 'shells':
      return t('about-tp-group-shells');
    case 'web':
      return t('about-tp-group-web');
    case 'assets':
      return t('about-tp-group-assets');
    case 'runtimeImage':
      return t('about-tp-group-image');
    default:
      return t('about-tp-group-build');
  }
}

function kindLabel(e: ThirdPartyEntry): string {
  switch (e.kind) {
    case 'crate':
      return t('about-tp-kind-crate');
    case 'npm':
      return t('about-tp-kind-npm');
    case 'font':
      return t('about-tp-kind-font');
    case 'icons':
      return t('about-tp-kind-icons');
    case 'image':
      return t('about-tp-kind-image');
    default:
      return t('about-tp-kind-os');
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

function imageLabel(i: string): string {
  if (i === 'runtime') return t('about-tp-image-runtime');
  if (i === 'runtime-static') return t('about-tp-image-static');
  return i;
}

function noteLabel(note: string | undefined): string | null {
  switch (note) {
    case 'bundled':
      return t('about-tp-note-bundled');
    case 'fonts-pull':
      return t('about-tp-note-fonts-pull');
    case 'first-party':
      return t('about-tp-note-first-party');
    case 'static-libc':
      return t('about-tp-note-static-libc');
    case 'alpine-ca':
      return t('about-tp-note-alpine-ca');
    default:
      return null;
  }
}

function matches(e: ThirdPartyEntry, q: string): boolean {
  if (!q) return true;
  return [e.name, e.version ?? '', e.licence].some((s) => s.toLowerCase().includes(q));
}

export function ThirdPartyList(props: ThirdPartyListProps): JSX.Element {
  const id = createUniqueId();
  const all = createMemo(() => props.entries ?? aboutEntries());
  const [query, setQuery] = createSignal('');
  const [licence, setLicence] = createSignal<string>(ALL);
  const [shown, setShown] = createSignal<Partial<Record<ThirdPartyGroup, number>>>({});
  const [dialogFor, setDialogFor] = createSignal<ThirdPartyEntry | null>(null);
  // One load per entry for the life of the list, so reopening a dialog is instant;
  // a failed load is forgotten so Try again asks once more.
  const loaded = new Map<string, Promise<Record<string, string>>>();
  const load: LoadTexts = (ids) => {
    const key = ids.join(',');
    let p = loaded.get(key);
    if (!p) {
      p = (props.loadTexts ?? loadLicenceTexts)(ids);
      p.catch(() => loaded.delete(key));
      loaded.set(key, p);
    }
    return p;
  };

  // Count per licence id over the whole list, most used first; an entry under
  // "MIT OR Apache-2.0" counts for both, so the counts can add up to more than the total.
  const counts = createMemo(() => {
    const m = new Map<string, number>();
    for (const e of all()) for (const l of licenceIdsOf(e.licence)) m.set(l, (m.get(l) ?? 0) + 1);
    return [...m.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0], 'en'));
  });

  const filtered = createMemo(() => {
    const q = query().trim().toLowerCase();
    const l = licence();
    return all().filter((e) => matches(e, q) && (l === ALL || licenceIdsOf(e.licence).includes(l)));
  });

  const groups = createMemo(() =>
    GROUPS.map((g) => ({ group: g, items: filtered().filter((e) => e.group === g) })).filter((x) => x.items.length > 0),
  );
  const limit = (g: ThirdPartyGroup): number => shown()[g] ?? PAGE_SIZE;
  const visibleCount = createMemo(() => groups().reduce((n, x) => n + Math.min(x.items.length, limit(x.group)), 0));

  function reset(): void {
    setShown({});
  }
  function clear(): void {
    setQuery('');
    setLicence(ALL);
    reset();
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
            onInput={(e) => {
              setQuery(e.currentTarget.value);
              reset();
            }}
          />
        </form>
        <div class={shell.chips} role="group" aria-label={t('about-tp-filter')}>
          <button
            type="button"
            class={shell.chip}
            aria-pressed={licence() === ALL}
            onClick={() => {
              setLicence(ALL);
              reset();
            }}
          >
            {t('about-tp-filter-all', { count: all().length, n: formatNumber(all().length) })}
          </button>
          <For each={counts()}>
            {([l, n]) => (
              <button
                type="button"
                class={shell.chip}
                aria-pressed={licence() === l}
                onClick={() => {
                  setLicence(licence() === l ? ALL : l);
                  reset();
                }}
              >
                {t('about-tp-filter-licence', { licence: l, count: n, n: formatNumber(n) })}
              </button>
            )}
          </For>
        </div>
        <p class={css.summary} role="status" aria-live="polite" data-testid="third-party-summary">
          {t('about-tp-showing', {
            count: filtered().length,
            shown: formatNumber(visibleCount()),
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
        <For each={groups()}>
          {(g) => {
            const headingId = createUniqueId();
            const visible = (): ThirdPartyEntry[] => g.items.slice(0, limit(g.group));
            return (
              <section class={css.group} aria-labelledby={headingId}>
                <h3 id={headingId} class={css.groupTitle}>
                  {t('about-tp-group-count', { title: groupTitle(g.group), n: formatNumber(g.items.length) })}
                </h3>
                <ul class={css.list}>
                  <For each={visible()}>{(e) => <Entry entry={e} onOpen={() => setDialogFor(e)} />}</For>
                </ul>
                <Show when={visible().length < g.items.length}>
                  <button
                    type="button"
                    class={['btn', css.more].join(' ')}
                    onClick={() => setShown({ ...shown(), [g.group]: limit(g.group) + PAGE_SIZE })}
                  >
                    {t('about-tp-more', { n: formatNumber(g.items.length - visible().length) })}
                  </button>
                </Show>
              </section>
            );
          }}
        </For>
      </Show>

      <Show when={dialogFor()}>
        {(e) => <LicenceDialog entry={e()} loadTexts={load} onClose={() => setDialogFor(null)} />}
      </Show>
    </section>
  );
}

function Entry(props: { entry: ThirdPartyEntry; onOpen: () => void }): JSX.Element {
  const e = (): ThirdPartyEntry => props.entry;
  const licenceText = (): string => {
    if (e().licence) return e().licence;
    return e().kind === 'os' ? t('about-tp-licence-in-file') : t('about-tp-licence-none');
  };
  return (
    <li class={css.item} data-testid="third-party-entry">
      <h4 class={css.head}>
        <span>{e().name}</span>
        <Show when={e().version}>{(v) => <span class={css.version}>{v()}</span>}</Show>
        <span class={css.kind}>{kindLabel(e())}</span>
      </h4>
      <p class={css.meta}>
        {t('about-tp-licence')} <span class={css.licence}>{licenceText()}</span>
      </p>
      <Show when={e().artifacts && e().artifacts!.length > 0}>
        <p class={css.meta}>{t('about-tp-used-in', { parts: e().artifacts!.map(artifactLabel).join(', ') })}</p>
      </Show>
      <Show when={e().images && e().images!.length > 0}>
        <p class={css.meta}>{t('about-tp-in-images', { images: e().images!.map(imageLabel).join(', ') })}</p>
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
        <button
          type="button"
          class={['btn', 'btn--ghost', css.textButton].join(' ')}
          aria-label={t('about-tp-text-open-for', { name: e().name })}
          aria-haspopup="dialog"
          onClick={() => props.onOpen()}
        >
          {t('about-tp-text-toggle', { count: e().texts.length })}
        </button>
      </Show>
    </li>
  );
}

function LicenceDialog(props: { entry: ThirdPartyEntry; loadTexts: LoadTexts; onClose: () => void }): JSX.Element {
  const id = createUniqueId();
  const [el, setEl] = createSignal<HTMLElement>();
  const [texts, setTexts] = createSignal<Record<string, string> | null>(null);
  const [failed, setFailed] = createSignal(false);
  let closeButton: HTMLButtonElement | undefined;
  const e = (): ThirdPartyEntry => props.entry;

  createFocusTrap(el, { onEscape: () => props.onClose(), initialFocus: () => closeButton });

  function load(): void {
    setFailed(false);
    props.loadTexts(e().texts.map((r) => r.id)).then(
      (all) => setTexts(all),
      () => setFailed(true),
    );
  }
  onMount(load);

  return (
    <Portal>
      <div class={shell.dialogScrim} onClick={() => props.onClose()} />
      <div
        ref={setEl}
        class={css.dialog}
        role="dialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        tabindex="-1"
        data-testid="third-party-dialog"
      >
        <h2 id={`${id}-title`} class={shell.dialogTitle}>
          {t('about-tp-dialog-title', { name: e().name, version: e().version ?? '' })}
        </h2>
        <Show
          when={!failed()}
          fallback={
            <div>
              <Feedback kind="error" message={t('about-tp-text-error')} />
              <button type="button" class="btn" onClick={load}>
                {t('about-tp-retry')}
              </button>
            </div>
          }
        >
          <Show when={texts()} fallback={<p class={css.meta}>{t('about-tp-text-loading')}</p>}>
            {(all) => (
              <For each={e().texts}>
                {(ref) => (
                  <div class={css.textBlock}>
                    <h3 class={css.textTitle}>
                      {ref.file ?? t('about-tp-standard', { licence: ref.standard ?? '' })}
                      <Show when={ref.standard}>
                        {' '}
                        <span class={css.textNote}>{t('about-tp-standard-note')}</span>
                      </Show>
                    </h3>
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
        <div class={shell.dialogActions}>
          <button ref={closeButton} type="button" class="btn" onClick={() => props.onClose()}>
            {t('about-tp-close')}
          </button>
        </div>
      </div>
    </Portal>
  );
}
