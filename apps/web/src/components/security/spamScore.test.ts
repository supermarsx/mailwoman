import { expect, it } from 'vitest';
import { parseSpamScores, sortRules, spamBand, type SpamScore } from './spamScore.ts';

it('reads folded SpamAssassin status, negative scores, thresholds and rule names',()=>{
  expect(parseSpamScores('X-Spam-Status: No, score=-2.3 required=5.0\r\n tests=DKIM_VALID,BAYES_00 autolearn=ham version=3.4.6\r\n\r\nbody')).toEqual([{source:'SpamAssassin',score:-2.3,threshold:5,spam:false,rules:[{name:'BAYES_00',score:null,description:''},{name:'DKIM_VALID',score:null,description:''}]}]);
});
it('takes SpamAssassin points and descriptions from X-Spam-Report and sorts by contribution',()=>{
  const raw=['X-Spam-Status: Yes, score=6.2 required=5.0 tests=BAYES_99,DKIM_VALID,\r\n\tURIBL_BLOCKED autolearn=no','X-Spam-Report: \r\n\t*  0.0 URIBL_BLOCKED ADMINISTRATOR NOTICE: The query to URIBL was blocked.\r\n\t*      See http://wiki.apache.org/spamassassin/DnsBlocklists for more\r\n\t*  6.3 BAYES_99 BODY: Bayes spam probability is 99 to 100%\r\n\t*      [score: 1.0000]\r\n\t* -0.1 DKIM_VALID Message has at least one valid DKIM or DK signature',''].join('\r\n');
  const [score]=parseSpamScores(raw);
  expect(score?.rules).toEqual([
    {name:'BAYES_99',score:6.3,description:'BODY: Bayes spam probability is 99 to 100% [score: 1.0000]'},
    {name:'URIBL_BLOCKED',score:0,description:'ADMINISTRATOR NOTICE: The query to URIBL was blocked. See http://wiki.apache.org/spamassassin/DnsBlocklists for more'},
    {name:'DKIM_VALID',score:-0.1,description:'Message has at least one valid DKIM or DK signature'},
  ]);
});
it('reads SpamAssassin tests that carry their own points',()=>{
  expect(parseSpamScores('X-Spam-Status: No, score=1.0 required=5.0 tests=RULE_A=1.1,\r\n RULE_B=-0.1 autolearn=no')[0]?.rules.map(rule=>[rule.name,rule.score])).toEqual([['RULE_A',1.1],['RULE_B',-0.1]]);
});
it('reads Rspamd result scores, symbol options and sorts symbols without relying on color or a guessed verdict',()=>{
  expect(parseSpamScores('X-Spamd-Result: default: True [16.20 / 15.00]; R_BAD_CTE(2.00)[]; DKIM_ALLOW(-0.20)[example.com:s=sel]; BAYES_SPAM(4.20)[99.90%]')[0]).toEqual({source:'Rspamd',score:16.2,threshold:15,spam:true,rules:[
    {name:'BAYES_SPAM',score:4.2,description:'99.90%'},{name:'R_BAD_CTE',score:2,description:''},{name:'DKIM_ALLOW',score:-0.2,description:'example.com:s=sel'},
  ]});
  expect(parseSpamScores('X-Rspamd-Score: -0.7')[0]?.spam).toBeNull();
});
it('does not interpret body text as headers, fabricate missing scores or accept unbounded values',()=>{
  expect(parseSpamScores('Subject: Example\r\n\r\nX-Spam-Status: Yes, score=20 required=5')).toEqual([]);
  expect(parseSpamScores('X-Spam-Status: Yes, score=99999999999999999 required=5')).toEqual([]);
  expect(parseSpamScores('X-Spam-Level: *****')).toEqual([]);
});
it('bands a score as ham, neutral or spam from the threshold and the verdict',()=>{
  const at=(score:number,threshold:number|null,spam:boolean|null=null):SpamScore=>({source:'Rspamd',score,threshold,spam,rules:[]});
  expect(spamBand(at(-1,5))).toBe('ham');
  expect(spamBand(at(0,5))).toBe('ham');
  expect(spamBand(at(3,5))).toBe('neutral');
  expect(spamBand(at(5,5))).toBe('spam');
  expect(spamBand(at(1,null,true))).toBe('spam');
  expect(spamBand(at(2,null))).toBe('neutral');
});
it('puts rules reported without points after the scored ones',()=>{
  expect(sortRules([{name:'B',score:null,description:''},{name:'A',score:-1,description:''},{name:'C',score:2,description:''},{name:'A2',score:null,description:''}]).map(rule=>rule.name)).toEqual(['C','A','A2','B']);
});
