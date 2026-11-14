import {afterEach,expect,it} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {DEFAULT_FILE_PREVIEWS,FILE_PREVIEWS_KEY} from './filePreviewPreferences.ts';
import {ViewerToolbar} from './ViewerToolbar.tsx';
afterEach(clearSettings);
it('puts selected tools in one menu and never exposes hidden tools',async()=>{
 await loadSettings({fetcher:async()=>new Response(JSON.stringify({settings:{[FILE_PREVIEWS_KEY]:{value:{...DEFAULT_FILE_PREVIEWS,tools:{download:'menu',details:'hidden'}},source:'global',locked:true}}}))});
 let downloads=0;
 render(()=><ViewerToolbar scope="preview-test" actions={[{id:'download',label:'Download file',icon:'download',run:()=>{downloads++;}},{id:'details',label:'File details',icon:'info',run:()=>{}}]}/>);
 expect(screen.queryByRole('button',{name:'Download file'})).toBeNull();expect(screen.queryByRole('button',{name:'File details'})).toBeNull();
 fireEvent.click(screen.getByRole('button',{name:'More actions'}));
 expect(screen.queryByRole('menuitem',{name:'File details'})).toBeNull();fireEvent.click(screen.getByRole('menuitem',{name:'Download file'}));expect(downloads).toBe(1);
});
