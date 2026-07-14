import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, screen, waitFor, cleanup } from '@solidjs/testing-library';
import { AssistPanel } from './AssistPanel.tsx';
import { ComposerTools } from './ComposerTools.tsx';
import { SemanticSearchToggle } from './SemanticSearchToggle.tsx';
import { AutoTag } from './AutoTag.tsx';
import { Dictation } from './Dictation.tsx';
import { AssistService } from './service.ts';
import { DISABLED_CONFIG, type AssistConfig, type InvokeResult, type TagAuditEntry } from './types.ts';

afterEach(() => cleanup());

function enabled(caps: AssistConfig['capabilities'], endpointHost = 'ai.example.com'): AssistConfig {
  return { availability: 'enabled', capabilities: caps, endpointHost, includeE2ee: false, includeAttachments: false };
}

/** A service double whose `invoke` returns a fixed result and records calls. */
function fakeService(result: InvokeResult): { service: AssistService; invoke: ReturnType<typeof vi.fn> } {
  const invoke = vi.fn(async () => result);
  const service = new AssistService();
  // Override just the network method; getConfig/transcribe untouched.
  (service as unknown as { invoke: typeof invoke }).invoke = invoke;
  return { service, invoke };
}

const noActions: InvokeResult = {
  text: 'ok',
  disclosure: { endpointHost: 'ai.example.com', sent: ['message text'], withheld: ['attachments'] },
  actions: [],
};

