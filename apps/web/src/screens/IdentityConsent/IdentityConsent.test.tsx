import { it,expect,vi } from 'vitest';
import { render,screen,fireEvent,waitFor } from '@solidjs/testing-library';
import IdentityConsent from './index.tsx';
const {request}=vi.hoisted(()=>({request:vi.fn()}));
vi.mock('../../api/http.ts',async original=>({...await original<typeof import('../../api/http.ts')>(),apiFetch:request}));
const context={loginRequired:false,clientName:'Independent RP',account:'user@example.org',scopes:['openid','email'],redirectUri:'https://rp.example/callback',csrfToken:'bound-proof'};
it('requires an explicit choice and sends only the bound flow and CSRF proof',async()=>{
 history.replaceState(null,'','/oidc/consent?flow=opaque-flow');request.mockReset();request.mockImplementation(async(_url:string,init?:RequestInit)=>new Response(JSON.stringify(init?.method==='POST'?{redirectUri:'https://rp.example/callback?code=opaque'}:context),{status:200}));const redirect=vi.fn();render(()=><IdentityConsent onRedirect={redirect}/>);
 await screen.findByText('Independent RP');expect(request).toHaveBeenCalledTimes(1);fireEvent.click(screen.getByRole('button',{name:'Allow identity access'}));await waitFor(()=>expect(redirect).toHaveBeenCalledWith('https://rp.example/callback?code=opaque'));
 expect(request.mock.calls[1]?.[1]).toMatchObject({method:'POST',headers:{'content-type':'application/json','x-csrf-token':'bound-proof'},body:JSON.stringify({flow:'opaque-flow',approve:true})});
});
it('retains the request on failure without automatically resubmitting a decision',async()=>{
 history.replaceState(null,'','/oidc/consent?flow=other-flow');request.mockReset();request.mockImplementation(async(_url:string,init?:RequestInit)=>new Response(JSON.stringify(init?.method==='POST'?{error:'expired_request'}:context),{status:init?.method==='POST'?400:200}));render(()=><IdentityConsent/>);await screen.findByText('Independent RP');fireEvent.click(screen.getByRole('button',{name:'Cancel'}));await screen.findByRole('alert');expect(request.mock.calls.filter(call=>call[1]?.method==='POST')).toHaveLength(1);expect(screen.getByRole('button',{name:'Cancel'})).toBeEnabled();
});
