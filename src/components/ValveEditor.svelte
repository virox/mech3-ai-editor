<script lang="ts">
  import type { Json, WorkspaceDocument, WorkspaceSnapshot } from '../types';
  import { STRATEGIES } from '../types';
  import { array, isLocked, pairList, parseScalar, referenceNames, referencesFor, scalar, valveActions, type ValveActionView } from '../workspace';
  let { workspace, document, onChanged }: {workspace:WorkspaceSnapshot;document:WorkspaceDocument;onChanged:()=>void}=$props();
  let selected=$state(0);let rawError=$state('');
  const actionTypes=['sound','sound_once','delayupdate','shutdown','destroy','teleport','attack_strategy','any_nonzero','all_nonzero','all_zero'];
  function current():ValveActionView|undefined{const list=valveActions(document);if(selected>=list.length)selected=Math.max(0,list.length-1);return list[selected]}
  function touch(){rawError='';onChanged()}
  function setEvent(action:ValveActionView,text:string){pairList(document.root)[action.pairIndex]=text;touch()}
  function defaultRecord(type:string):Json[]{switch(type){case'sound':case'sound_once':return[type,['',1]];case'delayupdate':return[type,['',0]];case'shutdown':return[type,['',0,1]];case'destroy':return[type,['',1]];case'teleport':return[type,['',[0,0,0],0,1]];case'attack_strategy':return[type,['',0,1]];case'any_nonzero':case'all_nonzero':case'all_zero':return[type,'namelist',[]];default:return[type,[]]}}
  function setType(action:ValveActionView,type:string){const root=pairList(document.root);const next=defaultRecord(type);root[action.pairIndex+1]=next;action.record=next;touch()}
  function args(action:ValveActionView):Json[]{let value=action.record[1];if(!Array.isArray(value)){value=[];action.record[1]=value}return value as Json[]}
  function setArg(action:ValveActionView,index:number,text:string){const a=args(action);while(a.length<=index)a.push(0);a[index]=parseScalar(text,a[index]);touch()}
  function setState(action:ValveActionView,index:number,text:string){const a=args(action);while(a.length<=index)a.push(0);a[index]=Number(text);touch()}
  function addAction(){const root=pairList(document.root);root.push('new_event',defaultRecord('delayupdate'));selected=valveActions(document).length-1;touch()}
  function duplicate(action:ValveActionView){pairList(document.root).splice(action.pairIndex+2,0,action.event,structuredClone(action.record));selected++;touch()}
  function remove(action:ValveActionView){pairList(document.root).splice(action.pairIndex,2);selected=Math.max(0,selected-1);touch()}
  function move(action:ValveActionView,d:number){const root=pairList(document.root);const target=action.pairIndex+d*2;if(target<0||target>=root.length)return;const pair=root.splice(action.pairIndex,2);root.splice(target,0,...pair);selected+=d;touch()}
  function names(action:ValveActionView):Json[]{if(action.record[1]!=='namelist')action.record[1]='namelist';if(!Array.isArray(action.record[2]))action.record[2]=[];return action.record[2] as Json[]}
  function setRaw(action:ValveActionView,text:string){try{const parsed=JSON.parse(text);if(!Array.isArray(parsed))throw new Error('Action record must be an array');pairList(document.root)[action.pairIndex+1]=parsed;action.record=parsed;touch()}catch(error){rawError=String(error)}}
  const roles=(name:string)=>referencesFor(workspace,name);
</script>

