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

/** Device preferences shown in the Keyboard and display section (presets, offline cache, direction). */
export const DEVICE_PREFERENCE_CARDS: readonly CardSpec[] = [
  { id: 'prefs-card-keyboard', section: 'shortcuts', title: 'settings-kbd-preset-title', help: 'settings-kbd-intro', rows: [{ id: 'prefs-keyboard-preset', label: 'settings-kbd-preset-title' }] },
  { id: 'prefs-card-offline', section: 'shortcuts', title: 'settings-offline-title', help: 'settings-offline-intro', rows: [{ id: 'prefs-offline-budget', label: 'settings-offline-budget-label' }, { id: 'prefs-offline-retention', label: 'settings-offline-retention-label' }, { id: 'prefs-offline-strategy', label: 'settings-offline-strategy-label' }] },
  { id: 'prefs-card-direction', section: 'shortcuts', title: 'settings-dir-title', help: 'settings-dir-intro', rows: [{ id: 'prefs-direction', label: 'settings-dir-title' }] },
];

/** Keyboard shortcuts table card (the rows are data: one per action). */
export const SHORTCUT_CARDS: readonly CardSpec[] = [
  { id: 'shortcuts-card-table', section: 'shortcuts', title: 'settings-kbd-title', help: 'settings-shortcuts-help', rows: [{ id: 'shortcuts-enabled', label: 'settings-shortcuts-enabled' }] },
];

/** Privacy & data section: remote content, trusted links, analytics, load timing, cookies, Your data and requests. */
export const PRIVACY_CARDS: readonly CardSpec[] = [
  { id: 'privacy-card-remote', section: 'privacy', title: 'privacy-title', help: 'privacy-help', rows: [
    { id: 'privacy-mode', label: 'privacy-remote-mode' }, { id: 'privacy-trackers', label: 'privacy-trackers' }, { id: 'privacy-clean-links', label: 'privacy-clean-links' },
    { id: 'privacy-image-downloads', label: 'privacy-image-downloads' }, { id: 'privacy-image-toast', label: 'privacy-image-toast' },
    { id: 'privacy-strip-mailer-headers', label: 'privacy-strip-mailer-headers', help: 'privacy-strip-mailer-help' },
    { id: 'privacy-image-retries', label: 'privacy-image-retry-attempts', help: 'privacy-image-retry-help' }, { id: 'privacy-image-retry-delay', label: 'privacy-image-retry-delay' },
    { id: 'privacy-unsafe-html', label: 'privacy-unsafe-html', help: 'privacy-unsafe-html-help' }, { id: 'privacy-tracker-choice', label: 'privacy-tracker-choice', help: 'privacy-tracker-choice-help' },
    { id: 'privacy-original-html', label: 'privacy-original-html-allow', help: 'privacy-original-html-help' },
  ] },
  { id: 'privacy-card-trusted-links', section: 'privacy', title: 'privacy-trusted-links-title', help: 'privacy-trusted-links-help', rows: [
    { id: 'trusted-link-domains', label: 'privacy-trusted-links-domains', help: 'privacy-trusted-links-domains-help' }, { id: 'trusted-link-subdomains', label: 'privacy-trusted-links-subdomains', help: 'privacy-trusted-links-subdomains-help' },
  ] },
  { id: 'privacy-card-trusted-senders', section: 'privacy', title: 'privacy-trusted-link-senders-title', help: 'privacy-trusted-link-senders-help', rows: [
    { id: 'trusted-link-senders', label: 'privacy-trusted-link-senders-list', help: 'privacy-trusted-link-senders-list-help' },
  ] },
  { id: 'privacy-card-analytics', section: 'privacy', title: 'privacy-analytics-title', help: 'privacy-analytics-help', rows: [{ id: 'analytics-consent', label: 'privacy-analytics-consent' }] },
  { id: 'privacy-card-load-time', section: 'privacy', title: 'privacy-load-time-title', help: 'privacy-load-time-help', rows: [
    { id: 'load-time-folder-control', label: 'privacy-load-time-folder', help: 'privacy-load-time-folder-help' }, { id: 'load-time-message-control', label: 'privacy-load-time-message', help: 'privacy-load-time-message-help' },
  ] },
  { id: 'privacy-card-cookies', section: 'privacy', title: 'cookies-settings-title', help: 'cookies-settings-help', rows: [] },
  { id: 'your-data-card-overview', section: 'privacy', title: 'yd-title', rows: [] },
  { id: 'your-data-card-consents', section: 'privacy', title: 'yd-consents-title', help: 'yd-consents-help', rows: [] },
  { id: 'your-data-card-art22', section: 'privacy', title: 'yd-art22-title', rows: [] },
  { id: 'your-data-card-sessions', section: 'privacy', title: 'yd-sessions-title', rows: [] },
  { id: 'your-data-card-keys', section: 'privacy', title: 'yd-keys-title', rows: [] },
  { id: 'your-data-card-requests', section: 'privacy', title: 'yd-shortcuts-title', help: 'yd-prefs-pointer', rows: [] },
  { id: 'your-data-requests', section: 'privacy', title: 'dr-title', rows: [
    { id: 'dr-kind', label: 'dr-kind' }, { id: 'dr-include-mail', label: 'dr-include-mail' }, { id: 'dr-details', label: 'dr-details' }, { id: 'dr-erasure-sure', label: 'dr-erasure-sure' },
  ] },
  { id: 'data-requests-card-list', section: 'privacy', title: 'dr-list', rows: [] },
];
