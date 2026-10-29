// The admin console's loading placeholders (t72), drawn with the same classes
// as the static boot placeholder in index.html (styles/boot.css, which ships in
// the entry stylesheet), so a placeholder that takes over from the static one,
// or shows without it, has the same shapes, sizes and pulse.
//
//   • AdminLoginSkeleton: the sign-in card while the session is checked.
//   • AdminNavSkeleton: the sidebar's navigation while the admin's roles load.
//   • AdminPageSkeleton: a section's page while its chunk or the roles load: the
//     page heading row and the same bounded loading panel the dashboard shows
//     while its data loads, so the dashboard takes over without moving a box.

import { For, Show, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { passkeySupported } from '../../Settings/webauthn.ts';
import { AsyncPending } from '../../../components/AsyncState.tsx';
import * as a11y from '../../../components/mailA11y.css.ts';
import * as shellCss from './shell.css.ts';

const NBSP = ' ';
/** One line of text: a transparent space sets its height, the bar is the placeholder. */
const Line = (): JSX.Element => <span class="mwb-tx">{NBSP}<span class="mwb-block" /></span>;

/** The sign-in card's shapes. Kept in step with `#mw-boot .mwb-admin` in index.html. */
export function AdminLoginSkeleton(): JSX.Element {
  return (
    <main class="mwb-admin mwb-admin--inline" data-testid="admin-login-skeleton">
      <span class={a11y.srOnly} role="status" aria-live="polite">{t('common-loading')}</span>
      <div class="mwb-admin-card" aria-hidden="true" inert>
        <span class="mwb-admin-logo"><span class="mwb-block" /></span>
        <span class="mwb-tx mwb-admin-title">{NBSP}<span class="mwb-block" /></span>
        <span class="mwb-admin-note"><Line /><Line /><Line /></span>
        <span class="field"><Line /><input type="text" tabindex="-1" autocomplete="off" aria-hidden="true" /></span>
        <span class="field"><Line /><input type="text" tabindex="-1" autocomplete="off" aria-hidden="true" /></span>
        <button type="button" class="btn" tabindex="-1">{NBSP}</button>
        <Show when={passkeySupported()}><button type="button" class="btn" tabindex="-1">{NBSP}</button></Show>
      </div>
    </main>
  );
}

/** Two groups of navigation entries, as the sidebar lists them. */
export function AdminNavSkeleton(): JSX.Element {
  return (
    <div class="mwb-console-nav" aria-hidden="true" data-testid="admin-nav-skeleton">
      <For each={[4, 5]}>
        {(n) => (
          <span class="mwb-console-group">
            <span class="mwb-block mwb-console-group-label" />
            <For each={Array.from({ length: n })}>{() => <span class="mwb-console-item"><span class="mwb-block" /></span>}</For>
          </span>
        )}
      </For>
    </div>
  );
}

/** The page heading row and the bounded loading panel. */
export function AdminPageSkeleton(props: { message?: string }): JSX.Element {
  return (
    <div class="mwb-console-page" data-testid="admin-page-skeleton">
      <div class="mwb-console-page-head" aria-hidden="true"><span class="mwb-block" /></div>
      <div class={shellCss.state}>
        <AsyncPending variant="page" message={props.message ?? t('admin-shell-loading')} />
      </div>
    </div>
  );
}
