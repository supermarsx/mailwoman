import { describe, it, expect } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { FormField } from './FormField.tsx';
import { HelpNote } from './HelpNote.tsx';

describe('FormField + HelpNote', () => {
  it('labels the control in plain language with the technical name secondary', () => {
    render(() => (
      <FormField label="Connection security" technical="STARTTLS">
        {(w) => <select id={w.id} aria-describedby={w.describedBy} />}
      </FormField>
    ));
    expect(screen.getByRole('combobox', { name: 'Connection security — STARTTLS' })).toBeInTheDocument();
  });

  it('ties help and error to the control with aria-describedby and aria-invalid', () => {
    const [error, setError] = createSignal<string | null>(null);
    render(() => (
      <FormField label="Port" help="The port the mail server listens on; it does not open a firewall." error={error()}>
        {(w) => <input id={w.id} aria-describedby={w.describedBy} aria-invalid={w.invalid} />}
      </FormField>
    ));
    const input = screen.getByRole('textbox', { name: 'Port' });
    expect(input).toHaveAccessibleDescription('The port the mail server listens on; it does not open a firewall.');
    expect(input).toHaveAttribute('aria-invalid', 'false');
    setError('Enter a number between 1 and 65535.');
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(input).toHaveAccessibleDescription(
      'The port the mail server listens on; it does not open a firewall. Enter a number between 1 and 65535.',
    );
    setError(null);
    expect(input).toHaveAttribute('aria-invalid', 'false');
    expect(screen.queryByText('Enter a number between 1 and 65535.')).toBeNull();
  });

  it('without help or error there is no dangling description', () => {
    render(() => <FormField label="Name">{(w) => <input id={w.id} aria-describedby={w.describedBy} />}</FormField>);
    expect(screen.getByRole('textbox', { name: 'Name' })).not.toHaveAttribute('aria-describedby');
  });

  it('HelpNote is one sentence with a decorative icon', () => {
    render(() => <HelpNote id="h">Blocks sign-in; it does not stop IMAP clients.</HelpNote>);
    const p = document.getElementById('h')!;
    expect(p).toHaveTextContent('Blocks sign-in; it does not stop IMAP clients.');
    expect(p.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
  });
});
