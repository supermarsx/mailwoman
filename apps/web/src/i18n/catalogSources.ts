// Production builds substitute catalogAssetSources.ts through the Vite plugin.
export default import.meta.glob<string>('../../locales/*/*.ftl',{query:'?raw',import:'default'});
