import { describe, expect, it } from 'vitest';
import { loadCatalog, t } from '../../i18n/index.ts';
import { deliveryStatusMeaning } from './deliveryStatusMeaning.ts';

// IANA's 81 enumerated detail codes, checked 2026-11-09.
const details = [[0,0],[1,10],[2,4],[3,6],[4,7],[5,6],[6,10],[7,30]];
describe('enhanced delivery status explanations',()=>{
  it('explains all registered codes for every supported class with real catalog messages',async()=>{
    await loadCatalog('mail-dsn');
    let count=0;
    for (const [subject,last] of details) for(let detail=0;detail<=last!;detail++) {
      count++;
      for(const kind of [2,4,5]) {
        const meaning=deliveryStatusMeaning(`${kind}.${subject}.${detail}`)!;
        expect(meaning.specific).toBe(true);
        for(const key of [meaning.classKey,meaning.reasonKey,meaning.adviceKey]) {
          expect(t(key)).not.toBe(key);
          expect(t(key).length).toBeGreaterThan(20);
        }
      }
    }
    expect(count).toBe(81);
  });
  it('keeps the reported severity independent of the detail',()=>{
    expect(deliveryStatusMeaning('5.2.2')?.classKey).toBe('dsn-class-5');
    expect(deliveryStatusMeaning('4.2.2')?.classKey).toBe('dsn-class-4');
    expect(deliveryStatusMeaning('2.6.4')?.classKey).toBe('dsn-class-2');
  });
  it('uses honest category fallbacks for vendor-specific and future codes',()=>{
    for(const code of ['5.7.133','5.7.999','4.4.999','5.1.999','5.999.999']) {
      const meaning=deliveryStatusMeaning(code)!;
      expect(meaning.specific).toBe(false);
      expect(meaning.reasonKey).toBe(`dsn-category-${code.split('.')[1]==='999'?'0':code.split('.')[1]}`);
    }
    expect(deliveryStatusMeaning('5.07.1')?.specific).toBe(false);
  });
  it('rejects malformed or unsupported status codes',()=>{
    for(const code of ['550','5.7','5.7.1234','3.1.1','5.7.1 extra','<b>5.7.1</b>','']) expect(deliveryStatusMeaning(code)).toBeNull();
  });
});
