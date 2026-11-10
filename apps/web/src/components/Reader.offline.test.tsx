import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,screen,waitFor,fireEvent} from '@solidjs/testing-library';
import {Reader} from './Reader.tsx';
import {renderWithApp,mkEmail} from './appHarness.tsx';
afterEach(()=>{cleanup();vi.restoreAllMocks();});
it('renders a cached body without the network bridge when the browser goes offline and restores it online',async()=>{
 let online=true;vi.spyOn(navigator,'onLine','get').mockImplementation(()=>online);
 const email=mkEmail('saved',{htmlBody:[{partId:'1',blobId:null,type:'text/html',size:20}],textBody:[{partId:'text',blobId:null,type:'text/plain',size:13}],bodyValues:{'1':{value:'<p>Saved message</p>',isEncodingProblem:false,isTruncated:false},text:{value:'Saved message',isEncodingProblem:false,isTruncated:false}}});
 const {app}=renderWithApp(()=><Reader/>,{emails:[email]});await app.login({username:'me@example.org',password:'p'});await app.openMessage('saved');
 await screen.findByText('Subject saved');
 online=false;fireEvent(window,new Event('offline'));
 const frame=await screen.findByTestId('reader-offline-frame');expect(frame).toHaveAttribute('sandbox','');expect(frame.getAttribute('srcdoc')).toContain('Saved message');expect(frame.getAttribute('srcdoc')).toContain("default-src 'none'");expect(frame.getAttribute('srcdoc')).not.toContain('img-src');
 online=true;fireEvent(window,new Event('online'));await waitFor(()=>expect(screen.queryByTestId('reader-offline-frame')).toBeNull());
});
