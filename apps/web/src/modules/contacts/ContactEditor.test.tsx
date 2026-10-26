import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { ContactEditor } from './ContactEditor.tsx';
import { confirmShellNavigation } from '../../shell/navigationGuard.ts';

describe('dedicated contact editor', () => {
  it('retains the draft on failure and prevents duplicate pending saves', async () => {
    let finish: (saved: boolean) => void = () => undefined;
    const save = vi.fn(() => new Promise<boolean>(resolve => {finish=resolve;}));
    const done = vi.fn();
    render(() => <ContactEditor mode="create" onCancel={vi.fn()} onSave={save} onSaved={done}/>);
    fireEvent.input(screen.getByLabelText('Full name'), {target:{value:'Unsaved name'}});
    fireEvent.click(screen.getByRole('button',{name:'Save'}));
    expect(screen.getByRole('button',{name:'Save'})).toBeDisabled();
    expect(screen.getByRole('button',{name:'Cancel'})).toBeDisabled();
    expect(save).toHaveBeenCalledTimes(1); finish(false);
    expect(await screen.findByRole('alert')).toHaveTextContent('could not be saved');
    expect(screen.getByLabelText('Full name')).toHaveValue('Unsaved name'); expect(done).not.toHaveBeenCalled();
  });
  it('requires explicit discard for cancel and shell navigation', async () => {
    const cancel = vi.fn();
    render(() => <ContactEditor mode="create" onCancel={cancel} onSave={async()=>false} onSaved={vi.fn()}/>);
    fireEvent.input(screen.getByLabelText('Full name'), {target:{value:'Draft'}});
    fireEvent.click(screen.getByRole('button',{name:'Cancel'}));
    expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button',{name:'Keep editing'})); expect(cancel).not.toHaveBeenCalled();
    const leaving = confirmShellNavigation();
    fireEvent.click(await screen.findByRole('button',{name:'Discard changes'})); expect(await leaving).toBe(true);
  });
  it('does not invoke completion after its account-owned editor is unmounted', async () => {
    let finish: (saved: boolean) => void = () => undefined; const done = vi.fn();
    const view=render(() => <ContactEditor mode="create" onCancel={vi.fn()} onSave={()=>new Promise(resolve=>{finish=resolve;})} onSaved={done}/>);
    fireEvent.click(screen.getByRole('button',{name:'Save'})); view.unmount(); finish(true);
    await waitFor(()=>expect(done).not.toHaveBeenCalled());
    expect(await confirmShellNavigation()).toBe(true);
  });
});
