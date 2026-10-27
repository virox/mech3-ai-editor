import type { Json, ValveGraphHost, ValveGraphNode, ValveGraphPort, ValveGraphSnapshot } from './types';

export interface GraphPoint { x:number; y:number }
export type GraphPositions = Record<string,GraphPoint>;

export const BLOCK_WIDTH=220;
export const BLOCK_HEIGHT=104;
export const CHANNEL_WIDTH=190;
export const CHANNEL_HEIGHT=62;
export const MAX_AUTO_LAYOUT_ROWS=20;
const AUTO_LAYOUT_ROW_GAP=38;

export interface GraphRect {x:number;y:number;w:number;h:number}

export function intersectsRect(point:GraphPoint,size:{w:number;h:number},rect:GraphRect){return point.x<rect.x+rect.w&&point.x+size.w>rect.x&&point.y<rect.y+rect.h&&point.y+size.h>rect.y}

export function translatePositions(positions:GraphPositions,origins:GraphPositions,dx:number,dy:number):GraphPositions{
  const next={...positions};
  for(const[id,origin]of Object.entries(origins))next[id]={x:origin.x+dx,y:origin.y+dy};
  return next;
}

const categoryColumn:Record<string,number>={producer:0,channel:1,logic:2,consumer:3,unknown:3};

export function channelNodeId(name:string){return `channel::${name}`}
export function endpointNodeId(graph:ValveGraphSnapshot,id:string){return id.startsWith('channel::')?id:graph.nodes.find(node=>node.id===id||node.ports.some(port=>port.id===id))?.id??id}

function recordValue(node:ValveGraphNode,index:number){return Array.isArray(node.nativeRecord)?node.nativeRecord[index]:undefined}
function enabled(value:unknown){return value===true||value===1||value==='1'}
function decimal(value:unknown){return typeof value==='number'&&Number.isFinite(value)?value.toFixed(3):String(value??'')}

export const READ_ONLY_KINDS=new Set(['unknown-action','animation-callback']);
export const READ_ONLY_PORT_KINDS=new Set(['animation-callback']);
const DETAILED_CARD_KINDS=new Set(['vlist-producer','target-health-producer','animation-callback']);
export function hasDetailedCard(node:ValveGraphNode){return DETAILED_CARD_KINDS.has(node.kind)||nodeCardLines(node).length>0}

const text=(value:unknown)=>typeof value==='string'?value:JSON.stringify(value??null);
const list=(value:unknown):unknown[]=>Array.isArray(value)?value:[];

// valves.zrd blocks carry [event, [action, args]]; argument meanings follow
// docs/editor-systems/valve-and-state-logic.md and stay technical where unproven.
function valveActionLines(node:ValveGraphNode):string[]{
  const record=list(recordValue(node,1)),action=text(record[0]),args=list(record[1]);
  switch(node.kind){
    case'valve-sound':return[`Sound: ${text(args[0])}`,`Value: ${text(args[1])}`];
    case'valve-sound_once':return[`Sound once: ${text(args[0])}`,`Value: ${text(args[1])}`];
    case'valve-shutdown':return[`Vehicle: ${text(args[0])}`,`Arguments: ${text(args[1])}, ${text(args[2])} (unproven)`];
    case'valve-destroy':return[`Vehicle: ${text(args[0])}`,`State: ${text(args[1])}`];
    case'valve-teleport':return[`Vehicle: ${text(args[0])}`,`Position: ${list(args[1]).map(decimal).join(', ')}`,`Yaw: ${decimal(args[2])}`,`State: ${text(args[3])}`];
    case'valve-attack_strategy':return[`Vehicle: ${text(args[0])}`,`Strategy: ${text(args[1])}`,`State: ${text(args[2])}`];
    case'unknown-action':return[`Raw: ${action} ${record.length>1?text(record.slice(1)):''}`.trim()];
    default:return[];
  }
}

