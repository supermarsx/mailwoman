import {it,expect,vi} from 'vitest';
import {fireEvent,screen} from '@solidjs/testing-library';
import {AuthLogs,type AuthLogsApi} from './AuthLogs.tsx';
import {ApplicationLogs,type ApplicationLogsApi} from './ApplicationLogs.tsx';
import {renderInShell} from './shell/testkit.tsx';

it.each(['auth','application'] as const)('separates %s pending and failed history from an empty result',async(kind)=>{
 let reject!:(cause:unknown)=>void;
 const auth=vi.fn<AuthLogsApi['history']>(()=>new Promise((_resolve,no)=>{reject=no;}));
 const application=vi.fn<ApplicationLogsApi['history']>(()=>new Promise((_resolve,no)=>{reject=no;}));
 renderInShell(()=>kind==='auth'?<AuthLogs api={{history:auth}}/>:<ApplicationLogs api={{history:application}}/>);
 const empty=kind==='auth'?'No matching attempts.':'No matching warnings or errors in this process buffer.';
 expect(screen.getByTestId('async-pending').querySelector('[data-skeleton="table"]')).not.toBeNull();
 expect(screen.queryByText(empty)).not.toBeInTheDocument();
 reject(new Error('history refused'));
 await vi.waitFor(()=>expect(screen.queryByTestId('async-pending')).not.toBeInTheDocument());
 expect(screen.getByRole('alert')).toBeInTheDocument();
 expect(screen.queryByText(empty)).not.toBeInTheDocument();
 auth.mockResolvedValue({kind:'ok',etag:null,data:{events:[],cursor:null,retentionDays:7}});
 application.mockResolvedValue({kind:'ok',etag:null,data:{events:[],cursor:null,discarded:0,capacity:1000,retentionSeconds:86400,processLocal:true}});
 fireEvent.click(screen.getByRole('button',{name:'Apply filters'}));
 expect(await screen.findByText(empty)).toBeInTheDocument();
});
