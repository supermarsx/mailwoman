import {expect,it} from 'vitest';
import {leetspeakHtml,leetspeakText} from './leetspeak.ts';

it('replaces readable letters including Portuguese accents without changing digits or punctuation',()=>{
  expect(leetspeakText('Olá! Está tudo bem? 123 €')).toBe('0l4! 3574 7ud0 83m? 123 €');
  expect(leetspeakText('PRIVATE MESSAGE')).toBe('PR1V473 M355493');
});

it('changes display text and descriptive attributes but preserves markup, destinations and security policy',()=>{
  const source='<!doctype html><html><head><meta http-equiv="Content-Security-Policy" content="default-src \'none\'"><style>.note{color:teal}</style></head><body><p class="note">Secret message</p><a href="https://example.test/private?a=1" rel="noopener noreferrer" target="_blank">Open</a><img src="/api/rc/i/signed-token" alt="Private image"><script>const state=1</script></body></html>';
  const document=new DOMParser().parseFromString(leetspeakHtml(source),'text/html');
  expect(document.querySelector('p')?.textContent).toBe('53cr37 m355493');
  expect(document.querySelector('a')?.getAttribute('href')).toBe('https://example.test/private?a=1');
  expect(document.querySelector('a')?.getAttribute('rel')).toBe('noopener noreferrer');
  expect(document.querySelector('img')?.getAttribute('src')).toBe('/api/rc/i/signed-token');
  expect(document.querySelector('img')?.getAttribute('alt')).toBe('Pr1v473 1m493');
  expect(document.querySelector('meta')?.getAttribute('content')).toBe("default-src 'none'");
  expect(document.querySelector('style')?.textContent).toBe('.note{color:teal}');
  expect(document.querySelector('script')?.textContent).toBe('const state=1');
  expect(source).toContain('Secret message');
});
