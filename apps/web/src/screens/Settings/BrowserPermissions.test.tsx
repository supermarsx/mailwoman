import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { BrowserPermissions } from './BrowserPermissions.tsx';
import { activateAccountPreferences, suspendAccountPreferences } from '../../api/accountPreferenceLifecycle.ts';
import { t } from '../../test/i18n.ts';
void t;
afterEach(() => { vi.unstubAllGlobals(); suspendAccountPreferences(); delete (navigator as unknown as { storage?: unknown }).storage; });
describe('explicit browser storage permissions', () => {
  it('reports real status and only requests persistence on an allowed click', async () => {
    const persist=vi.fn(async()=>true);
    Object.defineProperty(navigator,'storage',{configurable:true,value:{persist,persisted:async()=>false,estimate:async()=>({usage:1048576})}});
    render(() => <BrowserPermissions enabled={false} pushAllowed={false} persistentStorageAllowed={true} />);
    await screen.findByText('Current site storage: 1.0 MiB.');
    expect(persist).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button',{name:'Request persistent storage'}));
    await waitFor(()=>expect(persist).toHaveBeenCalledOnce());
    expect(await screen.findByText('The browser has granted persistent storage for this site.')).toBeInTheDocument();
  });
  it('does not request storage when administrator policy refuses it', async () => {
    const persist=vi.fn(async()=>true);
    Object.defineProperty(navigator,'storage',{configurable:true,value:{persist,persisted:async()=>false,estimate:async()=>({})}});
    activateAccountPreferences('account');
    render(() => <BrowserPermissions enabled={true} pushAllowed={false} persistentStorageAllowed={false} />);
    expect(screen.getByRole('button',{name:'Request persistent storage'})).toBeDisabled();
    expect(persist).not.toHaveBeenCalled();
  });
});
