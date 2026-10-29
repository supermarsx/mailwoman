// The Events view (t78): a sidebar entry under the folders, beside Attachments
// and Outbox, that lists every message of the account with calendar content
// (invitations, updates, cancellations, replies, counter-proposals and .ics
// files) in the ordinary message list. The server does the filtering:
// `Email/query { inMailbox: EVENTS_VIEW_ID }` runs over the search index's
// per-message calendar flag, so the list pages like any folder, opens messages
// under the same reading-pane and tab rules (t74), and shows each row's event
// hint (EventHint.tsx). The badge follows the folder count setting (t39).

import { createEffect, on, onCleanup, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t } from '../i18n/index.ts';
import { EVENTS_VIEW_ID } from '../api/jmap.ts';
import { FolderCountBadge } from './FolderCountBadge.tsx';
import { Icon } from './Icon.tsx';
import * as a11y from './mailA11y.css.ts';
import * as css from './eventsView.css.ts';

/** The sidebar entry. `active` is whether the mail surface shows the view;
 *  `onSelect` opens it the way a folder row does. */
export function EventsNavEntry(props: { active: boolean; onSelect: () => void }): JSX.Element {
  const app = useApp();
  // Recount whenever the folders' counts move (a new message, a read, a move):
  // they change on the same events this view's counts do. Debounced, so a burst
  // of folder updates costs one request.
  let timer: ReturnType<typeof setTimeout> | undefined;
  createEffect(on([app.mailboxes, app.accountId], ([, account]) => {
    if (timer !== undefined) clearTimeout(timer);
    if (account === null) return;
    timer = setTimeout(() => { timer = undefined; void app.refreshEventsCounts(); }, 250);
  }));
  onCleanup(() => { if (timer !== undefined) clearTimeout(timer); });
  return (
    <button
      type="button"
      class={`sidebar__box ${a11y.focusable}`}
      classList={{ 'sidebar__box--active': props.active }}
      aria-label={t('mail-nav-events')}
      title={t('mail-nav-events')}
      aria-current={props.active ? 'true' : undefined}
      data-testid="sidebar-events"
      onClick={() => props.onSelect()}
    >
      <Icon name="calendar" /><span class="sidebar__box-name">{t('mail-nav-events')}</span>
      <Show when={app.eventsCounts()}>{(counts) => <FolderCountBadge folder={{ role: null, unreadEmails: counts().unread, totalEmails: counts().total }} />}</Show>
    </button>
  );
}

/** The filter row above the list while the Events view is open: "Upcoming only". */
export function EventsViewBar(): JSX.Element {
  const app = useApp();
  return (
    <Show when={app.selectedMailboxId() === EVENTS_VIEW_ID && !app.searchActive()}>
      <div class={css.row} role="group" aria-label={t('mail-events-filter')} data-testid="events-view-bar">
        <button
          type="button"
          class={`${css.chip} ${a11y.focusable}`}
          aria-pressed={app.eventsUpcoming()}
          data-testid="events-upcoming"
          onClick={() => void app.setEventsUpcoming(!app.eventsUpcoming())}
        >
          <Icon name="calendar" />{t('mail-events-upcoming')}
        </button>
      </div>
    </Show>
  );
}
