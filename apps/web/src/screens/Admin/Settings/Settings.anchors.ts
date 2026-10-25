// Search anchors of the admin Settings section (t32-cfg UX10, worker f01): every
// registry key the console has a name for, so Ctrl/Cmd+K finds it and opens
// `#/settings/<key>?field=<key>`. The `signin.*` keys are anchored by the
// section that writes them (Sign-in policy), the server text family by its own.
// Ids are `settings:<key>`: the bare key may be the anchor of another screen's
// field for the same knob (the configuration catalogue), and ids are unique.

import { defineFieldAnchors } from '../shell/fieldAnchors.ts';
import { KNOWN_KEYS, keySlug, settingAnchorId } from './keys.ts';

export default defineFieldAnchors({
  section: 'settings',
  catalog: 'admin-settings',
  anchors: KNOWN_KEYS.filter((k) => !k.startsWith('signin.') && !k.endsWith('.*')).map((key) => ({
    id: settingAnchorId(key),
    rest: [key],
    labelId: `admin-settings-key-${keySlug(key)}`,
    keywords: [key],
    capability: 'settings.read' as const,
  })),
});
