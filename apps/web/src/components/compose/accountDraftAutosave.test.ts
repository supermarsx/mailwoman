import {afterEach,expect,it,vi} from 'vitest';
import {createDraftAutosave} from './accountDraftAutosave.ts';
import {DraftSaveError,type DraftEditorState,type AccountDraftDocument} from '../../api/accountDrafts.ts';
it('treats reordered server JSON and absent optional MIME types as the same saved state',async()=>{
 vi.useFakeTimers();const status=vi.fn();
 const value={...state(),attachments:[{blobId:'owned.1',name:'file',size:2}]} as unknown as DraftEditorState;
 const returned=Object.fromEntries(Object.entries({...value,attachments:[{blobId:'owned.1',name:'file',size:2,contentType:null}]}).reverse()) as unknown as DraftEditorState;
 const save=vi.fn().mockResolvedValue(saved(returned));
 const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status,failure:vi.fn(),saved:vi.fn()});
 controller.update(value);await vi.advanceTimersByTimeAsync(1000);expect(status).toHaveBeenLastCalledWith('saved');
 controller.update(value);await vi.advanceTimersByTimeAsync(60000);expect(save).toHaveBeenCalledTimes(1);controller.dispose();
});
const state=(subject='draft'):DraftEditorState=>({version:1,identityId:'',to:'unfinished',cc:'',bcc:'',subject,bodyFormat:'plain',bodyHtml:'',bodyText:'unsent',insertedSignatures:[],inReplyTo:[],references:[],attachments:[],inline:[],requestReceipt:false,requestDeliveryReceipt:false,openTracking:false,protectedContent:false});
const saved=(value:DraftEditorState,revision='r1'):AccountDraftDocument=>({id:'draft',accountId:'a',storageAuthority:'authority',revision,emailId:'email',savedAt:1,status:'saved',cleanupPending:false,summary:{subject:value.subject},state:value});
afterEach(()=>vi.useRealTimers());
it('debounces edits and serializes a newer edit behind confirmed revision',async()=>{
 vi.useFakeTimers();let resolve!:(value:AccountDraftDocument)=>void;
 const save=vi.fn().mockImplementationOnce(()=>new Promise<AccountDraftDocument>(done=>resolve=done)).mockImplementation(async(_id,_op,_rev,value)=>saved(value,'r2'));
 const status=vi.fn();const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status,failure:vi.fn(),saved:vi.fn(),operation:()=> 'operation'});
 controller.update(state('one'));await vi.advanceTimersByTimeAsync(700);controller.update(state('two'));await vi.advanceTimersByTimeAsync(1000);expect(save).toHaveBeenCalledTimes(1);
 controller.update(state('three'));resolve(saved(state('two')));await vi.advanceTimersByTimeAsync(0);await vi.advanceTimersByTimeAsync(1000);
 expect(save).toHaveBeenCalledTimes(2);expect(save.mock.calls[1]?.[2]).toBe('r1');expect(save.mock.calls[1]?.[3].subject).toBe('three');expect(status).toHaveBeenLastCalledWith('saved');controller.dispose();
});
it('never retries a network-uncertain APPEND even after later edits or manual save',async()=>{
 vi.useFakeTimers();const save=vi.fn().mockRejectedValue(new TypeError('connection lost'));const failure=vi.fn();
 const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status:vi.fn(),failure,saved:vi.fn()});controller.update(state());await vi.advanceTimersByTimeAsync(1000);controller.update(state('new'));await vi.advanceTimersByTimeAsync(60000);await controller.flush();
 expect(save).toHaveBeenCalledTimes(1);expect(failure).toHaveBeenCalledWith('draftSaveUncertain');controller.dispose();
});
it('periodic saving does not slide with every letter; manual mode never schedules',async()=>{
 vi.useFakeTimers();const save=vi.fn().mockImplementation(async(_id,_op,_rev,value)=>saved(value));const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status:vi.fn(),failure:vi.fn(),saved:vi.fn()});
 controller.configure({mode:'periodic',intervalSecs:5,debounceMs:500});controller.update(state('a'));await vi.advanceTimersByTimeAsync(4000);controller.update(state('ab'));await vi.advanceTimersByTimeAsync(1000);expect(save).toHaveBeenCalledTimes(1);
 controller.configure({mode:'manual',intervalSecs:5,debounceMs:500});controller.update(state('abc'));await vi.advanceTimersByTimeAsync(60000);expect(save).toHaveBeenCalledTimes(1);await controller.flush();expect(save).toHaveBeenCalledTimes(2);controller.dispose();
});
it('drops late acknowledgement and scheduled writes after account authority changes',async()=>{
 vi.useFakeTimers();let current=true;let resolve!:(value:AccountDraftDocument)=>void;const callback=vi.fn();const save=vi.fn(()=>new Promise<AccountDraftDocument>(done=>resolve=done));
 const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>current,status:vi.fn(),failure:vi.fn(),saved:callback});controller.update(state());await vi.advanceTimersByTimeAsync(1000);current=false;resolve(saved(state()));await vi.advanceTimersByTimeAsync(0);expect(callback).not.toHaveBeenCalled();controller.update(state('other'));await vi.advanceTimersByTimeAsync(60000);expect(save).toHaveBeenCalledTimes(1);controller.dispose();
});
it('preparation refusal reports failure without automatically looping',async()=>{
 vi.useFakeTimers();const save=vi.fn().mockRejectedValue(new DraftSaveError('draftPreparationRefused'));const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status:vi.fn(),failure:vi.fn(),saved:vi.fn()});controller.update(state());await vi.advanceTimersByTimeAsync(60000);expect(save).toHaveBeenCalledTimes(1);controller.dispose();
});

it('suspends pending plaintext saves and waits for an already-started save before sending',async()=>{
 vi.useFakeTimers();let resolve!:(value:AccountDraftDocument)=>void;const save=vi.fn(()=>new Promise<AccountDraftDocument>(done=>resolve=done));
 const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status:vi.fn(),failure:vi.fn(),saved:vi.fn()});
 controller.update(state());controller.suspend(true);await vi.advanceTimersByTimeAsync(2000);expect(save).not.toHaveBeenCalled();
 controller.suspend(false);await vi.advanceTimersByTimeAsync(1000);expect(save).toHaveBeenCalledTimes(1);
 controller.update(state('edited while saving'));controller.suspend(true);let settled=false;void controller.settled().then(()=>settled=true);await vi.advanceTimersByTimeAsync(0);expect(settled).toBe(false);
 resolve(saved(state()));await vi.advanceTimersByTimeAsync(0);expect(settled).toBe(true);await vi.advanceTimersByTimeAsync(60000);expect(save).toHaveBeenCalledTimes(1);controller.dispose();
});