// Network blocks carry [tag, value].
function networkLines(node:ValveGraphNode):string[]{
  const value=list(recordValue(node,1));
  // A bare `valve` derives its target from the outgoing-link index; a bare
  // `valveunion` member defaults to 1 (docs/editor-systems/ai-networks-and-regions.md).
  const condition=(member:unknown[],missing:string)=>`${text(member[1])}${member[2]===undefined?missing:` = ${text(member[2])}`}${enabled(member[0])?'':' · disabled'}`;
  switch(node.kind){
    case'net-condition':return[`Requires: ${condition(value,' (target from link index)')}`];
    case'net-union':return value.map(member=>`Any of: ${condition(list(member),' = 1 (default)')}`);
    // The assigned channel is the card's output row.
    case'net-assignment':return[];
    default:return[];
  }
}

function animationCallbackLines(node:ValveGraphNode):string[]{
  const p=node.properties as Record<string,unknown>;
  const lines=[`Callback ${String(p.callback??43)} in ${String(p.sequence||'sequence')}`];
  if(p.activation)lines.push(`Activation: ${String(p.activation)}`);
  if(p.executionRange!==null&&p.executionRange!==undefined)lines.push(`Execution range: ${String(p.executionRange)}`);
  const callers=Array.isArray(p.calledBy)?p.calledBy.map(String):[];
  lines.push(...(callers.length?callers.map(caller=>`Called by: ${caller}`):['Caller: executable or unloaded']));
  return lines;
}

export function nodeCardLines(node:ValveGraphNode):string[]{
  if(node.kind==='animation-callback')return animationCallbackLines(node);
  if(node.kind.startsWith('valve-')||node.kind==='unknown-action')return valveActionLines(node);
  if(node.kind.startsWith('net-'))return networkLines(node);
  if(node.kind==='target-health-producer'){
    const threshold=Array.isArray(node.nativeRecord)?node.nativeRecord[2]:node.properties.threshold;
    return [`Health Threshold: ${decimal(threshold)}`];
  }
  if(node.kind!=='vlist-producer')return [];
  const lines:string[]=[];
  const region=recordValue(node,9);
  if(enabled(recordValue(node,1)))lines.push(`Ammo Threshold: ${decimal(recordValue(node,2))}`);
  if(enabled(recordValue(node,3)))lines.push('Fired Upon');
  if(enabled(recordValue(node,4)))lines.push(`Health Threshold: ${decimal(recordValue(node,5))}`);
  if(enabled(recordValue(node,6)))lines.push(`Heat Threshold: ${decimal(recordValue(node,7))}`);
  if(enabled(recordValue(node,8)))lines.push(`Enters: ${String(region??'region')}`);
  if(enabled(recordValue(node,10)))lines.push(`Target LOS Range: ${decimal(recordValue(node,11))}`);
  if(enabled(recordValue(node,12)))lines.push(`Target Distance: ${decimal(recordValue(node,13))}`);
  if(enabled(recordValue(node,16)))lines.push(`Exits: ${String(region??'region')}`);
  if(!lines.length)lines.push('No trigger conditions');
  return lines;
}

/** Every producer shows what it writes on its own highlighted row below the subtitle. */
export function nodeOutputLine(node:ValveGraphNode){
  const writes=node.ports.filter(item=>item.direction==='write');
  if(!writes.length)return '';
  const port=writes[0];
  return `${port.channel}${port.value===undefined?'':` = ${port.value}`}${writes.length>1?` +${writes.length-1}`:''}`;
}

export const OUTPUT_ROW_Y=82;
const OUTPUT_ROW_HEIGHT=18;
/** Vertical shift of everything below the output row. */
export function nodeOutputOffset(node:ValveGraphNode){return nodeOutputLine(node)?OUTPUT_ROW_HEIGHT:0}

/** Read ports stack from the label row; write ports start at the output row so the highlighted channel sits next to its port. */
export function portOffsetY(node:ValveGraphNode,index:number){
  const port=node.ports[index];
  const sameDirection=node.ports.slice(0,index).filter(item=>item.direction===port.direction).length;
  return (port.direction==='write'?OUTPUT_ROW_Y-5:54)+sameDirection*18;
}

