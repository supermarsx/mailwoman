import {afterEach, describe, expect, it, vi} from 'vitest';
import {render, waitFor} from '@solidjs/testing-library';
import {AppearanceCustomizationRuntime, CUSTOMIZATION_CHANGED} from './AppearanceCustomizationRuntime.tsx';
import {loadCustomization, type CustomizationSnapshot} from '../api/appearanceCustomization.ts';
import {mountAppearanceCss, runAppearanceScript} from '../theme/appearanceSandbox.ts';
vi.mock('../api/appearanceCustomization.ts',()=>({loadCustomization:vi.fn()}));
vi.mock('../theme/appearanceSandbox.ts',()=>({mountAppearanceCss:vi.fn(),runAppearanceScript:vi.fn()}));
afterEach(()=>vi.resetAllMocks());
const initial:CustomizationSnapshot={etag:'"unchanged-document"',value:{ownerId:'one',document:{version:1,css:'body{color:red}',script:''},compiledCss:'.mw-custom-mail{color:red}',cssEnabled:true,cssAllowed:true,scriptEnabled:false,lockedFields:[],scopeClass:'mw-custom-mail'}};
describe('appearance runtime authority refresh',()=>{
 it('removes applied CSS when policy changes without changing the document ETag',async()=>{
  const remove=vi.fn();vi.mocked(mountAppearanceCss).mockReturnValue(remove);vi.mocked(loadCustomization).mockResolvedValue(initial);
  const view=render(()=><AppearanceCustomizationRuntime owner={()=>({kind:'mailbox',accountId:'one'})}/>);
  await waitFor(()=>expect(mountAppearanceCss).toHaveBeenCalledTimes(1));
  vi.mocked(loadCustomization).mockResolvedValue({...initial,value:{...initial.value,cssEnabled:false,lockedFields:['accent'],compiledCss:''}});
  window.dispatchEvent(new Event(CUSTOMIZATION_CHANGED));
  await waitFor(()=>expect(remove).toHaveBeenCalledTimes(1));
  expect(mountAppearanceCss).toHaveBeenCalledTimes(1);expect(runAppearanceScript).not.toHaveBeenCalled();view.unmount();
 });
 it('removes customizations on failed revalidation and on unmount',async()=>{
  const remove=vi.fn();vi.mocked(mountAppearanceCss).mockReturnValue(remove);vi.mocked(loadCustomization).mockResolvedValue(initial);
  const view=render(()=><AppearanceCustomizationRuntime owner={()=>({kind:'mailbox',accountId:'one'})}/>);
  await waitFor(()=>expect(mountAppearanceCss).toHaveBeenCalled());
  vi.mocked(loadCustomization).mockRejectedValue(new Error('unavailable'));window.dispatchEvent(new Event(CUSTOMIZATION_CHANGED));
  await waitFor(()=>expect(remove).toHaveBeenCalledTimes(1));view.unmount();
 });
});
