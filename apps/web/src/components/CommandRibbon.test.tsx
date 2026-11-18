import {describe,it,expect,vi} from 'vitest';
import {createSignal,Show} from 'solid-js';
import {screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {CommandRibbonContext,createCommandRibbon,useFileRibbon} from './CommandRibbonContext.tsx';
import {Ribbon} from './Ribbon.tsx';
import {ViewerToolbar} from '../viewers/ViewerToolbar.tsx';
import {ReaderToolbar} from './ReaderToolbar.tsx';
import {renderWithApp,mkEmail} from './appHarness.tsx';
describe('centralized command ribbon',()=>{
 it('keeps file titles and live preview controls in the File ribbon, then restores inline controls in standard layout',async()=>{
  const [enabled,setEnabled]=createSignal(true),[wrap,setWrap]=createSignal(false);const bridge=createCommandRibbon(enabled);
  const File=()=>{const centralized=useFileRibbon(()=> 'notes.txt');return <section data-testid="file"><h2 hidden={centralized()}>notes.txt</h2><ViewerToolbar scope="file" actions={[{id:'text.wrap',label:'Wrap lines',icon:'file',pressed:wrap(),run:()=>{setWrap(value=>!value);}}]}/></section>;};
  renderWithApp(()=><CommandRibbonContext.Provider value={bridge}><Show when={enabled()}><Ribbon onCompose={()=>{}} onOpenSettings={()=>{}} preview={bridge.preview()} onPreviewTarget={bridge.setFileTarget}/></Show><File/></CommandRibbonContext.Provider>);
  const action=await screen.findByRole('button',{name:'Wrap lines'});
  expect(action.closest('[data-bar="mailRibbon"]')).not.toBeNull();expect(screen.getByTestId('file').querySelector('button')).toBeNull();
  expect(screen.getByRole('tab',{name:'File previews'})).toHaveAttribute('aria-selected','true');
  fireEvent.click(action);expect(wrap()).toBe(true);
  fireEvent.click(screen.getByRole('tab',{name:'Home'}));fireEvent.click(screen.getByRole('tab',{name:'File previews'}));expect(screen.getByRole('button',{name:'Wrap lines'})).toHaveAttribute('aria-pressed','true');
  setEnabled(false);expect(screen.getByTestId('file').querySelector('button')).toBe(screen.getByRole('button',{name:'Wrap lines'}));expect(screen.getByRole('heading',{name:'notes.txt'})).toBeVisible();
 });
 it('uses the actual reader reply handler and keeps the reader action bar out of Ribbon layout',async()=>{
  const bridge=createCommandRibbon(()=>true),reply=vi.fn(),email=mkEmail('message');
  renderWithApp(()=><CommandRibbonContext.Provider value={bridge}><Ribbon email={email} readerActions={bridge.reader()?.actions} onCompose={()=>{}} onOpenSettings={()=>{}}/><ReaderToolbar email={email} onReply={reply} onPrint={()=>{}} preparing={false} active links={[]} linksTruncated={false} linkScope="message" security={{value:'full-sanitized',floor:null,onChange:()=>{}}}/></CommandRibbonContext.Provider>);
  await waitFor(()=>expect(bridge.reader()).toBeDefined());
  expect(document.querySelector('.reader__toolbar')).toBeNull();fireEvent.click(screen.getByRole('button',{name:'Reply'}));
  await waitFor(()=>expect(reply).toHaveBeenCalledWith('reply'));
 });
});
