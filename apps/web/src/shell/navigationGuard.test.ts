import {expect,it,vi} from 'vitest';
import {confirmShellNavigation,registerNavigationGuard} from './navigationGuard.ts';
it('closing a retained pane consults only its own editor guards',async()=>{
 const first=vi.fn(()=>false),second=vi.fn(()=>true);const removeFirst=registerNavigationGuard(first,'first'),removeSecond=registerNavigationGuard(second,'second');
 try{expect(await confirmShellNavigation('second')).toBe(true);expect(second).toHaveBeenCalledOnce();expect(first).not.toHaveBeenCalled();expect(await confirmShellNavigation()).toBe(false);}finally{removeFirst();removeSecond();}
});
