import { describe, it, expect, vi } from 'vitest';
import { screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { renderInShell as render } from './testkit.tsx';
import { Wizard } from './Wizard.tsx';
import { anyDirty } from './useDirtyGuard.ts';
import { AdminApiError } from '../../../state/slices/admin.ts';

function DomainWizard(props: { onSave: () => Promise<void> }) {
  const [name, setName] = createSignal('');
  const [port, setPort] = createSignal('993');
  return (
    <Wizard
      title="Add a domain"
      dirty={() => name() !== ''}
      steps={[
        {
          id: 'name',
          title: 'Name',
          content: () => <input aria-label="Domain name" value={name()} onInput={(e) => setName(e.currentTarget.value)} />,
          validate: () => (name().includes('.') ? null : 'Enter a domain name such as example.org.'),
        },
        {
          id: 'imap',
          title: 'Mail server',
          content: () => <input aria-label="IMAP port" value={port()} onInput={(e) => setPort(e.currentTarget.value)} />,
        },
      ]}
      review={() => [
        { label: 'Domain', value: name() },
        { label: 'IMAP port', value: port() },
      ]}
      onSave={props.onSave}
    />
  );
}

describe('Wizard', () => {
  it('shows the steps, marks the current one, and blocks Next on a validation problem', async () => {
    const save = vi.fn(async () => undefined);
    render(() => <DomainWizard onSave={save} />);
    const steps = screen.getByRole('list', { name: 'Steps' });
    expect(within(steps).getAllByRole('listitem').map((s) => s.textContent)).toEqual(['1. Name', '2. Mail server', '3. Review']);
    expect(within(steps).getAllByRole('listitem')[0]).toHaveAttribute('aria-current', 'step');
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    expect(screen.getByRole('alert')).toHaveTextContent('Enter a domain name such as example.org.');
    expect(screen.getByLabelText('Domain name')).toBeInTheDocument();
  });

  it('Back keeps what was entered; the review lists every value; nothing is saved before Save', async () => {
    const save = vi.fn(async () => undefined);
    render(() => <DomainWizard onSave={save} />);
    fireEvent.input(screen.getByLabelText('Domain name'), { target: { value: 'example.org' } });
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    expect(screen.getByRole('heading', { name: 'Step 2 of 3: Mail server' })).toBeInTheDocument();
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('heading', { name: 'Step 2 of 3: Mail server' })));
    fireEvent.click(screen.getByRole('button', { name: 'Back' }));
    expect(screen.getByLabelText('Domain name')).toHaveValue('example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    const review = document.querySelector('[data-step="review"]')!;
    expect(review).toHaveTextContent('Domainexample.org');
    expect(review).toHaveTextContent('IMAP port993');
    expect(save).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
    expect(await screen.findByRole('status')).toHaveTextContent('Saved.');
  });

  it('only toasts after a save gesture and replaces persistent progress without inline success',async()=>{
    let finish!:()=>void;
    render(()=><DomainWizard onSave={()=>new Promise<void>(resolve=>{finish=resolve;})}/>);
    expect(screen.queryByTestId('admin-wizard-save-toast')).toBeNull();
    fireEvent.input(screen.getByLabelText('Domain name'),{target:{value:'example.org'}});
    fireEvent.click(screen.getByRole('button',{name:'Next'}));
    fireEvent.click(screen.getByRole('button',{name:'Next'}));
    const button=screen.getByRole('button',{name:'Save'});
    fireEvent.click(button);
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('aria-busy','true');
    expect(screen.getByTestId('admin-wizard-save-toast')).toHaveTextContent('Saving');
    finish();
    await waitFor(()=>expect(screen.getByTestId('admin-wizard-save-toast')).toHaveTextContent('Saved.'));
    expect(screen.getAllByRole('status')).toHaveLength(1);
    expect(document.querySelector('section [role="status"]')).toBeNull();
  });

  it('a failed save stays on the review with the reason', async () => {
    render(() => <DomainWizard onSave={async () => Promise.reject(new AdminApiError(409, 'exists'))} />);
    fireEvent.input(screen.getByLabelText('Domain name'), { target: { value: 'example.org' } });
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Someone changed it in the meantime');
    expect(document.querySelector('[data-step="review"]')).not.toBeNull();
  });

  it('holds the dirty guard while there is unsaved input, and releases it after Save', async () => {
    render(() => <DomainWizard onSave={async () => undefined} />);
    expect(anyDirty()).toBe(false);
    fireEvent.input(screen.getByLabelText('Domain name'), { target: { value: 'example.org' } });
    expect(anyDirty()).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(screen.getByTestId('admin-wizard-save-toast')).toHaveTextContent('Saved.'));
    expect(anyDirty()).toBe(false);
  });
});
