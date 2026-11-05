import {describe,expect,it} from 'vitest';
import {imageHostLists,imageHostOf,imageHostState,setImageHostState} from './imageHostRules.ts';
describe('image host rules',()=>{
 const lists={trusted:['cdn.shop.test','news.test'],blocked:['ads.news.test']};
 it('matches whole labels, with blocked winning',()=>{
  expect(imageHostState('cdn.shop.test',lists)).toBe('allow');
  expect(imageHostState('img.cdn.shop.test',lists)).toBe('allow');
  expect(imageHostState('evilcdn.shop.test',lists)).toBe('ask');
  expect(imageHostState('static.news.test',lists)).toBe('allow');
  expect(imageHostState('ads.news.test',lists)).toBe('block');
  expect(imageHostState('x.ads.news.test',lists)).toBe('block');
  expect(imageHostState('news.test.evil.example',lists)).toBe('ask');
 });
 it('one choice replaces the other, never both',()=>{
  const blocked=setImageHostState('cdn.shop.test','block',lists)!;
  expect(blocked.trusted).toEqual(['news.test']);expect(blocked.blocked).toEqual(['ads.news.test','cdn.shop.test']);
  expect(imageHostState('cdn.shop.test',blocked)).toBe('block');
  const asked=setImageHostState('cdn.shop.test','ask',blocked)!;
  expect(imageHostState('cdn.shop.test',asked)).toBe('ask');expect(asked.blocked).toEqual(['ads.news.test']);
  const allowed=setImageHostState('ads.news.test','allow',lists)!;
  expect(imageHostState('ads.news.test',allowed)).toBe('allow');expect(allowed.blocked).toEqual([]);
 });
 it('refuses unlistable hosts and full lists; reads settings fail-closed',()=>{
  expect(setImageHostState('localhost','allow',lists)).toBeNull();expect(setImageHostState('127.0.0.1','block',lists)).toBeNull();
  const full={trusted:Array.from({length:100},(_,i)=>`h${i}.test`),blocked:[]};expect(setImageHostState('new.test','allow',full)).toBeNull();
  expect(imageHostLists({'images.trustedHosts':{value:['A.test','ok.test',3]},'images.blockedHosts':{value:'x'}})).toEqual({trusted:['ok.test'],blocked:[]});
  expect(imageHostOf('https://CDN.Shop.test./a.png')).toBe('cdn.shop.test');expect(imageHostOf('javascript:1')).toBeNull();
 });
});
