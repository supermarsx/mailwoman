import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { AdminAttachmentPolicy } from './AttachmentPolicy.tsx';
import { renderInShell } from './shell/testkit.tsx';
import type { AdminSetting } from '../../api/admin/settings.ts';
const setting: AdminSetting = { key: 'attachments.downloadAllowed', family: false, kind: { type: 'bool' }, default: true, nullable: false, scopes: 'global+domain+user', userOverride: true, category: 'attachments', mapsTo: '', env: null, dedicatedRoute: null, scopeId: 'account', hot: true, ceiling: false, userVisible: true, globalValue: false, allowUserOverride: false, globalLocked: true, updatedBy: 'root', updatedAt: 1, effective: { value: false, source: 'global' }, envState: null, overrides: {}, etag: '"v1"' };
describe('attachment policy overview',()=>{
  it('uses real settings values and links to the single scoped policy editor',async()=>{
    const load=vi.fn(async()=>({kind:'ok' as const,data:[setting],etag:null}));
    const {navigate}=renderInShell(()=><AdminAttachmentPolicy load={load}/>);
    await screen.findByText('Allow original attachment delivery');
    fireEvent.click(screen.getByRole('link',{name:'Edit policy'}));
    expect(navigate).toHaveBeenCalledWith('/admin#/settings/attachments.downloadAllowed');
  });
  it('reports unavailable configuration rather than inventing policy defaults',async()=>{
    renderInShell(()=><AdminAttachmentPolicy load={async()=>({kind:'unavailable'})}/>);
    expect(await screen.findByRole('alert')).toHaveTextContent('not available');
    expect(screen.queryByRole('link',{name:'Edit policy'})).toBeNull();
  });
});
