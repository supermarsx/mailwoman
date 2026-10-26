// useSetting + LockNote (worker f01): a held key shows the note, offers no edit
// and refuses a write before any request; a free key writes through.

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { Show, type JSX } from 'solid-js';
import { clearSettings, loadSettings, SettingsApiError, type Fetcher } from '../../api/settings.ts';
import { useSetting, settingErrorMessage } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { t } from '../../test/i18n.ts';

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

function Density(): JSX.Element {
  const density = useSetting('appearance.density', 'cozy');
  return (
    <div>
      <select
        aria-label="Density"
        aria-describedby="density-lock"
        disabled={density.locked() || density.saving()}
        value={density.value()}
        onChange={(e) => void density.set(e.currentTarget.value)}
      >
        <option value="compact">compact</option>
        <option value="cozy">cozy</option>
        <option value="relaxed">relaxed</option>
      </select>
      <LockNote id="density-lock" locked={density.locked()} />
      <Show when={density.error() !== ''}>
        <p role="alert">{density.error()}</p>
      </Show>
      <button type="button" onClick={() => void density.set('compact')}>
        force
      </button>
    </div>
  );
}

describe('useSetting + LockNote', () => {
  beforeEach(() => clearSettings());

  it('keeps saving active and ignores an obsolete failure while a newer edit is queued', async () => {
    const replies: Array<(response:Response)=>void> = [];
    await loadSettings({fetcher:async (_url,init)=>init?.method
      ? new Promise<Response>(resolve=>replies.push(resolve))
      : json(200,{settings:{'appearance.density':{value:'cozy',source:'default',locked:false}}})});
    render(()=><Density/>);
    fireEvent.click(screen.getByText('force'));
    fireEvent.click(screen.getByText('force'));
    replies.shift()!(json(503,{}));
    await waitFor(()=>expect(replies).toHaveLength(1));
    expect(screen.getByLabelText('Density')).toBeDisabled();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    replies.shift()!(json(200,{value:'compact',source:'user',locked:false}));
    await waitFor(()=>expect(screen.getByLabelText('Density')).toBeEnabled());
    expect(screen.getByLabelText('Density')).toHaveValue('compact');
  });

  it('falls back to the given value and stays free before the map loads', () => {
    render(() => <Density />);
    expect(screen.getByLabelText('Density')).toHaveValue('cozy');
    expect(screen.queryByTestId('setting-lock-note')).toBeNull();
  });

  it('a held key: value from the server, disabled, the note, and no request on write', async () => {
    const fetcher = vi.fn<Fetcher>(async () =>
      json(200, {
        settings: { 'appearance.density': { value: 'compact', source: 'domain', locked: true, lockedBy: 'domain' } },
      }),
    );
    await loadSettings({ fetcher });
    render(() => <Density />);
    const select = screen.getByLabelText('Density');
    expect(select).toHaveValue('compact');
    expect(select).toBeDisabled();
    const note = screen.getByTestId('setting-lock-note');
    expect(note).toHaveTextContent(t('settings-policy-locked'));
    expect(note).toHaveAttribute('id', 'density-lock');
    expect(note.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');

    fireEvent.click(screen.getByText('force'));
    expect(await screen.findByRole('alert')).toHaveTextContent(t('settings-policy-error-locked'));
    expect(fetcher).toHaveBeenCalledTimes(1); // the load only
  });

  it('a free key writes through and shows the new value', async () => {
    const fetcher = vi.fn<Fetcher>(async (_u, init) =>
      init?.method === 'PUT'
        ? json(200, { value: 'relaxed', source: 'user', locked: false })
        : json(200, { settings: { 'appearance.density': { value: 'cozy', source: 'default', locked: false } } }),
    );
    await loadSettings({ fetcher });
    render(() => <Density />);
    fireEvent.change(screen.getByLabelText('Density'), { target: { value: 'relaxed' } });
    await waitFor(() => expect(screen.getByLabelText('Density')).toHaveValue('relaxed'));
    expect(fetcher.mock.calls.some(([, init]) => init?.method === 'PUT')).toBe(true);
  });

  it('a value of the wrong type from the server is ignored', async () => {
    await loadSettings({
      fetcher: vi.fn<Fetcher>(async () =>
        json(200, { settings: { 'appearance.density': { value: 42, source: 'global', locked: false } } }),
      ),
    });
    render(() => <Density />);
    expect(screen.getByLabelText('Density')).toHaveValue('cozy');
  });

  it('maps refusal codes to localised text', () => {
    expect(settingErrorMessage(new SettingsApiError(403, 'setting_locked', null))).toBe(
      t('settings-policy-error-locked'),
    );
    expect(settingErrorMessage(new SettingsApiError(422, 'invalidValue', '/value'))).toBe(
      t('settings-policy-error-invalid'),
    );
    expect(settingErrorMessage(new SettingsApiError(401, 'http401', null))).toBe(
      t('settings-policy-error-signed-out'),
    );
    expect(settingErrorMessage(new Error('boom'))).toBe(t('settings-policy-error-generic'));
  });
});
