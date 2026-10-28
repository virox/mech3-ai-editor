import type { Json } from './types';

/** Tags that read or write named state channels; shown on the map next to their node or edge. */
export const CHANNEL_TAGS=new Set(['valve','valveunion','valve_assign']);

export type TagBadgeKind='condition'|'assign'|'gate'|'other';
export interface TagBadge { kind:TagBadgeKind; text:string; title:string }

const text=(value:Json|undefined)=>typeof value==='string'?value:JSON.stringify(value??null);
const list=(value:Json|undefined):Json[]=>Array.isArray(value)?value:[];
const enabled=(value:Json|undefined)=>value===1||value===true;

// Layouts per docs/editor-systems/valve-and-state-logic.md: valve [enabled, channel, target?],
// valveunion [[enabled, channel, target?], ...], valve_assign [channel, value], gate [vehicle].
function condition(member:Json[]){
  const target=member[2]===undefined?'':` = ${text(member[2])}`;
  return `${text(member[1])}${target}${enabled(member[0])?'':' (off)'}`;
}

export function tagBadge(tag:string,value:Json):TagBadge{
  const args=list(value);
  switch(tag){
    case'valve':return {kind:'condition',text:`⏸ ${condition(args)}`,title:`Waits until ${text(args[1])} ${args[2]===undefined?'is 1 on the last outgoing link (0 on the others)':`= ${text(args[2])}`} before leaving on this link`};
    case'valveunion':{const members=args.map(list).filter(member=>enabled(member[0]));return {kind:'condition',text:`⏸ ${members.map(member=>text(member[1])).join(' | ')||'(none enabled)'}`,title:`Waits until any of: ${members.map(member=>`${text(member[1])} = ${member[2]===undefined?'1':text(member[2])}`).join(', ')}`}}
    case'valve_assign':return {kind:'assign',text:`→ ${text(args[0])} = ${text(args[1])}`,title:`Sets ${text(args[0])} = ${text(args[1])} when the unit departs here`};
    case'gate':return {kind:'gate',text:`gate ${args.map(text).join(', ')}`,title:`Only ${args.map(text).join(', ')} may use this connection`};
    default:return {kind:'other',text:tag,title:`${tag} ${text(value)}`};
  }
}

/** Approximate badge width in map units for the 12px map label font. */
export function badgeWidth(label:string){return Math.max(54,Math.round(label.length*6.8+16))}