<datalist id="valve-channel-names">{#each referenceNames(workspace,'valve') as name}<option value={name}></option>{/each}</datalist>
<datalist id="valve-vehicle-names">{#each referenceNames(workspace,'vehicle') as name}<option value={name}></option>{/each}</datalist>
<datalist id="valve-sound-names">{#each referenceNames(workspace,'sound') as name}<option value={name}></option>{/each}</datalist>

<div class="editor-shell">
  <aside class="object-sidebar valve-sidebar">
    <div class="side-heading"><div><small>VALVES.ZRD</small><b>Valve Logic</b></div><button disabled={isLocked(workspace,document)} onclick={addAction}>＋</button></div>
    <div class="object-list">{#each valveActions(document) as action,i (action.id)}<button class:active={i===selected} onclick={()=>selected=i}><span><b>{action.event}</b><small>{scalar(action.record[0])}</small></span><small>#{i+1}</small></button>{/each}</div>
    {#if current()}<div class="side-actions"><button disabled={isLocked(workspace,document)} onclick={()=>duplicate(current()!)}>Duplicate</button><button disabled={isLocked(workspace,document)} onclick={()=>remove(current()!)}>Delete</button><button disabled={isLocked(workspace,document)} onclick={()=>move(current()!,-1)}>↑</button><button disabled={isLocked(workspace,document)} onclick={()=>move(current()!,1)}>↓</button></div>{/if}
  </aside>
  <section class="editor-workspace">
    {#if isLocked(workspace,document)}<div class="locked-banner">Valve Logic is read-only because {document.layer} is locked.</div>{/if}
    {#if current()}{@const action=current()!}{@const type=String(action.record[0]??'')}
      <fieldset disabled={isLocked(workspace,document)} class="editor-fields"><div class="page valve-page">
        <div class="card hero-card"><div><small>ORDERED EVENT / ACTION PAIR</small><h2>{action.event}</h2><p>Changing this channel can run every action registered under the same event name. Duplicate names are preserved.</p></div><div class="counter">#{selected+1}</div></div>
        <div class="card"><h2>Event channel</h2><div class="grid two"><label><span class="field-title">Event / state channel <small>(root[{action.pairIndex}])</small></span><input list="valve-channel-names" value={action.event} oninput={e=>setEvent(action,e.currentTarget.value)}></label><label><span class="field-title">Action type <small>(record[0])</small></span><select value={type} onchange={e=>setType(action,e.currentTarget.value)}>{#each actionTypes as known}<option value={known}>{known}</option>{/each}{#if !actionTypes.includes(type)}<option value={type}>{type} — unknown</option>{/if}</select></label></div>
          <div class="role-strip roomy">{#each roles(action.event) as ref}<span title={`${ref.documentId} · ${ref.location}`}>{ref.role} · {ref.layer}</span>{/each}</div>
        </div>
        {#if type==='sound'||type==='sound_once'}
          <div class="card"><h2>{type==='sound_once'?'Play sound once':'Play sound'}</h2><div class="grid two"><label><span class="field-title">Sound name <small>(args[0])</small></span><input list="valve-sound-names" value={scalar(args(action)[0])} oninput={e=>setArg(action,0,e.currentTarget.value)}></label>{@render State(action,1)}</div></div>
        {:else if type==='delayupdate'}
          <div class="card"><h2>Delayed Valve update</h2><div class="grid two"><label><span class="field-title">Valve / state channel <small>(args[0])</small></span><input list="valve-channel-names" value={scalar(args(action)[0])} oninput={e=>setArg(action,0,e.currentTarget.value)}></label><label><span class="field-title">Delay <small>(args[1])</small></span><input type="number" step="any" value={scalar(args(action)[1])} oninput={e=>setArg(action,1,e.currentTarget.value)}></label></div></div>
        {:else if type==='shutdown'}
          <div class="card"><h2>Vehicle shutdown state</h2><div class="grid"><label><span class="field-title">Vehicle <small>(args[0])</small></span><input list="valve-vehicle-names" value={scalar(args(action)[0])} oninput={e=>setArg(action,0,e.currentTarget.value)}></label><label><span class="field-title">Shutdown value <small>(args[1])</small></span><input value={scalar(args(action)[1])} oninput={e=>setArg(action,1,e.currentTarget.value)}></label>{@render State(action,2)}</div></div>
        {:else if type==='destroy'}
          <div class="card"><h2>Destroy vehicle</h2><div class="grid two"><label><span class="field-title">Vehicle <small>(args[0])</small></span><input list="valve-vehicle-names" value={scalar(args(action)[0])} oninput={e=>setArg(action,0,e.currentTarget.value)}></label>{@render State(action,1)}</div></div>
        {:else if type==='teleport'}
          {@const position=array(args(action)[1])}<div class="card"><h2>Teleport vehicle</h2><div class="grid"><label><span class="field-title">Vehicle <small>(args[0])</small></span><input list="valve-vehicle-names" value={scalar(args(action)[0])} oninput={e=>setArg(action,0,e.currentTarget.value)}></label>{#each ['X','Y','Z'] as axis,i}<label><span class="field-title">Position {axis} <small>(args[1][{i}])</small></span><input type="number" step="any" value={scalar(position[i])} oninput={e=>{while(position.length<3)position.push(0);position[i]=Number(e.currentTarget.value);args(action)[1]=position;touch()}}></label>{/each}<label><span class="field-title">Yaw <small>(args[2])</small></span><input type="number" step="any" value={scalar(args(action)[2])} oninput={e=>setArg(action,2,e.currentTarget.value)}></label>{@render State(action,3)}</div></div>
        {:else if type==='attack_strategy'}
          <div class="card"><h2>Change attack strategy</h2><div class="grid"><label><span class="field-title">Vehicle <small>(args[0])</small></span><input list="valve-vehicle-names" value={scalar(args(action)[0])} oninput={e=>setArg(action,0,e.currentTarget.value)}></label><label><span class="field-title">Strategy <small>(args[1])</small></span><select value={scalar(args(action)[1])} onchange={e=>setArg(action,1,e.currentTarget.value)}>{#each STRATEGIES as strategy,i}<option value={String(i)}>{i} — {strategy}</option>{/each}</select></label>{@render State(action,2)}</div></div>
        {:else if type==='any_nonzero'||type==='all_nonzero'||type==='all_zero'}
          <div class="card"><div class="title-row"><div><h2>Valve/state name list</h2><p>{type.replaceAll('_',' ')} evaluates the listed channels as a group.</p></div><button onclick={()=>{names(action).push('');touch()}}>Add name</button></div><div class="name-list">{#each names(action) as name,i}<div class="reference-input"><input list="valve-channel-names" value={scalar(name)} oninput={e=>{names(action)[i]=e.currentTarget.value;touch()}}><button onclick={()=>{names(action).splice(i,1);touch()}}>Remove</button></div>{/each}</div></div>
        {:else}
          <div class="card"><h2>Unknown action record</h2><p>This variant has no proven typed editor. Valid JSON arrays are written back losslessly.</p><textarea class="raw-editor" value={JSON.stringify(action.record,null,2)} onchange={e=>setRaw(action,e.currentTarget.value)}></textarea>{#if rawError}<div class="diagnostic error">{rawError}</div>{/if}</div>
        {/if}
        <div class="card"><details><summary>Raw action record</summary><pre>{JSON.stringify(action.record,null,2)}</pre></details></div>
      </div></fieldset>
    {:else}<div class="empty-state"><h2>No Valve actions</h2><p>Add an ordered event/action pair to begin.</p></div>{/if}
  </section>
</div>

{#snippet State(action:ValveActionView,index:number)}<label><span class="field-title">Active state <small>(args[{index}])</small></span><select value={scalar(args(action)[index])} onchange={e=>setState(action,index,e.currentTarget.value)}><option value="0">0 — Off</option><option value="1">1 — On</option></select></label>{/snippet}
