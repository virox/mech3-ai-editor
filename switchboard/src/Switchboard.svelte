<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { onMount, tick } from 'svelte';
  import AnimationBoard, { type AnimationSnapshot } from './AnimationBoard.svelte';

  type Candidate = { pid:number; name:string; path?:string; compatible:boolean; detail:string };
  type Valve = { name:string; value:number; changedAt?:number };
  type Status = { state:string; pid?:number; build?:string; valveCount:number; writeEnabled:boolean; lastSnapshotMs?:number; message?:string };
  type Snapshot = { status:Status; valves:Valve[]; animations:AnimationSnapshot };
  type View = 'valves'|'animations';
  type Filter = 'all'|'on'|'off'|'other';

  let candidates=$state<Candidate[]>([]);
  let status=$state<Status>({state:'disconnected',valveCount:0,writeEnabled:false,message:'Looking for a compatible Mech3.exe process…'});
  let valves=$state<Valve[]>([]);
  const emptyAnimations=():AnimationSnapshot=>({revision:0,animations:[],events:[]});
  let animations=$state<AnimationSnapshot>(emptyAnimations());
  let view=$state<View>('valves');
  let search=$state('');
  let filter=$state<Filter>('all');
  let selectedPid=$state<number|undefined>();
  let message=$state('');
  let messageTimer:ReturnType<typeof setTimeout>|undefined;
  let attaching=$state(false);
  let writingName=$state<string|undefined>();
  let editingName=$state<string|undefined>();
  let activeEditor=$state<HTMLInputElement|undefined>();
  let drafts=$state<Record<string,string>>({});
  let hasAutoAttached=$state<number|undefined>();
  let autoAttachPaused=$state(false);

  const visible=()=>valves.filter(valve=>{
    const needle=search.trim().toLowerCase();
    if(needle&&!valve.name.toLowerCase().includes(needle))return false;
    return filter==='all'||(filter==='on'&&valve.value===1)||(filter==='off'&&valve.value===0)||(filter==='other'&&valve.value!==0&&valve.value!==1);
  });
  const count=(kind:Filter)=>kind==='all'?valves.length:valves.filter(v=>kind==='on'?v.value===1:kind==='off'?v.value===0:v.value!==0&&v.value!==1).length;
  const updated=()=>status.lastSnapshotMs?new Date(status.lastSnapshotMs).toLocaleTimeString():'—';
  const led=(value:number)=>value===0?'off':value===1?'on':'other';
  const recentlyChanged=(valve:Valve)=>Boolean(valve.changedAt&&Date.now()-valve.changedAt<1100);
  function showMessage(next:string,duration=2600){
    message=next;
    if(messageTimer)clearTimeout(messageTimer);
    messageTimer=setTimeout(()=>{message='';messageTimer=undefined;},duration);
  }

  async function refreshStatus(){
    try{
      const snapshot=await invoke<Snapshot>('switchboard_snapshot');
      status=snapshot.status;
      valves=snapshot.valves;
      if(snapshot.animations.revision!==animations.revision)animations=snapshot.animations;
      const next=await invoke<Candidate[]>('switchboard_candidates');
      candidates=next;
      if(status.state==='disconnected'&&!autoAttachPaused){
        const compatible=next.filter(candidate=>candidate.compatible);
        if(compatible.length===1&&hasAutoAttached!==compatible[0].pid){hasAutoAttached=compatible[0].pid;await attach(compatible[0].pid);}
        else if(compatible.length!==1)hasAutoAttached=undefined;
      }
    }catch(error){showMessage(String(error),5000);}
  }
  async function attach(pid:number){
    attaching=true;selectedPid=pid;
    try{status=await invoke<Status>('switchboard_attach',{pid});autoAttachPaused=false;showMessage('Attached. Monitoring is active; direct writes remain disabled.');}
    catch(error){showMessage(String(error),5000);}
    finally{attaching=false;}
  }
  async function detach(){
    try{
      status=await invoke<Status>('switchboard_detach');
      valves=[];
      animations=emptyAnimations();
      hasAutoAttached=undefined;
      autoAttachPaused=true;
      showMessage('Detached. Automatic attach is paused; choose Attach to reconnect.',5000);
    }catch(error){showMessage(String(error),5000);}
  }
  async function setWriteEnabled(enabled:boolean){
    try{
      status=await invoke<Status>('switchboard_set_write_enabled',{enabled});
      showMessage(enabled?'Direct value writes enabled for this session.':'Direct value writes disabled. The switchboard is monitoring only.');
    }
    catch(error){showMessage(String(error),5000);}
  }
  async function writeValve(valve:Valve, requested:number){
    if(!Number.isInteger(requested)||requested< -2147483648||requested>2147483647){showMessage(`${valve.name} needs a 32-bit integer value.`,5000);return;}
    writingName=valve.name;
    try{const result=await invoke<Valve>('switchboard_set_valve',{name:valve.name,value:requested,expectedValue:valve.value});drafts={...drafts,[valve.name]:String(result.value)};editingName=undefined;showMessage(`Set ${result.name} to ${result.value}.`);}
    catch(error){showMessage(String(error),5000);}
    finally{writingName=undefined;}
  }
  async function setValve(valve:Valve){await writeValve(valve,Number(drafts[valve.name]??valve.value));}
  async function toggleValve(valve:Valve){await writeValve(valve,valve.value===0?1:0);}
  async function beginEdit(valve:Valve){
    drafts={...drafts,[valve.name]:drafts[valve.name]??String(valve.value)};
    editingName=valve.name;
    await tick();
    activeEditor?.focus();
    activeEditor?.select();
  }
  function handleValueKey(event:KeyboardEvent,valve:Valve){
    if(event.key==='Enter'){event.preventDefault();void setValve(valve);}
    if(event.key==='Escape'){drafts={...drafts,[valve.name]:String(valve.value)};editingName=undefined;}
  }

  onMount(()=>{
    let unlisten:UnlistenFn|undefined,unlistenAnimations:UnlistenFn|undefined;
    void listen<Snapshot>('switchboard:snapshot',event=>{status=event.payload.status;valves=event.payload.valves;})
      .then(stop=>unlisten=stop)
      .catch(error=>showMessage(`Live event stream unavailable; using snapshot polling. ${String(error)}`,5000));
    void listen<AnimationSnapshot>('switchboard:animations',event=>{animations=event.payload;})
      .then(stop=>unlistenAnimations=stop)
      .catch(()=>undefined);
    void refreshStatus();
    const timer=window.setInterval(()=>void refreshStatus(),1000);
    return()=>{window.clearInterval(timer);if(messageTimer)clearTimeout(messageTimer);unlisten?.();unlistenAnimations?.();};
  });
