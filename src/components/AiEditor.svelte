<script lang="ts">
  import type { FieldSection, Json, WorkspaceDocument, WorkspaceSnapshot } from '../types';
  import { CHASSIS, GROUPS, STRATEGIES, VLIST } from '../types';
  import { array, checked, ensureTag, isLocked, pairList, parseScalar, referenceNames, referencesFor, removeTag, scalar, tagArray, tags, vehicles, type VehicleView } from '../workspace';
  let { workspace, document, onChanged, onNavigate }: { workspace:WorkspaceSnapshot; document:WorkspaceDocument; onChanged:()=>void; onNavigate:(kind:string,name:string)=>void } = $props();
  let selected=$state(0); let tab=$state('Vehicle');
  const tabs=['Vehicle','Activation','Attack','Retreat','Scan / Jump','Fire','VList','Raw'];
  const booleanFields=new Set(['params[1]','p2[1]','p2[2]','p2[3]','p6[0]','p6[2]','p6[3]','p8[4]','p8[9]','p8[13]','p8[14]','p8[15]','p9[3]','p12[1]','p12[2]','p13[0]']);
  const booleanVlistFields=new Set([1,3,4,6,8,10,12,14,16]);
  const vlistSections:FieldSection[]=[{title:'Valve Result',indices:[0,14,15]},{title:'Vehicle-State Conditions',indices:[1,2,3,4,5,6,7]},{title:'Region Conditions',indices:[8,9,16]},{title:'Target Conditions',indices:[10,11,12,13]}];
  const touch=()=>onChanged();
  function current():VehicleView|undefined { const list=vehicles(document);if(selected>=list.length)selected=Math.max(0,list.length-1);return list[selected]; }
  function setField(values:Json[],i:number,text:string){values[i]=parseScalar(text,values[i]);touch()}
  function setBoolean(values:Json[],i:number,value:boolean){values[i]=value?1:0;touch()}
  function setBase(v:VehicleView,i:number,text:string){while(v.record.length<4)v.record.push(0);v.record[i]=parseScalar(text,v.record[i]);touch()}
  function setName(v:VehicleView,text:string){pairList(document.root)[v.pairIndex]=text;touch()}
  function addVehicle(){const root=pairList(document.root);root.push('New Vehicle',[0,[0,0,0],0,0]);selected=vehicles(document).length-1;touch()}
  function duplicate(v:VehicleView){const root=pairList(document.root);root.splice(v.pairIndex+2,0,`${v.name} Copy`,structuredClone(v.record));selected++;touch()}
  function remove(v:VehicleView){pairList(document.root).splice(v.pairIndex,2);selected=Math.max(0,selected-1);touch()}
  function move(v:VehicleView,d:number){const root=pairList(document.root);const target=v.pairIndex+d*2;if(target<0||target>=root.length)return;const pair=root.splice(v.pairIndex,2);root.splice(target,0,...pair);selected+=d;touch()}
  function setIdentity(v:VehicleView,key:string,text:string){const found=tags(v.record).find(x=>x.tag===key);if(found)v.record[found.at+1]=[text];else v.record.push(key,[text]);touch()}
  function addVlist(v:VehicleView){ensureTag(v.record,'vlist',0).push(Array(17).fill(0));touch()}
  function roles(name:Json|undefined){return typeof name==='string'?referencesFor(workspace,name):[]}
</script>

