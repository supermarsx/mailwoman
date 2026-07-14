import { Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';

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
            <span class="inbox-tabs__count">{app.focusedMessages().length}</span>
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
            <span class="inbox-tabs__count">{app.otherMessages().length}</span>
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

      <label class="inbox-tabs__unified">
        <input
          type="checkbox"
          checked={app.unifiedInbox()}
          onChange={(e) => app.setUnifiedInbox(e.currentTarget.checked)}
        />
        {t('mail-inbox-unified')}
      </label>
    </div>
  );
}
