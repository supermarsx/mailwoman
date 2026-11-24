import {expect,it} from 'vitest';
import {storagePressure} from './storagePermission.ts';
it('detects high quota usage or low estimated headroom without guessing when estimates are missing',()=>{
 expect(storagePressure({usage:900,quota:1000})).toBe(true);
 expect(storagePressure({usage:1,quota:32*1048576})).toBe(true);
 expect(storagePressure({usage:100,quota:2**30})).toBe(false);
 for(const status of [{},{usage:10},{usage:0,quota:1},{usage:NaN,quota:10},{usage:1,quota:Infinity}])expect(storagePressure(status)).toBe(false);
});