export function nodeIsActive(node:ValveGraphNode){return node.kind==='vlist-producer'&&enabled(recordValue(node,14))}

export function nodeCardHeight(node:ValveGraphNode){return (hasDetailedCard(node)?Math.max(BLOCK_HEIGHT,92+nodeCardLines(node).length*14):BLOCK_HEIGHT)+nodeOutputOffset(node)}

export function cycleNodeIds(graph:ValveGraphSnapshot):Set<string>{
  const links=new Map<string,string[]>();
  for(const node of graph.nodes){const reads=node.ports.filter(port=>port.direction==='read').map(port=>port.channel),writes=node.ports.filter(port=>port.direction==='write').map(port=>port.channel);for(const read of reads)links.set(read,[...(links.get(read)??[]),...writes])}
  const reachable=(from:string,to:string,seen=new Set<string>()):boolean=>from===to||!seen.has(from)&&((seen.add(from)),(links.get(from)??[]).some(next=>reachable(next,to,seen)));
  return new Set(graph.nodes.filter(node=>{const reads=node.ports.filter(port=>port.direction==='read'),writes=node.ports.filter(port=>port.direction==='write');return reads.some(read=>writes.some(write=>reachable(write.channel,read.channel)))}).map(node=>node.id));
}

export function autoLayout(graph:ValveGraphSnapshot):GraphPositions{
  const lanes=new Map<number,{id:string;order:number;label:string;height:number}[]>();
  const cycles=cycleNodeIds(graph);
  const add=(column:number,id:string,order:number,label:string,height:number)=>lanes.set(column,[...(lanes.get(column)??[]),{id,order,label,height}]);
  for(const node of graph.nodes)if(!cycles.has(node.id))add(categoryColumn[node.category]??3,node.id,node.order,node.label,nodeCardHeight(node));
  for(const channel of graph.channels)add(1,channelNodeId(channel.name),0,channel.name,CHANNEL_HEIGHT);
  const positions:GraphPositions={};
  const sortedLanes=new Map<number,{id:string;order:number;label:string;height:number}[]>();
  for(const [column,items] of lanes){
    items.sort((a,b)=>a.order-b.order||a.label.localeCompare(b.label)||a.id.localeCompare(b.id));
    sortedLanes.set(column,items);
  }
  for(const [column,items] of sortedLanes){
    const expandedColumns=[...sortedLanes.entries()]
      .filter(([lane])=>lane<column)
      .reduce((total,[,laneItems])=>total+Math.max(0,Math.ceil(laneItems.length/MAX_AUTO_LAYOUT_ROWS)-1),0);
    const columnRowHeights=Array.from({length:Math.ceil(items.length/MAX_AUTO_LAYOUT_ROWS)},()=>Array.from({length:MAX_AUTO_LAYOUT_ROWS},()=>0));
    items.forEach((item,index)=>{const wrapColumn=Math.floor(index/MAX_AUTO_LAYOUT_ROWS);const row=index%MAX_AUTO_LAYOUT_ROWS;columnRowHeights[wrapColumn][row]=Math.max(columnRowHeights[wrapColumn][row],item.height)});
    items.forEach((item,index)=>{
      const wrapColumn=Math.floor(index/MAX_AUTO_LAYOUT_ROWS);
      const row=index%MAX_AUTO_LAYOUT_ROWS;
      const yOffset=columnRowHeights[wrapColumn].slice(0,row).reduce((total,height)=>total+height+AUTO_LAYOUT_ROW_GAP,0);
      positions[item.id]={x:70+(column+expandedColumns+wrapColumn)*340,y:70+yOffset};
    });
  }
  if(cycles.size){
    const startY=Math.max(70,...graph.nodes.filter(node=>!cycles.has(node.id)).map(node=>(positions[node.id]?.y??70)+nodeCardHeight(node)))+116;
    const cycleNodes=graph.nodes.filter(node=>cycles.has(node.id)).sort((a,b)=>a.order-b.order||a.id.localeCompare(b.id));
    const cycleChannels=[...new Set(cycleNodes.flatMap(node=>node.ports.map(port=>port.channel)))].sort();
    cycleChannels.forEach((name,index)=>positions[channelNodeId(name)]={x:410+Math.floor(index/MAX_AUTO_LAYOUT_ROWS)*340,y:startY+(index%MAX_AUTO_LAYOUT_ROWS)*92});
    cycleNodes.forEach((node,index)=>positions[node.id]={x:750+Math.floor(index/MAX_AUTO_LAYOUT_ROWS)*340,y:startY+(index%MAX_AUTO_LAYOUT_ROWS)*142});
  }
  return positions;
}

