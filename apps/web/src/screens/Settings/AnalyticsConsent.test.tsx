import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { AnalyticsConsent } from './AnalyticsConsent.tsx';
import { setAccountUiPreferences } from '../../api/accountPreferences.ts';
let consent=false;
let privacy=false;
let tagConsent: string|undefined;
vi.mock('../../api/accountPreferences.ts',()=>({accountUiPreferences:()=>({analyticsConsent:consent,analyticsTagConsent:tagConsent}),accountUiPreferenceStatus:()=> 'synced',setAccountUiPreferences:vi.fn()}));
vi.mock('../../api/analytics.ts',()=>({analyticsPrivacySignal:()=>privacy}));
beforeEach(()=>{vi.clearAllMocks();consent=false;privacy=false;tagConsent=undefined;});
const KEY=`posthog:${'c'.repeat(64)}`;
const noTag=async()=>null;
const posthog=async()=>({provider:'posthog',consentKey:KEY,consent:false});
describe('analytics consent',()=>{
  it('starts unchecked and changes only the consent field',async()=>{
    render(()=><AnalyticsConsent loadTag={noTag}/>);
    const consent=await screen.findByRole('checkbox');expect(consent).not.toBeChecked();
    fireEvent.click(consent);
    expect(setAccountUiPreferences).toHaveBeenCalledWith({analyticsConsent:true});
  });
  it('lets a user withdraw existing consent while a browser privacy signal is active',async()=>{
    consent=true;privacy=true;render(()=><AnalyticsConsent loadTag={noTag}/>);
    const checkbox=await screen.findByRole('checkbox');expect(checkbox).toBeEnabled();
    fireEvent.click(checkbox);expect(setAccountUiPreferences).toHaveBeenCalledWith({analyticsConsent:false});
  });
});

// t63 F1: a browser tag is a separate, provider- and script-bound consent.
describe('browser analytics tag consent',()=>{
  it('is off even when counts are allowed, names the provider and binds the exact script',async()=>{
    consent=true;render(()=><AnalyticsConsent loadTag={posthog}/>);
    const tag=await screen.findByLabelText('Allow PostHog to run in this app for this account');
    expect(tag).not.toBeChecked();
    expect(screen.getByText(/messages and addresses shown on screen/)).toBeInTheDocument();
    fireEvent.click(tag);
    expect(setAccountUiPreferences).toHaveBeenCalledWith({analyticsTagConsent:KEY});
  });
  it('asks again when the provider or script changed',async()=>{
    tagConsent=`posthog:${'d'.repeat(64)}`;render(()=><AnalyticsConsent loadTag={posthog}/>);
    const tag=await screen.findByLabelText('Allow PostHog to run in this app for this account');
    expect(tag).not.toBeChecked();
    expect(screen.getByText(/changed since you last agreed/)).toBeInTheDocument();
  });
  it('withdraws by clearing the key',async()=>{
    tagConsent=KEY;render(()=><AnalyticsConsent loadTag={posthog}/>);
    const tag=await screen.findByLabelText('Allow PostHog to run in this app for this account');
    expect(tag).toBeChecked();
    fireEvent.click(tag);
    expect(setAccountUiPreferences).toHaveBeenCalledWith({analyticsTagConsent:''});
  });
  it('shows nothing extra when no tag is enabled',async()=>{
    render(()=><AnalyticsConsent loadTag={noTag}/>);
    await screen.findAllByRole('checkbox');
    expect(screen.queryByTestId('analytics-tag-consent')).not.toBeInTheDocument();
  });
});
