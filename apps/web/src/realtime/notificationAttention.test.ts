import {afterEach,expect,it,vi} from 'vitest';
import {newMailAttention,incomingHighlighted,clearNotificationAttention} from './notificationAttention.ts';
afterEach(()=>{clearNotificationAttention();vi.useRealTimers();vi.restoreAllMocks();});
it('highlights incoming rows and clears attention at the configured deadline',()=>{
 vi.useFakeTimers();document.title='Inbox';vi.spyOn(document,'hasFocus').mockReturnValue(false);
 newMailAttention(['mail'],'New mail',{'notifications.incoming.tab':true,'notifications.incoming.durationSeconds':2});
 expect(incomingHighlighted('mail')).toBe(true);expect(document.title).toBe('● New mail');
 vi.advanceTimersByTime(1000);expect(document.title).toBe('Inbox');vi.advanceTimersByTime(1000);
 expect(incomingHighlighted('mail')).toBe(false);expect(document.title).toBe('Inbox');
});
it('honours disabled channels and restores the tab title on focus',()=>{
 document.title='Inbox';vi.spyOn(document,'hasFocus').mockReturnValue(false);
 newMailAttention(['mail'],'New mail',{'notifications.incoming.tab':false,'notifications.incoming.highlight':false});expect(incomingHighlighted('mail')).toBe(false);expect(document.title).toBe('Inbox');
 newMailAttention([],'New mail',{'notifications.incoming.tab':true});window.dispatchEvent(new Event('focus'));expect(document.title).toBe('Inbox');
});
