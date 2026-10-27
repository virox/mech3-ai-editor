import type { Json } from './types';

export interface AnimationLink { eventIndex:number; kind:string; target?:string; value?:number; start:Json }
export interface AnimationSequenceSummary { name:string; seqState:Json; eventCount:number; links:AnimationLink[] }
export interface AnimationPrerequisite { kind:string; name:string; required?:boolean|null }
export interface AnimationDefinitionSummary { index:number; fileName:string; name:string; animName:string; rootName:string; activation:Json; execution:Json; health:Json; resetTime:Json; hasCallbacks:boolean; prerequisites:AnimationPrerequisite[]; sequences:AnimationSequenceSummary[] }
export interface AnimationIndex { path:string; relativePath:string; mission:Json; definitions:AnimationDefinitionSummary[] }

/** Event kinds that reference another animation definition by `anim_name`. */
export const ANIMATION_EDGE_KINDS=['CallAnimation','StopAnimation','InvalidateAnimation','ResetAnimation'] as const;
export type AnimationEdgeKind=typeof ANIMATION_EDGE_KINDS[number]|'Prerequisite';

export interface AnimationEdge { id:string; from:number; sequence:number; eventIndex:number; kind:AnimationEdgeKind; target:string; to:number[]; start:Json }

/** Names the executable calls by itself at mission start (format strings in Mech3.exe). */
export function executableEntry(animName:string){return /^start_(single_player|multiplayer)(_m\d+)?$/.test(animName)}

const variant=(value:Json):[string,Json]|undefined=>value!==null&&typeof value==='object'&&!Array.isArray(value)?Object.entries(value)[0] as [string,Json]|undefined:typeof value==='string'?[value,null]:undefined;

export function activationLabel(activation:Json){const entry=variant(activation);return entry?entry[0]:'—'}

export function executionLabel(execution:Json){
  const entry=variant(execution);if(!entry)return 'None';
  const [kind,data]=entry;
  // anim.zbd stores range limits squared; show the authored radius.
  if(kind==='ByRange'&&data&&typeof data==='object'&&!Array.isArray(data)){const max=Number((data as Record<string,Json>).max),min=Number((data as Record<string,Json>).min);return `ByRange ${min?`${Math.sqrt(min).toFixed(0)}–`:'≤ '}${Math.sqrt(max).toFixed(0)}`}
  return data===null||typeof data!=='object'?kind:`${kind} ${JSON.stringify(data)}`;
}

export function startLabel(start:Json){
  if(!start||typeof start!=='object'||Array.isArray(start))return '';
  const {offset,time}=start as Record<string,Json>;
  return `${String(offset).toLowerCase()} +${Number(time).toFixed(time&&Number(time)%1?2:0)}s`;
}

/** What a CALLBACK value does, per docs/aiv-research/animation-callbacks.md. */
export function callbackMeaning(value:number){
  if(value>=42&&value<=49){const depth=(value-42)>>1,state=value&1;return depth?`writes the ${['parent','grandparent','great-grandparent'][depth-1]} node's name = ${state}`:`writes this definition's NAME = ${state}`}
  if(value>=50&&value<=55)return 'owner virtual callback only; ignored by state collector';
  if(value===56)return 'notifies the game object owning the node';
  if(value>=57&&value<=60)return `transition mode ${({57:1,58:0,59:3,60:2} as Record<number,number>)[value]}`;
  return 'unknown callback';
}

export function stateChannel(definition:AnimationDefinitionSummary,value:number):string|undefined{return value===42||value===43?definition.name:undefined}

export function indexByAnimName(index:AnimationIndex){const map=new Map<string,number[]>();for(const definition of index.definitions)map.set(definition.animName,[...(map.get(definition.animName)??[]),definition.index]);return map}

export function animationEdges(index:AnimationIndex):AnimationEdge[]{
  const byName=indexByAnimName(index),edges:AnimationEdge[]=[];
  for(const definition of index.definitions){
    // Self references (e.g. INVALIDATE_ANIMATION of its own name) stay in the event list, not the graph.
    definition.sequences.forEach((sequence,sequenceIndex)=>{for(const link of sequence.links){if(!link.target||link.target===definition.animName||!(ANIMATION_EDGE_KINDS as readonly string[]).includes(link.kind))continue;edges.push({id:`${definition.index}:${sequenceIndex}:${link.eventIndex}`,from:definition.index,sequence:sequenceIndex,eventIndex:link.eventIndex,kind:link.kind as AnimationEdgeKind,target:link.target,to:(byName.get(link.target)??[]).filter(index=>index!==definition.index),start:link.start})}});
    // A prerequisite animation must have run before this one may activate.
    definition.prerequisites.forEach((prerequisite,position)=>{if(prerequisite.kind!=='Animation'||!prerequisite.name)return;for(const from of byName.get(prerequisite.name)??[])if(from!==definition.index)edges.push({id:`pre:${definition.index}:${position}:${from}`,from,sequence:-1,eventIndex:-1,kind:'Prerequisite',target:definition.animName,to:[definition.index],start:null})});
  }
  return edges;
}

