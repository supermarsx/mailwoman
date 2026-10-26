import {test, expect} from '../csrf.ts';

test('appearance code stays in an opaque bounded worker @phone', async ({page}) => {
  await page.goto('/');
  const result = await page.evaluate(async () => {
    const run = (code: string): Promise<Array<Record<string, unknown>>> => new Promise((resolve, reject) => {
      const frame = document.createElement('iframe');
      frame.hidden = true; frame.sandbox.add('allow-scripts'); frame.src = '/api/appearance/sandbox';
      const channel = new MessageChannel(); const messages: Array<Record<string, unknown>> = [];
      const timeout = setTimeout(() => {frame.remove(); channel.port1.close(); reject(new Error('Sandbox did not answer'));}, 5000);
      channel.port1.onmessage = ({data}) => {
        messages.push(data);
        if (data.kind === 'ready') channel.port1.postMessage({kind:'run', code, context:{dark:true,reducedMotion:false}});
        if (data.kind === 'done' || data.kind === 'error') {clearTimeout(timeout); frame.remove(); channel.port1.close(); resolve(messages);}
      };
      frame.onload = () => frame.contentWindow?.postMessage({kind:'appearance-connect'}, '*', [channel.port2]);
      document.body.append(frame);
    });
    return {
      good: await run("appearance.set({density:'compact', accent: typeof document, font: typeof indexedDB, mode: typeof fetch});"),
      loop: await run('while(true){}'),
    };
  });
  expect(result.good.at(-1)?.kind).toBe('done');
  expect(result.good.find(message => message.kind === 'appearance-set')?.value).toEqual({density:'compact',accent:'undefined',font:'undefined',mode:'undefined'});
  expect(result.loop.at(-1)?.kind).toBe('error');
  expect(result.loop.at(-1)?.message).toContain('time limit');
});
