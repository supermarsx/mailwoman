import {afterEach, expect, it} from 'vitest';
import {render, waitFor} from '@solidjs/testing-library';
import {clearSettings, loadSettings} from '../api/settings.ts';
import {useInterfacePreferences} from './interfacePreferences.ts';
afterEach(()=>{clearSettings();document.documentElement.removeAttribute('style');});
it('applies policy text settings and clears them on account reset and unmount',async()=>{
 const values:Record<string,unknown>={'appearance.fontSize':20,'appearance.fontWeight':'500','appearance.fontColor':'#123abc','appearance.lowContrast':true};
 await loadSettings({fetcher:async()=>new Response(JSON.stringify({settings:Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'domain',locked:true}]))}))});
 const view=render(()=>{useInterfacePreferences();return null;});const style=document.documentElement.style;
 await waitFor(()=>expect(style.getPropertyValue('--mw-ui-font-size')).toBe('20px'));
 expect(style.getPropertyValue('--mw-ui-font-weight')).toBe('500');expect(style.getPropertyValue('--mw-ui-text-color')).toBe('#123abc');expect(style.getPropertyValue('--mw-ui-text-mix')).toBe('82%');
 clearSettings();await waitFor(()=>expect(style.getPropertyValue('--mw-ui-text-color')).toBe(''));view.unmount();expect(style.getPropertyValue('--mw-ui-font-size')).toBe('');
});
