import {describe,expect,it} from 'vitest';
import {MAX_AUTO_LAYOUT_ROWS,autoLayout,channelNodeId,compileDraft,connectedIds,cycleNodeIds,draftError,intersectsRect,mergePositions,newDraft,nodeCardHeight,nodeCardLines,nodeCardRows,nodeIsActive,nodeOutputLine,portOffsetY,portsCompatible,translatePositions,visibleIds,wrapCardLine} from './behavior-graph';
import type {Json,ValveGraphHost,ValveGraphSnapshot} from './types';

const graph:ValveGraphSnapshot={scopeKey:'m4',nodes:[
  {id:'p',kind:'target-health-producer',category:'producer',label:'Bridge',subtitle:'',layer:'mission',locked:false,order:2,binding:{documentId:'t',archiveRelativePath:'r',entryName:'targets.zrd',recordKind:'health-below',locator:'x',containerPath:[],start:0,width:3,mode:'slice'},ports:[{id:'pp',label:'Writes',direction:'write',channel:'m4factbrg',value:1,binding:{documentId:'t',path:[0]}}],properties:{},nativeRecord:[]},
  {id:'c',kind:'objective-observer',category:'consumer',label:'Mission failure',subtitle:'outside_c3m4',layer:'campaign-base',locked:true,order:4,binding:{documentId:'o',archiveRelativePath:'c3/reader.zbd',entryName:'objectives.zrd',recordKind:'objective-observer',locator:'x',containerPath:[],start:0,width:2,mode:'slice'},ports:[{id:'cp',label:'Observed',direction:'read',channel:'m4factbrg',value:1,binding:{documentId:'o',path:[0]}}],properties:{},nativeRecord:[]}
],channels:[{id:'channel::m4factbrg',name:'m4factbrg',producerCount:1,consumerCount:1,layers:['mission','campaign-base']}],edges:[{id:'a',from:'pp',to:'channel::m4factbrg',channel:'m4factbrg'},{id:'b',from:'channel::m4factbrg',to:'cp',channel:'m4factbrg'}],hosts:[],diagnostics:[]};

