import {afterEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {loadCatalog} from '../../i18n';
import {SpamScoreBreakdown} from './SpamScore.tsx';

afterEach(()=>vi.unstubAllGlobals());
it.each(['SpamAssassin','Rspamd'] as const)('copies the complete %s breakdown as readable text',async source=>{
 await loadCatalog('spam-score');
 const writeText=vi.fn<(text:string)=>Promise<void>>().mockResolvedValue(undefined);vi.stubGlobal('navigator',{clipboard:{writeText}});
 render(()=><SpamScoreBreakdown score={{source,score:6.2,threshold:5,spam:true,rules:[
  {name:'BAYES_99',score:6.3,description:'BODY: Bayes probability'},
  {name:'DKIM_VALID',score:-0.1,description:'Valid signature'},
  {name:'UNKNOWN_POINTS',score:null,description:''},
 ]}}/>);
 fireEvent.click(screen.getByRole('button',{name:'Copy breakdown'}));
 await screen.findByText('Breakdown copied.');
 expect(writeText).toHaveBeenCalledOnce();
 const text=writeText.mock.calls[0]?.[0] as string|undefined;
 expect(text).toContain(`${source} score breakdown\nTotal: 6.2\nThreshold: 5\nRating: likely spam`);
 expect(text).toContain('BAYES_99\t+6.3\tBODY: Bayes probability\nDKIM_VALID\t−0.1\tValid signature\nUNKNOWN_POINTS\tNot given\t');
 expect(text).toContain('not independently verified');
 expect(text).not.toContain('Copy breakdown');
});
it('reports clipboard refusal and supports retry without inventing missing scores or rules',async()=>{
 await loadCatalog('spam-score');
 const writeText=vi.fn<(text:string)=>Promise<void>>().mockRejectedValueOnce(new Error('denied')).mockResolvedValueOnce(undefined);
 vi.stubGlobal('navigator',{clipboard:{writeText}});
 render(()=><SpamScoreBreakdown score={{source:'SpamAssassin',score:0,threshold:null,spam:null,rules:[]}}/>);
 const button=screen.getByRole('button',{name:'Copy breakdown'});fireEvent.click(button);
 await screen.findByText('The breakdown could not be copied. Try again.');
 expect(screen.queryByText('Breakdown copied.')).toBeNull();
 expect(writeText.mock.calls[0]?.[0]).toContain('Threshold: Not reported');
 expect(writeText.mock.calls[0]?.[0]).toContain('The header lists no individual rules.');
 fireEvent.click(button);await screen.findByText('Breakdown copied.');
 await waitFor(()=>expect(button).toBeEnabled());expect(writeText).toHaveBeenCalledTimes(2);
});
