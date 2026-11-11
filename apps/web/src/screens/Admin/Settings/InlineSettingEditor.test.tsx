// The inline setting editor: dirty, save with If-Match, reset, lock, read-only,
// env pin and 412 handling for one representative key.

import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { Show } from 'solid-js';
import { InlineSettingEditor } from './InlineSettingEditor.tsx';
import type { AdminSetting } from '../../../api/admin/settings.ts';
import { holding, renderInShell } from '../shell/testkit.tsx';
import { useShell } from '../shell/context.ts';

const base: AdminSetting = {
  key: 'privacy.maxRedirects', family: false, kind: { type: 'int', min: 0, max: 10 }, default: 3, nullable: false, scopes: 'global', userOverride: false,
  category: 'imageProxy', mapsTo: '', env: null, dedicatedRoute: null, scopeId: 'account', hot: true, ceiling: false, userVisible: false, globalValue: null,
  allowUserOverride: true, globalLocked: false, updatedBy: null, updatedAt: null, effective: { value: 3, source: 'default' }, envState: null, overrides: {}, etag: '"e0"',
};
const json = (status: number, body: unknown, etag?: string): Response => new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json', ...(etag ? { ETag: etag } : {}) } });

afterEach(() => vi.unstubAllGlobals());

describe('InlineSettingEditor', () => {
  it('is clean until edited, saves with If-Match, and shows the saved value with its source', async () => {
    const saved: AdminSetting = { ...base, globalValue: 5, effective: { value: 5, source: 'global' }, etag: '"e1"' };
    const calls: { ifMatch: string | null; body: unknown }[] = [];
    vi.stubGlobal('fetch', vi.fn(async (_url: string, init?: RequestInit) => {
      calls.push({ ifMatch: new Headers(init?.headers).get('If-Match'), body: JSON.parse(String(init?.body)) });
      return json(200, saved, '"e1"');
    }));
    const onChanged = vi.fn();
    const onDirty = vi.fn();
    renderInShell(() => <InlineSettingEditor settingKey={base.key} setting={base} onChanged={onChanged} onDirty={onDirty} />);
    const save = await screen.findByRole('button', { name: 'Save' });
    expect(save).toBeDisabled();
    const input = screen.getByRole('spinbutton') as HTMLInputElement;
    fireEvent.input(input, { target: { value: '5' } });
    await waitFor(() => expect(save).toBeEnabled());
    expect(onDirty).toHaveBeenLastCalledWith(base.key, true);
    fireEvent.click(save);
    await waitFor(() => expect(onChanged).toHaveBeenCalledWith(saved));
    expect(calls[0]).toEqual({ ifMatch: '"e0"', body: { globalValue: 5 } });
    await waitFor(() => expect(onDirty).toHaveBeenLastCalledWith(base.key, false));
    expect(screen.getByTestId('inline-effective')).toHaveTextContent('5');
  });

  it('Discard changes puts the stored value back', async () => {
    renderInShell(() => <InlineSettingEditor settingKey={base.key} setting={base} />);
    const input = (await screen.findByRole('spinbutton')) as HTMLInputElement;
    fireEvent.input(input, { target: { value: '9' } });
    fireEvent.click(screen.getByRole('button', { name: 'Discard changes' }));
    await waitFor(() => expect(input.value).toBe('3'));
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('refuses a value outside the registry bounds before any request', async () => {
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);
    renderInShell(() => <InlineSettingEditor settingKey={base.key} setting={base} />);
    fireEvent.input(await screen.findByRole('spinbutton'), { target: { value: '50' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText(/10/)).toBeInTheDocument();
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('writes the lock with the value for a lockable key', async () => {
    const lockable: AdminSetting = { ...base, scopes: 'global+domain' };
    const bodies: unknown[] = [];
    vi.stubGlobal('fetch', vi.fn(async (_url: string, init?: RequestInit) => {
      bodies.push(JSON.parse(String(init?.body)));
      return json(200, { ...lockable, globalLocked: true, etag: '"e1"' }, '"e1"');
    }));
    renderInShell(() => <InlineSettingEditor settingKey={lockable.key} setting={lockable} />);
    fireEvent.click(await screen.findByRole('switch'));
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(bodies).toEqual([{ locked: true }]));
  });

  it('renders read-only without settings.write and says why', async () => {
    renderInShell(() => { const shell = useShell(); return <Show when={shell.can.grants().kind === 'listed'}><InlineSettingEditor settingKey={base.key} setting={base} /></Show>; }, { loadMe: holding(['settings.read']) });
    expect(await screen.findByRole('spinbutton')).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
    expect(screen.getByTestId('inline-readonly')).toBeInTheDocument();
  });

  it('is read-only when an environment variable pins the key, and names the variable', async () => {
    const pinned: AdminSetting = { ...base, envState: { var: 'MW_MAX_REDIRECTS', precedence: 'pin', set: true, value: 2 } };
    renderInShell(() => <InlineSettingEditor settingKey={base.key} setting={pinned} />);
    expect(await screen.findByRole('spinbutton')).toBeDisabled();
    expect(screen.getByTestId('inline-readonly')).toHaveTextContent('MW_MAX_REDIRECTS');
  });

  it('keeps the edit and offers Reload / Keep mine on a 412', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => json(412, { code: 'stale', updatedBy: 'alice', updatedAt: '2026-01-01T00:00:00Z' })));
    renderInShell(() => <InlineSettingEditor settingKey={base.key} setting={base} />);
    const input = (await screen.findByRole('spinbutton')) as HTMLInputElement;
    fireEvent.input(input, { target: { value: '7' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByRole('button', { name: /Keep mine/ })).toBeInTheDocument();
    expect(input.value).toBe('7');
  });

  it('loads the setting by key when the list did not hand it in', async () => {
    const load = vi.fn(async () => ({ setting: base, etag: '"e0"' }));
    renderInShell(() => <InlineSettingEditor settingKey={base.key} load={load} />);
    expect(await screen.findByRole('spinbutton')).toBeInTheDocument();
    expect(load).toHaveBeenCalledWith(base.key);
  });
});

it.each(['accounts.linkingEnabled','accounts.storeLinkedCredentials'])('confirms disabling %s before credentials can be cleared',async key=>{
 const policy:AdminSetting={...base,key,kind:{type:'bool'},globalValue:true,effective:{value:true,source:'global'},default:false};
 const saved={...policy,globalValue:false,effective:{value:false,source:'global' as const},etag:'"e1"'};
 const fetcher=vi.fn<typeof fetch>(async()=>json(200,saved,'"e1"'));vi.stubGlobal('fetch',fetcher);
 renderInShell(()=><InlineSettingEditor settingKey={key} setting={policy}/>);
 const toggle=await screen.findByRole('checkbox');expect(toggle).toBeChecked();fireEvent.click(toggle);
 const dialog=await screen.findByRole('alertdialog');expect(dialog).toHaveTextContent('sign in to their linked accounts again');expect(fetcher).not.toHaveBeenCalled();
 fireEvent.click(within(dialog).getByRole('button',{name:'Cancel'}));expect(toggle).toBeChecked();expect(fetcher).not.toHaveBeenCalled();
 fireEvent.click(toggle);fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button',{name:'Disable and clear saved access'}));
 await waitFor(()=>expect(screen.queryByRole('alertdialog')).toBeNull());expect(toggle).not.toBeChecked();expect(fetcher).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Save'}));await waitFor(()=>expect(fetcher).toHaveBeenCalled());
 expect(JSON.parse(fetcher.mock.calls[0]![1]!.body as string)).toEqual({globalValue:false});
});