describe('behavior graph helpers',()=>{
  it('keeps card sizes and port anchors on the 30 unit grid',()=>{const node={...graph.nodes[0],ports:[graph.nodes[0].ports[0],{...graph.nodes[0].ports[0],id:'w2'}]};expect(nodeCardHeight(node)%60).toBe(0);for(const index of [0,1])expect(portOffsetY(node,index)%30).toBe(0)});
  it('keeps automatic positions on the 30 unit grid',()=>{for(const point of Object.values(autoLayout(graph))){expect(point.x%30).toBe(0);expect(point.y%30).toBe(0)}});
  it('lays roles out deterministically',()=>{const a=autoLayout(graph),b=autoLayout(graph);expect(a).toEqual(b);expect(a.p.x).toBeLessThan(a[channelNodeId('m4factbrg')].x);expect(a.c.x).toBeGreaterThan(a[channelNodeId('m4factbrg')].x)});
  it('wraps every automatic lane after twenty items',()=>{
    const manyProducers=Array.from({length:MAX_AUTO_LAYOUT_ROWS+1},(_,index)=>({...graph.nodes[0],id:`producer-${index}`,label:`Producer ${index}`,order:index,ports:[{...graph.nodes[0].ports[0],id:`producer-port-${index}`,channel:`producer-channel-${index}`}] }));
    const manyChannels=Array.from({length:MAX_AUTO_LAYOUT_ROWS+1},(_,index)=>({id:channelNodeId(`channel-${index}`),name:`channel-${index}`,producerCount:1,consumerCount:0,layers:['mission']}));
    const laidOut=autoLayout({...graph,nodes:manyProducers,channels:manyChannels});
    expect(laidOut[`producer-${MAX_AUTO_LAYOUT_ROWS-1}`].y).toBe(90+(MAX_AUTO_LAYOUT_ROWS-1)*(Math.ceil(nodeCardHeight(manyProducers[0])/30)*30+60));
    expect(laidOut[`producer-${MAX_AUTO_LAYOUT_ROWS}`]).toEqual({x:450,y:90});
    expect(laidOut[channelNodeId('channel-9')]).toEqual({x:1170,y:90});
  });
  it('summarizes VList triggers and grows the card for multiple conditions',()=>{
    const vlist={...graph.nodes[0],kind:'vlist-producer',label:'AI / region condition',subtitle:'atlas',ports:[{...graph.nodes[0].ports[0],channel:'door_open',value:1}],nativeRecord:[1,1,.8,0,1,.35,0,.8,1,'hangar',0,0,0,60,0,'door_open',1]};
    const lines=nodeCardLines(vlist);
    expect(lines).toEqual(['Health ≤ 0.350','or Ammo ≤ 0.800','or Always (enters + exits hangar)']);
    expect(nodeOutputLine(vlist)).toBe('door_open = 1');
    expect(nodeIsActive(vlist)).toBe(false);
    expect(nodeCardHeight(vlist)).toBe(180);
  });
  it('labels C1M1 Owens rows and rows that can never fire',()=>{
    const row=(record:Json[])=>nodeCardLines({...graph.nodes[0],kind:'vlist-producer',nativeRecord:record});
    expect(row([1,0,.25,0,1,0,0,.8,0,0,0,0,0,60,1,'firefly_02_go',0])).toEqual(['Health ≤ 0.000']);
    expect(row([1,0,.25,0,0,.999,0,.8,1,'reg_firefly_retreat',0,0,0,60,1,'firefly_retreat',0])).toEqual(['Inside reg_firefly_retreat']);
    expect(row([1,0,.25,0,0,.999,0,.8,1,0,0,0,0,60,1,'firefly_02_go',0])).toEqual(['Never fires: region selector without region']);
    expect(row([1,0,.25,0,0,.999,0,.8,0,'reg_x',0,0,0,60,1,'x',0])).toEqual(['Never fires: no trigger or region']);
  });
  it('wraps long card lines instead of cutting them off',()=>{
    expect(wrapCardLine('Never fires: region selector without region')).toEqual(['Never fires: region selector','  without region']);
    expect(wrapCardLine('Requires: firefly_02_go (target from link index)')).toEqual(['Requires: firefly_02_go (target','  from link index)']);
    expect(wrapCardLine('x'.repeat(40))).toEqual(['x'.repeat(34),`  ${'x'.repeat(6)}`]);
    expect(wrapCardLine('Health ≤ 0.000')).toEqual(['Health ≤ 0.000']);
    const never={...graph.nodes[0],kind:'vlist-producer',ports:[{...graph.nodes[0].ports[0],channel:'firefly_02_go',value:1}],nativeRecord:[1,0,.25,0,0,.999,0,.8,1,0,0,0,0,60,1,'firefly_02_go',0]};
    expect(nodeCardRows(never)).toHaveLength(2);
    expect(nodeCardHeight(never)).toBeGreaterThanOrEqual(nodeCardHeight({...never,nativeRecord:[1,0,.25,0,1,0,0,.8,0,0,0,0,0,60,1,'firefly_02_go',0]}));
  });
  it('summarizes target health output and threshold',()=>{
    const health={...graph.nodes[0],kind:'target-health-producer',subtitle:'satdish',ports:[{...graph.nodes[0].ports[0],channel:'comcntr_hit',value:1}],nativeRecord:['comcntr_hit',1,35]};
    expect(nodeOutputLine(health)).toBe('comcntr_hit = 1');
    expect(nodeCardLines(health)).toEqual(['Health Threshold: 35.000']);
    expect(nodeCardHeight(health)).toBe(120);
  });
  it('summarizes animation callback producers with their callers',()=>{
    const callback={...graph.nodes[0],kind:'animation-callback',label:'Animation callback',subtitle:'bramp / call_ramp',ports:[{...graph.nodes[0].ports[0],channel:'bramp',value:1}],properties:{callback:43,sequence:'effect_sequence',activation:'ON_CALL',executionRange:500,calledBy:['start_single_player_m1 (horizon)']},nativeRecord:['CALLBACK',['VALUE',[43]]]};
    expect(nodeOutputLine(callback)).toBe('bramp = 1');
    expect(nodeCardLines(callback)).toEqual(['Callback 43 in effect_sequence','Activation: ON_CALL','Execution range: 500','Called by: start_single_player_m1 (horizon)']);
    expect(nodeCardLines({...callback,properties:{callback:43,sequence:'s',executionRange:null,calledBy:[]}})).toEqual(['Callback 43 in s','Caller: executable or unloaded']);
  });
  it('shows what valve actions and network tags act on',()=>{
    const consumer=(kind:string,nativeRecord:unknown[])=>({...graph.nodes[1],kind,nativeRecord:nativeRecord as never});
    expect(nodeCardLines(consumer('valve-shutdown',['bramp',['shutdown',['owens_01',0,1]]]))).toEqual(['Vehicle: owens_01','Arguments: 0, 1 (unproven)']);
    expect(nodeCardLines(consumer('valve-sound_once',['bramp',['sound_once',['com_mech',1]]]))).toEqual(['Sound once: com_mech','Value: 1']);
    expect(nodeCardLines(consumer('valve-teleport',['go',['teleport',['atlas',[1,2,3],90,1]]]))).toEqual(['Vehicle: atlas','Position: 1.000, 2.000, 3.000','Yaw: 90.000','State: 1']);
    expect(nodeCardLines(consumer('net-condition',['valve',[1,'bramp']]))).toEqual(['Requires: bramp (target from link index)']);
    expect(nodeCardLines(consumer('net-union',['valveunion',[[1,'bramp',1],[0,'truck_explode',1],[1,'gate_open']]]))).toEqual(['Any of: bramp = 1','Any of: truck_explode = 1 · disabled','Any of: gate_open = 1 (default)']);
    expect(nodeCardLines(consumer('net-assignment',['valve_assign',['destroy_truck_01',1]]))).toEqual([]);
    expect(nodeCardLines(consumer('objective-observer',[]))).toEqual([]);
  });
  it('gives every producer the same output row with its write port beside it',()=>{
    const write={...graph.nodes[0].ports[0],direction:'write' as const,channel:'destroy_truck_01',value:1};
    const read={...graph.nodes[0].ports[0],direction:'read' as const};
    const assignment={...graph.nodes[1],kind:'net-assignment',ports:[write]};
    expect(nodeOutputLine(assignment)).toBe('destroy_truck_01 = 1');
    expect(nodeCardHeight(assignment)).toBe(120);
    expect(nodeOutputLine({...assignment,ports:[write,{...write,channel:'other'}]})).toBe('destroy_truck_01 = 1 +1');
    expect(nodeOutputLine({...assignment,ports:[read]})).toBe('');
    const delay={...assignment,kind:'valve-delayupdate',ports:[read,write]};
    expect([portOffsetY(delay,0),portOffsetY(delay,1)]).toEqual([60,60]);
    expect([portOffsetY({...delay,ports:[write,{...write,id:'w2'}]},0),portOffsetY({...delay,ports:[write,{...write,id:'w2'}]},1)]).toEqual([60,90]);
  });
  it('isolates a complete connected chain',()=>expect([...connectedIds(graph,'p')].sort()).toEqual(['c','channel::m4factbrg','p']));
  it('continues an isolated chain across a network route edge',()=>{
    const assignment={...graph.nodes[0],id:'assign',kind:'net-assignment',ports:[{...graph.nodes[0].ports[0],id:'assign-port',channel:'destroy_truck_01'}]};
    const routed:ValveGraphSnapshot={...graph,nodes:[...graph.nodes,assignment],edges:[...graph.edges,{id:'r',from:'c',to:'assign',channel:'route node_08 → node_17',kind:'route'},{id:'w',from:'assign-port',to:'channel::destroy_truck_01',channel:'destroy_truck_01'}]};
    expect([...connectedIds(routed,'p')].sort()).toEqual(['assign','c','channel::destroy_truck_01','channel::m4factbrg','p']);
  });
  it('keeps connected base-layer consumers in mission scope',()=>expect(visibleIds(graph,{allLoaded:false,search:'',layers:new Set()}).has('c')).toBe(true));
  it('falls back to the loaded graph when an IA/MP mission has no mission nodes',()=>{
    const inherited=structuredClone(graph);
    inherited.nodes=inherited.nodes.map(node=>({...node,layer:'campaign-base'}));
    inherited.channels=inherited.channels.map(channel=>({...channel,layers:['campaign-base']}));
    const visible=visibleIds(inherited,{allLoaded:false,search:'',layers:new Set()});
    expect(visible.has('p')).toBe(true);
    expect(visible.has('c')).toBe(true);
    expect(visible.has(channelNodeId('m4factbrg'))).toBe(true);
  });
  it('requires opposite port directions',()=>expect(portsCompatible(graph.nodes[0].ports[0],graph.nodes[1].ports[0])).toBe(true));
  it('compiles a draft into the exact native aggregate pair',()=>{const draft=newDraft('derived-any-nonzero');draft.channel='bridge_fail';draft.members='bridge_a, bridge_b';const host={kind:'valves-document',locked:false} as unknown as ValveGraphHost;expect(draftError(draft,host)).toBe('');expect(compileDraft(draft,host)).toEqual(['bridge_fail',['any_nonzero','namelist',['bridge_a','bridge_b']]])});
  it('places existing cycles in a separate lower lane',()=>{const cyclic=structuredClone(graph);cyclic.nodes.push({...cyclic.nodes[0],id:'loop',kind:'derived-state',category:'logic',order:7,ports:[{...cyclic.nodes[1].ports[0],id:'loop-in',channel:'m4factbrg'},{...cyclic.nodes[0].ports[0],id:'loop-out',channel:'m4factbrg'}]});expect(cycleNodeIds(cyclic).has('loop')).toBe(true);expect(autoLayout(cyclic).loop.y).toBeGreaterThan(autoLayout(graph).c.y)});
  it('rectangle-selects intersecting nodes and translates a selected group',()=>{expect(intersectsRect({x:100,y:100},{w:220,h:104},{x:250,y:150,w:80,h:80})).toBe(true);expect(intersectsRect({x:100,y:100},{w:50,h:50},{x:200,y:200,w:20,h:20})).toBe(false);expect(translatePositions({a:{x:10,y:20},b:{x:30,y:40},c:{x:50,y:60}},{a:{x:10,y:20},c:{x:50,y:60}},7,-3)).toEqual({a:{x:17,y:17},b:{x:30,y:40},c:{x:57,y:57}})});
  it('places a newly discovered locked node outside persisted node positions',()=>{const expanded=structuredClone(graph);expanded.nodes.push({...expanded.nodes[0],id:'locked-producer',label:'Locked producer',layer:'campaign-base',locked:true,order:3,ports:[{...expanded.nodes[0].ports[0],id:'locked-port',channel:'locked-output'}]});expanded.channels.push({id:'channel::locked-output',name:'locked-output',producerCount:1,consumerCount:0,layers:['campaign-base']});const automatic=autoLayout(expanded),saved={p:{...automatic['locked-producer']}};const merged=mergePositions(expanded,saved);expect(merged.p).toEqual(saved.p);expect(intersectsRect(merged['locked-producer'],{w:220,h:104},{x:saved.p.x,y:saved.p.y,w:220,h:104})).toBe(false)});
});
