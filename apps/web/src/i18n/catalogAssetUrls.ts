import {catalogAssetLoaders} from './catalogAssetLoaders.ts';
export default catalogAssetLoaders(import.meta.glob<string>('../../locales/*/*.ftl',{query:'?url',import:'default',eager:true}));
