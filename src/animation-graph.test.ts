import {describe,expect,it} from 'vitest';
import {animationEdges,callbackMeaning,eventSummary,executableEntry,executionLabel,layoutNeighborhood,neighborhood,startLabel,stateChannel,type AnimationDefinitionSummary,type AnimationIndex} from './animation-graph';

const definition=(index:number,name:string,animName:string,links:AnimationDefinitionSummary['sequences'][number]['links'],extra:Partial<AnimationDefinitionSummary>={}):AnimationDefinitionSummary=>({index,fileName:`${name}-${animName}`,name,animName,rootName:name,activation:'OnCall',execution:null,health:0,resetTime:-1,hasCallbacks:false,prerequisites:[],sequences:[{name:'seq',seqState:'Initial',eventCount:links.length,links}],...extra});
const call=(eventIndex:number,target:string,kind='CallAnimation')=>({eventIndex,kind,target,start:null});

// C1M1: start_single_player_m1 -> call_ramp -> hit_ramp -> call_trap; expstn01.flt is not defined here.
const index:AnimationIndex={path:'anim.zbd',relativePath:'zbd/c1/anim.zbd',mission:null,definitions:[
  definition(0,'horizon','start_single_player_m1',[call(0,'call_ramp')]),
  definition(1,'bramp','call_ramp',[{eventIndex:0,kind:'Callback',value:43,start:null},call(1,'call_ramp','InvalidateAnimation'),{...call(2,'hit_ramp'),start:{offset:'Animation',time:20}}],{execution:{ByRange:{min:0,max:250000}}}),
  definition(2,'bramp','hit_ramp',[call(0,'call_trap'),call(1,'expstn01.flt')]),
  definition(3,'trap','call_trap',[]),
  definition(4,'gate','open_gate',[],{prerequisites:[{kind:'Animation',name:'call_trap',required:true}]}),
]};

describe('animation graph helpers',()=>{
  it('resolves calls by anim_name and drops self references from targets',()=>{
    const edges=animationEdges(index);
    expect(edges.find(edge=>edge.from===0)?.to).toEqual([1]);
    expect(edges.some(edge=>edge.kind==='InvalidateAnimation')).toBe(false);
    expect(edges.find(edge=>edge.target==='expstn01.flt')?.to).toEqual([]);
    expect(edges.find(edge=>edge.kind==='Prerequisite')).toMatchObject({from:3,to:[4]});
  });
  it('expands callees rightward and callers leftward only',()=>{
    const hood=neighborhood(animationEdges(index),1,2);
    expect(Object.fromEntries(hood.layers)).toEqual({0:-1,1:0,2:1,3:2});
    expect(hood.external).toEqual([]);
    const fromHit=neighborhood(animationEdges(index),2,1);
    expect(fromHit.external.map(item=>item.name)).toEqual(['expstn01.flt']);
    const positions=layoutNeighborhood(index,hood);
    expect(positions['def:0'].x).toBeLessThan(positions['def:1'].x);
    expect(positions['def:3'].x).toBeGreaterThan(positions['def:2'].x);
  });
  it('caps each layer and reports the rest',()=>{
    const hub:AnimationIndex={...index,definitions:[definition(0,'hub','hub',Array.from({length:5},(_,i)=>call(i,`leaf${i}`))),...Array.from({length:5},(_,i)=>definition(i+1,`leaf${i}`,`leaf${i}`,[]))]};
    const hood=neighborhood(animationEdges(hub),0,1,3);
    expect(hood.layers.size).toBe(4);
    expect(hood.truncated).toBe(2);
  });
  it('labels triggers, ranges, and callbacks',()=>{
    expect(executableEntry('start_single_player_m1')).toBe(true);
    expect(executableEntry('start_single_player')).toBe(true);
    expect(executableEntry('call_ramp')).toBe(false);
    expect(executionLabel({ByRange:{min:0,max:250000}})).toBe('ByRange ≤ 500');
    expect(executionLabel(null)).toBe('None');
    expect(startLabel({offset:'Animation',time:20})).toBe('animation +20s');
    expect(callbackMeaning(43)).toBe("writes this definition's NAME = 1");
    expect(callbackMeaning(47)).toBe("writes the grandparent node's name = 1");
    expect(callbackMeaning(56)).toBe('notifies the game object owning the node');
    expect(stateChannel(index.definitions[1],43)).toBe('bramp');
    expect(stateChannel(index.definitions[1],47)).toBeUndefined();
  });
  it('summarizes full events for the inspector timeline',()=>{
    expect(eventSummary({start:{offset:'Animation',time:2.1},data:{ObjectActiveState:{node:'stage2',state:true}}})).toEqual({kind:'ObjectActiveState',start:'animation +2.10s',text:'stage2 active'});
    expect(eventSummary({start:null,data:{CallAnimation:{name:'xplflare.flt',parameters:{AtNode:{node:'part5'}},wait_for_completion:null}}}).text).toBe('xplflare.flt AtNode part5');
    expect(eventSummary({start:null,data:{Endif:{}}})).toEqual({kind:'Endif',start:'',text:''});
  });
});