</script>

<svelte:head><meta name="theme-color" content="#111a21" /></svelte:head>

<main>
  <header>
    <div class="brand"><span class="mark">V</span><div><small>MECHWARRIOR 3 · LIVE DEBUGGER</small><h1>Valve Switchboard</h1></div></div>
    <nav class="view-tabs" aria-label="Switchboard view"><button class:active={view==='valves'} onclick={()=>view='valves'}>Valves<em>{status.valveCount}</em></button><button class:active={view==='animations'} onclick={()=>view='animations'}>Animations<em>{animations.animations.length}</em></button></nav>
    <div class:attached={status.state==='attached'} class="connection"><i></i><div><b>{status.state==='attached'?(status.writeEnabled?'Live monitor + direct writer':'Live monitor'):status.state==='waitingForRegistry'?'Waiting for mission registry':'Not attached'}</b><span>{status.pid?`PID ${status.pid} · ${status.build??'checking build'}`:'No game memory is being read'}</span></div></div>
  </header>

  <section class="status-bar">
    {#if view==='animations'}<div><b>{animations.animations.length}</b><span>animation definitions</span></div><div><b>{animations.animations.filter(animation=>animation.state===2||animation.state===6).length}</b><span>running</span></div><div class="status-message">{message||animations.message||'Reading the live animation table (read-only).'}</div>
    {:else}<div><b>{status.valveCount}</b><span>live valves</span></div><div><b>{updated()}</b><span>last snapshot</span></div><div class="status-message">{message||status.message}</div>{/if}
    {#if status.state!=='disconnected'&&view==='animations'}<button class="secondary" onclick={detach}>Detach</button>{:else if status.state!=='disconnected'}<label class="write-toggle"><input type="checkbox" checked={status.writeEnabled} onchange={event=>setWriteEnabled(event.currentTarget.checked)}/> Enable direct writes</label><button class="secondary" onclick={detach}>Detach</button>{/if}
  </section>

  {#if status.state==='disconnected'&&candidates.length}
    <section class="candidates"><h2>Mech3.exe processes</h2><p>Only the verified 1.2.22 build can be opened. Monitoring uses query/read access; direct writes require the separate session control.</p>{#each candidates as candidate}<div class:unsupported={!candidate.compatible} class="candidate"><div><b>{candidate.name} · PID {candidate.pid}</b><span>{candidate.path??candidate.detail}</span><small>{candidate.detail}</small></div><button disabled={!candidate.compatible||attaching} onclick={()=>attach(candidate.pid)}>{attaching&&selectedPid===candidate.pid?'Attaching…':'Attach'}</button></div>{/each}</section>
  {:else if status.state==='disconnected'}
    <section class="empty"><div class="empty-led"></div><h2>Waiting for Mech3.exe</h2><p>Launch a supported game build. This companion will detect and attach to one compatible process automatically.</p></section>
  {:else if view==='animations'}
    <AnimationBoard snapshot={animations}/>
  {:else}
    <section class="controls"><input aria-label="Search valves" placeholder="Filter valve names…" bind:value={search}/><div class="filters">{#each [['all','All'],['on','On'],['off','Off'],['other','Other']] as item}<button class:active={filter===item[0]} onclick={()=>filter=item[0] as Filter}>{item[1]} <em>{count(item[0] as Filter)}</em></button>{/each}</div></section>
    {#if visible().length}
      <section class="grid" aria-live="polite">{#each visible() as valve (valve.name)}<article class:changed={recentlyChanged(valve)}>{#if status.writeEnabled}<button class="led" class:off={led(valve.value)==='off'} class:on={led(valve.value)==='on'} class:other={led(valve.value)==='other'} aria-label={`Set ${valve.name} to ${valve.value===0?1:0}`} title={valve.value===0?'Set to 1':'Set to 0'} disabled={writingName===valve.name} onclick={()=>toggleValve(valve)}></button>{:else}<i class:off={led(valve.value)==='off'} class:on={led(valve.value)==='on'} class:other={led(valve.value)==='other'}></i>{/if}<div><h2>{valve.name}</h2><span>{valve.value===0?'OFF':valve.value===1?'ON':'INTEGER VALUE'}</span></div>{#if status.writeEnabled&&editingName===valve.name}<input bind:this={activeEditor} class="inline-value" aria-label={`Value for ${valve.name}`} type="number" value={drafts[valve.name]??valve.value} onblur={()=>editingName=undefined} oninput={event=>drafts={...drafts,[valve.name]:event.currentTarget.value}} onkeydown={event=>handleValueKey(event,valve)}/>{:else if status.writeEnabled}<button class="editable-value" type="button" aria-label={`Edit ${valve.name} value`} onclick={()=>beginEdit(valve)}>{valve.value}</button>{:else}<strong>{valve.value}</strong>{/if}</article>{/each}</section>
    {:else}<section class="empty compact"><h2>No matching valves</h2><p>Adjust the text filter or state filter.</p></section>{/if}
  {/if}
</main>
