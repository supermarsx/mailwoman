import { describe, expect, it } from 'vitest';

// Console and settings text that states what the server does. These pin the corrected
// wording (checked against the server code when written) so it is not reverted by accident.
const sources = import.meta.glob<string>('../../locales/en/*.ftl', { query: '?raw', import: 'default', eager: true });

const messageOf = (id: string): string => {
  for (const source of Object.values(sources)) {
    const lines = source.split(/\r?\n/);
    const at = lines.findIndex((l) => l.startsWith(`${id} = `));
    if (at < 0) continue;
    let end = at + 1;
    while (end < lines.length && /^(\s|\})/.test(lines[end] ?? '')) end += 1;
    return lines.slice(at, end).join('\n');
  }
  throw new Error(`missing ${id}`);
};

const CASES: Array<[string, RegExp]> = [
  ['admin-sso-want-encrypted-help', /refuses every encrypted assertion/],
  ['admin-sso-sp-key-help', /does not decrypt SAML assertions/],
  ['admin-sso-claim-display-help', /does not read the display name/],
  ['admin-sso-claim-groups-help', /does not read group membership/],
  ['admin-egress-intro', /socks5h route is the exception/],
  ['avatars-brand-avatars-help', /hidden only when the message failed DKIM or DMARC/],
  ['admin-settings-help-reader-attachments-collapsed', /On by default: the bar starts collapsed/],
  ['admin-settings-help-webhooks-max-attempts', /tried 3 times/],
  ['admin-settings-help-webhooks-timeout-secs', /10 seconds/],
  ['admin-restrictions-intro', /permanent deletion of messages and deletion of folders are refused/],
  ['yd-banner', /permanently deleting messages and deleting folders is refused/],
  ['res-recovery-help', /marked recovery verified/],
  ['admin-geoip-refusing', /unless the sign-in policy without a database is set to allow/],
  ['admin-dash-warn-geoip-missing', /unless the sign-in policy without a database is set to allow/],
  ['admin-settings-help-security-signin-geo-scope', /unless the sign-in policy without a database is set to allow/],
  ['admin-session-batch-confirm', /^admin-session-batch-confirm = Revoke only the selected sessions\. Failed/],
  ['admin-policy-group-sign-in-help', /upgraded at its next successful sign-in/],
  ['admin-settings-help-security-admin-probe-audit-ip', /only when IP addresses in logs is set to Full/],
  ['admin-protect-not-country', /Recognized scan paths mode is Enforce \(whatever the adaptive mode is\)/],
  ['admin-protect-summary-step-detail', /whatever the adaptive mode is/],
  ['admin-protect-rule-signals-help', /adaptive mode when their own mode is Enforce/],
  ['dr-reauth', /confirm your identity/],
  ['admin-settings-help-dsr-reauth-required', /password and second factor, or a passkey/],
  ['admin-settings-help-security-session-binding-webmail-action', /confirm their identity again/],
  ['session-alerts-action-step-up', /confirm your identity/],
];

describe('accurate console wording', () => {
  it.each(CASES)('%s', (id, pattern) => {
    expect(messageOf(id)).toMatch(pattern);
  });
});
