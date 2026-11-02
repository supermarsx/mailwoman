import {defineFieldAnchors} from './shell/fieldAnchors.ts';

export default defineFieldAnchors({
  section: 'preferences',
  catalog: 'appearance',
  anchors: [{
    id: 'personal.leetspeak',
    rest: ['appearance'],
    labelId: 'appearance-text-mode-label',
    keywords: ['leetspeak', '1337', 'normal text', 'text style'],
  }],
});