<datalist id="ai-valve-names">{#each referenceNames(workspace,'valve') as name}<option value={name}></option>{/each}</datalist>
<datalist id="ai-region-names">{#each referenceNames(workspace,'region') as name}<option value={name}></option>{/each}</datalist>

<div class="editor-shell">
  <aside class="object-sidebar">
    <div class="side-heading"><div><small>AIV.ZRD</small><b>AI Vehicles</b></div><button title="Add vehicle" disabled={isLocked(workspace,document)} onclick={addVehicle}>＋</button></div>
    <div class="object-list">{#each vehicles(document) as item,i (item.id)}<button class:active={i===selected} onclick={()=>selected=i}><span>{item.name}</span><small>#{i+1}</small></button>{/each}</div>
    {#if current()}<div class="side-actions"><button disabled={isLocked(workspace,document)} onclick={()=>duplicate(current()!)}>Duplicate</button><button disabled={isLocked(workspace,document)} onclick={()=>remove(current()!)}>Delete</button><button disabled={isLocked(workspace,document)} onclick={()=>move(current()!,-1)}>↑</button><button disabled={isLocked(workspace,document)} onclick={()=>move(current()!,1)}>↓</button></div>{/if}
  </aside>
  <section class="editor-workspace">
    {#if isLocked(workspace,document)}<div class="locked-banner">This document belongs to a locked {document.layer} Reader. Unlock that archive to edit it.</div>{/if}
    <nav class="subnav">{#each tabs as name}<button class:active={tab===name} onclick={()=>tab=name}>{name}</button>{/each}</nav>
    {#if current()}{@const v=current()!}<fieldset disabled={isLocked(workspace,document)} class="editor-fields"><div class="page">
      {#if tab==='Vehicle'}
        <div class="card"><h2>Vehicle</h2><div class="subsection"><h3>Identity</h3><div class="grid"><label class="wide"><span class="field-title">Name <small>(name)</small></span><input value={v.name} oninput={e=>setName(v,e.currentTarget.value)}></label></div></div><div class="subsection"><h3>Initial Vehicle Position</h3><div class="grid">{#each ['X','Y','Z'] as axis,i}<label><span class="field-title">Position {axis} <small>(record[1][{i}])</small></span><input value={scalar(array(v.record[1])[i])} oninput={e=>{const p=array(v.record[1]);while(p.length<3)p.push(0);p[i]=parseScalar(e.currentTarget.value,p[i]);v.record[1]=p;touch()}}></label>{/each}<label><span class="field-title">Yaw <small>(record[2])</small></span><input value={scalar(v.record[2])} oninput={e=>setBase(v,2,e.currentTarget.value)}></label></div></div><div class="subsection"><h3>AI Network & Team</h3><div class="grid"><label><span class="field-title">Net ID <small>(record[0])</small></span><input value={scalar(v.record[0])} oninput={e=>setBase(v,0,e.currentTarget.value)}></label><label><span class="field-title">Team / allegiance <small>(record[3])</small></span><input value={scalar(v.record[3])} oninput={e=>setBase(v,3,e.currentTarget.value)}></label></div></div></div>
        {@render Group(v,'p12',GROUPS.p12.title,GROUPS.p12.description,GROUPS.p12.labels,GROUPS.p12.sections)}
        <div class="card"><h2>Identity tags</h2><div class="grid">{#each ['pilot','skin'] as key}<label><span class="field-title">{key[0].toUpperCase()+key.slice(1)} <small>({key})</small></span><input value={scalar(tagArray(v.record,key)?.[0] ?? '')} oninput={e=>setIdentity(v,key,e.currentTarget.value)}></label>{/each}</div></div>
      {:else if tab==='VList'}
        <div class="card"><div class="title-row"><div><h2>Conditional Valve List <small>(vlist)</small></h2><p>Conditions on this vehicle write named state channels shared with Valve Logic, networks, targets, and objectives.</p></div><button onclick={()=>addVlist(v)}>Add condition</button></div>
        {#each tagArray(v.record,'vlist') ?? [] as row,ri}{@const values=array(row)}<div class="vrow"><div class="row-head"><b>Condition {ri+1}</b><button onclick={()=>{tagArray(v.record,'vlist')!.splice(ri,1);touch()}}>Remove</button></div>{#each vlistSections as section}<div class="subsection"><h3>{section.title}</h3><div class="grid">{#each section.indices as i}<label><span class="field-title">{VLIST[i]} <small>(vlist[{ri}][{i}])</small></span>{#if i===0}<select value={scalar(values[i])} onchange={e=>setField(values,i,e.currentTarget.value)}><option value="0">0 — Close</option><option value="1">1 — Open</option></select>{:else if i===15}<input list="ai-valve-names" value={scalar(values[i])} oninput={e=>setField(values,i,e.currentTarget.value)}>{#if roles(values[i]).length}<div class="role-strip">{#each roles(values[i]).slice(0,4) as ref}<span title={ref.location}>{ref.role} · {ref.layer}</span>{/each}</div>{/if}{:else if i===9}<div class="reference-input"><input list="ai-region-names" value={scalar(values[i])} oninput={e=>setField(values,i,e.currentTarget.value)}><button type="button" disabled={typeof values[i]!=='string'} onclick={()=>onNavigate('region',String(values[i]))}>Open</button></div>{:else if booleanVlistFields.has(i)}<span class="boolean-control"><input type="checkbox" checked={checked(values[i])} onchange={e=>setBoolean(values,i,e.currentTarget.checked)}><span>{checked(values[i])?'On':'Off'}</span></span>{:else}<input value={scalar(values[i])} oninput={e=>setField(values,i,e.currentTarget.value)}>{/if}</label>{/each}</div></div>{/each}</div>{/each}
        {#if !(tagArray(v.record,'vlist')?.length)}<div class="empty">No VList conditions in this vehicle.</div>{/if}</div>
      {:else if tab==='Raw'}
        <div class="card"><h2>Raw vehicle record</h2><p>Read-only diagnostic view. Unknown tags remain in their original order.</p><pre>{JSON.stringify(v.record,null,2)}</pre></div>
      {:else}
        {#each Object.entries(GROUPS).filter(([,group])=>group.tab===tab) as [key,group]}{@render Group(v,key,group.title,group.description,group.labels,group.sections)}{/each}
      {/if}
    </div></fieldset>{:else}<div class="empty-state"><h2>No AI vehicles</h2><p>This aiv.zrd contains no editable name/record pairs.</p></div>{/if}
  </section>
</div>

{#snippet Group(v:VehicleView,key:string,title:string,description:string,labels:string[],sections:FieldSection[])}
  {@const value=tagArray(v.record,key)}<div class="card"><div class="title-row"><div><h2>{title} <small>({key})</small></h2><p>{description}</p></div><label class="switch"><input type="checkbox" checked={!!value} onchange={e=>{e.currentTarget.checked?ensureTag(v.record,key,labels.length):removeTag(v.record,key);touch()}}> Present</label></div>{#if value}{#each sections as section}<div class="subsection"><h3>{section.title}</h3><div class="grid">{#each section.indices as i}<label><span class="field-title">{labels[i]} <small>({key}[{i}])</small></span>{#if key==='p2'&&i===0}<select value={scalar(value[i])} onchange={e=>setField(value,i,e.currentTarget.value)}>{#each STRATEGIES as strategy,si}<option value={String(si)}>{si} — {strategy}</option>{/each}</select>{:else if key==='p8'&&i===10}<select value={scalar(value[i])} onchange={e=>setField(value,i,e.currentTarget.value)}><option value="0">0</option><option value="1">1</option></select>{:else if key==='p8'&&i===16}<select value={scalar(value[i])} onchange={e=>setField(value,i,e.currentTarget.value)}><option value="0">0 — Scan</option><option value="1">1 — Jump</option></select>{:else if key==='p12'&&i===0}<select value={scalar(value[i])} onchange={e=>setField(value,i,e.currentTarget.value)}>{#each CHASSIS as chassis}<option value={chassis}>{chassis}</option>{/each}</select>{:else if booleanFields.has(`${key}[${i}]`)}<span class="boolean-control"><input type="checkbox" checked={checked(value[i])} onchange={e=>setBoolean(value,i,e.currentTarget.checked)}><span>{checked(value[i])?'On':'Off'}</span></span>{:else}<input value={scalar(value[i])} oninput={e=>setField(value,i,e.currentTarget.value)}>{/if}</label>{/each}</div></div>{/each}{:else}<div class="empty">This optional section is not present for this vehicle.</div>{/if}</div>
{/snippet}
