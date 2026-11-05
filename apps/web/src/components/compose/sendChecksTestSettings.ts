import { loadSettings } from '../../api/settings.ts';

/** The pre-send checks turned off, for tests that send without a subject or with odd recipients. */
export const SEND_CHECKS_OFF = {
  'mail.compose.checkInvalidRecipients': { value: 'allow', source: 'user', locked: false },
  'mail.compose.checkMissingAttachment': { value: 'allow', source: 'user', locked: false },
  'mail.compose.checkEmptySubject': { value: 'allow', source: 'user', locked: false },
  'mail.externalRecipientWarning': { value: 'off', source: 'user', locked: false },
} as const;

export async function allowSendChecks(extra: Record<string, unknown> = {}): Promise<void> {
  await loadSettings({ refresh: true, fetcher: async () => new Response(JSON.stringify({ settings: { ...SEND_CHECKS_OFF, ...extra } }), { status: 200, headers: { 'content-type': 'application/json' } }) });
}
