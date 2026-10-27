<script lang="ts">
  import type { Json, WorkspaceDocument, WorkspaceSnapshot } from '../types';
  import { CHASSIS, STRATEGIES } from '../types';
  import { DEFAULT_AI_LAYOUT, addRepeaterItem, booleanValue, displayBinding, displayValue, fieldValue, groupPresent, removeRepeaterItem, repeaterItems, setFieldValue, setGroupPresent, tagValue, type AiLayout, type AiLayoutField, type AiLayoutGroup, type AiLayoutOption, type AiLayoutRepeater } from '../ai-layout';
  import { inspectPackedTeams } from '../team';
  import { array, isLocked, pairList, referenceNames, referencesFor, removeTag, tags, vehicles, type VehicleView } from '../workspace';
  import TeamEditor from './TeamEditor.svelte';

  let { workspace, document, layout=DEFAULT_AI_LAYOUT, onChanged, onNavigate }: { workspace:WorkspaceSnapshot; document:WorkspaceDocument; layout?:AiLayout; onChanged:()=>void; onNavigate:(kind:string,name:string)=>void } = $props();
  let selected=$state(0);let tab=$state(localStorage.getItem('mech-reader-studio:ai-tab')??'');
  $effect(()=>{if(!layout.tabs.some(candidate=>candidate.id===tab)){tab=layout.tabs[0]?.id??'';if(tab)localStorage.setItem('mech-reader-studio:ai-tab',tab)}});
  const touch=()=>onChanged();
  function current():VehicleView|undefined { const list=vehicles(document);if(selected>=list.length)selected=Math.max(0,list.length-1);return list[selected] }
  function setName(v:VehicleView,text:string){pairList(document.root)[v.pairIndex]=text;touch()}
  function addVehicle(){const root=pairList(document.root);root.push('New Vehicle',[0,[0,0,0],0,0]);selected=vehicles(document).length-1;touch()}
  function duplicate(v:VehicleView){const root=pairList(document.root);root.splice(v.pairIndex+2,0,`${v.name} Copy`,structuredClone(v.record));selected++;touch()}
  function remove(v:VehicleView){pairList(document.root).splice(v.pairIndex,2);selected=Math.max(0,selected-1);touch()}
  function move(v:VehicleView,d:number){const root=pairList(document.root);const target=v.pairIndex+d*2;if(target<0||target>=root.length)return;const pair=root.splice(v.pairIndex,2);root.splice(target,0,...pair);selected+=d;touch()}
  function changeField(v:VehicleView,field:AiLayoutField,value:string,item?:Json[]){setFieldValue(v,field,value,item,name=>setName(v,name));touch()}
  function setTeam(v:VehicleView,value:number){while(v.record.length<4)v.record.push(0);v.record[3]=value;touch()}
  function setLancemate(v:VehicleView,state:'auto'|'on'|'off'){
    if(state==='auto')removeTag(v.record,'lancemate');
    else {const found=tags(v.record).find(entry=>entry.tag==='lancemate');if(found)v.record[found.at+1]=[state];else v.record.push('lancemate',[state])}
    touch();
  }
  function controlledTeam(){
    const candidates=workspace.documents.filter(d=>d.kind==='aiv').flatMap(d=>vehicles(d).filter(v=>v.name.toLowerCase()==='player').map(v=>({vehicle:v,layer:d.layer})));
    candidates.sort((a,b)=>(a.layer==='mission'?0:1)-(b.layer==='mission'?0:1));const candidate=candidates[0];if(!candidate)return undefined;
    const info=inspectPackedTeams(candidate.vehicle.record[3]);return info.valid?{name:candidate.vehicle.name,teamIds:info.teamIds,layer:candidate.layer}:undefined;
  }
  function options(field:AiLayoutField):AiLayoutOption[]{
    if(field.optionSource==='strategies')return STRATEGIES.map((label,value)=>({value:String(value),label:`${value} — ${label}`}));
    if(field.optionSource==='chassis')return CHASSIS.map(value=>({value,label:value}));
    if(field.optionSource==='binary')return [{value:'0',label:'0'},{value:'1',label:'1'}];
    if(field.optionSource==='scan-jump')return [{value:'0',label:'0 — Scan'},{value:'1',label:'1 — Jump'}];
    return field.options??[];
  }
  function referenceValue(v:VehicleView,field:AiLayoutField,item?:Json[]):string { const value=fieldValue(v,field.binding,item);return typeof value==='string'?value:value==null||value===0?'':String(value) }
  function referenceList(field:AiLayoutField):string[]{return field.referenceKind&&field.referenceKind!=='none'?referenceNames(workspace,field.referenceKind):[]}
  function roles(v:VehicleView,field:AiLayoutField,item?:Json[]){const value=fieldValue(v,field.binding,item);return typeof value==='string'?referencesFor(workspace,value):[]}
  function selectTab(id:string){tab=id;localStorage.setItem('mech-reader-studio:ai-tab',id)}
