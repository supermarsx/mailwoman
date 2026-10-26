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
