import {describe,expect,it} from 'vitest';
import {describeSessionDevice,sessionDate} from './sessionDevice.ts';

describe('session client descriptions',()=>{
  it('identifies the supplied Firefox computer and phone without claiming Windows 10 or distinct hardware',()=>{
    expect(describeSessionDevice('Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:153.0) Gecko/20100101 Firefox/153.0')).toEqual({device:'desktop',browser:'Firefox 153',os:'Windows'});
    expect(describeSessionDevice('Mozilla/5.0 (Android 16; Mobile; rv:160.0) Gecko/160.0 Firefox/160.0')).toEqual({device:'phone',browser:'Firefox 160',os:'Android 16'});
  });
  it.each([
    ['Mozilla/5.0 (Windows NT 10.0) Chrome/140.0 Safari/537.36 Edg/140.0','Microsoft Edge 140','desktop'],
    ['Mozilla/5.0 (Android 16) Chrome/140.0 Safari/537.36 SamsungBrowser/28.0','Samsung Internet 28','tablet'],
    ['Mozilla/5.0 (iPhone) AppleWebKit/605.1 Version/18.0 Mobile/15 Safari/604.1','Safari 18','phone'],
    ['Mozilla/5.0 (Macintosh) Chrome/140.0 Safari/537.36 OPR/120.0','Opera 120','desktop'],
    ['Mozilla/5.0 (iPad) FxiOS/140.0 Mobile/15 Safari/605.1','Firefox 140','tablet'],
  ])('prefers the real client over compatibility tokens: %s',(ua,browser,device)=>{
    expect(describeSessionDevice(ua)).toMatchObject({browser,device});
  });
  it('does not expose unknown raw strings as a device name',()=>{
    expect(describeSessionDevice('unrecognized client')).toEqual({device:'unknown',browser:'',os:''});
    expect(describeSessionDevice(null).device).toBe('unknown');
  });
  it('normalizes nanosecond timestamps and rejects invalid dates',()=>{
    expect(sessionDate('2026-11-23T21:51:53.995352385+00:00')?.toISOString()).toBe('2026-11-23T21:51:53.995Z');
    expect(sessionDate('invalid')).toBeNull();expect(sessionDate(null)).toBeNull();
  });
});
