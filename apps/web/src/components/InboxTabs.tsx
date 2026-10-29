import { Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import { folderCountMode, listCount } from '../api/countBadges.ts';
import type { Email } from '../api/jmap-types.ts';

// The inbox header control (plan §1.5): the opt-in rules-based Focused inbox
// (two tabs, Focused / Other). Focused mode is off by default so the list shows
// every message; when on, `app.listMessages()` resolves to the focused/other
// split by `app.inboxTab()`.
//
// There is no Unified-inbox control: a session holds one account, so there is
// nothing to unify. The checkbox that used to be here set a flag nothing read.

export function InboxTabs(): JSX.Element {
  const app = useApp();
  // The tab counts follow the folder count setting (t39): hidden, unread only
  // (optionally only while there is any), or the number of messages.
  const count = (messages: readonly Email[]): number | null =>
    listCount(folderCountMode(), messages.length, messages.filter((m) => m.keywords?.['$seen'] !== true).length);

  return (
    <div class="inbox-tabs">
      <Show
        when={app.focusedInbox()}
        fallback={
          <button
            type="button"
            class={`btn btn--ghost inbox-tabs__enable ${a11y.focusable}`}
            onClick={() => app.setFocusedInbox(true)}
          >
            {t('mail-inbox-focused-enable')}
          </button>
        }
      >
        <div class="inbox-tabs__row" role="tablist" aria-label={t('mail-inbox-filter')}>
          <button
            type="button"
            role="tab"
            class={`inbox-tabs__tab ${a11y.focusable}`}
            classList={{ 'inbox-tabs__tab--active': app.inboxTab() === 'focused' }}
            aria-selected={app.inboxTab() === 'focused'}
            onClick={() => app.setInboxTab('focused')}
          >
            {t('mail-inbox-focused')}
            <Show when={count(app.focusedMessages())}>{(n) => <span class="inbox-tabs__count">{n()}</span>}</Show>
          </button>
          <button
            type="button"
            role="tab"
            class={`inbox-tabs__tab ${a11y.focusable}`}
            classList={{ 'inbox-tabs__tab--active': app.inboxTab() === 'other' }}
            aria-selected={app.inboxTab() === 'other'}
            onClick={() => app.setInboxTab('other')}
          >
            {t('mail-inbox-other')}
            <Show when={count(app.otherMessages())}>{(n) => <span class="inbox-tabs__count">{n()}</span>}</Show>
          </button>
          <button
            type="button"
            class={`btn btn--ghost inbox-tabs__enable ${a11y.focusable}`}
            onClick={() => app.setFocusedInbox(false)}
          >
            {t('mail-inbox-turn-off')}
          </button>
        </div>
      </Show>
    </div>
  );
}
