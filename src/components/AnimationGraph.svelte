<script lang="ts">
  import {invoke} from '@tauri-apps/api/core';
  import type {Json,WorkspaceSnapshot} from '../types';
  import {CARD_HEADER,CARD_WIDTH,ROW_HEIGHT,activationLabel,animationEdges,callbackMeaning,cardHeight,eventSummary,executableEntry,executionLabel,layoutNeighborhood,neighborhood,startLabel,stateChannel,type AnimationDefinitionSummary,type AnimationEdge,type AnimationIndex} from '../animation-graph';

  let {workspace}:{workspace:WorkspaceSnapshot}=$props();

  let index=$state<AnimationIndex|null>(null);let loadedFor=$state('');let error=$state('');let loading=$state(false);
  let search=$state('');let filter=$state<'all'|'entry'|'callbacks'|'startup'|'hit'>('all');let depth=$state(2);
  let focus=$state<number|null>(null);let selected=$state<number|null>(null);let detail=$state<Json|null>(null);let detailFor=$state<number|null>(null);
  let viewX=$state(0);let viewY=$state(0);let viewWidth=$state(1500);let viewHeight=$state(850);let panPointer=$state<number|null>(null);let panX=$state(0);let panY=$state(0);

  let edges=$derived.by(()=>index?animationEdges(index):[] as AnimationEdge[]);
  let hood=$derived.by(()=>index&&focus!==null?neighborhood(edges,focus,depth):null);
  let positions=$derived.by(()=>index&&hood?layoutNeighborhood(index,hood):{} as Record<string,{x:number;y:number}>);
  let visibleDefinitions=$derived.by(()=>index&&hood?[...hood.layers.keys()].map(item=>index!.definitions[item]):[]);
  let visibleEdges=$derived.by(()=>{if(!hood)return[];const shown=hood.layers;return edges.flatMap(edge=>edge.to.filter(to=>shown.has(edge.from)&&shown.has(to)).map(to=>({edge,to})))});
  let matches=$derived.by(()=>{
    if(!index)return[];const query=search.trim().toLowerCase();
    return index.definitions.filter(item=>{
      if(filter==='entry'&&!executableEntry(item.animName))return false;
      if(filter==='callbacks'&&!item.hasCallbacks)return false;
      const activation=activationLabel(item.activation);
      if(filter==='startup'&&activation!=='OnStartup')return false;
      if(filter==='hit'&&!/Hit/.test(activation))return false;
      return !query||`${item.animName} ${item.name} ${item.rootName} ${item.fileName}`.toLowerCase().includes(query);
    });
  });

  // anim.zbd is campaign-wide; reload only when the campaign folder changes.
  $effect(()=>{const mission=workspace.selectedMission,key=mission.slice(0,Math.max(mission.lastIndexOf('\\'),mission.lastIndexOf('/'))).toLowerCase();if(key&&key!==loadedFor){loadedFor=key;load(mission)}});

  async function load(mission:string){
    loading=true;error='';index=null;focus=null;selected=null;detail=null;
    try{
      const next=await invoke<AnimationIndex>('load_animation_index',{rootPath:workspace.rootPath,missionPath:mission});
      index=next;
      const number=/readerm(\d+)\.zbd$/i.exec(mission)?.[1];
      const entry=next.definitions.find(item=>number&&item.animName===`start_single_player_m${number}`)??next.definitions.find(item=>item.animName==='start_single_player');
      if(entry)focusOn(entry.index);
    }catch(reason){error=String(reason)}finally{loading=false}
  }
  async function select(item:number){
    selected=item;if(detailFor===item)return;detail=null;detailFor=item;
    try{const next=await invoke<Json>('load_animation_definition',{path:index!.path,index:item});if(detailFor===item)detail=next}catch(reason){error=String(reason)}
  }
  function focusOn(item:number){focus=item;select(item);queueMicrotask(fit)}

  function graphPoint(event:PointerEvent|WheelEvent,element:SVGSVGElement){const rect=element.getBoundingClientRect();return{x:viewX+(event.clientX-rect.left)*viewWidth/rect.width,y:viewY+(event.clientY-rect.top)*viewHeight/rect.height}}
  function zoomAt(factor:number,x=viewX+viewWidth/2,y=viewY+viewHeight/2){const width=Math.min(9000,Math.max(420,viewWidth*factor)),height=width*(viewHeight/viewWidth);const rx=(x-viewX)/viewWidth,ry=(y-viewY)/viewHeight;viewX=x-rx*width;viewY=y-ry*height;viewWidth=width;viewHeight=height}
  function wheel(event:WheelEvent){event.preventDefault();const point=graphPoint(event,event.currentTarget as SVGSVGElement);zoomAt(event.deltaY<0?.82:1.22,point.x,point.y)}
  function down(event:PointerEvent){if(event.button!==0&&event.button!==1)return;panPointer=event.pointerId;panX=event.clientX;panY=event.clientY;(event.currentTarget as SVGSVGElement).setPointerCapture(event.pointerId)}
  function move(event:PointerEvent){if(panPointer!==event.pointerId)return;const rect=(event.currentTarget as SVGSVGElement).getBoundingClientRect();viewX-=(event.clientX-panX)*viewWidth/rect.width;viewY-=(event.clientY-panY)*viewHeight/rect.height;panX=event.clientX;panY=event.clientY}
  function up(event:PointerEvent){if(panPointer===event.pointerId)panPointer=null}
  function fit(){
    const boxes=Object.entries(positions).map(([id,point])=>{const definition=id.startsWith('def:')?index?.definitions[Number(id.slice(4))]:undefined;return{point,h:definition?cardHeight(definition):44}});
    if(!boxes.length)return;
    const minX=Math.min(...boxes.map(box=>box.point.x)),minY=Math.min(...boxes.map(box=>box.point.y)),maxX=Math.max(...boxes.map(box=>box.point.x+CARD_WIDTH)),maxY=Math.max(...boxes.map(box=>box.point.y+box.h));
    viewX=minX-80;viewY=minY-80;viewWidth=Math.max(800,maxX-minX+160);viewHeight=Math.max(460,maxY-minY+160);
  }

  function rowY(sequence:number){return CARD_HEADER+sequence*ROW_HEIGHT+ROW_HEIGHT/2}
  function edgePath(edge:AnimationEdge,to:number){
    const from=positions[`def:${edge.from}`],target=positions[`def:${to}`];if(!from||!target)return'';
    const a={x:from.x+CARD_WIDTH,y:from.y+(edge.sequence<0?28:rowY(edge.sequence))},b={x:target.x,y:target.y+28};
    if(b.x<=a.x){const lift=Math.min(a.y,b.y)-60;return`M ${a.x} ${a.y} C ${a.x+120} ${a.y}, ${a.x+120} ${lift}, ${(a.x+b.x)/2} ${lift} S ${b.x-120} ${b.y}, ${b.x} ${b.y}`}
    const curve=Math.max(50,(b.x-a.x)*.45);return`M ${a.x} ${a.y} C ${a.x+curve} ${a.y}, ${b.x-curve} ${b.y}, ${b.x} ${b.y}`;
  }
  function externalPath(item:{from:number;id:string}){const from=positions[`def:${item.from}`],target=positions[item.id];if(!from||!target)return'';const a={x:from.x+CARD_WIDTH,y:from.y+28},b={x:target.x,y:target.y+22};return`M ${a.x} ${a.y} C ${a.x+60} ${a.y}, ${b.x-60} ${b.y}, ${b.x} ${b.y}`}
  function sequenceCallbacks(definition:AnimationDefinitionSummary,sequence:number){return definition.sequences[sequence].links.filter(link=>link.kind==='Callback'&&link.value!==undefined).map(link=>link.value!)}
  function callbackChip(definition:AnimationDefinitionSummary,value:number){const channel=stateChannel(definition,value);return channel?`◆ ${channel}=${value&1}`:`cb ${value}`}
  function callers(item:number){return edges.filter(edge=>edge.to.includes(item))}
  function callees(item:number){return edges.filter(edge=>edge.from===item)}
  function detailSequences(){const value=detail as Record<string,Json>|null;return Array.isArray(value?.sequences)?value!.sequences as Record<string,Json>[]:[]}
  function label(item:number){const definition=index!.definitions[item];return`${definition.animName} (${definition.name})`}
