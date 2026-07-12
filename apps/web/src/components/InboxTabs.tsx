import { Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';

// The inbox header controls (plan §1.5): the opt-in rules-based Focused inbox
// (two tabs, Focused / Other) and the Unified-inbox toggle. Focused mode is off
// by default so the list shows every message; when on, `app.listMessages()`
// resolves to the focused/other split by `app.inboxTab()`.

export function InboxTabs(): JSX.Element {
  const app = useApp();

  return (
    <div class="inbox-tabs">
      <Show
        when={app.focusedInbox()}
        fallback={
          <button
            type="button"
            class="btn btn--ghost inbox-tabs__enable"
            onClick={() => app.setFocusedInbox(true)}
          >
            Focused inbox
          </button>
        }
      >
        <div class="inbox-tabs__row" role="tablist" aria-label="Inbox filter">
          <button
            type="button"
            role="tab"
            class="inbox-tabs__tab"
            classList={{ 'inbox-tabs__tab--active': app.inboxTab() === 'focused' }}
            aria-selected={app.inboxTab() === 'focused'}
            onClick={() => app.setInboxTab('focused')}
          >
            Focused
            <span class="inbox-tabs__count">{app.focusedMessages().length}</span>
          </button>
          <button
            type="button"
            role="tab"
            class="inbox-tabs__tab"
            classList={{ 'inbox-tabs__tab--active': app.inboxTab() === 'other' }}
            aria-selected={app.inboxTab() === 'other'}
            onClick={() => app.setInboxTab('other')}
          >
            Other
            <span class="inbox-tabs__count">{app.otherMessages().length}</span>
          </button>
          <button
            type="button"
            class="btn btn--ghost inbox-tabs__enable"
            onClick={() => app.setFocusedInbox(false)}
          >
            Turn off
          </button>
        </div>
      </Show>

      <label class="inbox-tabs__unified">
        <input
          type="checkbox"
          checked={app.unifiedInbox()}
          onChange={(e) => app.setUnifiedInbox(e.currentTarget.checked)}
        />
        Unified inbox
      </label>
    </div>
  );
}
