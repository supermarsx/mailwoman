import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {createMessageListMotion,type MessageListMotionFrame} from './messageListMotion.ts';

class Paint extends EventTarget {
  cancelled=false;
  cancel():void {this.cancelled=true;this.dispatchEvent(new Event('cancel'));}
  finish():void {this.dispatchEvent(new Event('finish'));}
}
interface Call {element:HTMLElement;frames:Keyframe[];paint:Paint}
const rect=(top:number,height=72,width=400):DOMRect=>({x:0,y:top,top,left:0,right:width,bottom:top+height,width,height,toJSON:()=>({})});
const flush=async():Promise<void>=>{await Promise.resolve();await Promise.resolve();};

describe('optimistic message list motion',()=>{
  let calls:Call[];
  let disposers:(()=>void)[];
  const originalAnimate=Object.getOwnPropertyDescriptor(HTMLElement.prototype,'animate');
  beforeEach(()=>{
    calls=[];disposers=[];
    document.documentElement.removeAttribute('data-reduced-motion');
    vi.stubGlobal('matchMedia',()=>({matches:false,addEventListener:vi.fn(),removeEventListener:vi.fn()}));
    Object.defineProperty(HTMLElement.prototype,'animate',{configurable:true,value:function(this:HTMLElement,frames:Keyframe[]){const paint=new Paint();calls.push({element:this,frames,paint});return paint;}});
  });
  afterEach(()=>{
    for(const dispose of disposers)dispose();
    document.body.replaceChildren();
    document.documentElement.removeAttribute('data-reduced-motion');
    if(originalAnimate)Object.defineProperty(HTMLElement.prototype,'animate',originalAnimate);else Reflect.deleteProperty(HTMLElement.prototype,'animate');
    vi.unstubAllGlobals();
  });
  function harness(ids=['a','b','c']){
    const host=document.createElement('div'),list=document.createElement('ul');
    list.className='list__items';host.append(list);document.body.append(host);
    host.getBoundingClientRect=()=>rect(0,600);
    const nodes=new Map<string,HTMLElement>(),tops=new Map<string,number>();
    for(const [index,id] of ids.entries()){
      const slot=document.createElement('li');slot.dataset.motionRow=id;slot.style.transform=`translateY(${index*72}px)`;
      const button=document.createElement('button');button.id='message-'+id;button.dataset.emailId=id;button.textContent='Subject '+id;slot.append(button);
      tops.set(id,index*72);slot.getBoundingClientRect=()=>rect(tops.get(id)??0);
      nodes.set(id,slot);list.append(slot);
    }
    const motion=createMessageListMotion();disposers.push(motion.dispose);
    const update=(currentIds:readonly string[],over:Partial<MessageListMotionFrame>={}):void=>motion.update({host,scope:'account1/inbox',ids:currentIds,busy:false,...over});
    const remove=(id:string):void=>{nodes.get(id)?.remove();};
    return {host,list,nodes,tops,motion,update,remove};
  }

  it('removes the live row immediately and animates only inert departure paint and the surviving rows',async()=>{
    const h=harness();h.update(['a','b','c']);await flush();
    h.remove('a');h.tops.set('b',0);h.tops.set('c',72);h.update(['b','c']);
    expect(h.list.querySelector('[data-motion-row="a"]')).toBeNull();
    await flush();
    const ghost=h.list.querySelector<HTMLElement>('[data-list-exit-ghost]')!;
    expect(ghost.getAttribute('aria-hidden')).toBe('true');expect(ghost.hasAttribute('inert')).toBe(true);
    expect(ghost.querySelector('[data-email-id], [id]')).toBeNull();
    expect(ghost.querySelector('button')?.tabIndex).toBe(-1);
    expect(calls.find(call=>call.element===h.nodes.get('b'))?.frames).toEqual([{translate:'0 72px'},{translate:'0 0'}]);
    expect(h.nodes.get('b')?.style.transform).toBe('translateY(72px)');
    calls.forEach(call=>call.paint.finish());expect(h.list.querySelector('[data-list-exit-ghost]')).toBeNull();
  });

  it('never animates a virtual-window change, paging append or keyword-only refresh',async()=>{
    const h=harness(),ids=['a','b','c'];h.update(ids);await flush();
    h.remove('a');h.update(ids);await flush();
    h.update([...ids,'d']);await flush();h.update([...ids,'d']);await flush();
    expect(calls).toHaveLength(0);
  });

  it('resets paging geometry when an arriving sibling turns a singleton into a conversation head',async()=>{
    const h=harness(['m:a','m:b']);h.update(['m:a','m:b']);await flush();
    h.update(['m:a','m:b'],{busy:true});await flush();
    h.remove('m:a');h.nodes.get('m:b')!.dataset.motionRow='h:conversation';
    h.update(['h:conversation'],{busy:false});await flush();
    expect(calls).toHaveLength(0);
    expect(h.list.querySelector('[data-list-exit-ghost]')).toBeNull();
  });

  it('cancels on scroll then records the new window so the next actual deletion can animate',async()=>{
    const h=harness();h.update(['a','b','c']);await flush();
    h.host.scrollTop=72;h.host.dispatchEvent(new Event('scroll'));h.update(['a','b','c']);await flush();
    expect(calls).toHaveLength(0);
    h.remove('b');h.tops.set('c',72);h.update(['a','c']);await flush();
    expect(calls.some(call=>call.element===h.nodes.get('c'))).toBe(true);
    h.host.dispatchEvent(new Event('scroll'));
    expect(calls.every(call=>call.paint.cancelled)).toBe(true);
    expect(h.list.querySelector('[data-list-exit-ghost]')).toBeNull();
    expect(h.nodes.get('c')?.style.transform).toBe('translateY(144px)');
  });

  it('keeps the next removal animated when the list scroll handler runs before the native motion listener',async()=>{
    const h=harness(),ids=['a','b','c'];
    // Real MessageList installs its onScroll before the helper subscribes.
    h.host.addEventListener('scroll',()=>h.update(ids));
    h.update(ids);await flush();h.host.scrollTop=72;h.host.dispatchEvent(new Event('scroll'));await flush();
    expect(calls).toHaveLength(0);
    h.remove('a');h.tops.set('b',0);h.update(['b','c']);await flush();
    expect(calls.some(call=>call.element===h.nodes.get('b'))).toBe(true);
  });

  it('does not animate across authority/query scopes, whole-list loads or layout changes',async()=>{
    const h=harness();h.update(['a','b','c']);await flush();
    h.remove('a');h.update(['b','c'],{scope:'account2/inbox'});await flush();
    h.remove('b');h.update(['c'],{scope:'account2/inbox',busy:true});await flush();
    h.update(['c'],{scope:'account2/inbox'});await flush();
    h.host.getBoundingClientRect=()=>rect(0,400,320);h.remove('c');h.update([],{scope:'account2/inbox'});await flush();
    expect(calls).toHaveLength(0);
  });

  it('honors both reduced-motion sources and cancels active paint when the preference changes',async()=>{
    const h=harness();h.update(['a','b','c']);await flush();
    h.remove('a');h.tops.set('b',0);h.update(['b','c']);await flush();expect(calls.length).toBeGreaterThan(0);
    document.documentElement.setAttribute('data-reduced-motion','');await flush();
    expect(calls.every(call=>call.paint.cancelled)).toBe(true);
    h.remove('b');h.update(['c']);await flush();const count=calls.length;
    document.documentElement.removeAttribute('data-reduced-motion');
    vi.stubGlobal('matchMedia',()=>({matches:true,addEventListener:vi.fn(),removeEventListener:vi.fn()}));
    const reduced=createMessageListMotion();disposers.push(reduced.dispose);
    reduced.update({host:h.host,scope:'one',ids:['c'],busy:false});await flush();
    h.remove('c');reduced.update({host:h.host,scope:'one',ids:[],busy:false});await flush();expect(calls).toHaveLength(count);
  });

  it('bounds bulk departure ghosts and cancels queued/active work on disposal',async()=>{
    const ids=Array.from({length:20},(_,index)=>String(index)),h=harness(ids);
    h.update(ids);await flush();ids.slice(0,12).forEach(h.remove);h.update(ids.slice(12));await flush();
    expect(h.list.querySelectorAll('[data-list-exit-ghost]')).toHaveLength(8);
    h.motion.dispose();expect(h.list.querySelector('[data-list-exit-ghost]')).toBeNull();expect(calls.every(call=>call.paint.cancelled)).toBe(true);
    h.motion.update({host:h.host,scope:'one',ids:[],busy:false});await flush();expect(h.list.querySelector('[data-list-exit-ghost]')).toBeNull();
  });
});
