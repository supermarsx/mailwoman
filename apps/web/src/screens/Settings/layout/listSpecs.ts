// Search-index entries for list-style cards (identities, signatures): their
// content is data rows, so only the card itself is searchable.
import type { CardSpec } from './rows.tsx';

export const LIST_CARDS: readonly CardSpec[] = [
  { id: 'identities-card', section: 'identities', title: 'settings-ident-title', help: 'settings-ident-intro', rows: [] },
  { id: 'signatures-card', section: 'signatures', title: 'settings-sig-title', help: 'settings-sig-intro', rows: [] },
];

/** The Security section's cards (factors, geography, alerts, sessions, history, devices, activity). */
export const SECURITY_SECTION_CARDS: readonly CardSpec[] = [
  { id: 'twofa-card-overview', section: 'security', title: 'settings-2fa-title', help: 'settings-2fa-intro', rows: [] },
  { id: 'twofa-card-totp', section: 'security', title: 'settings-2fa-totp-title', rows: [] },
  { id: 'twofa-card-passkeys', section: 'security', title: 'settings-2fa-passkey-title', rows: [] },
  { id: 'signin-geo-card', section: 'security', title: 'signin-geo-title', help: 'signin-geo-help', rows: [{ id: 'signin-geo-mode', label: 'signin-geo-mode' }, { id: 'signin-geo-countries', label: 'signin-geo-countries' }, { id: 'signin-geo-ips', label: 'signin-geo-ips' }] },
  { id: 'session-alerts-card', section: 'security', title: 'session-alerts-title', help: 'session-alerts-help', rows: [] },
  { id: 'session-alerts-prefs-card', section: 'security', title: 'session-alerts-prefs', rows: [{ id: 'session-alerts-country', label: 'session-alerts-country', help: 'session-alerts-country-help' }, { id: 'session-alerts-channels', label: 'session-alerts-channels' }, { id: 'session-alerts-sensitivity', label: 'session-alerts-sensitivity' }] },
  { id: 'sessions-card', section: 'security', title: 'settings-sessions-title', help: 'settings-sessions-intro', rows: [] },
  { id: 'security-history-card', section: 'security', title: 'security-center-history', rows: [{ id: 'security-history-result', label: 'security-center-result' }] },
  { id: 'security-devices-card', section: 'security', title: 'security-center-devices', help: 'security-center-remember-help', rows: [] },
  { id: 'security-activity-card', section: 'security', title: 'security-center-activity', rows: [] },
];
