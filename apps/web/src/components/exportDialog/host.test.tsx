import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {render,screen} from '@solidjs/testing-library';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {installLinkedMailbox} from '../../api/linkedMailboxSession.ts';
import {clearSettings} from '../../api/settings.ts';
import type {Email} from '../../api/jmap-types.ts';
import {ExportDialogHost,openExportDialog,closeExportDialog,exportDialogOpen} from './host.tsx';
vi.mock('./ExportDialog.tsx',()=>({ExportDialog:()=> <section role="dialog" aria-label="Export fixture"/>}));
const request={source:{message:[{id:'message'} as Email]}};
beforeEach(()=>{clearSettings();closeExportDialog();activateAccountPreferences('same-account');});
afterEach(()=>{closeExportDialog();suspendAccountPreferences();});
it('drops an open request on host unmount instead of replaying it in a new host',async()=>{
 const first=render(()=><ExportDialogHost/>);expect(openExportDialog(request)).toBe(true);await screen.findByRole('dialog');
 first.unmount();expect(exportDialogOpen()).toBe(false);
 render(()=><ExportDialogHost/>);expect(screen.queryByRole('dialog')).toBeNull();
});
it('drops a request on authority replacement before the lazy dialog mounts',()=>{
 render(()=><ExportDialogHost/>);openExportDialog(request);suspendAccountPreferences();activateAccountPreferences('same-account');
 expect(exportDialogOpen()).toBe(false);
});
it('drops a request on linked mailbox replacement before the lazy dialog mounts',()=>{
 render(()=><ExportDialogHost/>);openExportDialog(request);installLinkedMailbox(null);
 expect(exportDialogOpen()).toBe(false);
});
