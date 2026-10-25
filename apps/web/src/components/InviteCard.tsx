import { Show, createResource, createSignal, onMount, type JSX } from 'solid-js';
import type { Client } from '../api/client.ts';
import { CAP_CORE, type Email } from '../api/jmap-types.ts';
import { CAP_CALENDARS } from '../api/pim-types.ts';
import { createConfiguredClient } from '../api/transport.ts';
import { pimResponse, PimMethodError } from '../modules/calendar/api.ts';
import { isolate, loadCatalog, t } from '../i18n';
import { toast } from './toast/index.ts';
import * as css from './invite-card.css.ts';

type Outcome = 'new' | 'update' | 'current' | 'stale' | 'cancel' | 'reply' |
  'notOrganizer' | 'notAttendee' | 'unknownEvent' | 'unsupported';

export interface Invitation {
  accountId: string;
  emailId: string;
  method: string;
  sender: string;
  event: { title?: string };
  outcome: Outcome;
  applied: boolean;
}

/** Calendar alternatives and attached invitations both use the server's MIME type. */
export function hasCalendarPart(email: Email): boolean {
  const parts = email as Email & { attachments?: Array<{ type?: string }> };
  return [...(parts.attachments ?? []), ...(email.textBody ?? []), ...(email.htmlBody ?? [])]
    .some((part) => part.type?.split(';', 1)[0]?.trim().toLowerCase() === 'text/calendar');
}

const ACTIONS: Partial<Record<Outcome, string>> = {
  new: 'calendar-invite-add',
  update: 'calendar-invite-update',
  cancel: 'calendar-invite-cancel',
  reply: 'calendar-invite-reply',
};
const STATUS: Record<Outcome, string> = {
  new: 'calendar-invite-new', update: 'calendar-invite-newer', current: 'calendar-invite-current',
  stale: 'calendar-invite-stale', cancel: 'calendar-invite-cancelled', reply: 'calendar-invite-response',
  notOrganizer: 'calendar-invite-not-organizer', notAttendee: 'calendar-invite-not-attendee',
  unknownEvent: 'calendar-invite-unknown', unsupported: 'calendar-invite-unsupported',
};

/** Opening a message only previews it; the server checks the sender and freshness again on apply. */
export function InviteCard(props: { accountId: string; emailId: string; client?: Pick<Client, 'jmap'> }): JSX.Element {
  onMount(() => void loadCatalog('calendar'));
  const client = props.client ?? createConfiguredClient();
  const [busy, setBusy] = createSignal(false);
  async function read(accountId: string, emailId: string, apply: boolean): Promise<Invitation | null> {
    try {
      return pimResponse<Invitation>(await client.jmap({
        using: [CAP_CORE, CAP_CALENDARS],
        methodCalls: [['CalendarEvent/itip', { accountId, emailId, apply }, 'invite']],
      }), 'invite');
    } catch (error) {
      // Plain calendar files have no iTIP METHOD; proxy servers may not support the method.
      if (!apply && error instanceof PimMethodError &&
        ['invalidArguments', 'unknownMethod', 'notFound'].includes(error.type)) return null;
      throw error;
    }
  }
  const [invitation, { mutate, refetch }] = createResource(
    () => [props.accountId, props.emailId] as const,
    ([accountId, emailId]) => read(accountId, emailId, false),
  );
  const ready = (): Invitation | null => {
    if (invitation.loading || invitation.error) return null;
    return invitation() ?? null;
  };
  async function apply(): Promise<void> {
    const current = ready();
    if (!current || current.applied || !ACTIONS[current.outcome] || busy()) return;
    const { accountId, emailId } = props;
    setBusy(true);
    try {
      const result = await read(accountId, emailId, true);
      if (accountId === props.accountId && emailId === props.emailId) mutate(result);
      if (result?.applied) toast.success(t('calendar-invite-applied'));
      else toast.info(t('calendar-invite-unchanged'));
    } catch {
      toast.error(t('calendar-invite-apply-failed'));
    } finally {
      setBusy(false);
    }
  }
  return (
    <Show when={invitation.error || ready()}>
      <section class={css.card} aria-label={t('calendar-invite-title')}>
        <h3>{t('calendar-invite-title')}</h3>
        <Show when={invitation.error}>
          <p role="alert">{t('calendar-invite-load-failed')}</p>
          <button type="button" class="btn btn--ghost" onClick={() => void refetch()}>{t('calendar-invite-retry')}</button>
        </Show>
        <Show when={ready()}>{(invite) => (
          <>
            <p dir="auto">{invite().event.title || t('calendar-untitled')}</p>
            <p>{t('calendar-invite-sender', { sender: isolate(invite().sender) })}</p>
            <p>{t(invite().applied ? 'calendar-invite-applied' : STATUS[invite().outcome])}</p>
            <Show when={!invite().applied && ACTIONS[invite().outcome]}>{(action) => (
              <>
                <p>{t('calendar-invite-no-reply')}</p>
                <button type="button" class="btn btn--primary" disabled={busy()} onClick={() => void apply()}>{t(action())}</button>
              </>
            )}</Show>
          </>
        )}</Show>
      </section>
    </Show>
  );
}
