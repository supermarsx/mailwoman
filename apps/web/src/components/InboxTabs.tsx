import { Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import * as css from './InboxTabs.css.ts';
import { folderCountMode, listCount } from '../api/countBadges.ts';
import type { Email } from '../api/jmap-types.ts';

// The Focused / Other tabs above the Inbox list (plan §1.5, t73). The Focused
// inbox is the user preference `mail.focusedInbox`, off by default and turned on
// or off in Settings > Folders and sidebar (an administrator can set the default
// or lock it). While it is off, or another folder or a search is shown, this
// renders nothing; while it is on, one slim segmented pair with counts, and
// `app.listMessages()` resolves to the split chosen by `app.inboxTab()`.
//
// There is no Unified-inbox control: a session holds one account, so there is
// nothing to unify.

export function InboxTabs(): JSX.Element {
  const app = useApp();
  // The tab counts follow the folder count setting (t39): hidden, unread only
  // (optionally only while there is any), or the number of messages.
  const count = (messages: readonly Email[]): number | null =>
    listCount(folderCountMode(), messages.length, messages.filter((m) => m.keywords?.['$seen'] !== true).length);

  const tab = (id: 'focused' | 'other', label: string, messages: () => readonly Email[]): JSX.Element => (
    <button
      type="button"
      role="tab"
      class={`${css.tab} inbox-tabs__tab ${a11y.focusable}`}
      classList={{ 'inbox-tabs__tab--active': app.inboxTab() === id }}
      aria-selected={app.inboxTab() === id}
      tabIndex={app.inboxTab() === id ? 0 : -1}
      data-testid={`inbox-tab-${id}`}
      onClick={() => app.setInboxTab(id)}
      onKeyDown={(e) => {
        if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return;
        e.preventDefault();
        const next = id === 'focused' ? 'other' : 'focused';
        app.setInboxTab(next);
        (e.currentTarget.parentElement?.querySelector(`[data-testid="inbox-tab-${next}"]`) as HTMLElement | null)?.focus();
      }}
    >
      {label}
      <Show when={count(messages())}>{(n) => <span class={`${css.count} inbox-tabs__count`}>{n()}</span>}</Show>
    </button>
  );

  return (
    <Show when={app.focusedInboxShown()}>
      <div class={`${css.row} inbox-tabs`} role="tablist" aria-label={t('mail-inbox-filter')} data-testid="inbox-tabs">
        {tab('focused', t('mail-inbox-focused'), app.focusedMessages)}
        {tab('other', t('mail-inbox-other'), app.otherMessages)}
      </div>
    </Show>
  );
}
