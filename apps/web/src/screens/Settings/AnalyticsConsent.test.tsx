import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { AnalyticsConsent } from './AnalyticsConsent.tsx';
import { setAccountUiPreferences } from '../../api/accountPreferences.ts';
let consent=false;
let privacy=false;
vi.mock('../../api/accountPreferences.ts',()=>({accountUiPreferences:()=>({analyticsConsent:consent}),accountUiPreferenceStatus:()=> 'synced',setAccountUiPreferences:vi.fn()}));
vi.mock('../../api/analytics.ts',()=>({analyticsPrivacySignal:()=>privacy}));
beforeEach(()=>{vi.clearAllMocks();consent=false;privacy=false;});
describe('analytics consent',()=>{
  it('starts unchecked and changes only the consent field',async()=>{
    render(()=><AnalyticsConsent/>);
    const consent=await screen.findByRole('checkbox');expect(consent).not.toBeChecked();
    fireEvent.click(consent);
    expect(setAccountUiPreferences).toHaveBeenCalledWith({analyticsConsent:true});
  });
  it('lets a user withdraw existing consent while a browser privacy signal is active',async()=>{
    consent=true;privacy=true;render(()=><AnalyticsConsent/>);
    const checkbox=await screen.findByRole('checkbox');expect(checkbox).toBeEnabled();
    fireEvent.click(checkbox);expect(setAccountUiPreferences).toHaveBeenCalledWith({analyticsConsent:false});
  });
});
