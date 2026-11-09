import {expect,it,vi} from 'vitest';
import {mentionQuery,mentionSuggestions,toWithMention,mentionedAddresses,mentionsAny} from './mentions.ts';
import {cardForAddress} from '../contacts/share.ts';
it('recognises cursor-local triggers without matching ordinary email/URL content',()=>{
 expect(mentionQuery('Hi @Jane Doe')).toEqual({from:3,to:12,query:'Jane Doe'});expect(mentionQuery('@')).toEqual({from:0,to:1,query:''});
 for(const text of ['jane@example.org','https://site/@Jane','Hi @@Jane','Hi @Jane, now'])expect(mentionQuery(text),text).toBeNull();
 expect(mentionQuery('Hi @Jane rest',8)?.query).toBe('Jane');
});
it('ranks contacts, validates and deduplicates addresses instead of suggesting arbitrary text',()=>{
 const a=cardForAddress('jane@example.org','Jane Doe'),b=cardForAddress('jane@example.org','Jane Duplicate'),bad=cardForAddress('javascript:bad','Jane Bad');
 expect(mentionSuggestions([a,b,bad],'Jane').map(item=>item.email)).toEqual(['jane@example.org']);expect(mentionSuggestions([a],'')).toHaveLength(1);
});
it('adds only explicitly selected recipients and preserves Cc/Bcc privacy and custom separators',()=>{
 const pick={name:'Doe, Jane',email:'jane@example.org'};
 expect(toWithMention('existing@example.org','','',pick)).toBe('existing@example.org, "Doe, Jane" <jane@example.org>');
 expect(toWithMention('existing@example.org','','',pick,';')).toBe('existing@example.org; "Doe, Jane" <jane@example.org>');
 for(const field of ['to','cc','bcc']){const fields={to:'',cc:'',bcc:'',[field]:'Jane <JANE@example.org>'};expect(toWithMention(fields.to,fields.cc,fields.bcc,pick)).toBe(fields.to);}
});
it('extracts structured mailto mentions, nested labels and explicit addresses, excluding ordinary recipient links and quotes',()=>{
 const html='<p><a href="mailto:Jane%40example.org"><strong>@Jane</strong> Doe</a> and @alex@example.org.</p><a href="mailto:other@example.org">Contact Other</a><blockquote>@quoted@example.org <a href="mailto:q@example.org">@Q</a></blockquote><div class="gmail_signature">@sig@example.org</div><script>@script@example.org</script>';
 const parser=vi.spyOn(DOMParser.prototype,'parseFromString').mockImplementation(()=>{throw Error('browser DOM must not parse sender resources');});
 try{expect(mentionedAddresses(html)).toEqual(['jane@example.org','alex@example.org']);}finally{parser.mockRestore();}
 expect(mentionedAddresses('', 'Hello @jane@example.org\n> @quoted@example.org\n-- \n@sig@example.org')).toEqual(['jane@example.org']);
 expect(mentionsAny(['alias@example.org'],['primary@example.org','ALIAS@example.org'])).toBe(true);expect(mentionsAny(['alias@example.org.evil'],['alias@example.org'])).toBe(false);
});
