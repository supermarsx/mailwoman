// The cookie consent control: a banner, card or modal chosen by the administrator,
// with Accept all, Reject all and a per-category view. It is also the Cookie
// preferences dialog that Settings, the sign-in page and the footer reopen.
//
// Accessibility: a dialog with a name and description; the modal layout traps
// focus and closes on Escape when reopened from a link; the banner and card do not
// take focus from the page. Toggles are native checkboxes with visible labels.

import { For, Show, createEffect, createSignal, createUniqueId, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../i18n';
import { activeLocaleSignal } from '../i18n/registry.ts';
import { createFocusTrap } from '../components/a11y';
import * as css from './consent.css.ts';
import { policyLinks } from './policyLinks.ts';
import {
  bootConsent, closeCookiePreferences, consentConfig, consentEffective, consentStored, decide, openCookiePreferences, preferencesVisible, previewing,
} from './runtime.ts';
import { localized, OPTIONAL, type Category, type Choices, type InventoryEntry } from './policy.ts';

const DEFAULT_LABEL: Record<Category, string> = {
  necessary: 'cookies-category-necessary',
  functional: 'cookies-category-functional',
  analytics: 'cookies-category-analytics',
  marketing: 'cookies-category-marketing',
};

/** The visitor-facing text of an inventory entry; falls back to the server's English description. */
export function entryPurpose(entry: Pick<InventoryEntry, 'id' | 'purpose'>): string {
  const id = `cookies-inv-${entry.id}`;
  const text = t(id);
  return text === id ? entry.purpose : text;
}

export function ConsentLayer(): JSX.Element {
  onMount(() => { void loadCatalog('cookies'); void bootConsent(); });
  const id = createUniqueId();
  const [details, setDetails] = createSignal(false);
  const [showItems, setShowItems] = createSignal(false);
  const [choices, setChoices] = createSignal<Choices>({ necessary: true, functional: false, analytics: false, marketing: false });
  let panel: HTMLDivElement | undefined;

  const config = consentConfig;
  const effective = consentEffective;
  const visible = (): boolean => {
    const c = config();
    const e = effective();
    return !!c && c.enabled && !!e && (e.needsDecision || preferencesVisible());
  };
  const reopened = (): boolean => preferencesVisible() && !(effective()?.needsDecision ?? false);
  const layout = (): 'banner' | 'card' | 'modal' => (reopened() ? 'modal' : config()?.layout ?? 'banner');
  const position = (): 'bottom' | 'top' | 'start' | 'end' | 'center' => (layout() === 'modal' ? 'center' : config()?.position ?? 'bottom');
  const trapped = (): boolean => visible() && layout() === 'modal';
  const mode = (): string => config()?.mode ?? 'optIn';

  // Each time the control opens, the toggles start from what applies now.
  createEffect(() => {
    if (!visible()) return;
    const e = effective();
    if (e) setChoices({ ...e.allowed, necessary: true });
    setDetails(reopened() && mode() !== 'noticeOnly');
    setShowItems(false);
  });
  createFocusTrap(() => panel, { active: trapped, onEscape: () => { if (preferencesVisible()) closeCookiePreferences(); } });

  const label = (c: Category): string => localized(config()?.categories?.find((x) => x.id === c)?.label, activeLocaleSignal()) ?? t(DEFAULT_LABEL[c]);
  const describe = (c: Category): string => localized(config()?.categories?.find((x) => x.id === c)?.description, activeLocaleSignal()) ?? t(`${DEFAULT_LABEL[c]}-help`);
  const titleText = (): string => localized(config()?.text?.title, activeLocaleSignal()) ?? t('cookies-title');
  const bodyText = (): string => localized(config()?.text?.body, activeLocaleSignal()) ?? t(mode() === 'noticeOnly' ? 'cookies-body-notice' : mode() === 'optOut' ? 'cookies-body-optout' : 'cookies-body-optin');
  const reason = (): string | null => {
    const s = effective()?.stale;
    return s === 'version' ? t('cookies-reconsent-version') : s === 'expired' ? t('cookies-reconsent-expired') : null;
  };
  const offered = (): Category[] => OPTIONAL.filter((c) => config()?.categories?.find((x) => x.id === c)?.enabled === true);
  const itemsOf = (c: Category): InventoryEntry[] => (config()?.inventory ?? []).filter((i) => i.category === c);
  const equal = (): boolean => config()?.rejectProminent !== false;
  const signalNote = (): string | null => {
    const s = effective()?.signal;
    return s === 'gpc' ? t('cookies-signal-gpc') : s === 'dnt' ? t('cookies-signal-dnt') : null;
  };

  const links = (): JSX.Element => (
    <ul class={css.links}>
      <Show when={policyLinks(config()).cookies}>{(url) => <li><a class={css.link} href={url()} target="_blank" rel="noopener noreferrer">{t('cookies-policy-link')}</a></li>}</Show>
      <Show when={policyLinks(config()).privacy}>{(url) => <li><a class={css.link} href={url()} target="_blank" rel="noopener noreferrer">{t('cookies-privacy-link')}</a></li>}</Show>
    </ul>
  );

  const categoryList = (): JSX.Element => (
    <ul class={css.categories}>
      <li class={css.category}>
        <input class={css.check} id={`${id}-necessary`} type="checkbox" checked disabled aria-describedby={`${id}-necessary-help`} />
        <label for={`${id}-necessary`} class={css.categoryName}>{label('necessary')} <span class={css.note}>({t('cookies-always-on')})</span></label>
        <p id={`${id}-necessary-help`} class={css.categoryText}>{describe('necessary')}</p>
        <Show when={showItems()}><ItemList items={itemsOf('necessary')} /></Show>
      </li>
      <For each={offered()}>{(c) => (
        <li class={css.category}>
          <input class={css.check} id={`${id}-${c}`} type="checkbox" checked={choices()[c]} disabled={!effective()?.adjustable[c]} aria-describedby={`${id}-${c}-help`}
            onChange={(event) => setChoices({ ...choices(), [c]: event.currentTarget.checked })} />
          <label for={`${id}-${c}`} class={css.categoryName}>{label(c)}</label>
          <p id={`${id}-${c}-help`} class={css.categoryText}>{describe(c)}</p>
          <Show when={showItems()}><ItemList items={itemsOf(c)} /></Show>
        </li>
      )}</For>
    </ul>
  );

  return (
    <Show when={visible()}>
      <Show when={layout() === 'modal'}><div class={css.backdrop} aria-hidden="true" /></Show>
      <div
        ref={panel}
        class={`${css.panel} ${css.layout[layout()]} ${css.place[position()]} ${layout() !== 'banner' && position() !== 'bottom' && position() !== 'top' ? css.cornerWidth : ''}`}
        role="dialog"
        aria-modal={layout() === 'modal' ? 'true' : 'false'}
        aria-labelledby={`${id}-title`}
        aria-describedby={`${id}-body`}
        tabindex="-1"
        data-testid="cookie-consent"
        data-layout={layout()}
        data-mode={mode()}
      >
        <h2 id={`${id}-title`} class={css.title}>{reopened() ? t('cookies-preferences-title') : titleText()}</h2>
        <Show when={reason()}>{(text) => <p class={css.note} role="status">{text()}</p>}</Show>
        <p id={`${id}-body`} class={css.body}>{bodyText()}</p>
        <Show when={signalNote()}>{(text) => <p class={css.note} role="status">{text()}</p>}</Show>
        <Show when={details() && mode() !== 'noticeOnly'}>
          {categoryList()}
          <button type="button" class={`${css.quiet} ${css.quietFocus}`} aria-expanded={showItems()} onClick={() => setShowItems(!showItems())}>
            {showItems() ? t('cookies-hide-items') : t('cookies-show-items')}
          </button>
        </Show>
        {links()}
        <div class={css.actions}>
          <Show when={mode() === 'noticeOnly'} fallback={
            <>
              <Show when={details()} fallback={
                <>
                  <button type="button" class={`btn btn--primary ${equal() ? css.equal : ''}`} onClick={() => void decide('acceptAll')}>{t('cookies-accept-all')}</button>
                  <button type="button" class={equal() ? `btn btn--primary ${css.equal}` : `${css.quiet} ${css.quietFocus}`} onClick={() => void decide('rejectAll')}>{t('cookies-reject-all')}</button>
                  <button type="button" class="btn btn--ghost" onClick={() => setDetails(true)}>{t('cookies-customize')}</button>
                </>
              }>
                <button type="button" class="btn btn--primary" onClick={() => void decide('custom', choices())}>{t('cookies-save')}</button>
                <button type="button" class="btn btn--ghost" onClick={() => void decide('acceptAll')}>{t('cookies-accept-all')}</button>
                <button type="button" class="btn btn--ghost" onClick={() => void decide('rejectAll')}>{t('cookies-reject-all')}</button>
              </Show>
              <Show when={reopened() || previewing()}><button type="button" class="btn btn--ghost" onClick={() => closeCookiePreferences()}>{t('cookies-close')}</button></Show>
            </>
          }>
            <button type="button" class="btn btn--primary" onClick={() => void decide('notice')}>{t('cookies-ok')}</button>
            <Show when={reopened() || previewing()}><button type="button" class="btn btn--ghost" onClick={() => closeCookiePreferences()}>{t('cookies-close')}</button></Show>
          </Show>
        </div>
        <Show when={consentStored() && reopened()}>
          <p class={css.note}>{t('cookies-decided', { date: new Date(consentStored()!.decidedAt).toLocaleDateString(activeLocaleSignal()) })}</p>
        </Show>
      </div>
    </Show>
  );
}

function ItemList(props: { items: InventoryEntry[] }): JSX.Element {
  return (
    <ul class={css.detailsList} style={{ 'grid-column': '2' }}>
      <For each={props.items}>{(item) => (
        <li><span class={css.mono}>{item.pattern}</span>: {entryPurpose(item)} ({item.duration})</li>
      )}</For>
    </ul>
  );
}

/** A link that reopens the preferences dialog (Settings, sign-in page, footer). */
export function CookiePreferencesLink(props: { class?: string }): JSX.Element {
  const [shown, setShown] = createSignal(false);
  onMount(() => { void loadCatalog('cookies'); void bootConsent().then(() => setShown(consentConfig()?.enabled === true)); });
  return (
    <Show when={shown() || consentConfig()?.enabled === true}>
      <button type="button" class={props.class ?? `${css.quiet} ${css.quietFocus}`} data-testid="cookie-preferences-link" onClick={() => openCookiePreferences()}>
        {t('cookies-preferences-link')}
      </button>
    </Show>
  );
}