</script>

<div class="behavior-shell animation-shell">
  <aside class="behavior-palette">
    <div class="side-heading"><div><small>ANIM.ZBD</small><b>Definitions</b></div>{#if index}<span class="count">{matches.length}</span>{/if}</div>
    <input class="graph-search" placeholder="Search anim / node name…" bind:value={search}>
    <div class="anim-filters">{#each [['all','All'],['entry','Exe entry'],['startup','OnStartup'],['hit','Hit'],['callbacks','Callbacks']] as [value,text]}<button class:active={filter===value} onclick={()=>filter=value as typeof filter}>{text}</button>{/each}</div>
    {#if loading}<p class="palette-help">Reading anim.zbd…</p>{/if}
    {#if error}<p class="field-error">{error}</p>{/if}
    {#each matches.slice(0,400) as item (item.index)}<button class="palette-block anim-entry" class:active={focus===item.index} onclick={()=>focusOn(item.index)}><b>{item.animName}</b><small>{item.name} · {activationLabel(item.activation)}{item.hasCallbacks?' · callbacks':''}{executableEntry(item.animName)?' · exe entry':''}</small></button>{/each}
    {#if matches.length>400}<p class="palette-help">{matches.length-400} more; refine the search.</p>{/if}
  </aside>

  <section class="behavior-workspace">
    <div class="behavior-toolbar">
      <div><small>{index?.relativePath.toUpperCase()??'ANIM.ZBD'}</small><h2>Animation Graph</h2></div>
      <div class="graph-tools"><label>Depth <select bind:value={depth}>{#each [1,2,3,4] as value}<option value={value}>{value}</option>{/each}</select></label><div class="zoom-controls"><button title="Zoom out" onclick={()=>zoomAt(1.22)}>−</button><span>{Math.round(1500/viewWidth*100)}%</span><button title="Zoom in" onclick={()=>zoomAt(.82)}>+</button><button onclick={fit}>Fit</button></div></div>
    </div>
    <div class="layer-filter"><span>Read-only view of the compiled campaign animations. Callers expand left, callees right.</span><span class="graph-count">{visibleDefinitions.length} definitions · {visibleEdges.length} links{hood?.truncated?` · ${hood.truncated} hidden by layer limit`:''}</span></div>
    <div class="behavior-layout">
      <div class="behavior-canvas">
        {#if focus===null}<div class="anim-empty">{loading?'Loading…':'Select a definition on the left.'}</div>{/if}
        <svg class:panning={panPointer!==null} viewBox={`${viewX} ${viewY} ${viewWidth} ${viewHeight}`} role="application" aria-label="Animation call graph. Drag to pan, wheel to zoom, click a card to inspect it, double-click to focus it." onwheel={wheel} onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up}>
          <defs><pattern id="anim-grid" width="30" height="30" patternUnits="userSpaceOnUse"><path d="M 30 0 L 0 0 0 30" fill="none" stroke="#203138" stroke-width="1"/></pattern><marker id="anim-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#7f9aa3"/></marker></defs>
          <rect x={viewX-100} y={viewY-100} width={viewWidth+200} height={viewHeight+200} fill="#081115"/><rect x={viewX-100} y={viewY-100} width={viewWidth+200} height={viewHeight+200} fill="url(#anim-grid)"/>
          {#each visibleEdges as {edge,to} (`${edge.id}>${to}`)}<path class="anim-edge {edge.kind}" d={edgePath(edge,to)} marker-end="url(#anim-arrow)"><title>{edge.kind} {edge.target}{startLabel(edge.start)?` · ${startLabel(edge.start)}`:''}</title></path>{/each}
          {#each hood?.external??[] as item (item.id)}{@const point=positions[item.id]}{#if point}<path class="anim-edge external" d={externalPath(item)}/><g class="anim-external" transform={`translate(${point.x} ${point.y})`}><rect width={CARD_WIDTH} height="44" rx="8"/><text x="14" y="19">{item.name}</text><text class="anim-muted" x="14" y="34">not defined in this anim.zbd</text></g>{/if}{/each}
          {#each visibleDefinitions as definition (definition.index)}{@const point=positions[`def:${definition.index}`]}{#if point}<g class="anim-card" class:focus={focus===definition.index} class:selected={selected===definition.index} class:entry={executableEntry(definition.animName)} transform={`translate(${point.x} ${point.y})`} role="button" tabindex="0" aria-label={`Animation ${definition.animName}`} onpointerdown={event=>event.stopPropagation()} onclick={()=>select(definition.index)} ondblclick={()=>focusOn(definition.index)} onkeydown={event=>{if(event.key==='Enter')focusOn(definition.index);else if(event.key===' '){event.preventDefault();select(definition.index)}}}>
            <rect width={CARD_WIDTH} height={cardHeight(definition)} rx="10"/><rect class="anim-stripe" width="6" height={cardHeight(definition)} rx="3"/>
            <text class="node-kind" x="16" y="19">{activationLabel(definition.activation)} · {executionLabel(definition.execution)}{executableEntry(definition.animName)?' · EXE ENTRY':''}</text>
            <text class="node-label" x="16" y="40">{definition.animName.slice(0,30)}<title>{definition.animName}</title></text>
            <text class="node-subtitle" x="16" y="58">{definition.name}{definition.rootName&&definition.rootName!==definition.name?` · root ${definition.rootName}`:''}{definition.prerequisites.length?` · ${definition.prerequisites.length} prereq`:''}</text>
            {#each definition.sequences as sequence,position}{@const y=CARD_HEADER+position*ROW_HEIGHT}<g class="anim-row"><rect x="10" y={y} width={CARD_WIDTH-20} height={ROW_HEIGHT-3} rx="4"/><text x="18" y={y+14}>{sequence.name.slice(0,20)}</text><text class="anim-muted" x="150" y={y+14}>{sequence.eventCount} ev</text>{#if sequenceCallbacks(definition,position).length}{@const values=sequenceCallbacks(definition,position)}<text class="anim-callback" class:state={values.some(value=>!!stateChannel(definition,value))} x={CARD_WIDTH-16} y={y+14} text-anchor="end">{values.map(value=>callbackChip(definition,value)).join(' ')}<title>{values.map(value=>`CALLBACK ${value}: ${callbackMeaning(value)}`).join('\n')}</title></text>{/if}{#if sequence.links.some(link=>link.kind!=='Callback'&&link.target!==definition.animName)}<circle class="anim-port" cx={CARD_WIDTH} cy={y+ROW_HEIGHT/2} r="4"/>{/if}</g>{/each}
            {#if !definition.sequences.length}<text class="anim-muted" x="18" y={CARD_HEADER+14}>no sequences</text>{/if}
            <circle class="anim-port in" cx="0" cy="28" r="5"/>
          </g>{/if}{/each}
        </svg>
        <div class="graph-legend anim-legend"><span class="call">Call</span><span class="stop">Stop</span><span class="invalidate">Invalidate / Reset</span><span class="prereq">Prerequisite</span><em>Click: inspect · Double-click: focus · Drag: pan</em></div>
      </div>

      <aside class="graph-inspector">
        {#if index&&selected!==null}{@const definition=index.definitions[selected]}
          <div class="property-title"><div><small>ANIMATION DEFINITION</small><h3>{definition.animName}</h3></div>{#if focus!==selected}<button onclick={()=>focusOn(definition.index)}>Focus</button>{/if}</div>
          <dl class="binding-data"><dt>Node name</dt><dd>{definition.name}</dd><dt>Root</dt><dd>{definition.rootName||'—'}</dd><dt>Activation</dt><dd>{activationLabel(definition.activation)}</dd><dt>Execution</dt><dd>{executionLabel(definition.execution)}</dd><dt>Health</dt><dd>{String(definition.health)}</dd><dt>Reset time</dt><dd>{String(definition.resetTime)}</dd><dt>File</dt><dd>{definition.fileName}</dd></dl>
          {#if executableEntry(definition.animName)}<p class="warning-text">Called by Mech3.exe at mission start (start_single_player / _m%d format strings).</p>{/if}
          <div class="property-section"><h4>Called by ({callers(definition.index).length})</h4>{#each callers(definition.index) as edge}<button class="anim-link" onclick={()=>focusOn(edge.from)}><b>{label(edge.from)}</b><small>{edge.kind}{edge.sequence>=0?` · ${index.definitions[edge.from].sequences[edge.sequence]?.name}`:''}{startLabel(edge.start)?` · ${startLabel(edge.start)}`:''}</small></button>{:else}<p class="anim-muted-text">{executableEntry(definition.animName)?'The executable.':`No caller in this anim.zbd. Trigger: ${activationLabel(definition.activation)}.`}</p>{/each}</div>
          {#if definition.prerequisites.length}<div class="property-section"><h4>Prerequisites</h4>{#each definition.prerequisites as prerequisite}<p class="anim-muted-text">{prerequisite.kind}: {prerequisite.name}{prerequisite.required?' (required)':''}</p>{/each}</div>{/if}
          <div class="property-section"><h4>Calls ({callees(definition.index).length})</h4>{#each callees(definition.index) as edge}<div class="anim-call-row"><span>{edge.kind}</span>{#each edge.to as to}<button class="anim-link inline" onclick={()=>focusOn(to)}>{label(to)}</button>{:else}<code>{edge.target}</code>{/each}</div>{:else}<p class="anim-muted-text">None.</p>{/each}</div>
          <div class="property-section"><h4>Sequences</h4>{#if detailFor===selected&&!detail}<p class="anim-muted-text">Loading events…</p>{/if}{#each detailSequences() as sequence}<details class="anim-sequence" open={detailSequences().length<=3}><summary><b>{String(sequence.name)}</b><small>{String(sequence.seq_state??'')} · {Array.isArray(sequence.events)?sequence.events.length:0} events</small></summary><ol>{#each (Array.isArray(sequence.events)?sequence.events:[]) as event}{@const line=eventSummary(event)}<li class:callback={line.kind==='Callback'}><span class="anim-event-kind">{line.kind}</span>{#if line.start}<span class="anim-event-start">{line.start}</span>{/if}<span class="anim-event-text">{line.text}</span></li>{/each}</ol></details>{/each}</div>
          {#if detail}<details class="property-section"><summary>Raw definition JSON</summary><pre class="small-raw">{JSON.stringify(detail,null,2)}</pre></details>{/if}
        {:else}<div class="empty"><h3>Select an animation</h3><p>Definitions come from the campaign's compiled anim.zbd (read with mech3ax). Callback chips marked ◆ write a state channel that also appears in the Behavior Graph.</p></div>{/if}
      </aside>
    </div>
  </section>
</div>
