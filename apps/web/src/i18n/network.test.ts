import {expect,it} from 'vitest';
import {countryDisplay,isInternalIp} from './network.ts';
it('labels local ranges, including mapped IPv4, without treating public or documentation addresses as internal',()=>{
 for(const ip of ['10.1.2.3','172.16.0.1','172.31.255.1','192.168.1.1','127.0.0.2','169.254.2.3','100.64.1.1','100.127.255.254','0.0.0.0','::','::1','fc00::1','fdab::1','fe80::1','febf::1','::ffff:192.168.1.1','::ffff:c0a8:101'])expect(isInternalIp(ip),ip).toBe(true);
 for(const ip of ['8.8.8.8','172.15.0.1','172.32.0.1','100.128.1.1','192.0.2.1','2001:db8::1','fec0::1','::ffff:8.8.8.8','bad','10.999.1.1','10.0.0.1.example.org'])expect(isInternalIp(ip),ip).toBe(false);
 expect(countryDisplay('10.1.2.3','PT','Unknown')).toBe('Internal network');expect(countryDisplay('8.8.8.8',null,'Unknown')).toBe('Unknown');
});