export interface Neighborhood { layers:Map<number,number>; external:{id:string;name:string;layer:number;from:number}[]; truncated:number }

/**
 * Callees expand only rightward and callers only leftward, so shared effect
 * animations (called from hundreds of places) do not pull in all their callers.
 */
export function neighborhood(edges:AnimationEdge[],focus:number,depth:number,maxPerLayer=24):Neighborhood{
  const layers=new Map<number,number>([[focus,0]]),external:Neighborhood['external']=[];let truncated=0;
  const outgoing=new Map<number,AnimationEdge[]>(),incoming=new Map<number,AnimationEdge[]>();
  for(const edge of edges){outgoing.set(edge.from,[...(outgoing.get(edge.from)??[]),edge]);for(const to of edge.to)incoming.set(to,[...(incoming.get(to)??[]),edge])}
  for(const direction of [1,-1]){
    let frontier=[focus];
    for(let step=1;step<=depth&&frontier.length;step++){
      const next:number[]=[];
      for(const node of frontier){
        const candidates=direction>0?(outgoing.get(node)??[]).flatMap(edge=>{if(!edge.to.length&&node===focus&&!external.some(item=>item.name===edge.target&&item.from===node))external.push({id:`external:${node}:${edge.target}`,name:edge.target,layer:1,from:node});return edge.to}):(incoming.get(node)??[]).map(edge=>edge.from);
        for(const candidate of candidates){if(layers.has(candidate))continue;if(next.length>=maxPerLayer){truncated++;continue}layers.set(candidate,step*direction);next.push(candidate)}
      }
      frontier=next;
    }
  }
  return {layers,external,truncated};
}

export const CARD_WIDTH=280;
export const CARD_HEADER=74;
export const ROW_HEIGHT=22;
export function cardHeight(definition:AnimationDefinitionSummary){return CARD_HEADER+Math.max(1,definition.sequences.length)*ROW_HEIGHT+10}

export function layoutNeighborhood(index:AnimationIndex,hood:Neighborhood){
  const columns=new Map<number,{id:string;height:number;order:string}[]>();
  for(const [definition,layer] of hood.layers){const item=index.definitions[definition];columns.set(layer,[...(columns.get(layer)??[]),{id:`def:${definition}`,height:cardHeight(item),order:`${item.animName}\u0000${item.name}`}])}
  for(const item of hood.external)columns.set(item.layer,[...(columns.get(item.layer)??[]),{id:item.id,height:44,order:`~${item.name}`}]);
  const minLayer=Math.min(0,...columns.keys()),positions:Record<string,{x:number;y:number}>={};
  for(const [layer,items] of columns){items.sort((a,b)=>a.order.localeCompare(b.order));let y=60;for(const item of items){positions[item.id]={x:60+(layer-minLayer)*(CARD_WIDTH+120),y};y+=item.height+28}}
  return positions;
}

type EventData=Record<string,Json>;
const field=(data:EventData,key:string)=>data[key];
const atNode=(value:Json)=>value&&typeof value==='object'&&!Array.isArray(value)?String((value as EventData).name??(value as EventData).node??''):'';
const number=(value:Json)=>typeof value==='number'?Number(value.toFixed(3)).toString():String(value??'');

/** One readable line per event for the inspector timeline. */
export function eventSummary(event:Json):{kind:string;start:string;text:string}{
  const record=(event&&typeof event==='object'&&!Array.isArray(event)?event:{}) as EventData;
  const entry=variant(record.data??null),kind=entry?.[0]??'Unknown';
  const data=(entry?.[1]&&typeof entry[1]==='object'&&!Array.isArray(entry[1])?entry[1]:{}) as EventData;
  const name=String(data.name??data.node??'');
  let text:string;
  switch(kind){
    case'CallAnimation':{const params=variant(data.parameters??null);text=`${name}${params?` ${params[0]} ${atNode(params[1])}`:''}${data.wait_for_completion!=null?' (wait)':''}`;break}
    case'ObjectActiveState':text=`${name} ${data.state?'active':'inactive'}`;break;
    case'ObjectMotion':case'ObjectMotionFromTo':case'ObjectOpacityFromTo':case'LightAnimation':case'FbfxColorFromTo':text=`${name}${data.run_time!=null?` for ${number(field(data,'run_time'))}s`:''}${data.gravity?' with gravity':''}`;break;
    case'Sound':text=`${name}${data.at_node?` at ${atNode(data.at_node)}`:''}`;break;
    case'Callback':{const value=Number(data.value);text=`${value} – ${callbackMeaning(value)}`;break}
    case'If':case'Elseif':text=JSON.stringify(data.condition??null);break;
    case'Loop':text=JSON.stringify(data);break;
    case'DetonateWeapon':text=`${String(data.weapon)} at ${atNode(data.at_node)}`;break;
    case'Else':case'Endif':text='';break;
    default:text=name||JSON.stringify(data).slice(0,90);
  }
  return {kind,start:startLabel(record.start??null),text};
}
