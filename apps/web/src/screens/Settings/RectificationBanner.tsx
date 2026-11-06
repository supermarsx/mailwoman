// Shown on Settings > Identities when it was opened from a correction request
// (data-requests.ftl dr-rectify-edit): the request's reference stays in view while
// the person edits their details, and Mark as done dismisses it. The route state
// is read once, so it survives opening and closing an identity editor.
import { createSignal, onMount, Show, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { useSettingsNavigation } from './navigation.tsx';
import * as css from './styles.css.ts';

export function RectificationBanner(): JSX.Element {
  const navigation = useSettingsNavigation();
  const [state, setState] = createSignal(navigation?.route().rectification);
  onMount(() => { void loadCatalog('data-requests'); });
  return <Show when={state()}>{rect =>
    <div class={css.item} role="status" data-testid="rectification-banner">
      <span class={css.itemMain}>{t('dr-rectify-banner', { ref: rect().reference })}</span>
      <button type="button" class={css.ghost} data-testid="rectification-done" onClick={() => setState(undefined)}>{t('dr-rectify-done')}</button>
    </div>
  }</Show>;
}
