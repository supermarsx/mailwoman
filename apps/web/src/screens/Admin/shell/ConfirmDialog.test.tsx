import {setAdminLeetspeak} from '../../../i18n/presentation.ts';
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { ConfirmDialog } from './ConfirmDialog.tsx';
import { isolate } from '../../../i18n';
import { AdminApiError } from '../../../state/slices/admin.ts';

function Harness(props: { onConfirm: () => void | Promise<void>; typed?: string }) {
  const [open, setOpen] = createSignal(false);
  return (
    <>
      <button type="button" onClick={() => setOpen(true)}>
        Revoke
      </button>
      <ConfirmDialog
        open={open()}
        title="Revoke all sessions of alice@example.org?"
        body="Every device signed in as alice@example.org is signed out."
        confirmLabel="Revoke sessions"
        {...(props.typed !== undefined ? { typedConfirmation: props.typed } : {})}
        onConfirm={async () => {
          await props.onConfirm();
          setOpen(false);
        }}
        onCancel={() => setOpen(false)}
      />
    </>
  );
}

afterEach(()=>setAdminLeetspeak(null));

describe('ConfirmDialog', () => {
  it('keeps the typed confirmation instruction and required token canonical in leetspeak mode',async()=>{
    setAdminLeetspeak(true);const confirm=vi.fn();render(()=><Harness onConfirm={confirm} typed="Trash"/>);fireEvent.click(screen.getByRole('button',{name:'Revoke'}));
    const field=await screen.findByLabelText(`Type ${isolate('Trash')} to confirm`);const button=screen.getByRole('button',{name:'Revoke sessions'});
    fireEvent.input(field,{target:{value:'7r45h'}});expect(button).toBeDisabled();
    fireEvent.input(field,{target:{value:'Trash'}});expect(button).toBeEnabled();fireEvent.click(button);await waitFor(()=>expect(confirm).toHaveBeenCalledOnce());
  });
  it('is a modal alertdialog named after the object, with the consequence as its description', async () => {
    render(() => <Harness onConfirm={() => undefined} />);
    fireEvent.click(screen.getByRole('button', { name: 'Revoke' }));
    const dlg = await screen.findByRole('alertdialog', { name: 'Revoke all sessions of alice@example.org?' });
    expect(dlg).toHaveAttribute('aria-modal', 'true');
    expect(dlg).toHaveAccessibleDescription('Every device signed in as alice@example.org is signed out.');
  });

  it('focus starts on Cancel, Escape cancels, and focus returns to the opener', async () => {
    const confirm = vi.fn();
    render(() => <Harness onConfirm={confirm} />);
    const opener = screen.getByRole('button', { name: 'Revoke' });
    opener.focus();
    fireEvent.click(opener);
    const dlg = await screen.findByRole('alertdialog');
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Cancel' })));
    fireEvent.keyDown(dlg, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('alertdialog')).toBeNull());
    expect(confirm).not.toHaveBeenCalled();
    expect(document.activeElement).toBe(opener);
  });

  it('confirm runs the action and closes', async () => {
    const confirm = vi.fn(async () => undefined);
    render(() => <Harness onConfirm={confirm} />);
    fireEvent.click(screen.getByRole('button', { name: 'Revoke' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Revoke sessions' }));
    await waitFor(() => expect(confirm).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.queryByRole('alertdialog')).toBeNull());
  });

  it('a failed action keeps the dialog open and says what failed', async () => {
    render(() => <Harness onConfirm={async () => Promise.reject(new AdminApiError(403, 'no'))} />);
    fireEvent.click(screen.getByRole('button', { name: 'Revoke' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Revoke sessions' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Your role does not allow this action');
    expect(screen.getByRole('alertdialog')).toBeInTheDocument();
  });

  it('typed confirmation: the button works only once the exact name is typed', async () => {
    const confirm = vi.fn();
    render(() => <Harness onConfirm={confirm} typed="example.org" />);
    fireEvent.click(screen.getByRole('button', { name: 'Revoke' }));
    const field = await screen.findByLabelText(`Type ${isolate('example.org')} to confirm`);
    const go = screen.getByRole('button', { name: 'Revoke sessions' });
    await waitFor(() => expect(document.activeElement).toBe(field));
    expect(go).toBeDisabled();
    fireEvent.input(field, { target: { value: 'example.or' } });
    expect(go).toBeDisabled();
    fireEvent.submit(go.closest('form')!);
    expect(confirm).not.toHaveBeenCalled();
    fireEvent.input(field, { target: { value: 'example.org' } });
    expect(go).toBeEnabled();
    fireEvent.click(go);
    await waitFor(() => expect(confirm).toHaveBeenCalled());
  });
});
