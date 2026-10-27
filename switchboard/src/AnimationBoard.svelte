<script lang="ts" module>
  export type Sequence = { name:string; state:number; loopCount:number; hasCurrentEvent:boolean };
  export type Animation = { index:number; animName:string; name:string; rootName:string; state:number; activation:number; flags:number; instances:number; runningInstances:number; resetTime:number; timer:number; maxHealth:number; curHealth:number; sequences:Sequence[]; changedAt?:number };
  export type Transition = { at:number; index:number; animName:string; name:string; fromState:number; toState:number; runningInstances:number };
  export type AnimationSnapshot = { revision:number; lastReadMs?:number; message?:string; animations:Animation[]; events:Transition[] };

  // anim_state at AnimDef+0x98; see src-tauri/src/animations.rs for the evidence per value.
  export const STATE_LABELS:Record<number,string>={0:'Starting',1:'Ready',2:'Running',3:'Executed',4:'Invalidated',5:'Disabled',6:'Running, then invalidate'};
  export const stateLabel=(state:number)=>STATE_LABELS[state]??`State ${state}`;
  export const stateClass=(state:number)=>state===2||state===6?'running':state===3?'executed':state===4||state===5?'invalid':state===1?'ready':'other';
</script>

<script lang="ts">
  type Filter='all'|'running'|'executed'|'ready'|'invalid'|'other';
  type Sort='index'|'name'|'changed';
  let { snapshot }:{ snapshot:AnimationSnapshot }=$props();

  const ACTIVATION=['Weapon hit','Collide hit','Weapon or collide hit','On call','On startup'];
  const RENDER_LIMIT=400;
  let search=$state('');
  let filter=$state<Filter>('all');
  let sort=$state<Sort>('index');
  let hideEffects=$state(true);
  let now=$state(Date.now());

  const isEffect=(animation:Animation)=>animation.animName.endsWith('.flt')||animation.name.endsWith('.flt');
  const matchesState=(animation:Animation,kind:Filter)=>kind==='all'||stateClass(animation.state)===kind;
  const base=$derived(snapshot.animations.filter(animation=>{
    if(hideEffects&&isEffect(animation))return false;
    const needle=search.trim().toLowerCase();
    return !needle||animation.animName.toLowerCase().includes(needle)||animation.name.toLowerCase().includes(needle);
  }));
  const visible=$derived.by(()=>{
    const list=base.filter(animation=>matchesState(animation,filter));
    if(sort==='name')list.sort((a,b)=>a.animName.localeCompare(b.animName));
    if(sort==='changed')list.sort((a,b)=>(b.changedAt??0)-(a.changedAt??0)||a.index-b.index);
    return list;
  });
  const count=(kind:Filter)=>base.filter(animation=>matchesState(animation,kind)).length;
  const events=$derived(snapshot.events.filter(event=>!hideEffects||!(event.animName.endsWith('.flt')||event.name.endsWith('.flt'))));
  const recentlyChanged=(animation:Animation)=>Boolean(animation.changedAt&&now-animation.changedAt<1500);
  const seconds=(value:number)=>value<60?`${value.toFixed(1)} s`:`${Math.floor(value/60)}:${String(Math.floor(value%60)).padStart(2,'0')} min`;
  const time=(ms:number)=>new Date(ms).toLocaleTimeString();

  $effect(()=>{const timer=window.setInterval(()=>now=Date.now(),500);return()=>window.clearInterval(timer)});
</script>

{#if snapshot.message&&!snapshot.animations.length}
  <section class="empty compact"><h2>No animation table yet</h2><p>{snapshot.message}</p></section>
{:else}
  <section class="controls">
    <input aria-label="Search animations" placeholder="Filter animation or node names…" bind:value={search}/>
    <div class="filters">{#each [['all','All'],['running','Running'],['executed','Executed'],['ready','Ready'],['invalid','Invalid'],['other','Other']] as item}<button class:active={filter===item[0]} onclick={()=>filter=item[0] as Filter}>{item[1]} <em>{count(item[0] as Filter)}</em></button>{/each}</div>
    <label class="anim-toggle"><input type="checkbox" bind:checked={hideEffects}/> Hide .flt effects</label>
    <select aria-label="Sort animations" bind:value={sort}><option value="index">Table order</option><option value="name">Name</option><option value="changed">Last change</option></select>
  </section>
  <div class="anim-layout">
    <div>
      {#if visible.length}
        <section class="anim-grid" aria-live="polite">
          {#each visible.slice(0,RENDER_LIMIT) as animation (animation.index)}
            <article class:changed={recentlyChanged(animation)}>
              <i class={stateClass(animation.state)}></i>
              <div class="anim-main">
                <h2 title={`#${animation.index}`}>{animation.animName}</h2>
                <span>{animation.name}{animation.rootName&&animation.rootName!==animation.name?` · root ${animation.rootName}`:''} · {ACTIVATION[animation.activation]??`activation ${animation.activation}`}</span>
                {#if animation.maxHealth>0}<div class="anim-health" title={`Health ${animation.curHealth.toFixed(0)} / ${animation.maxHealth.toFixed(0)}`}><b style={`width:${Math.max(0,Math.min(100,animation.curHealth/animation.maxHealth*100))}%`}></b></div>{/if}
                {#if animation.sequences.length}<ul class="anim-sequences">{#each animation.sequences as sequence}<li title="Raw runtime sequence state; only 0 (initial) and 3 (on call) are confirmed from file data.">{sequence.name} <em>s{sequence.state}{sequence.loopCount?` · loop ${sequence.loopCount}`:''}</em></li>{/each}</ul>{/if}
              </div>
              <div class="anim-side">
                <strong class={stateClass(animation.state)}>{stateLabel(animation.state)}</strong>
                {#if animation.instances>1}<small>{animation.runningInstances}/{animation.instances} instances</small>{/if}
                {#if animation.state===2||animation.state===6}<small>{seconds(animation.timer)}</small>{:else if animation.state===3&&animation.resetTime>0}<small>reset {seconds(Math.max(0,animation.resetTime-animation.timer))}</small>{/if}
              </div>
            </article>
          {/each}
        </section>
        {#if visible.length>RENDER_LIMIT}<p class="anim-more">{visible.length-RENDER_LIMIT} more — refine the filter to see them.</p>{/if}
      {:else}<section class="empty compact"><h2>No matching animations</h2><p>Adjust the text filter, state filter, or effect toggle.</p></section>{/if}
    </div>
    <aside class="anim-log">
      <h2>Transitions <em>{events.length}</em></h2>
      {#if events.length}
        <ol>{#each events as event (event.at+':'+event.index+':'+event.toState)}<li><time>{time(event.at)}</time><b>{event.animName}</b><span><span class={stateClass(event.fromState)}>{stateLabel(event.fromState)}</span> → <span class={stateClass(event.toState)}>{stateLabel(event.toState)}</span>{event.runningInstances>1?` · ${event.runningInstances} running`:''}</span></li>{/each}</ol>
      {:else}<p>State changes appear here while the mission runs. The table is read 5 times per second, so very short runs can be missed.</p>{/if}
    </aside>
  </div>
{/if}
