// Regression (t43-e1): submit() passed Solid store proxies (emails/phones) to onSave,
// which reaches the network worker through postMessage ("could not be cloned").
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { ContactEditor } from './ContactEditor.tsx';

describe('ContactEditor save payload', () => {
  it('is structured-cloneable with an email and a phone entered', async () => {
    const onSave = vi.fn(async (_draft: unknown) => 'id1');
    render(() => <ContactEditor mode="create" onCancel={() => undefined} onSaved={() => undefined} onSave={onSave as never} />);
    fireEvent.input(await screen.findByLabelText(/^email 1$/i), { target: { value: 'a@example.com' } });
    fireEvent.input(screen.getByLabelText(/^phone 1$/i), { target: { value: '+1 555 0100' } });
    fireEvent.submit(document.querySelector('form')!);
    await waitFor(() => expect(onSave).toHaveBeenCalledTimes(1));
    const saved = onSave.mock.calls[0]![0] as { emails: unknown[]; phones: unknown[] };
    const cloned = structuredClone(saved); // throws DataCloneError on a Proxy
    expect(cloned.emails).toHaveLength(1);
    expect(cloned.phones).toHaveLength(1);
  }, 60_000);
});
