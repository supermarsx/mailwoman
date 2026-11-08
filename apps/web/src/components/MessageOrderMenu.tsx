import type {JSX} from 'solid-js';
import {createContextMenu} from './ContextMenu.tsx';
import type {MessageOrderController} from './messageOrder.ts';
import {Icon} from './Icon.tsx';
import {t} from '../i18n';
import * as thread from './threadList.css.ts';
import * as a11y from './mailA11y.css.ts';
/** The order button stays available in cards, table mode and an empty list. */
export function MessageOrderMenu(props:{order:MessageOrderController;scope:()=>string}):JSX.Element {
 let button:HTMLButtonElement|undefined;
 const menu=createContextMenu({label:()=>t('mail-order-by'),scope:props.scope,actions:props.order.actions,focusCurrent:true,ownerDocument:()=>button?.ownerDocument});
 return <><button ref={button} type="button" class={`${thread.segBtn} ${a11y.focusable}`} data-testid="list-order-menu" aria-label={t('mail-order-by')} title={props.order.locked()?t('mail-list-order-locked'):`${t('mail-order-by')}: ${t(`mail-list-order-${props.order.value()}`)}`} aria-haspopup="menu" aria-expanded={menu.isOpen()} disabled={props.order.locked()||props.order.saving()} onClick={()=>{if(menu.isOpen())menu.close();else if(button)menu.openBelow(button);}}><Icon name="sort"/></button>{menu.menu()}</>;
}
