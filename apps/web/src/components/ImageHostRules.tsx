// One control per image host: Always show / Ask / Never show. Choosing a state
// replaces the previous one (the hosts live in two settings lists, but the reader
// only ever shows the single resulting state). Presentational: the reader owns
// the settings write and the hot-patched re-render that follows.
import { For, Show, type JSX } from 'solid-js';
import { t } from '../i18n/index.ts';
import { imageHostState, type ImageHostLists, type ImageHostState } from '../viewers/imageHostRules.ts';
import * as css from './ImageHostRules.css.ts';

const STATES: readonly ImageHostState[] = ['allow', 'ask', 'block'];
const isolate = (s: string): string => `⁨${s}⁩`;

export function ImageHostRules(props: {
  hosts: readonly string[];
  lists: ImageHostLists;
  /** Settings are locked or still loading. */
  disabled?: boolean;
  onChange: (host: string, state: ImageHostState) => void;
}): JSX.Element {
  return (
    <section class={css.panel} aria-label={t('image-hosts-title')} data-testid="image-host-rules">
      <Show when={props.hosts.length > 0} fallback={<p class={css.hint}>{t('image-hosts-none')}</p>}>
        <p class={css.hint}>{t('image-hosts-help')}</p>
        <For each={props.hosts}>{(host) => {
          const state = (): ImageHostState => imageHostState(host, props.lists);
          return (
            <div class={css.row}>
              <span class={css.host}>{isolate(host)}</span>
              <div class={css.segmented} role="radiogroup" aria-label={t('image-hosts-for', { host: isolate(host) })}>
                <For each={STATES}>{(option) => (
                  <button type="button" role="radio" class={css.option} data-testid={`image-host-${option}`} aria-checked={state() === option} disabled={props.disabled} onClick={() => { if (state() !== option) props.onChange(host, option); }}>
                    {t(`image-hosts-${option}`)}
                  </button>
                )}</For>
              </div>
            </div>
          );
        }}</For>
      </Show>
    </section>
  );
}
