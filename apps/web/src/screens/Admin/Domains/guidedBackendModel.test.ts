import {expect,it} from 'vitest';
import {guidedBackendProblem,guidedHost,newGuidedBackend} from './guidedBackendModel.ts';
import {domainBackends} from '../../../api/admin/domains.ts';
import type {DomainStatus} from '../../../api/admin/domains.ts';
const status:DomainStatus={mode:'engine',upstreamChoices:[],blockedAccounts:{total:0,byDomain:[]},transportSecurity:{runtimeReady:true,auto:true,customCa:true,serverName:true,protocols:['imap']},authentication:{runtimeReady:true,protocols:{imap:['plain','login']}}};
const valid=()=>({...newGuidedBackend('imap','primary'),endpoint:{host:'mail.example.test',port:993,tls:'implicit' as const,auth:'password' as const}});
it('checks bounded IDs, priorities and preferred service conflicts before final review',()=>{
 expect(guidedBackendProblem([valid()],status)).toBeNull();
 expect(guidedBackendProblem([],status)).toContain('count');
 expect(guidedBackendProblem([valid(),valid()],status)).toContain('id');
 expect(guidedBackendProblem([{...valid(),priority:-1}],status)).toContain('priority');
 expect(guidedBackendProblem([{...valid(),preferred:true},{...valid(),id:'second',preferred:true}],status)).toContain('preferred');
});
it('never treats a JMAP provider as an engine socket backend or invents unsupported mechanisms',()=>{
 expect(guidedBackendProblem([{...newGuidedBackend('jmap','remote'),sessionUrl:'https://mail.test/session'}],status)).toContain('mode');
 const row=valid();expect(guidedBackendProblem([{...row,endpoint:{...row.endpoint,authentication:{mode:'fixed',mechanisms:['scram-sha-256-plus']}}}],status)).toContain('auth');
 expect(guidedBackendProblem([{...row,endpoint:{...row.endpoint,authentication:{mode:'auto',mechanisms:['plain','plain']}}}],status)).toContain('auth');
});
it('requires explicit plaintext permission and rejects duplicate transport order',()=>{
 const row=valid();
 expect(guidedBackendProblem([{...row,endpoint:{...row.endpoint,security:{mode:'auto',allowed:['implicit','none'],allowPlaintext:false}}}],status)).toContain('security');
 expect(guidedBackendProblem([{...row,endpoint:{...row.endpoint,security:{mode:'auto',allowed:['implicit','implicit'],allowPlaintext:false}}}],status)).toContain('security');
});
it('accepts IPv6 host fields but refuses embedded credentials, paths and ports',()=>{
 expect(guidedHost('2001:db8::1')).toBe(true);expect(guidedHost('[2001:db8::1]')).toBe(true);
 for(const host of ['mail.test:443','secret@mail.test','mail.test/path',' mail.test'])expect(guidedHost(host)).toBe(false);
});

it('validates the actual EWS HTTPS Basic contract and selected service conflicts',()=>{
 const ews={...newGuidedBackend('ews','exchange'),priority:1,ews:{endpointUrl:'https://exchange.example.test/EWS/Exchange.asmx',authentication:'basic' as const}};
 expect(guidedBackendProblem([ews],status)).toBeNull();
 for(const endpointUrl of ['http://exchange.test/EWS','https://user:secret@exchange.test/EWS','https://exchange.test/EWS?token=x','https://exchange.test/EWS#x','https://exchange.test/EWS?','https://exchange.test/\nEWS'])expect(guidedBackendProblem([{...ews,ews:{...ews.ews,endpointUrl}}],status)).toBe('backend-ews-invalid');
 expect(guidedBackendProblem([{...ews,ews:{...ews.ews,authentication:'ntlm' as 'basic'}}],status)).toBe('backend-ews-invalid');
 expect(guidedBackendProblem([{...ews,ews:{...ews.ews,caPem:'-----BEGIN PRIVATE KEY-----'}}],status)).toBe('backend-ews-invalid-ca');
 const smtp={...newGuidedBackend('smtp','sender'),endpoint:{host:'smtp.test',port:465,tls:'implicit' as const,auth:'password' as const}};
 expect(guidedBackendProblem([ews,smtp],status)).toBe('backend-ews-service-conflict');
 expect(guidedBackendProblem([ews,{...smtp,enabled:false}],status)).toBeNull();
 expect(guidedBackendProblem([ews,{...valid(),preferred:true},smtp],status)).toBeNull();
 expect(guidedBackendProblem([ews],{...status,mode:'proxy'})).toContain('mode');
});

it('preserves legacy EWS configuration when upgrading to backend lists and bounds CA drafts',()=>{
 const cert='-----BEGIN CERTIFICATE-----\nYWJj\n-----END CERTIFICATE-----';
 const config={endpointUrl:'https://exchange.example.test/EWS/Exchange.asmx',authentication:'basic' as const,caPem:cert};
 const rows=domainBackends({ews:config});expect(rows[0]).toMatchObject({id:'legacy-ews',protocol:'ews',ews:config});
 expect(guidedBackendProblem(rows,status)).toBeNull();
 rows[0]!.ews!.caPem=cert.repeat(9);expect(guidedBackendProblem(rows,status)).toBe('backend-ews-invalid-ca');
 rows[0]!.ews!.caPem='x'.repeat(65537);expect(guidedBackendProblem(rows,status)).toBe('backend-ews-invalid-ca');
 expect(config.caPem).toBe(cert);
});