export function mergePositions(graph:ValveGraphSnapshot,saved:GraphPositions|undefined):GraphPositions{
  const automatic=autoLayout(graph);
  if(!saved)return automatic;
  const merged:GraphPositions={};
  for(const [id,point] of Object.entries(saved))if(id in automatic&&Number.isFinite(point.x)&&Number.isFinite(point.y))merged[id]=point;
  const size=(id:string)=>{if(id.startsWith('channel::'))return{w:CHANNEL_WIDTH,h:CHANNEL_HEIGHT};const node=graph.nodes.find(item=>item.id===id);return{w:BLOCK_WIDTH,h:node?nodeCardHeight(node):BLOCK_HEIGHT}};
  const occupied=(id:string,point:GraphPoint)=>{
    const current=size(id),gap=18;
    return Object.entries(merged).some(([otherId,other])=>{
      const otherSize=size(otherId);
      return point.x<other.x+otherSize.w+gap&&point.x+current.w+gap>other.x&&point.y<other.y+otherSize.h+gap&&point.y+current.h+gap>other.y;
    });
  };
  const missing=Object.entries(automatic).filter(([id])=>!merged[id]).sort((a,b)=>a[1].x-b[1].x||a[1].y-b[1].y||a[0].localeCompare(b[0]));
  for(const[id,automaticPoint]of missing){
    const point={...automaticPoint};
    while(occupied(id,point))point.y+=142;
    merged[id]=point;
  }
  return merged;
}

export function layoutStorageKey(root:string,mission:string){return`mech-reader-studio:behavior-layout:${root.toLowerCase()}::${mission.toLowerCase()}`}

export function connectedIds(graph:ValveGraphSnapshot,start:string):Set<string>{
  const found=new Set<string>([start]);
  let changed=true;
  while(changed){
    changed=false;
    for(const edge of graph.edges){const from=endpointNodeId(graph,edge.from),to=endpointNodeId(graph,edge.to);if(found.has(from)||found.has(to)){if(!found.has(from)){found.add(from);changed=true}if(!found.has(to)){found.add(to);changed=true}}}
  }
  return found;
}

export function scopedNodeIds(graph:ValveGraphSnapshot,allLoaded:boolean):Set<string>{
  if(allLoaded)return new Set([...graph.nodes.map(node=>node.id),...graph.channels.map(channel=>channelNodeId(channel.name))]);
  const missionNodes=graph.nodes.filter(node=>node.layer==='mission');
  // Some Reader variants (notably IA/MP archives) contain no graph-producing
  // records in the mission archive. In that case an empty mission seed would
  // hide the entire inherited campaign/global graph even though its channels
  // are indexed and shown in the module tab.
  if(!missionNodes.length)return new Set([...graph.nodes.map(node=>node.id),...graph.channels.map(channel=>channelNodeId(channel.name))]);
  const ids=new Set(missionNodes.map(node=>node.id));
  let changed=true;
  while(changed){
    changed=false;
    for(const edge of graph.edges){const from=endpointNodeId(graph,edge.from),to=endpointNodeId(graph,edge.to);if(ids.has(from)||ids.has(to)){if(!ids.has(from)){ids.add(from);changed=true}if(!ids.has(to)){ids.add(to);changed=true}}}
  }
  return ids;
}

