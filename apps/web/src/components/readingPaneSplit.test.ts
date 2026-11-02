import {expect,it} from 'vitest';
import {readingPaneSplit} from './readingPaneSplit.ts';

it('preserves a usable saved split and reserves the divider and reader',()=>{
  expect(readingPaneSplit(1000,400)).toEqual({min:240,max:714,size:400});
  expect(readingPaneSplit(1000,1200)).toEqual({min:240,max:714,size:714});
  expect(readingPaneSplit(600,320,true)).toEqual({min:160,max:314,size:314});
  expect(readingPaneSplit(1000,-10)).toEqual({min:240,max:714,size:240});
});

it.each([false,true])('fits narrow or short workspaces without losing either pane (bottom=%s)',bottom=>{
  for(const extent of [280,320,390,440,526,600,1000]) {
    for(const preferred of [0,240,320,1200]) {
      const split=readingPaneSplit(extent,preferred,bottom);
      expect(split.min).toBeGreaterThan(0);
      expect(split.size).toBeGreaterThanOrEqual(split.min);
      expect(split.size).toBeLessThanOrEqual(split.max);
      expect(extent-6-split.size).toBeGreaterThan(0);
      if(extent>= (bottom?446:526))expect(extent-6-split.size).toBeGreaterThanOrEqual(280);
    }
  }
});

it('handles an unmeasured, hidden, or malformed extent and invalid saved sizes',()=>{
  expect(readingPaneSplit(0,340)).toEqual({min:0,max:0,size:0});
  expect(readingPaneSplit(Number.NaN,340)).toEqual({min:0,max:0,size:0});
  expect(readingPaneSplit(1000,Number.NaN).size).toBe(340);
  expect(readingPaneSplit(1000,Number.POSITIVE_INFINITY,true).size).toBe(320);
});
