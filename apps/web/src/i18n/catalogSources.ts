// Production builds substitute catalogAssetSources.ts through the Vite plugin.
export default import.meta.glob<string>('../../locales/*/*.ftl',{query:'?raw',import:'default'});
/** Development and tests load one file per namespace; English is a separate chain entry. */
export const mergedFallback = false;
