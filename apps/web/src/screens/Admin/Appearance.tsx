// Admin › Appearance (§19).
//
// This screen has no controls. `PUT /admin/appearance` changes a value held only
// in the server's memory (`mw_admin::Admin::set_appearance`,
// `crates/mw-admin/src/lib.rs`): it is not persisted, so a restart returned every
// deployment to the built-in default while the form went on showing "Saved."
// The form returns when the server persists the value (26.20, t28-e8).
//
// A user's own appearance (Settings) is unaffected: that is stored per account.

import { type JSX } from 'solid-js';
import { t } from '../../i18n';
import * as css from './admin.css.ts';

export function Appearance(): JSX.Element {
  return (
    <section class={css.section} aria-label={t('admin-appearance-title')}>
      <h2 class={css.heading}>{t('admin-appearance-title')}</h2>
      <div class={css.card}>
        <p class={css.note}>{t('admin-appearance-none')}</p>
      </div>
    </section>
  );
}
