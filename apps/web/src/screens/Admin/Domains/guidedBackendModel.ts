import type {DomainBackend,DomainStatus} from '../../../api/admin/domains.ts';

export function newGuidedBackend(protocol:DomainBackend['protocol'],id:string):DomainBackend {
  return {id,protocol,role:protocol==='smtp'?'submission':protocol==='sieve'?'rules':'mail',enabled:true,preferred:false,priority:100,username:{format:'fullEmail'},...(protocol==='jmap'?{sessionUrl:''}:{endpoint:{host:'',port:protocol==='smtp'?465:protocol==='pop3'?995:protocol==='sieve'?4190:993,tls:protocol==='sieve'?'starttls' as const:'implicit' as const,auth:'password' as const,allowPrivate:false}})};
}
export function guidedHost(value:string):boolean {
  if(!value||value!==value.trim()||/[\s/@?#\\]/u.test(value))return false;
  try {const host=value.includes(':')&&!value.startsWith('[')?`[${value}]`:value;const u=new URL(`https://${host}`);return !!u.hostname&&!u.port&&u.pathname==='/'&&!u.username&&!u.password;}catch{return false;}
}
export function guidedSessionUrl(value:string):boolean {
  try {const u=new URL(value);return u.protocol==='https:'&&!!u.hostname&&!u.username&&!u.password&&!u.search&&!u.hash;}catch{return false;}
}
/** Client review guard; the server remains authoritative for TLS/CA and auth validation. */
export function guidedBackendProblem(rows:readonly DomainBackend[],status:DomainStatus|undefined):string|null {
  if(!status)return 'admin-domain-guided-status-required';
  if(!rows.length||rows.length>32)return 'admin-domain-guided-backend-count';
  const ids=new Set<string>(),preferred=new Set<string>();
  for(const row of rows){
    if(!/^[A-Za-z0-9_-]{1,64}$/u.test(row.id)||ids.has(row.id))return 'admin-domain-guided-backend-id';ids.add(row.id);
    if((status.mode==='proxy')!==(row.protocol==='jmap'))return 'admin-domain-guided-backend-mode';
    if(!Number.isInteger(row.priority??100)||(row.priority??100)<0||(row.priority??100)>65535)return 'admin-domain-guided-backend-priority';
    if(row.preferred){if(row.enabled===false||preferred.has(row.role))return 'admin-domain-guided-backend-preferred';preferred.add(row.role);}
    if(row.protocol==='jmap'){if(!guidedSessionUrl(row.sessionUrl??''))return 'admin-domain-guided-invalid-url';}
    else {
      const wire=row.endpoint;
      if(!wire||!guidedHost(wire.host)||!Number.isInteger(wire.port)||wire.port<1||wire.port>65535)return 'admin-domain-guided-invalid-endpoint';
      const security=wire.security;
      if(security&&(!status.transportSecurity?.runtimeReady||!security.allowed.length||new Set(security.allowed).size!==security.allowed.length||(security.mode==='fixed'&&security.allowed.length!==1)||(security.allowed.includes('none')&&!security.allowPlaintext)))return 'admin-domain-guided-backend-security';
      const auth=wire.authentication;
      if(auth&&(!status.authentication?.runtimeReady||!auth.mechanisms.length||new Set(auth.mechanisms).size!==auth.mechanisms.length||(auth.mode==='fixed'&&auth.mechanisms.length!==1)||auth.mechanisms.some(m=>!status.authentication?.protocols[row.protocol]?.includes(m))))return 'admin-domain-guided-backend-auth';
    }
    const username=row.username;
    if(username?.format==='domainUser'&&!username.domain.trim()||username?.format==='userRealm'&&!username.realm.trim()||username?.format==='custom'&&!username.template.trim())return 'admin-domain-guided-backend-username';
  }
  return null;
}
