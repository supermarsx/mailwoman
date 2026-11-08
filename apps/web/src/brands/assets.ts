// Existing official marks bundled by the sender catalogue. Keep provider/header
// rendering local too, including brands without a compact vector path.
import meo from '../components/senderIcons/assets/meo-site.png';
export function brandAsset(id:string):string|undefined {return id==='meo'?meo:undefined;}