export function visibleIds(graph:ValveGraphSnapshot,options:{allLoaded:boolean;search:string;layers:Set<string>;isolate?:string}):Set<string>{
  const scoped=scopedNodeIds(graph,options.allLoaded);
  const isolated=options.isolate?connectedIds(graph,options.isolate):undefined;
  const query=options.search.trim().toLowerCase();
  const visible=new Set<string>();
  for(const node of graph.nodes){
    if(!scoped.has(node.id)||isolated&&!isolated.has(node.id)||options.layers.size&&!options.layers.has(node.layer))continue;
    const haystack=`${node.label} ${node.subtitle} ${node.kind} ${node.layer} ${node.binding.entryName} ${node.ports.map(port=>port.channel).join(' ')}`.toLowerCase();
    if(!query||haystack.includes(query))visible.add(node.id);
  }
  for(const channel of graph.channels){
    const id=channelNodeId(channel.name);
    if(!scoped.has(id)||isolated&&!isolated.has(id)||options.layers.size&&!channel.layers.some(layer=>options.layers.has(layer)))continue;
    if(!query||channel.name.toLowerCase().includes(query)||graph.edges.some(edge=>edge.channel===channel.name&&(visible.has(endpointNodeId(graph,edge.from))||visible.has(endpointNodeId(graph,edge.to)))))visible.add(id);
  }
  if(query){
    for(const edge of graph.edges){const from=endpointNodeId(graph,edge.from),to=endpointNodeId(graph,edge.to);if(visible.has(from)||visible.has(to)){if(scoped.has(from))visible.add(from);if(scoped.has(to))visible.add(to)}}
  }
  return visible;
}

export function portsCompatible(a:ValveGraphPort,b:ValveGraphPort){return a.direction!==b.direction}

export interface PaletteBlock {kind:string;label:string;category:'producer'|'logic'|'consumer';hostKind:ValveGraphHost['kind'];description:string}
export const PALETTE:PaletteBlock[]=[
  {kind:'vlist-producer',label:'AI VList',category:'producer',hostKind:'vehicle-vlist',description:'Vehicle condition writes a state channel.'},
  {kind:'target-health-producer',label:'Target Health Below',category:'producer',hostKind:'target-health',description:'Target health threshold writes a state channel.'},
  {kind:'net-valve-assign',label:'Network Assign',category:'producer',hostKind:'network-tags',description:'A network connection assigns a channel value.'},
  {kind:'derived-any-nonzero',label:'Any Nonzero',category:'logic',hostKind:'valves-document',description:'Output is true when any input is nonzero.'},
  {kind:'derived-all-nonzero',label:'All Nonzero',category:'logic',hostKind:'valves-document',description:'Output is true when every input is nonzero.'},
  {kind:'derived-all-zero',label:'All Zero',category:'logic',hostKind:'valves-document',description:'Output is true when every input is zero.'},
  {kind:'valve-delayupdate',label:'Delay Update',category:'logic',hostKind:'valves-document',description:'Copies the trigger value after a delay.'},
  {kind:'valve-sound',label:'Sound',category:'consumer',hostKind:'valves-document',description:'Controls a named sound.'},
  {kind:'valve-sound_once',label:'Sound Once',category:'consumer',hostKind:'valves-document',description:'Plays a named sound once.'},
  {kind:'valve-shutdown',label:'Shutdown',category:'consumer',hostKind:'valves-document',description:'Changes shutdown state for a vehicle.'},
  {kind:'valve-destroy',label:'Destroy',category:'consumer',hostKind:'valves-document',description:'Destroys a vehicle at the requested state.'},
  {kind:'valve-teleport',label:'Teleport',category:'consumer',hostKind:'valves-document',description:'Teleports a vehicle to a position.'},
  {kind:'valve-attack_strategy',label:'Attack Strategy',category:'consumer',hostKind:'valves-document',description:'Changes a vehicle attack strategy.'},
  {kind:'net-valve',label:'Network Valve',category:'consumer',hostKind:'network-tags',description:'Gates a network node or edge by one channel.'},
  {kind:'net-valveunion',label:'Network Valve Union',category:'consumer',hostKind:'network-tags',description:'Gates a network node or edge by any listed state.'},
  {kind:'objective-observer',label:'Objective Observer',category:'consumer',hostKind:'objective-list',description:'Completes or fails an existing objective phase.'},
];

