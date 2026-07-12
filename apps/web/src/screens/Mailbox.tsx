import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { useRealtime } from '../realtime/context.ts';
import { MessageList } from '../components/MessageList.tsx';
import { Reader } from '../components/Reader.tsx';
import { Compose } from '../components/Compose.tsx';
import { Outbox } from '../components/Outbox.tsx';
import { InboxTabs } from '../components/InboxTabs.tsx';
import { UndoToast } from '../components/UndoToast.tsx';
import { SubTabStrip } from '../components/SubTabStrip.tsx';
import { Ribbon } from '../components/Ribbon.tsx';
import { Settings } from './Settings.tsx';
import { Attachments } from './Attachments.tsx';

type View = 'mail' | 'outbox' | 'attachments';

/** The search box above the message list; submits an `Email/query` (engine →
 *  mw-search online, reduced cached search offline). */
function SearchBox(): JSX.Element {
  const app = useApp();
  const [query, setQuery] = createSignal(app.search());

  return (
    <form
      class="mail-search"
      role="search"
      onSubmit={(e) => {
        e.preventDefault();
        void app.searchMessages(query());
      }}
    >
      <input
        class="mail-search__input"
        type="search"
        aria-label="Search mail"
        placeholder="Search mail — from:alice subject:invoice larger:1mb"
        value={query()}
        onInput={(e) => setQuery(e.currentTarget.value)}
      />
      <button type="submit" class="btn btn--ghost mail-search__submit">
        Search
      </button>
      <Show when={app.searchActive()}>
        <button
          type="button"
          class="btn btn--ghost mail-search__clear"
          onClick={() => {
            setQuery('');
            void app.clearSearch();
          }}
        >
          Clear
        </button>
      </Show>
    </form>
  );
}

export function MailboxScreen(): JSX.Element {
  const app = useApp();
  const { subTabs } = useRealtime();
  const [composing, setComposing] = createSignal(false);
  const [settingsOpen, setSettingsOpen] = createSignal(false);
  const [view, setView] = createSignal<View>('mail');

  // Seed a single "messages" sub-tab so the multi-surface strip is live.
  onMount(() => {
    if (subTabs.tabs().length === 0) {
      subTabs.open({ kind: 'messages', title: 'Mail', id: 'mail', pinned: true });
    }
  });

  return (
    <div class="shell">
      <Show when={app.layout() === 'ribbon'}>
        <Ribbon onCompose={() => setComposing(true)} onOpenSettings={() => setSettingsOpen(true)} />
      </Show>
      <aside class="sidebar">
        <div class="sidebar__head">
          <span class="sidebar__brand">Mailwoman</span>
          <Show when={app.me()}>{(m) => <span class="sidebar__user">{m().username}</span>}</Show>
          <button
            type="button"
            class="btn btn--ghost sidebar__settings"
            aria-label="Settings"
            onClick={() => setSettingsOpen(true)}
          >
            ⚙
          </button>
        </div>
        <button type="button" class="btn btn--primary sidebar__compose" onClick={() => setComposing(true)}>
          Compose
        </button>
        <nav class="sidebar__nav" aria-label="Mailboxes">
          <For each={app.mailboxes()}>
            {(box) => (
              <button
                type="button"
                class="sidebar__box"
                classList={{ 'sidebar__box--active': view() === 'mail' && app.selectedMailboxId() === box.id }}
                onClick={() => {
                  setView('mail');
                  void app.selectMailbox(box.id);
                }}
              >
                <span class="sidebar__box-name">{box.name}</span>
                <Show when={box.unreadEmails > 0}>
                  <span class="sidebar__badge">{box.unreadEmails}</span>
                </Show>
              </button>
            )}
          </For>
          <button
            type="button"
            class="sidebar__box"
            classList={{ 'sidebar__box--active': view() === 'attachments' }}
            onClick={() => setView('attachments')}
          >
            <span class="sidebar__box-name">Attachments</span>
          </button>
          <button
            type="button"
            class="sidebar__box"
            classList={{ 'sidebar__box--active': view() === 'outbox' }}
            onClick={() => {
              setView('outbox');
              void app.refreshOutbox();
            }}
          >
            <span class="sidebar__box-name">Outbox</span>
            <Show when={app.cancelableOutbox().length > 0}>
              <span class="sidebar__badge">{app.cancelableOutbox().length}</span>
            </Show>
          </button>
        </nav>
        <button type="button" class="btn btn--ghost sidebar__logout" onClick={() => void app.logout()}>
          Log out
        </button>
        <Show when={!app.online()}>
          <span class="sidebar__offline" aria-live="polite">
            Offline
          </span>
        </Show>
      </aside>

      <Show when={view() === 'mail'}>
        <div class="mail-pane">
          <SubTabStrip />
          <SearchBox />
          <InboxTabs />
          <MessageList />
        </div>
        <Reader />
      </Show>
      <Show when={view() === 'outbox'}>
        <Outbox />
      </Show>
      <Show when={view() === 'attachments'}>
        <Attachments
          load={() => app.listAttachments()}
          onOpen={(item) => {
            setView('mail');
            void app.openMessage(item.emailId);
          }}
        />
      </Show>

      <Show when={composing()}>
        <Compose onClose={() => setComposing(false)} />
      </Show>
      <Show when={settingsOpen()}>
        <Settings onClose={() => setSettingsOpen(false)} />
      </Show>
      <UndoToast />
    </div>
  );
}
