import {normalizeComposerFont} from './fontCatalog.ts';

// Explicit, URL-free CSS subset for imported email layout. Revalidate on output
// too: document attributes can also be constructed by editor commands.
const lengths=/^(?:0|\d{1,4}(?:\.\d{1,3})?(?:px|pt|em|rem|%))$/;
const colors=/^(?:#[0-9a-f]{3}(?:[0-9a-f]{3})?|rgba?\([\d\s,.%]+\)|transparent|black|white)$/i;
export function safeEmailStyle(property:string,value:string):boolean {
 if(value.length>160)return false;
 if(/^(?:width|min-width|max-width|height|min-height|max-height|margin-(?:top|right|bottom|left)|padding-(?:top|right|bottom|left))$/.test(property))return lengths.test(value)||value==='auto';
 if(/^border-(?:top|right|bottom|left)-width$/.test(property))return lengths.test(value)&&!value.includes('%');
 if(/^border-(?:top|right|bottom|left)-style$/.test(property))return /^(?:none|solid|dashed|dotted|double)$/.test(value);
 if(property==='color'||property==='background-color'||/^border-(?:top|right|bottom|left)-color$/.test(property))return colors.test(value);
 if(property==='font-family')return !!normalizeComposerFont(value);
 if(property==='font-size')return /^\d{1,2}(?:\.\d{1,2})?(?:px|pt)$/.test(value);
 if(property==='line-height')return /^(?:[0-3](?:\.\d{1,3})?|\d{1,2}(?:px|pt))$/.test(value);
 const enums:Record<string,RegExp>={'text-align':/^(left|right|center|justify)$/,'vertical-align':/^(top|middle|bottom|baseline)$/,'direction':/^(ltr|rtl)$/,'font-weight':/^(normal|bold|[1-9]00)$/,'font-style':/^(normal|italic|oblique)$/,'border-collapse':/^(collapse|separate)$/,'white-space':/^(normal|pre-wrap)$/};
 return Object.hasOwn(enums,property)&&enums[property]!.test(value);
}
const properties=['width','min-width','max-width','height','min-height','max-height','color','background-color','font-family','font-size','line-height','text-align','vertical-align','direction','font-weight','font-style','border-collapse','white-space',...['top','right','bottom','left'].flatMap(side=>[`margin-${side}`,`padding-${side}`,`border-${side}-width`,`border-${side}-style`,`border-${side}-color`])];
export function readEmailStyles(element:HTMLElement):Record<string,string> {
 const result:Record<string,string>={};
 for(const property of properties){const value=element.style.getPropertyValue(property);if(safeEmailStyle(property,value))result[property]=value;}
 const legacy=(property:string,value:string|null,unit='')=>{if(!result[property]&&value){const candidate=/^\d+$/.test(value)?value+unit:value;if(safeEmailStyle(property,candidate))result[property]=candidate;}};
 legacy('width',element.getAttribute('width'),'px');legacy('height',element.getAttribute('height'),'px');legacy('text-align',element.getAttribute('align'));legacy('vertical-align',element.getAttribute('valign'));legacy('background-color',element.getAttribute('bgcolor'));
 if(element.matches('td,th')){const padding=element.closest('table')?.getAttribute('cellpadding')??null;for(const side of ['top','right','bottom','left'])legacy(`padding-${side}`,padding,'px');}
 return result;
}
export function applyEmailStyles(element:HTMLElement,styles:unknown):void {
 if(!styles||typeof styles!=='object')return;
 for(const [property,value]of Object.entries(styles))if(typeof value==='string'&&safeEmailStyle(property,value))element.style.setProperty(property,value);
}
