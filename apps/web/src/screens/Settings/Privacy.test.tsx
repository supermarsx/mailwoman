import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { Privacy } from './Privacy.tsx';
import { clearSettings,loadSettings } from '../../api/settings.ts';
import { remotePolicy } from '../../api/remote-content.ts';
import {clearFeatures,loadFeatures} from '../../state/features.ts';
vi.mock('../../api/remote-content.ts', () => ({ remotePolicy: vi.fn() }));
const free = (value: unknown, locked = false) => ({ value, source: 'global', locked, userValue: value });
let writes: unknown[]; let held = false;
beforeEach(() => {
  clearSettings(); writes = []; held = false;
  clearFeatures();
  vi.mocked(remotePolicy).mockResolvedValue({ mode: 'ask', trackers: true, cleanLinks: true, allowDirect: false, inlineParts: true, locked: { mode: false, trackers: false, cleanLinks: false } });
  vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
    const body = init?.method === 'PUT' ? JSON.parse(String(init.body)) as {value:unknown} : null;
    if (body) { writes.push({ url, value: body.value }); return new Response(JSON.stringify(free(body.value))); }
    if(init?.method==='DELETE'){writes.push({url,reset:true});return new Response(JSON.stringify(free(false)));}
    return new Response(JSON.stringify({ settings: {'privacy.imageRetryAttempts':free(3,held),'privacy.imageRetryDelayMs':free(5000,held), 'mail.reader.allowOriginalHtml':free(false,held),'privacy.trackerOverrideAllowed':free(true,held),'mail.reader.unsafeHtmlMode':free('ask',held),'privacy.remoteContent': free('ask', held), 'privacy.trackers': free(true, held), 'privacy.cleanLinks': free(true, held), 'privacy.stripMailerHeaders': free(true, held) } }));
  }));
});
describe('Privacy settings', () => {
  it('saves bounded image retry preferences in seconds and allows inherited defaults',async()=>{
    render(()=><Privacy/>);const attempts=await screen.findByLabelText('Automatic image retry attempts');const delay=screen.getByLabelText('Image retry delay (seconds)');await waitFor(()=>expect(attempts).toBeEnabled());expect(attempts).toHaveValue(3);expect(delay).toHaveValue(5);
    fireEvent.change(attempts,{target:{value:'0'}});await waitFor(()=>expect(writes).toContainEqual({url:'/api/settings/privacy.imageRetryAttempts',value:0}));
    fireEvent.change(delay,{target:{value:'0.5'}});await waitFor(()=>expect(writes).toContainEqual({url:'/api/settings/privacy.imageRetryDelayMs',value:500}));
    fireEvent.change(attempts,{target:{value:'9'}});fireEvent.change(delay,{target:{value:'0.1'}});expect(writes).toHaveLength(2);
    const row=attempts.closest('div')!;await waitFor(()=>expect(row.querySelector('button')).toBeEnabled());fireEvent.click(row.querySelector('button')!);await waitFor(()=>expect(writes).toContainEqual({url:'/api/settings/privacy.imageRetryAttempts',reset:true}));
  });
  it('offers original HTML as a disabled-by-default preference with inheritance, locks and feature gates',async()=>{
    await loadFeatures({refresh:true,fetcher:async()=>new Response(JSON.stringify({features:{'reading.original_html':{enabled:true}}}))});
    render(()=><Privacy/>);const toggle=await screen.findByLabelText('Allow Original HTML per message');await waitFor(()=>expect(toggle).toBeEnabled());expect(toggle).not.toBeChecked();
    fireEvent.click(toggle);await waitFor(()=>expect(writes).toContainEqual({url:'/api/settings/mail.reader.allowOriginalHtml',value:true}));
    const row=toggle.closest('div')!;await waitFor(()=>expect(row.querySelector('button')).toBeEnabled());fireEvent.click(row.querySelector('button')!);await waitFor(()=>expect(writes).toContainEqual({url:'/api/settings/mail.reader.allowOriginalHtml',reset:true}));
    await loadFeatures({refresh:true,fetcher:async()=>new Response(JSON.stringify({features:{'reading.original_html':{enabled:false}}}))});expect(toggle).toBeDisabled();expect(toggle).not.toBeChecked();
  });
  it('keeps tracker choice and original HTML unavailable under administrator locks',async()=>{
    held=true;await loadFeatures({refresh:true,fetcher:async()=>new Response(JSON.stringify({features:{'reading.original_html':{enabled:true}}}))});render(()=><Privacy/>);
    await waitFor(()=>expect(screen.getByLabelText('Allow Original HTML per message')).toBeDisabled());expect(screen.getByLabelText('Allow a per-message tracker choice')).toBeDisabled();await waitFor(()=>expect(screen.getByLabelText('Allow a per-message tracker choice')).not.toBeChecked());expect(writes).toEqual([]);
  });
  it('disables original HTML while strict handling is locked and retains its preference after unlock',async()=>{
    await loadFeatures({refresh:true,fetcher:async()=>new Response(JSON.stringify({features:{'reading.original_html':{enabled:true}}}))});render(()=><Privacy/>);
    const toggle=await screen.findByLabelText('Allow Original HTML per message');await waitFor(()=>expect(toggle).toBeEnabled());
    const strict=async(locked:boolean)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.reader.allowOriginalHtml':free(true),'mail.reader.unsafeHtmlMode':free('strict',locked)}}))});
    await strict(false);expect(toggle).toBeEnabled();expect(toggle).toBeChecked();
    await strict(true);expect(toggle).toBeDisabled();expect(toggle).not.toBeChecked();expect(toggle.closest('div')!.querySelector('button')).toBeDisabled();
    await strict(false);expect(toggle).toBeEnabled();expect(toggle).toBeChecked();expect(writes).toEqual([]);
  });
  it('hides native outgoing-header controls for upstream proxy accounts', async () => {
    vi.mocked(remotePolicy).mockResolvedValue({mode:'ask', trackers:true, cleanLinks:true, allowDirect:false, inlineParts:false, locked:{mode:false,trackers:false,cleanLinks:false}});
    render(() => <Privacy />);
    await waitFor(() => expect(screen.getByLabelText('Remote content')).toBeEnabled());
    expect(screen.queryByLabelText('Remove outgoing mailer headers')).not.toBeInTheDocument();
    expect(screen.queryByText(/Native SMTP and EWS sends/)).not.toBeInTheDocument();
  });

  it('honors locks and does not offer direct mode without server permission', async () => {
    held = true; render(() => <Privacy />);
    const mode = await screen.findByLabelText('Remote content'); await waitFor(() => expect(mode).toBeDisabled());
    expect(screen.getByLabelText('Block tracking images and beacons')).toBeDisabled();
    expect(screen.getByLabelText('Remove tracking parameters from links')).toBeDisabled();
    expect(screen.getByLabelText('Remove outgoing mailer headers')).toBeDisabled();
    expect(screen.getByRole('option', { name: "Load directly from the sender's servers" })).toBeDisabled();
    expect(screen.getByLabelText('Potentially unsafe message HTML')).toBeDisabled();
    expect(screen.getByLabelText('Automatic image retry attempts')).toBeDisabled();expect(screen.getByLabelText('Image retry delay (seconds)')).toBeDisabled();
    expect(writes).toEqual([]);
  });
  it('writes through the settings policy API and preserves unrelated preferences', async () => {
    render(() => <Privacy />);
    const mode = await screen.findByLabelText('Remote content'); await waitFor(() => expect(mode).toBeEnabled());
    fireEvent.change(screen.getByLabelText('Potentially unsafe message HTML'),{target:{value:'strict'}});
    await waitFor(()=>expect(writes).toContainEqual({url:'/api/settings/mail.reader.unsafeHtmlMode',value:'strict'}));
    fireEvent.change(mode, { target: { value: 'proxy' } });
    await waitFor(() => expect(writes).toContainEqual({ url: '/api/settings/privacy.remoteContent', value: 'proxy' }));
    fireEvent.click(screen.getByLabelText('Block tracking images and beacons'));
    await waitFor(() => expect(writes).toContainEqual({ url: '/api/settings/privacy.trackers', value: false }));
    expect(screen.getByLabelText('Remove tracking parameters from links')).toBeChecked();
    const mailer = screen.getByLabelText('Remove outgoing mailer headers');
    expect(mailer).toBeChecked();
    fireEvent.click(mailer);
    await waitFor(() => expect(writes).toContainEqual({url:'/api/settings/privacy.stripMailerHeaders',value:false}));
  });
});
