// The English UI catalogue (apps/web/locales/en/*.ftl), so manifest locators can
// name a control by its catalogue key ("@compose-new") and break loudly when the
// label changes.
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { WEB_APP, importFromWeb } from './paths.mjs';

let bundle;

async function load() {
  if (bundle) return bundle;
  const { FluentBundle, FluentResource } = await importFromWeb('@fluent/bundle');
  bundle = new FluentBundle('en-US', { useIsolating: false });
  const dir = join(WEB_APP, 'locales', 'en');
  for (const file of readdirSync(dir).sort()) {
    if (!file.endsWith('.ftl')) continue;
    const errors = bundle.addResource(new FluentResource(readFileSync(join(dir, file), 'utf8')), { allowOverrides: true });
    if (errors.length) throw new Error(`${file}: ${errors[0]}`);
  }
  return bundle;
}

const SENTINEL = '\u0000VAR\u0000';
const escapeRe = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/**
 * Resolve "@key" or "@key.attribute" to its English text. With `args` the
 * variables are filled in; without, a message that has variables becomes a
 * RegExp in which each variable matches any text.
 */
export async function catalogue(ref, args) {
  const b = await load();
  const [id, attr] = ref.slice(1).split('.');
  const message = b.getMessage(id);
  if (!message) throw new Error(`catalogue key ${ref} does not exist in apps/web/locales/en`);
  const pattern = attr ? message.attributes[attr] : message.value;
  if (!pattern) throw new Error(`catalogue key ${ref} has no ${attr ? `attribute .${attr}` : 'value'}`);
  if (args) {
    const errors = [];
    const text = b.formatPattern(pattern, args, errors);
    if (errors.length) throw new Error(`catalogue key ${ref}: ${errors[0].message}`);
    return text;
  }
  const errors = [];
  const proxy = new Proxy({}, { get: () => SENTINEL, has: () => true });
  const text = b.formatPattern(pattern, proxy, errors);
  if (!text.includes(SENTINEL)) return text;
  return new RegExp(`^${text.split(SENTINEL).map(escapeRe).join('.*?')}$`);
}

/** True if `ref` is a catalogue reference. */
export const isRef = (v) => typeof v === 'string' && /^@[a-z0-9][a-z0-9-]*(\.[a-zA-Z0-9-]+)?$/i.test(v);
