import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {beforeEach,it,expect,vi} from 'vitest';
import {onCleanup} from 'solid-js';
import {SignatureManager} from './SignatureManager.tsx';
import {useSettingsNavigation} from '../../screens/Settings/navigation.tsx';
const state=vi.hoisted(()=>({allowed:false,checks:0}));
vi.mock('../../screens/Settings/Signatures.tsx',()=>({default:()=>{
 const navigation=useSettingsNavigation()!;
 onCleanup(navigation.beforeLeave(()=>{state.checks++;return state.allowed;},()=>!state.allowed));
 return <button type="button">Signature editor</button>;
}}));
beforeEach(()=>{state.allowed=false;state.checks=0;});
it('keeps the composer-local manager open when its signature editor refuses an unsaved close',async()=>{
 const close=vi.fn();render(()=><SignatureManager document={document} onClose={close}/>);
 await screen.findByRole('button',{name:'Signature editor'});
 fireEvent.click(screen.getByRole('button',{name:'Close'}));await waitFor(()=>expect(state.checks).toBe(1));expect(close).not.toHaveBeenCalled();
 await new Promise(resolve=>setTimeout(resolve,0));
 state.allowed=true;fireEvent.click(screen.getByRole('button',{name:'Close'}));await waitFor(()=>expect(close).toHaveBeenCalledOnce());
});
it('runs the same unsaved guard for Escape',async()=>{
 const close=vi.fn();render(()=><SignatureManager document={document} onClose={close}/>);
 await screen.findByRole('button',{name:'Signature editor'});
 fireEvent.keyDown(screen.getByTestId('signature-manager'),{key:'Escape'});await waitFor(()=>expect(state.checks).toBe(1));expect(close).not.toHaveBeenCalled();
 await new Promise(resolve=>setTimeout(resolve,0));
 state.allowed=true;fireEvent.keyDown(screen.getByTestId('signature-manager'),{key:'Escape'});await waitFor(()=>expect(close).toHaveBeenCalledOnce());
});
