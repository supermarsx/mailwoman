import {t} from './index.ts';
/** Matches the server's geo_lookup::is_local, including mapped IPv4 and CGNAT. */
export function isInternalIp(value?:string|null):boolean {
 if(!value)return false;let ip=value.trim();
 const localV4=(octets:number[])=>octets[0]===10||octets[0]===127||octets.every(n=>n===0)||octets[0]===169&&octets[1]===254||octets[0]===172&&octets[1]!>=16&&octets[1]!<=31||octets[0]===192&&octets[1]===168||octets[0]===100&&octets[1]!>=64&&octets[1]!<=127;
 if(/^\d{1,3}(\.\d{1,3}){3}$/.test(ip)){const octets=ip.split('.').map(Number);return octets.every(n=>n<=255)&&localV4(octets);}
 ip=ip.replace(/^\[|\]$/g,'');if(!ip.includes(':')||!/^[a-f\d:.]+$/i.test(ip))return false;
 try{
  const canonical=new URL(`http://[${ip}]/`).hostname.slice(1,-1);
  if(canonical==='::'||canonical==='::1')return true;
  const mapped=/^::ffff:([a-f\d]{1,4}):([a-f\d]{1,4})$/i.exec(canonical);
  if(mapped){const high=parseInt(mapped[1]!,16),low=parseInt(mapped[2]!,16);return localV4([high>>>8,high&255,low>>>8,low&255]);}
  const first=parseInt(canonical.split(':')[0]||'0',16);
  return (first&0xfe00)===0xfc00||(first&0xffc0)===0xfe80;
 }catch{return false;}
}
export function countryDisplay(ip:string|null|undefined,country:string|null|undefined,unavailable:string):string{return isInternalIp(ip)?t('common-network-internal'):country||unavailable;}
