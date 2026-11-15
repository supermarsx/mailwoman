import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {afterEach,expect,it,vi} from 'vitest';
import {ProviderConnections,ProviderOauthCallback} from './ProviderConnections.tsx';
import {providerRequest,pendingProviderFlow,rememberProviderFlow,forgetProviderFlow,ProviderOauthError} from '../../api/providerOauth.ts';
vi.mock('../../i18n',()=>({t:(key:string)=>key,loadCatalog:async()=>undefined}));
vi.mock('../../api/providerOauth.ts',()=>({ProviderOauthError:class extends Error{constructor(readonly code:string){super(code);}},providerRequest:vi.fn(),pendingProviderFlow:vi.fn(()=> 'flow'),rememberProviderFlow:vi.fn(),forgetProviderFlow:vi.fn(),providerHttpsUrl:(value:string)=>value?.startsWith('https://')?value:undefined}));
afterEach(()=>{cleanup();history.replaceState(null,'','/');vi.clearAllMocks();vi.useRealTimers();});
it('ignores a rejected old poll after cancellation and a new device flow',async()=>{
 vi.useFakeTimers();let starts=0,rejectOld!:(error:Error)=>void;
 vi.mocked(providerRequest).mockImplementation(async(path='',method='GET')=>{
  if(path==='/start'){const n=++starts;return {flowId:`flow-${n}`,userCode:`CODE-${n}`,verificationUri:'https://microsoft.com/devicelogin',expiresIn:600,interval:5} as never;}
  if(path==='/complete')return await new Promise((_resolve,reject)=>{rejectOld=reject;}) as never;
  if(method==='DELETE')return undefined;
  return {configured:false,connected:false,flows:['authorizationCode','deviceCode']} as never;
 });
 render(()=><ProviderConnections/>);await Promise.resolve();await Promise.resolve();
 fireEvent.input(screen.getByLabelText('provider-oauth-client-id'),{target:{value:'app-id'}});fireEvent.click(screen.getByRole('button',{name:'provider-oauth-device'}));
 await vi.advanceTimersByTimeAsync(5000);expect(rejectOld).toBeTypeOf('function');
 fireEvent.click(screen.getByRole('button',{name:'common-cancel'}));await Promise.resolve();await Promise.resolve();fireEvent.click(screen.getByRole('button',{name:'provider-oauth-device'}));
 await Promise.resolve();await Promise.resolve();expect(screen.getByText('CODE-2')).toBeVisible();
 rejectOld(new Error('old request failed'));await Promise.resolve();await Promise.resolve();
 expect(screen.getByText('CODE-2')).toBeVisible();
});
it('keeps correlation after cancellation is refused during commit and finishes by polling',async()=>{
 vi.useFakeTimers();let release!:(value:{status:string})=>void;
 vi.mocked(providerRequest).mockImplementation(async(path='',method='GET')=>{
  if(path==='/start')return {flowId:'committing',userCode:'SAVE-CODE',verificationUri:'https://microsoft.com/devicelogin',expiresIn:600,interval:5} as never;
  if(path==='/complete')return await new Promise(resolve=>{release=resolve;}) as never;
  if(method==='DELETE')throw new ProviderOauthError('provider_committing');
  return {configured:false,connected:false,flows:['authorizationCode','deviceCode']} as never;
 });
 render(()=><ProviderConnections/>);await Promise.resolve();await Promise.resolve();
 fireEvent.input(screen.getByLabelText('provider-oauth-client-id'),{target:{value:'app-id'}});fireEvent.click(screen.getByRole('button',{name:'provider-oauth-device'}));
 await vi.advanceTimersByTimeAsync(5000);expect(release).toBeTypeOf('function');
 fireEvent.click(screen.getByRole('button',{name:'common-cancel'}));expect(screen.getByRole('button',{name:'common-cancel'})).toBeDisabled();
 await Promise.resolve();await Promise.resolve();expect(screen.getByText('SAVE-CODE')).toBeVisible();expect(forgetProviderFlow).not.toHaveBeenCalled();
 await vi.advanceTimersByTimeAsync(10000);expect(vi.mocked(providerRequest).mock.calls.filter(call=>call[0]==='/complete')).toHaveLength(1);
 release({status:'connected'});await Promise.resolve();await Promise.resolve();
 expect(screen.getByText('provider-oauth-connected')).toBeVisible();expect(forgetProviderFlow).toHaveBeenCalledOnce();
});
it('does not clear a completed flow or report a late cancellation error after completion wins',async()=>{
 vi.useFakeTimers();let complete!:(value:{status:string})=>void,rejectCancel!:(error:Error)=>void;
 vi.mocked(providerRequest).mockImplementation(async(path='',method='GET')=>{
  if(path==='/start')return {flowId:'finishing',userCode:'FINISH-CODE',verificationUri:'https://microsoft.com/devicelogin',expiresIn:600,interval:5} as never;
  if(path==='/complete')return await new Promise(resolve=>{complete=resolve;}) as never;
  if(method==='DELETE')return await new Promise((_resolve,reject)=>{rejectCancel=reject;}) as never;
  return {configured:false,connected:false,flows:['authorizationCode','deviceCode']} as never;
 });
 render(()=><ProviderConnections/>);await Promise.resolve();await Promise.resolve();
 fireEvent.input(screen.getByLabelText('provider-oauth-client-id'),{target:{value:'app-id'}});fireEvent.click(screen.getByRole('button',{name:'provider-oauth-device'}));
 await vi.advanceTimersByTimeAsync(5000);fireEvent.click(screen.getByRole('button',{name:'common-cancel'}));complete({status:'connected'});await Promise.resolve();await Promise.resolve();
 expect(screen.getByText('provider-oauth-connected')).toBeVisible();rejectCancel(new ProviderOauthError('provider_committing'));await Promise.resolve();await Promise.resolve();
 expect(screen.getByText('provider-oauth-connected')).toBeVisible();expect(forgetProviderFlow).toHaveBeenCalledOnce();
});
it('starts Microsoft device authorization and cancels the exact pending flow',async()=>{
 vi.mocked(providerRequest).mockImplementation(async(path='',method='GET')=>path==='/start'?{flowId:'flow',userCode:'ABCD',verificationUri:'https://microsoft.com/devicelogin',expiresIn:600,interval:5}:method==='DELETE'?undefined:{configured:false,connected:false,flows:['authorizationCode','deviceCode']} as never);
 render(()=><ProviderConnections/>);await waitFor(()=>expect(screen.getByLabelText('provider-oauth-client-id')).toBeEnabled());fireEvent.input(screen.getByLabelText('provider-oauth-client-id'),{target:{value:'app-id'}});fireEvent.click(screen.getByRole('button',{name:'provider-oauth-device'}));
 await screen.findByText('ABCD');expect(rememberProviderFlow).toHaveBeenCalledWith(expect.objectContaining({flowId:'flow'}));expect(screen.getByRole('link',{name:'common-open'})).toHaveAttribute('rel','noopener noreferrer');fireEvent.click(screen.getByRole('button',{name:'common-cancel'}));await waitFor(()=>expect(providerRequest).toHaveBeenCalledWith('/flow','DELETE',undefined,expect.any(AbortSignal)));
});
it('does not offer Gmail device flow or reuse another providers client ID',async()=>{
 vi.mocked(providerRequest).mockResolvedValue({bridgeId:'bridge-graph',configured:true,connected:true,clientId:'microsoft-app',flows:['authorizationCode','deviceCode']} as never);render(()=><ProviderConnections/>);await waitFor(()=>expect(screen.getByLabelText('provider-oauth-client-id')).toHaveAttribute('placeholder','microsoft-app'));
 fireEvent.input(screen.getByLabelText('provider-oauth-client-id'),{target:{value:'manually-entered-ms-app'}});fireEvent.input(screen.getByLabelText('provider-oauth-tenant'),{target:{value:'my-tenant'}});
 fireEvent.change(screen.getByLabelText('provider-oauth-provider'),{target:{value:'bridge-gmail'}});expect(screen.queryByRole('button',{name:'provider-oauth-device'})).toBeNull();expect(screen.getByRole('button',{name:'provider-oauth-connect'})).toBeDisabled();expect(screen.getByLabelText('provider-oauth-client-id')).toHaveValue('');
});
it('uses the configured Graph tenant when its unchanged value is shown as a placeholder',async()=>{
 vi.mocked(providerRequest).mockImplementation(async(path='')=>path==='/start'?{flowId:'configured',userCode:'CONFIGURED',verificationUri:'https://microsoft.com/devicelogin',expiresIn:600,interval:5}:{bridgeId:'bridge-graph',clientId:'tenant-app',tenant:'configured-tenant',connected:false,configured:true,flows:['authorizationCode','deviceCode']} as never);
 render(()=><ProviderConnections/>);await waitFor(()=>expect(screen.getByLabelText('provider-oauth-tenant')).toHaveAttribute('placeholder','configured-tenant'));
 fireEvent.click(screen.getByRole('button',{name:'provider-oauth-device'}));await screen.findByText('CONFIGURED');
 expect(providerRequest).toHaveBeenCalledWith('/start','POST',{bridgeId:'bridge-graph',clientId:'tenant-app',tenant:'configured-tenant',flow:'deviceCode'},expect.any(AbortSignal));
});
it('completes a correlated callback and removes authorization values from the URL before dispatch',async()=>{
 history.replaceState(null,'','/?provider-oauth-callback=1&code=secret-code&state=opaque');vi.mocked(providerRequest).mockImplementation(async(path,_method,body)=>{expect(location.search).toBe('');expect(path).toBe('/complete');expect(body).toEqual({flowId:'flow',code:'secret-code',state:'opaque'});return {status:'connected'} as never;});render(()=><ProviderOauthCallback/>);await screen.findByText('provider-oauth-connected');
});
it('refuses callbacks without local correlation and never exchanges the code',()=>{
 vi.mocked(pendingProviderFlow).mockReturnValueOnce(undefined);history.replaceState(null,'','/?provider-oauth-callback=1&code=secret-code&state=opaque');render(()=><ProviderOauthCallback/>);expect(screen.getByText('provider-oauth-failed')).toBeVisible();expect(providerRequest).not.toHaveBeenCalled();expect(location.search).toBe('');
});
it('refuses a callback without state and scrubs its code before showing failure',()=>{
 history.replaceState(null,'','/?provider-oauth-callback=1&code=secret-code');render(()=><ProviderOauthCallback/>);
 expect(screen.getByText('provider-oauth-failed')).toBeVisible();expect(providerRequest).not.toHaveBeenCalled();expect(location.search).toBe('');
});
