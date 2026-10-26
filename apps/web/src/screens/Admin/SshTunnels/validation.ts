import type {SshTunnelConfig} from '../../../api/admin/ssh-tunnels.ts';
export interface TunnelIssue {field:string;key:string}
export function validSshHost(value:string):boolean {
 if(!value||value.length>253||/[\s/@\\*?#[\]]/.test(value))return false;
 try {const parsed=new URL(`http://${value.includes(':')?`[${value}]`:value}`);return !!parsed.hostname&&!parsed.port&&!parsed.username&&parsed.pathname==='/';}catch{return false;}
}
const controls=(value:string)=>[...value].some(char=>char.charCodeAt(0)<32||char.charCodeAt(0)===127);
const within=(value:number,min:number,max:number)=>Number.isInteger(value)&&value>=min&&value<=max;
export function validateTunnel(config:SshTunnelConfig):TunnelIssue[]{
 const issues:TunnelIssue[]=[];const add=(field:string,key:string)=>issues.push({field,key});
 if(!config.name.trim()||new TextEncoder().encode(config.name).length>128||controls(config.name))add('name','admin-ssh-validation-name');
 if(!validSshHost(config.host))add('host','admin-ssh-validation-host');
 if(!within(config.port,1,65535))add('port','admin-ssh-validation-port');
 if(!config.username.trim()||new TextEncoder().encode(config.username).length>128||controls(config.username))add('username','admin-ssh-validation-user');
 if(!within(config.keepaliveSeconds,10,300))add('keepaliveSeconds','admin-ssh-validation-keepalive');
 if(!within(config.connectTimeoutMs,100,60000))add('connectTimeoutMs','admin-ssh-validation-timeout');
 if(!within(config.maxChannels,1,128))add('maxChannels','admin-ssh-validation-channels');
 if(config.hostKeyPins.length>16||config.hostKeyPins.some(pin=>!['ssh-ed25519','ecdsa-sha2-nistp256','ecdsa-sha2-nistp384','ssh-rsa'].includes(pin.algorithm)||!/^SHA256:[A-Za-z0-9+/]{42}[AEIMQUYcgkosw048]$/.test(pin.sha256)))add('hostKeyPins','admin-ssh-validation-pin');
 const targets=new Set<string>();
 if(config.targets.length>128||config.targets.some(target=>{const key=`${target.host.toLowerCase()}:${target.port}`;const duplicate=targets.has(key);targets.add(key);return duplicate||!validSshHost(target.host)||!within(target.port,1,65535);} ))add('targets','admin-ssh-validation-target');
 if(config.enabled&&(!config.targets.length||!config.hostKeyPins.length))add('enabled','admin-ssh-validation-enabled');
 return issues;
}
