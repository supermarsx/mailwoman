import {expect,it} from 'vitest';
import {draftToCard} from '../../state/slices/contacts.ts';
import {checkRecipientNames} from './recipientNames.ts';
const alice=draftToCard('a','book',{name:{full:'Alice Example',given:'Alice',surname:'Example',prefix:'',suffix:''},emails:[{value:'alice@example.org',context:'work',pref:1}]});
it('resolves exact contact names while preserving quoted recipients and valid addresses',()=>{
 expect(checkRecipientNames('"Smith, Bob" <bob@example.org>; Alice Example',[alice])).toEqual({value:'"Smith, Bob" <bob@example.org>, "Alice Example" <alice@example.org>',unresolved:[],resolved:1});
});
it('never guesses an ambiguous or partial contact name',()=>{
 const other={...alice,id:'b',emails:[{value:'other@example.org',context:'work',pref:1}]};
 expect(checkRecipientNames('Alice Example, Alice',[alice,other]).unresolved).toEqual(['Alice Example','Alice']);
});
