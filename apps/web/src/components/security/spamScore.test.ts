import { expect, it } from 'vitest';
import { parseSpamScores } from './spamScore.ts';

it('reads folded SpamAssassin status, negative scores, thresholds and rule names',()=>{
  expect(parseSpamScores('X-Spam-Status: No, score=-2.3 required=5.0\r\n tests=DKIM_VALID,BAYES_00 autolearn=ham\r\n\r\nbody')).toEqual([{source:'SpamAssassin',score:-2.3,threshold:5,spam:false,rules:['DKIM_VALID','BAYES_00']}]);
});
it('reads Rspamd result scores and symbols without relying on color or a guessed verdict',()=>{
  expect(parseSpamScores('X-Spamd-Result: default: True [16.20 / 15.00]; BAYES_SPAM(4.20)[]; R_BAD_CTE(2.00)[]')[0]).toEqual({source:'Rspamd',score:16.2,threshold:15,spam:true,rules:['BAYES_SPAM (4.20)','R_BAD_CTE (2.00)']});
  expect(parseSpamScores('X-Rspamd-Score: -0.7')[0]?.spam).toBeNull();
});
it('does not interpret body text as headers, fabricate missing scores or accept unbounded values',()=>{
  expect(parseSpamScores('Subject: Example\r\n\r\nX-Spam-Status: Yes, score=20 required=5')).toEqual([]);
  expect(parseSpamScores('X-Spam-Status: Yes, score=99999999999999999 required=5')).toEqual([]);
  expect(parseSpamScores('X-Spam-Level: *****')).toEqual([]);
});
