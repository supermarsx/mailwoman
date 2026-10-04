// Admin › Security policy (§19).
//
// This screen has no controls. Every field of the stored security-policy record
// was a value the server saved and nothing applied (26.20, t28-e8):
//   * min TLS, capture policy, Argon2 parameters — no defined meaning; the server
//     no longer returns or accepts them (`crates/mw-server/src/admin.rs`,
//     `SecurityPolicyDto`).
//   * "Require 2FA" — a duplicate of the enforced control on the Require
//     two-factor screen (`TwoFactorPolicy.tsx`).
//   * DLP rules, maximum-security floor — still stored by
//     `PUT /admin/security-policy`, not applied; they return here when a reader
//     exists.
// What it shows instead is where each of those is actually decided.

import { type JSX } from 'solid-js';
import { t } from '../../i18n';
import * as css from './admin.css.ts';

export function SecurityPolicy(): JSX.Element {
  return (
    <section class={css.section} aria-label={t('admin-security-title')}>
      <h2 class={css.heading}>{t('admin-security-title')}</h2>
      <div class={css.card}>
        <p class={css.note}>{t('admin-security-none')}</p>
        <ul>
          <li>{t('admin-security-where-2fa')}</li>
          <li>{t('admin-security-where-dlp')}</li>
          <li>{t('admin-security-where-tls')}</li>
        </ul>
      </div>
    </section>
  );
}
