import {expect,it} from 'vitest';
import {resolvePreviewLines} from './messagePreview.ts';
import type {SettingEntry} from '../api/settings.ts';
const entry=(value:unknown,locked=false,source='user')=>({value,locked,source}) as SettingEntry;
it('supports off and all five counts, validates fallback, and preserves legacy choices',()=>{
 for(const value of [0,1,2,3,4,5])expect(resolvePreviewLines(()=>entry(value))).toBe(value);
 expect(resolvePreviewLines(()=>undefined)).toBe(1);
 expect(resolvePreviewLines(key=>key.endsWith('Lines')?entry(8):undefined)).toBe(1);
 expect(resolvePreviewLines(key=>key.endsWith('Lines')?entry(1,false,'global'):entry(false))).toBe(0);
});
it('administrator locks override folder preferences and saved visibility',()=>{
 expect(resolvePreviewLines(key=>key.endsWith('Lines')?entry(0,true):undefined,{previewLines:5},['previewLines'])).toBe(0);
 expect(resolvePreviewLines(key=>key.endsWith('Lines')?entry(2):undefined,{previewLines:5},['previewLines'])).toBe(5);
 expect(resolvePreviewLines(key=>key.endsWith('Lines')?entry(2):undefined,{previewLines:5},[])).toBe(2);
 expect(resolvePreviewLines(key=>key.endsWith('Lines')?entry(5):entry(false,true),{previewLines:5},['previewLines'])).toBe(0);
});
