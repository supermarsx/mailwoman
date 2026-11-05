import { beforeEach, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { ComposePreferences } from './ComposePreferences.tsx';

const handles = vi.hoisted(() => ({ locked: false, save: vi.fn<(key: string, value: unknown) => Promise<boolean>>(async () => true) }));
vi.mock('./useSetting.ts', () => ({ useSetting: (key: string, fallback: unknown) => ({
  value: () => fallback, entry: () => ({ value: fallback }), locked: () => handles.locked,
  saving: () => false, error: () => '', reset: async () => true,
  set: (value: unknown) => handles.save(key, value),
}) }));

beforeEach(() => { handles.locked = false; handles.save.mockClear(); });

it('normalizes and deduplicates domains before saving to account settings', () => {
  render(() => <ComposePreferences />);
  fireEvent.input(screen.getByLabelText('Trusted recipient domains'), { target: { value: 'BÜCHER.example\nPARTNER.test\npartner.test' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save trusted domains' }));
  expect(handles.save).toHaveBeenCalledWith('mail.trustedRecipientDomains', ['xn--bcher-kva.example', 'partner.test']);
});

it('rejects wildcard or URL entries without silently changing the saved policy', () => {
  render(() => <ComposePreferences />);
  fireEvent.input(screen.getByLabelText('Trusted recipient domains'), { target: { value: '*.example.org' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save trusted domains' }));
  expect(screen.getByRole('alert')).toHaveTextContent('valid domain names');
  expect(handles.save).not.toHaveBeenCalled();
});

it('disables both controls when an administrator locks them', () => {
  handles.locked = true;
  render(() => <ComposePreferences />);
  expect(screen.getByLabelText('External recipient warnings')).toBeDisabled();
  expect(screen.getByLabelText('Trusted recipient domains')).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Save trusted domains' })).toBeDisabled();
  expect(screen.getByLabelText('Quick reply')).toBeDisabled();
});

it('keeps quick replies off by default and saves an explicit account opt-in', () => {
  render(() => <ComposePreferences />);
  expect(screen.getByLabelText('Quick reply')).not.toBeChecked();
  fireEvent.click(screen.getByLabelText('Quick reply'));
  expect(handles.save).toHaveBeenCalledWith('mail.quickReply', true);
});

it('keeps From hidden by default and saves an explicit account preference', () => {
  render(() => <ComposePreferences />);
  const input = screen.getByLabelText('Show From when composing');
  expect(input).not.toBeChecked();
  fireEvent.click(input);
  expect(handles.save).toHaveBeenCalledWith('mail.showFrom', true);
});

it('preserves the administrator lock on the default From visibility', () => {
  handles.locked = true;
  render(() => <ComposePreferences />);
  expect(screen.getByLabelText('Show From when composing')).toBeDisabled();
});

it('offers the four policies for each pre-send check and saves the choice', () => {
  render(() => <ComposePreferences />);
  for (const [label, key] of [['Invalid recipient addresses', 'mail.compose.checkInvalidRecipients'], ['Attachment mentioned but missing', 'mail.compose.checkMissingAttachment'], ['Empty subject', 'mail.compose.checkEmptySubject']] as const) {
    const select = screen.getByLabelText(label) as HTMLSelectElement;
    expect(Array.from(select.options).map(option => option.value)).toEqual(['allow', 'warn', 'warn-remember', 'deny']);
    expect(select.value).toBe('warn');
    fireEvent.change(select, { target: { value: 'warn-remember' } });
    expect(handles.save).toHaveBeenCalledWith(key, 'warn-remember');
  }
});

it('saves the recipient entry behaviour and separator, and honours the administrator lock', () => {
  render(() => <ComposePreferences />);
  expect(screen.getByLabelText('Finish a recipient when I type a space, comma or semicolon')).toBeChecked();
  fireEvent.click(screen.getByLabelText('Finish a recipient when I type a space, comma or semicolon'));
  expect(handles.save).toHaveBeenCalledWith('mail.compose.autoChipRecipients', false);
  fireEvent.change(screen.getByLabelText('Recipient separator'), { target: { value: 'semicolon' } });
  expect(handles.save).toHaveBeenCalledWith('mail.compose.recipientSeparator', 'semicolon');
});

it('locks the pre-send checks and recipient entry settings for an administrator', () => {
  handles.locked = true;
  render(() => <ComposePreferences />);
  expect(screen.getByLabelText('Empty subject')).toBeDisabled();
  expect(screen.getByLabelText('Recipient separator')).toBeDisabled();
});
