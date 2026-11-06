import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { CollapsibleBar, RibbonPanel } from '../../bars/index.ts';

const off = vi.hoisted(() => ({ ids: new Set<string>() }));
vi.mock('../../../api/settings.ts', () => ({
  settings: () => Object.fromEntries([...off.ids].map((id) => [`compose.feature.${id}`, { value: false, locked: true, source: 'domain' }])),
  loadSettings: vi.fn(async () => undefined),
}));

import { MessageOptions, datePresets, fromLocalInput, toLocalInput } from './MessageOptions.tsx';
import { DEFAULT_GATES, buildHeaders, type HeaderGates, type MessageHeaderOptions } from './messageHeaders.ts';

afterEach(() => { cleanup(); off.ids.clear(); });

function mount(gates: Partial<HeaderGates> = {}, threading = { inReplyTo: ['a@b'], references: ['x@y', 'a@b'] }) {
  const [opts, setOpts] = createSignal<MessageHeaderOptions>({});
  const g = { ...DEFAULT_GATES, ...gates };
  let newThread = 0;
  render(() => (
    <CollapsibleBar id="composerRibbon" revealPolicy="overlay" label="Ribbon" header={<span />}>
      <RibbonPanel label="Delivery">
        <MessageOptions options={opts} update={(c) => setOpts(c)} gates={() => g} threading={() => threading} startNewThread={() => { newThread += 1; }} fromAddress={() => 'me@example.com'} externalCount={() => 2} />
      </RibbonPanel>
    </CollapsibleBar>
  ));
  fireEvent.click(screen.getByTestId('compose-message-options-button'));
  return { opts, newThread: () => newThread, gates: g };
}

it('sets sensitivity, reply-by, expires and supersedes, and they serialise as headers', async () => {
  const m = mount();
  fireEvent.change(await screen.findByTestId('mo-sensitivity'), { target: { value: 'company-confidential' } });
  fireEvent.input(screen.getByTestId('mo-reply-by'), { target: { value: '2026-11-03T17:00' } });
  fireEvent.input(screen.getByTestId('mo-expires'), { target: { value: '2026-12-01T09:00' } });
  fireEvent.input(screen.getByTestId('mo-supersedes'), { target: { value: '<old@example.com>' } });
  const h = Object.fromEntries(buildHeaders(m.opts(), m.gates).headers);
  expect(h['Sensitivity']).toBe('company-confidential');
  expect(h['Reply-By']).toMatch(/^\w{3}, \d{2} \w{3} 2026 \d{2}:\d{2}:00 \+0000$/);
  expect(h['Expires']).toBe(h['Expiry-Date']);
  expect(h['Supersedes']).toBe('<old@example.com>');
  expect(screen.getByTestId('mo-confidential-external')).toHaveTextContent('2 recipients');
});

it('quick choices fill the date and clear removes it', async () => {
  const m = mount();
  fireEvent.change(await screen.findByTestId('mo-expires-preset'), { target: { value: 'tomorrow' } });
  expect(m.opts().expires).toBeInstanceOf(Date);
  fireEvent.click(screen.getAllByRole('button', { name: 'Clear' })[1]!);
  expect(m.opts().expires).toBeNull();
  const now = new Date(2026, 10, 3, 12, 0);
  expect(datePresets(now).map((p) => p.id)).toEqual(['today', 'tomorrow', 'in3days', 'nextWeek']);
  expect(datePresets(new Date(2026, 10, 3, 18, 0)).map((p) => p.id)).not.toContain('today');
  expect(fromLocalInput(toLocalInput(now))?.getTime()).toBe(now.getTime());
  expect(fromLocalInput('garbage')).toBeNull();
});

it('hides gated headers until the administrator allows them', async () => {
  mount();
  await screen.findByTestId('mo-sensitivity');
  expect(screen.queryByTestId('mo-return-receipt')).toBeNull();
  expect(screen.queryByTestId('mo-auto')).toBeNull();
  expect(screen.queryByTestId('mo-custom')).toBeNull();
});

it('offers auto-submitted, legacy receipt and custom headers when allowed, and validates custom ones', async () => {
  const m = mount({ allowAutoSubmitted: true, allowLegacyReturnReceipt: true, allowCustom: true });
  fireEvent.change(await screen.findByTestId('mo-auto'), { target: { value: 'auto-replied' } });
  fireEvent.click(screen.getByTestId('mo-return-receipt'));
  fireEvent.input(screen.getByTestId('mo-custom-name'), { target: { value: 'From' } });
  fireEvent.input(screen.getByTestId('mo-custom-value'), { target: { value: 'evil@example.com' } });
  fireEvent.click(screen.getByTestId('mo-custom-add'));
  expect(screen.getByTestId('mo-custom-error')).toHaveTextContent('set by Mailwoman');
  fireEvent.input(screen.getByTestId('mo-custom-name'), { target: { value: 'X-Ticket' } });
  fireEvent.input(screen.getByTestId('mo-custom-value'), { target: { value: '42' } });
  fireEvent.click(screen.getByTestId('mo-custom-add'));
  const names = buildHeaders(m.opts(), m.gates).headers.map(([n]) => n);
  expect(names).toEqual(['Return-Receipt-To', 'Auto-Submitted', 'X-Ticket']);
});

it('shows the threading headers of a reply and starts a new thread only when allowed', async () => {
  const locked = mount();
  expect(await screen.findByTestId('mo-in-reply-to')).toHaveTextContent('<a@b>');
  expect(screen.getByTestId('mo-references')).toHaveTextContent('<x@y> <a@b>');
  expect((screen.getByTestId('mo-new-thread') as HTMLButtonElement).disabled).toBe(true);
  expect(locked.newThread()).toBe(0);
  cleanup();
  const open = mount({ allowThreadingEdit: true });
  await screen.findByTestId('mo-new-thread');
  fireEvent.click(screen.getByTestId('mo-new-thread'));
  expect(open.newThread()).toBe(1);
});

it('feature flags remove their part of the dialog', async () => {
  off.ids.add('options.sensitivity');
  off.ids.add('options.expires');
  mount();
  await screen.findByTestId('mo-reply-by');
  expect(screen.queryByTestId('mo-sensitivity')).toBeNull();
  expect(screen.queryByTestId('mo-expires')).toBeNull();
});
