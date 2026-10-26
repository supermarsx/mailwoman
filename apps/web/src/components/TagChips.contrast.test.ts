import {expect,it} from 'vitest';
import {labelTextColor} from './TagChips.tsx';
it('keeps user-selected light and dark label colours readable',()=>{
  expect(labelTextColor('#ffffff')).toBe('#000');expect(labelTextColor('#ffff00')).toBe('#000');
  expect(labelTextColor('#000000')).toBe('#fff');expect(labelTextColor('#112233')).toBe('#fff');
});
