import {afterEach,expect,it} from 'vitest';
import {createRoot,createSignal} from 'solid-js';
import {configureMailLeetspeak,displayText,leetspeakEnabled,setAdminLeetspeak} from './presentation.ts';
import {rawT,seedLocaleSync,t} from './registry.ts';

afterEach(()=>{setAdminLeetspeak(null);configureMailLeetspeak(()=>false);});

it('transforms interface translations reactively while preserving raw generated content',()=>{
  createRoot(dispose=>{
    const [enabled,setEnabled]=createSignal(false);
    configureMailLeetspeak(enabled);
    seedLocaleSync('en',['leet-test-label = Private message from { $name }']);
    expect(t('leet-test-label',{name:'Alice'})).toBe('Private message from Alice');
    setEnabled(true);
    expect(t('leet-test-label',{name:'Alice'})).toBe('Pr1v473 m355493 fr0m 4l1c3');
    expect(rawT('leet-test-label',{name:'Alice'})).toBe('Private message from Alice');
    expect(displayText('Secret subject')).toBe('53cr37 5u8j3c7');
    setEnabled(false);
    expect(t('leet-test-label',{name:'Alice'})).toBe('Private message from Alice');
    dispose();
  });
});

it('keeps console preference independent and falls back to the current mailbox policy on disposal',()=>{
  createRoot(dispose=>{
    const [mail,setMail]=createSignal(true);
    configureMailLeetspeak(mail);
    expect(leetspeakEnabled()).toBe(true);
    setAdminLeetspeak(false);
    expect(leetspeakEnabled()).toBe(false);
    setAdminLeetspeak(true);
    setMail(false);
    expect(leetspeakEnabled()).toBe(true);
    setAdminLeetspeak(null);
    expect(leetspeakEnabled()).toBe(false);
    dispose();
  });
});
