import {expect,it,vi} from 'vitest';
import {render} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {FolderCountBadge} from './FolderCountBadge.tsx';
const state=vi.hoisted(()=>({read:():Record<string,{value:unknown}>=>({})}));
vi.mock('../api/settings.ts',()=>({settings:()=>state.read()}));
it('reacts to the effective count mode and compact badge policy',()=>{
 const [mode,setMode]=createSignal('unread'),[compact,setCompact]=createSignal(true);state.read=()=>({'mail.folderCounts':{value:mode()},'mail.compactFolderBadges':{value:compact()}});
 const {container}=render(()=><FolderCountBadge folder={{role:'archive',unreadEmails:2,totalEmails:7}}/>);
 expect(container.textContent).toBe('2');expect(container.firstElementChild).toHaveAttribute('data-secondary','true');
 setMode('total');setCompact(false);expect(container.textContent).toBe('7');expect(container.firstElementChild).not.toHaveAttribute('data-secondary');
 setMode('both');expect(container.textContent).toBe('2 / 7');setMode('none');expect(container).toBeEmptyDOMElement();
});

// t39: scope (all / main folders) x mode, with no unread and with unread mail.
const MODES=['none','unread','unreadIfAny','total','both'] as const;
const expected:Record<(typeof MODES)[number],[string,string]>={none:['',''],unread:['0','3'],unreadIfAny:['','3'],total:['7','7'],both:['0 / 7','3 / 7']};
for(const scope of ['all','main'] as const)for(const mode of MODES)for(const role of ['inbox',null] as const)for(const unread of [0,3]){
 it(`${scope} folders, ${mode}, ${role??'custom'} folder, ${unread} unread`,()=>{
  state.read=()=>({'mail.folderCounts':{value:mode},'mail.folderCountScope':{value:scope}});
  const {container}=render(()=><FolderCountBadge folder={{role,unreadEmails:unread,totalEmails:7}}/>);
  const want=scope==='main'&&role===null?'':expected[mode][unread===0?0:1];
  expect(container.textContent).toBe(want);
  if(want==='')expect(container).toBeEmptyDOMElement();
 });
}
