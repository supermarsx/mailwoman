// Shared test harness for e7 component tests: a programmable fake JMAP client and
// a render-with-`AppContext` helper. Not a spec itself (excluded by the test
// glob); imported by the *.test.tsx files in this directory.

import { vi } from 'vitest';
import { render } from '@solidjs/testing-library';
import type { JSX } from 'solid-js';
import { AppContext } from '../state/context.ts';
import { createAppState, type AppState } from '../state/store.ts';
import type { Client, Me } from '../api/client.ts';
import {
  CAP_MAIL,
  type Email,
  type Identity,
  type JmapRequest,
  type JmapResponse,
  type JmapSession,
  type Mailbox,
} from '../api/jmap-types.ts';

export const MAILBOXES: Mailbox[] = [
  { id: 'inbox', name: 'Inbox', parentId: null, role: 'inbox', sortOrder: 0, totalEmails: 0, unreadEmails: 0 },
  { id: 'archive', name: 'Archive', parentId: null, role: 'archive', sortOrder: 1, totalEmails: 0, unreadEmails: 0 },
  { id: 'trash', name: 'Trash', parentId: null, role: 'trash', sortOrder: 2, totalEmails: 0, unreadEmails: 0 },
];

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_MAIL]: 'acct1' },
  username: 'me@example.org',
  apiUrl: '/a', downloadUrl: '/d', uploadUrl: '/u', eventSourceUrl: '/e', state: 's0',
};

export function mkEmail(id: string, over: Partial<Email> = {}): Email {
  return {
    id,
    mailboxIds: { inbox: true },
    from: [{ name: null, email: `${id}@example.org` }],
    to: [{ name: null, email: 'me@example.org' }],
    subject: `Subject ${id}`,
    receivedAt: '2026-01-01T00:00:00Z',
    preview: `preview ${id}`,
    keywords: { $seen: true },
    ...over,
  };
}

export interface HarnessOpts {
  emails?: Email[];
  identities?: Identity[];
}

export function makeClient(opts: HarnessOpts = {}): Client {
  const emails = opts.emails ?? [];
  const identities = opts.identities ?? [];
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const names = body.methodCalls.map((c) => c[0]);
    if (names.includes('Mailbox/get')) {
      return { methodResponses: [['Mailbox/get', { accountId: 'acct1', state: 's', list: MAILBOXES, notFound: [] }, 'c0']], sessionState: 's' };
    }
    if (names.includes('Identity/get')) {
      return { methodResponses: [['Identity/get', { accountId: 'acct1', state: 's', list: identities, notFound: [] }, 'i']], sessionState: 's' };
    }
    if (names.includes('EmailSubmission/query')) {
      return {
        methodResponses: [
          ['EmailSubmission/query', { accountId: 'acct1', ids: [] }, 'q'],
          ['EmailSubmission/get', { accountId: 'acct1', state: 's', list: [], notFound: [] }, 'g'],
        ],
        sessionState: 's',
      };
    }
    if (names.includes('EmailSubmission/set')) {
      return {
        methodResponses: [
          ['Email/set', { accountId: 'acct1', created: { draft: { id: 'draft1' } }, notCreated: null }, 'set'],
          ['EmailSubmission/set', { accountId: 'acct1', created: { send: { id: 'sub1' } }, notCreated: null }, 'submit'],
        ],
        sessionState: 's',
      };
    }
    if (names.includes('Email/get')) {
      return {
        methodResponses: [
          ['Email/query', { accountId: 'acct1', ids: emails.map((e) => e.id) }, 'q'],
          ['Email/get', { accountId: 'acct1', state: 's', list: emails, notFound: [] }, 'g'],
        ],
        sessionState: 's',
      };
    }
    return { methodResponses: body.methodCalls.map((c) => [c[0], {}, c[2]] as JmapResponse['methodResponses'][number]), sessionState: 's' };
  });
  return {
    login: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    logout: vi.fn(async () => undefined),
    me: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    session: vi.fn(async () => SESSION),
    jmap,
    sanitize: vi.fn(async (h: string) => h),
    onNetwork: vi.fn(() => () => undefined),
  };
}

/** Render `ui` inside a live `AppState`; returns the RTL result plus the state. */
export function renderWithApp(ui: () => JSX.Element, opts: HarnessOpts = {}): {
  app: AppState;
  result: ReturnType<typeof render>;
} {
  const app = createAppState(makeClient(opts));
  const result = render(() => <AppContext.Provider value={app}>{ui()}</AppContext.Provider>);
  return { app, result };
}