</script>

{#each ['valve','region','vehicle','network'] as kind}<datalist id={`ai-reference-${kind}`}>{#each referenceNames(workspace,kind) as name}<option value={name}></option>{/each}</datalist>{/each}

<div class="editor-shell">
  <aside class="object-sidebar">
    <div class="side-heading"><div><small>AIV.ZRD</small><b>AI Vehicles</b></div><button title="Add vehicle" disabled={isLocked(workspace,document)} onclick={addVehicle}>＋</button></div>
    <div class="object-list">{#each vehicles(document) as item,i (item.id)}<button class:active={i===selected} onclick={()=>selected=i}><span>{item.name}</span><small>#{i+1}</small></button>{/each}</div>
    {#if current()}<div class="side-actions"><button disabled={isLocked(workspace,document)} onclick={()=>duplicate(current()!)}>Duplicate</button><button disabled={isLocked(workspace,document)} onclick={()=>remove(current()!)}>Delete</button><button disabled={isLocked(workspace,document)} onclick={()=>move(current()!,-1)}>↑</button><button disabled={isLocked(workspace,document)} onclick={()=>move(current()!,1)}>↓</button></div>{/if}
  </aside>
  <section class="editor-workspace">
    {#if isLocked(workspace,document)}<div class="locked-banner">This document belongs to a locked {document.layer} Reader. Unlock that archive to edit it.</div>{/if}
    <nav class="subnav">{#each layout.tabs as candidate}<button class:active={tab===candidate.id} onclick={()=>selectTab(candidate.id)}>{candidate.title}</button>{/each}</nav>
    {#if current()}{@const v=current()!}{@const activeTab=layout.tabs.find(candidate=>candidate.id===tab)}<fieldset disabled={isLocked(workspace,document)} class="editor-fields"><div class="page">
      {#if activeTab?.description}<p class="tab-description">{activeTab.description}</p>{/if}
      {#each activeTab?.blocks??[] as block (block.id)}{#if block.type==='group'}{@render Group(v,block)}{:else}{@render Repeater(v,block)}{/if}{/each}
      {#if !activeTab?.blocks.length}<div class="empty">This tab has no groups.</div>{/if}
    </div></fieldset>{:else}<div class="empty-state"><h2>No AI vehicles</h2><p>This aiv.zrd contains no editable name/record pairs.</p></div>{/if}
  </section>
</div>

{#snippet Group(v:VehicleView,group:AiLayoutGroup,item?:Json[])}
  {@const present=groupPresent(v,group)}
  <div class="card layout-group"><div class="title-row"><div><h2>{group.title}{#if group.presenceTag} <small>({group.presenceTag})</small>{/if}</h2>{#if group.description}<p>{group.description}</p>{/if}</div>{#if group.presenceTag}<label class="switch"><input type="checkbox" checked={present} onchange={event=>{setGroupPresent(v,group,event.currentTarget.checked);touch()}}> Present</label>{/if}</div>
    {#if present}<div class="layout-field-grid columns-{group.columns}">{#each group.fields as field (field.id)}{@render Field(v,field,item)}{/each}</div>{:else}<div class="empty">This optional section is not present for this vehicle.</div>{/if}
  </div>
{/snippet}

{#snippet Repeater(v:VehicleView,repeater:AiLayoutRepeater)}
  {@const items=repeaterItems(v,repeater)}
  <div class="card repeater-card"><div class="title-row"><div><h2>{repeater.title} <small>({repeater.binding})</small></h2>{#if repeater.description}<p>{repeater.description}</p>{/if}</div><button type="button" onclick={()=>{addRepeaterItem(v,repeater);touch()}}>{repeater.addLabel}</button></div>
    {#each items as rawItem,index (index)}{#if Array.isArray(rawItem)}<div class="repeater-item"><div class="row-head"><b>{repeater.itemLabel} {index+1}</b><button type="button" onclick={()=>{removeRepeaterItem(v,repeater,index);touch()}}>Remove</button></div>{#each repeater.groups as group (group.id)}{@render Group(v,group,array(rawItem))}{/each}</div>{/if}{/each}
    {#if !items.length}<div class="empty">No {repeater.itemLabel.toLowerCase()} entries.</div>{/if}
  </div>
{/snippet}

{#snippet Field(v:VehicleView,field:AiLayoutField,item?:Json[])}
  {#if field.control==='team'}
    <div class="layout-field width-{field.width}"><TeamEditor value={v.record[3]} lancemate={tagValue(v,'lancemate')} reference={controlledTeam()} title={field.label} description={field.description} sourceLabel={displayBinding(field.binding)} onChange={value=>setTeam(v,value)} onLancemateChange={state=>setLancemate(v,state)}/></div>
  {:else if field.control==='json'}
    <div class="layout-field width-{field.width}"><span class="field-title">{field.label} <small>({displayBinding(field.binding)})</small></span>{#if field.description}<small class="field-description">{field.description}</small>{/if}<pre class="layout-json">{JSON.stringify(fieldValue(v,field.binding,item),null,2)}</pre></div>
  {:else}
    <label class="layout-field width-{field.width}"><span class="field-title">{field.label} <small>({displayBinding(field.binding)})</small></span>{#if field.description}<small class="field-description">{field.description}</small>{/if}
      {#if field.control==='checkbox'}<span class="boolean-control"><input type="checkbox" disabled={field.readOnly} checked={booleanValue(v,field,item)} onchange={event=>changeField(v,field,event.currentTarget.checked?'1':'0',item)}><span>{booleanValue(v,field,item)?'On':'Off'}</span></span>
      {:else if field.control==='select'}<select disabled={field.readOnly} value={displayValue(v,field,item)} onchange={event=>changeField(v,field,event.currentTarget.value,item)}>{#each options(field) as option}<option value={option.value}>{option.label}</option>{/each}</select>
      {:else if field.control==='reference'}<div class="reference-input"><input disabled={field.readOnly} list={`ai-reference-${field.referenceKind??'none'}`} value={referenceValue(v,field,item)} oninput={event=>changeField(v,field,event.currentTarget.value,item)}>{#if field.referenceKind==='region'}<button type="button" disabled={!referenceList(field).includes(referenceValue(v,field,item))} onclick={()=>onNavigate('region',referenceValue(v,field,item))}>Open</button>{/if}</div>{#if roles(v,field,item).length}<div class="role-strip">{#each roles(v,field,item).slice(0,4) as reference}<span title={reference.location}>{reference.role} · {reference.layer}</span>{/each}</div>{/if}
      {:else}<input disabled={field.readOnly} type={field.control==='number'?'number':'text'} step={field.format==='integer'?'1':'any'} value={displayValue(v,field,item)} oninput={event=>changeField(v,field,event.currentTarget.value,item)}>{/if}
    </label>
  {/if}
{/snippet}

<style>
  .tab-description{margin:0 0 15px;color:#8fa3aa}.layout-group+.layout-group,.layout-group+.repeater-card,.repeater-card+.layout-group{margin-top:14px}.layout-field-grid{display:flex;flex-wrap:wrap;align-items:flex-start;gap:13px;margin-top:15px}.layout-field{min-width:0}.layout-field.width-small{flex:0 0 calc((100% - 26px)/3)}.layout-field.width-normal{flex:0 0 calc((100% - 13px)/2)}.layout-field.width-wide{flex:1 1 0%;min-width:0}.layout-field.width-full{flex:0 0 100%}.field-description{display:block;min-height:14px;margin:3px 0 6px;color:#7f939a;font-size:10px;line-height:1.35}.layout-json{max-height:480px;margin:0;overflow:auto}.repeater-item{margin-top:16px;padding:12px;border:1px solid #2e4047;border-radius:7px;background:#0d1519}.repeater-item .layout-group{margin-top:10px;background:#111b20}.repeater-item .layout-group:first-of-type{margin-top:0}.team-editor{margin-top:0}@media(max-width:1350px){.layout-field-grid.columns-3{grid-template-columns:repeat(2,minmax(0,1fr))}}
</style>