// ─────────────────────────────────────────────────────────────────────────────
// HARD RULE 1: unconfigured (Disabled) gateway ⇒ ZERO Assist UI.
// ─────────────────────────────────────────────────────────────────────────────
describe('Assist UI is absent when the gateway is Disabled', () => {
  it('renders nothing for every Assist component', () => {
    const { service } = fakeService(noActions);
    const a = render(() => <AssistPanel config={DISABLED_CONFIG} service={service} />);
    expect(a.container.textContent).toBe('');

    const b = render(() => (
      <ComposerTools config={DISABLED_CONFIG} service={service} text="hi" account="a" onApply={() => undefined} />
    ));
    expect(b.container.textContent).toBe('');

    const c = render(() => (
      <SemanticSearchToggle config={DISABLED_CONFIG} enabled={false} onChange={() => undefined} />
    ));
    expect(c.container.textContent).toBe('');

    const d = render(() => (
      <AutoTag
        config={DISABLED_CONFIG}
        messageId="m1"
        suggestions={[{ keyword: 'work', label: 'Work', confidence: 0.9 }]}
        onApply={() => undefined}
      />
    ));
    expect(d.container.textContent).toBe('');

    // Even a specific granted cap does nothing while availability is 'disabled'.
    const stillDisabled: AssistConfig = { ...DISABLED_CONFIG, capabilities: ['assistant', 'grammar', 'auto-tag'] };
    const e = render(() => <AssistPanel config={stillDisabled} service={service} />);
    expect(e.container.textContent).toBe('');
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// HONESTY: the "what left the device" disclosure is present + names the endpoint.
// ─────────────────────────────────────────────────────────────────────────────
describe('disclosure', () => {
  it('is shown in the chat panel and names the endpoint host', () => {
    const { service } = fakeService(noActions);
    render(() => <AssistPanel config={enabled(['assistant'])} service={service} />);
    const disc = screen.getByTestId('assist-disclosure');
    expect(disc).toBeInTheDocument();
    expect(disc.textContent).toContain('ai.example.com');
    // The default-deny ceilings are stated plainly.
    expect(disc.textContent).toMatch(/never sends end-to-end-encrypted content/i);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// HARD RULE 2: send is NEVER offered by any Assist path.
// ─────────────────────────────────────────────────────────────────────────────
describe('send is never offered', () => {
  it('AssistService exposes no send/delete/accept/transmit method', () => {
    const service = new AssistService();
    const names = Object.getOwnPropertyNames(Object.getPrototypeOf(service) as object);
    expect(names.some((n) => /send|delete|accept|transmit/i.test(n))).toBe(false);
    // The methods it DOES have are read-only / config only.
    expect(names).toContain('invoke');
    expect(names).toContain('getConfig');
    expect(names).toContain('transcribe');
  });

  it('a would-send proposed action offers only Review, never a Send button', async () => {
    const withSend: InvokeResult = {
      text: 'I can draft a reply.',
      disclosure: noActions.disclosure,
      actions: [{ id: 'a1', tool: 'mail.send', summary: 'Draft and queue a reply to Bob', wouldSend: true }],
    };
    const { service } = fakeService(withSend);
    const onReviewAction = vi.fn();
    render(() => <AssistPanel config={enabled(['assistant'])} service={service} onReviewAction={onReviewAction} />);

    fireEvent.input(screen.getByLabelText('Message the assistant'), { target: { value: 'reply to bob' } });
    fireEvent.click(screen.getByRole('button', { name: 'Ask' }));

    await waitFor(() => expect(screen.getByTestId('proposed-action')).toBeInTheDocument());
    // No control anywhere transmits.
    expect(screen.queryByRole('button', { name: /^send/i })).not.toBeInTheDocument();
    // The proposal routes to the Outbox for human confirmation.
    const review = screen.getByRole('button', { name: 'Review in Outbox' });
    fireEvent.click(review);
    expect(onReviewAction).toHaveBeenCalledWith(expect.objectContaining({ tool: 'mail.send', wouldSend: true }));
    // The panel itself never called anything that could send.
    expect(screen.getByTestId('proposed-action').textContent).toMatch(/Nothing is sent until you confirm/i);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Composer tools: transform → suggestion → explicit apply (never auto-applied).
// ─────────────────────────────────────────────────────────────────────────────
describe('composer tools', () => {
  it('offers a suggestion and applies it only on the user action', async () => {
    const result: InvokeResult = { text: 'Corrected text.', disclosure: noActions.disclosure, actions: [] };
    const { service, invoke } = fakeService(result);
    const onApply = vi.fn();
    const onDisclosure = vi.fn();
    render(() => (
      <ComposerTools
        config={enabled(['grammar'])}
        service={service}
        text="teh quick brown"
        account="acct-1"
        onApply={onApply}
        onDisclosure={onDisclosure}
      />
    ));

    fireEvent.click(screen.getByRole('button', { name: 'Fix grammar' }));
    await waitFor(() => expect(screen.getByTestId('composer-suggestion')).toBeInTheDocument());
    expect(screen.getByTestId('composer-suggestion').textContent).toBe('Corrected text.');
    // Nothing applied yet.
    expect(onApply).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'Apply to draft' }));
    expect(onApply).toHaveBeenCalledWith('Corrected text.');
    expect(onDisclosure).toHaveBeenCalledTimes(1);
    // Grammar maps to the grammar capability.
    expect(invoke).toHaveBeenCalledWith(expect.objectContaining({ capability: 'grammar' }));
  });

  it('hides tools whose capability is not granted', () => {
    const { service } = fakeService(noActions);
    // Only grammar granted ⇒ translate/tone (draft cap) are absent.
    render(() => (
      <ComposerTools config={enabled(['grammar'])} service={service} text="x" account="a" onApply={() => undefined} />
    ));
    expect(screen.getByRole('button', { name: 'Fix grammar' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Translate' })).not.toBeInTheDocument();
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Auto-tag: suggest-mode records the audit, apply is explicit + attributed.
// ─────────────────────────────────────────────────────────────────────────────
describe('auto-tag suggest → apply audit', () => {
  it('records suggested on mount and applied on the user click', async () => {
    const audit: TagAuditEntry[] = [];
    const onApply = vi.fn();
    const { service } = fakeService(noActions);
    void service;
    render(() => (
      <AutoTag
        config={enabled(['auto-tag'])}
        messageId="m-42"
        suggestions={[{ keyword: 'work', label: 'Work', confidence: 0.91 }]}
        onApply={onApply}
        onAudit={(e) => audit.push(e)}
      />
    ));

    // Suggest-mode: the suggestion was audited but NOT applied.
    expect(audit.some((e) => e.action === 'suggested' && e.actor === 'assist' && e.keyword === 'work')).toBe(true);
    expect(onApply).not.toHaveBeenCalled();
    expect(screen.getByTestId('tag-suggestion')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Apply Work' }));
    expect(onApply).toHaveBeenCalledWith('work');
    const applied = audit.find((e) => e.action === 'applied');
    expect(applied?.actor).toBe('user');
    expect(applied?.messageId).toBe('m-42');
  });

  it('auto-mode applies immediately on Assist behalf, still audited + reversible', () => {
    const audit: TagAuditEntry[] = [];
    const onApply = vi.fn();
    const onRevert = vi.fn();
    render(() => (
      <AutoTag
        config={enabled(['auto-tag'])}
        messageId="m-7"
        mode="auto"
        suggestions={[{ keyword: 'receipts', label: 'Receipts', confidence: 0.8 }]}
        onApply={onApply}
        onRevert={onRevert}
        onAudit={(e) => audit.push(e)}
      />
    ));
    expect(onApply).toHaveBeenCalledWith('receipts');
    expect(audit.find((e) => e.action === 'applied')?.actor).toBe('assist');

    fireEvent.click(screen.getByRole('button', { name: 'Remove Receipts' }));
    expect(onRevert).toHaveBeenCalledWith('receipts');
    expect(audit.some((e) => e.action === 'reverted')).toBe(true);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Semantic search toggle: controlled, hidden without the capability.
// ─────────────────────────────────────────────────────────────────────────────
describe('semantic search toggle', () => {
  it('emits changes when granted and is hidden otherwise', () => {
    const onChange = vi.fn();
    const { unmount } = render(() => (
      <SemanticSearchToggle config={enabled(['search-semantic'])} enabled={false} onChange={onChange} />
    ));
    fireEvent.click(screen.getByRole('checkbox'));
    expect(onChange).toHaveBeenCalledWith(true);
    unmount();

    const r = render(() => (
      <SemanticSearchToggle config={enabled(['grammar'])} enabled={false} onChange={onChange} />
    ));
    expect(r.container.textContent).toBe('');
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Dictation: browser SpeechRecognition path feeds onTranscript (nothing leaves).
// ─────────────────────────────────────────────────────────────────────────────
describe('dictation', () => {
  it('uses the browser recogniser and reports the transcript', async () => {
    interface FakeRec {
      lang: string;
      continuous: boolean;
      interimResults: boolean;
      onresult: ((e: unknown) => void) | null;
      onerror: ((e: unknown) => void) | null;
      onend: (() => void) | null;
      start(): void;
      stop(): void;
    }
    const holder: { current: FakeRec | null } = { current: null };
    class Fake implements FakeRec {
      lang = '';
      continuous = false;
      interimResults = false;
      onresult: ((e: unknown) => void) | null = null;
      onerror: ((e: unknown) => void) | null = null;
      onend: (() => void) | null = null;
      constructor() {
        holder.current = this;
      }
      start(): void {
        /* no-op */
      }
      stop(): void {
        this.onend?.();
      }
    }
    (globalThis as unknown as { SpeechRecognition: unknown }).SpeechRecognition = Fake;
    try {
      const { service } = fakeService(noActions);
      const onTranscript = vi.fn();
      render(() => (
        <Dictation config={enabled(['dictation'])} service={service} onTranscript={onTranscript} />
      ));
      const btn = screen.getByRole('button', { name: 'Hold to dictate' });
      fireEvent.pointerDown(btn);
      // Feed a recognition result.
      const inner = {
        length: 1,
        isFinal: true,
        item: (_j: number) => ({ transcript: 'hello world' }),
        0: { transcript: 'hello world' },
      };
      const evt = {
        resultIndex: 0,
        results: { length: 1, item: (_i: number) => inner, 0: inner },
      };
      holder.current?.onresult?.(evt);
      await waitFor(() => expect(onTranscript).toHaveBeenCalledWith('hello world'));
    } finally {
      delete (globalThis as unknown as { SpeechRecognition?: unknown }).SpeechRecognition;
    }
  });
});