export interface BlockDraft {kind:string;hostId:string;channel:string;value:number;name:string;members:string;targetChannel:string;delay:number;region:string;threshold:number;x:number;y:number;z:number;yaw:number;description:string;nav:string;enters:boolean;exits:boolean}

export function newDraft(kind:string):BlockDraft{return{kind,hostId:'',channel:'',value:1,name:'',members:'',targetChannel:'',delay:1,region:'',threshold:.5,x:0,y:0,z:0,yaw:0,description:'',nav:'',enters:true,exits:false}}
const members=(draft:BlockDraft)=>draft.members.split(',').map(value=>value.trim()).filter(Boolean);

export function compileDraft(draft:BlockDraft,host:ValveGraphHost):Json{
  const channel=draft.channel.trim();
  switch(draft.kind){
    case'vlist-producer':return[draft.value,0,.25,0,0,0,0,.8,draft.enters?1:0,draft.region.trim(),0,0,0,60,1,channel,draft.exits?1:0];
    case'target-health-producer':return[channel,draft.value,draft.threshold];
    case'net-valve-assign':return['valve_assign',[channel,draft.value]];
    case'net-valve':return['valve',[1,channel,draft.value]];
    case'net-valveunion':return['valveunion',members(draft).map(name=>[1,name,draft.value])];
    case'derived-any-nonzero':return[channel,['any_nonzero','namelist',members(draft)]];
    case'derived-all-nonzero':return[channel,['all_nonzero','namelist',members(draft)]];
    case'derived-all-zero':return[channel,['all_zero','namelist',members(draft)]];
    case'valve-delayupdate':return[channel,['delayupdate',[draft.targetChannel.trim(),draft.delay]]];
    case'valve-sound':return[channel,['sound',[draft.name.trim(),draft.value]]];
    case'valve-sound_once':return[channel,['sound_once',[draft.name.trim(),draft.value]]];
    case'valve-shutdown':return[channel,['shutdown',[draft.name.trim(),0,draft.value]]];
    case'valve-destroy':return[channel,['destroy',[draft.name.trim(),draft.value]]];
    case'valve-teleport':return[channel,['teleport',[draft.name.trim(),[draft.x,draft.y,draft.z],draft.yaw,draft.value]]];
    case'valve-attack_strategy':return[channel,['attack_strategy',[draft.name.trim(),Math.trunc(draft.threshold),draft.value]]];
    case'objective-observer':return['valve_change',['desc',[draft.description.trim()],'nav',[draft.nav.trim()],'valve',[channel],'value',[draft.value]]];
    default:throw new Error(`Unsupported block kind ${draft.kind}`);
  }
}

export function draftError(draft:BlockDraft,host:ValveGraphHost|undefined):string{
  if(!host)return'Select an existing writable host document.';
  if(host.locked)return`${host.archiveRelativePath} is locked.`;
  if(!draft.channel.trim()&&draft.kind!=='net-valveunion')return'Enter an exact, case-sensitive channel name.';
  if(draft.kind==='vlist-producer'&&!draft.region.trim())return'Choose an existing region name.';
  if(['derived-any-nonzero','derived-all-nonzero','derived-all-zero','net-valveunion'].includes(draft.kind)&&!members(draft).length)return'Add at least one input channel.';
  if(draft.kind==='valve-delayupdate'&&!draft.targetChannel.trim())return'Enter the output channel.';
  if(['valve-sound','valve-sound_once','valve-shutdown','valve-destroy','valve-teleport','valve-attack_strategy'].includes(draft.kind)&&!draft.name.trim())return'Enter the required sound or vehicle name.';
  if(draft.kind==='objective-observer'&&!draft.description.trim())return'Enter an existing objective description resource key.';
  return'';
}
