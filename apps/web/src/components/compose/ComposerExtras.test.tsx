import { describe, it, expect } from 'vitest';
import { createSignal } from 'solid-js';
import { render, screen, fireEvent, within } from '@solidjs/testing-library';
import {
  SignaturePicker,
  SendOptions,
  RecallPanel,
  DraftsDrawer,
  DEFAULT_SEND_OPTIONS,
  type ComposeSignature,
  type SendOptionsState,
} from './ComposerExtras.tsx';
import type { EmailSubmission } from '../../api/jmap-types.ts';
import type { StoredDraft } from './drafts-store.ts';

const SIGS: ComposeSignature[] = [
  { id: 's1', name: 'Personal', text: 'Cheers', html: '<p>Cheers</p>' },
  { id: 's2', name: 'Work', text: 'Regards', html: null },
];

describe('SignaturePicker (W12)', () => {
  it('inserts the chosen signature', () => {
    let picked: ComposeSignature | null = null;
    render(() => <SignaturePicker signatures={() => SIGS} onInsert={(s) => (picked = s)} />);
    fireEvent.change(screen.getByTestId('compose-signature').querySelector('select')!, {
      target: { value: 's2' },
    });
    expect(picked).not.toBeNull();
    expect(picked!.name).toBe('Work');
  });

  it('renders nothing when there are no signatures', () => {
    render(() => <SignaturePicker signatures={() => []} onInsert={() => undefined} />);
    expect(screen.queryByTestId('compose-signature')).toBeNull();
  });
});

describe('SendOptions (W11)', () => {
  it('toggles the read-receipt and tracking-pixel flags', () => {
    const [state, setState] = createSignal<SendOptionsState>(DEFAULT_SEND_OPTIONS);
    render(() => <SendOptions state={state} onChange={setState} />);
    fireEvent.click(screen.getByTestId('opt-receipt'));
    expect(state().requestReceipt).toBe(true);
    fireEvent.click(screen.getByTestId('opt-tracking'));
    expect(state().trackingPixel).toBe(true);
    // Both are off by default.
    expect(DEFAULT_SEND_OPTIONS).toEqual({ requestReceipt: false, trackingPixel: false });
  });
});

describe('RecallPanel (W10)', () => {
  const sub = (id: string, over: Partial<EmailSubmission> = {}): EmailSubmission => ({
    id,
    emailId: 'e',
    identityId: null,
    sendAt: null,
    undoStatus: 'pending',
    mailwomanHoldSeconds: 10,
    ...over,
  });

  it('lists cancelable submissions and recalls one', () => {
    let recalled: string | null = null;
    render(() => (
      <RecallPanel submissions={() => [sub('a'), sub('b')]} onRecall={(id) => (recalled = id)} />
    ));
    fireEvent.click(screen.getByTestId('recall-a'));
    expect(recalled).toBe('a');
  });

  it('renders nothing when there is nothing to recall', () => {
    render(() => <RecallPanel submissions={() => []} onRecall={() => undefined} />);
    expect(screen.queryByTestId('compose-recall')).toBeNull();
  });
});

describe('DraftsDrawer (W9)', () => {
  const d = (id: string, over: Partial<StoredDraft> = {}): StoredDraft => ({
    id,
    to: 'a@b.c',
    subject: `subject-${id}`,
    bodyHtml: '<p>h</p>',
    bodyText: 'h',
    savedAt: 1,
    ...over,
  });

  it('lists drafts and resumes / deletes them', () => {
    let resumed: string | null = null;
    let deleted: string | null = null;
    render(() => (
      <DraftsDrawer
        open={true}
        drafts={() => [d('d1'), d('d2')]}
        onResume={(x) => (resumed = x.id)}
        onDelete={(id) => (deleted = id)}
        onClose={() => undefined}
      />
    ));
    const drawer = screen.getByTestId('compose-drafts');
    // Subjects render through bidi `isolate()`, so match on a substring.
    expect(within(drawer).getByText(/subject-d1/)).toBeInTheDocument();
    fireEvent.click(screen.getByTestId('draft-resume-d2'));
    expect(resumed).toBe('d2');
    fireEvent.click(screen.getByTestId('draft-delete-d1'));
    expect(deleted).toBe('d1');
  });

  it('shows an empty state and hides when closed', () => {
    const [open, setOpen] = createSignal(true);
    render(() => (
      <DraftsDrawer
        open={open()}
        drafts={() => []}
        onResume={() => undefined}
        onDelete={() => undefined}
        onClose={() => undefined}
      />
    ));
    expect(screen.getByTestId('compose-drafts')).toBeInTheDocument();
    setOpen(false);
    expect(screen.queryByTestId('compose-drafts')).toBeNull();
  });
});
