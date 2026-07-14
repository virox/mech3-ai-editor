<script lang="ts">
  import type { Json, WorkspaceDocument, WorkspaceSnapshot } from '../types';
  import { array, documentTitle, documentsOf, isLocked, netNodes, pairList, parseScalar, scalar, tags, type NetNodeView } from '../workspace';
  let { workspace, onChanged, focusName='' }: {workspace:WorkspaceSnapshot;onChanged:()=>void;focusName?:string}=$props();
  const docs=()=>documentsOf(workspace,['network','region']);
  let selectedDoc=$state(Math.max(0,docs().findIndex(d=>documentTitle(d).toLowerCase()===focusName.toLowerCase())));let selectedNode=$state('');let dragIndex=$state<number|null>(null);let newTag=$state('');let tagError=$state('');
  function doc():WorkspaceDocument|undefined{const all=docs();if(selectedDoc>=all.length)selectedDoc=Math.max(0,all.length-1);return all[selectedDoc]}
  function nodes():NetNodeView[]{const d=doc();return d?netNodes(d):[]}
  function spatial():NetNodeView[]{return nodes().filter(n=>n.spatial&&n.position.length>=3)}
  function selected():NetNodeView|undefined{return nodes().find(n=>n.key===selectedNode)??spatial()[0]}
  function bounds(){const list=spatial();const xs=list.map(n=>Number(n.position[0])||0),zs=list.map(n=>Number(n.position[2])||0);let minX=Math.min(...xs,0),maxX=Math.max(...xs,1),minZ=Math.min(...zs,0),maxZ=Math.max(...zs,1);if(maxX-minX<10){minX-=5;maxX+=5}if(maxZ-minZ<10){minZ-=5;maxZ+=5}return{minX,maxX,minZ,maxZ,sx:900/(maxX-minX),sz:560/(maxZ-minZ)}}
  function point(position:Json[]){const b=bounds();return{x:50+(Number(position[0])-b.minX)*b.sx,y:50+(Number(position[2])-b.minZ)*b.sz}}
  function world(x:number,y:number){const b=bounds();return{x:b.minX+(x-50)/b.sx,z:b.minZ+(y-50)/b.sz}}
  function edgeTarget(index:number){return spatial().find(n=>n.index===index)}
  function regionCycle():NetNodeView[]{const list=spatial();if(doc()?.kind!=='region'||list.length<3)return[];const byIndex=new Map(list.map(n=>[n.index,n]));if(list.some(n=>n.links.filter(x=>typeof x==='number'&&byIndex.has(Number(x))).length!==2))return[];const ordered=[list[0]];let previous=-1,current=list[0];while(ordered.length<=list.length){const choices=current.links.map(Number).filter(i=>byIndex.has(i)&&i!==previous);const next=byIndex.get(choices[0]);if(!next)return[];if(next.index===ordered[0].index)return ordered.length===list.length?ordered:[];if(ordered.some(n=>n.index===next.index))return[];ordered.push(next);previous=current.index;current=next}return[]}
  function setPosition(node:NetNodeView,index:number,text:string){node.position[index]=parseScalar(text,node.position[index]);node.record[1]=node.position;onChanged()}
  function setLinks(node:NetNodeView,text:string){node.links=text.split(',').map(x=>x.trim()).filter(Boolean).map(Number).filter(Number.isFinite);node.record[2]=node.links;onChanged()}
  function setTag(node:NetNodeView,at:number,text:string){try{node.record[at+1]=JSON.parse(text);tagError='';onChanged()}catch(error){tagError=`Invalid JSON for ${node.record[at]}: ${String(error)}`}}
  function addTag(node:NetNodeView){const name=newTag.trim();if(!name)return;node.record.push(name,[]);newTag='';tagError='';onChanged()}
  function removeTag(node:NetNodeView,at:number){node.record.splice(at,2);tagError='';onChanged()}
  function addNode(){const d=doc();if(!d)return;const list=nodes();const index=Math.max(-1,...list.map(n=>n.index))+1;const b=bounds();pairList(d.root).push(`node_${String(index).padStart(2,'0')}`,[12,[(b.minX+b.maxX)/2,0,(b.minZ+b.maxZ)/2],[]]);selectedNode=`node_${String(index).padStart(2,'0')}`;onChanged()}
  function removeNode(node:NetNodeView){const d=doc();if(!d)return;const root=pairList(d.root);for(const other of netNodes(d).filter(n=>!n.spatial&&n.specialEdge?.some(x=>Number(x)===node.index)).sort((a,b)=>b.pairIndex-a.pairIndex))root.splice(other.pairIndex,2);for(const other of netNodes(d)){if(other.spatial){other.links=other.links.filter(x=>Number(x)!==node.index);other.record[2]=other.links}}for(let i=0;i+1<root.length;i+=2){if(root[i]===node.key){root.splice(i,2);break}}selectedNode='';onChanged()}
  function pointerDown(event:PointerEvent,node:NetNodeView){if(!doc()||isLocked(workspace,doc()!))return;dragIndex=node.index;selectedNode=node.key;(event.currentTarget as SVGCircleElement).setPointerCapture(event.pointerId)}
  function pointerMove(event:PointerEvent){if(dragIndex===null)return;const svg=event.currentTarget as SVGSVGElement;const rect=svg.getBoundingClientRect();const p=world((event.clientX-rect.left)*1000/rect.width,(event.clientY-rect.top)*660/rect.height);const node=spatial().find(n=>n.index===dragIndex);if(node){node.position[0]=p.x;node.position[2]=p.z;node.record[1]=node.position;onChanged()}}
  function pointerUp(){dragIndex=null}
</script>

<div class="editor-shell network-shell">
  <aside class="object-sidebar">
    <div class="side-heading"><div><small>ZRdrAiNet</small><b>Networks & Regions</b></div></div>
    <div class="section-label">Mission layer</div><div class="object-list compact">{#each docs() as item,i (item.id)}<button class:active={i===selectedDoc} onclick={()=>{selectedDoc=i;selectedNode=''}}><span><b>{documentTitle(item)}</b><small>{item.kind} · {item.layer}</small></span><small>{netNodes(item).length}</small></button>{/each}</div>
  </aside>
  <section class="network-workspace">
    {#if doc()}{@const currentDoc=doc()!}{@const currentNode=selected()}
      {#if isLocked(workspace,currentDoc)}<div class="locked-banner">{currentDoc.entryName} is in a locked {currentDoc.layer} Reader.</div>{/if}
      <div class="network-toolbar"><div><small>{currentDoc.kind.toUpperCase()}</small><h2>{documentTitle(currentDoc)}</h2></div><div><span>{spatial().length} spatial nodes</span><button disabled={isLocked(workspace,currentDoc)} onclick={addNode}>Add node</button></div></div>
      <div class="network-layout">
        <div class="viewport-card">
          <svg class="network-viewport" viewBox="0 0 1000 660" role="application" aria-label="AI network and region graph" onpointermove={pointerMove} onpointerup={pointerUp} onpointercancel={pointerUp}>
            <defs><pattern id="minor-grid" width="25" height="25" patternUnits="userSpaceOnUse"><path d="M 25 0 L 0 0 0 25" fill="none" stroke="#1b2a30" stroke-width="1"/></pattern><pattern id="major-grid" width="100" height="100" patternUnits="userSpaceOnUse"><rect width="100" height="100" fill="url(#minor-grid)"/><path d="M 100 0 L 0 0 0 100" fill="none" stroke="#2b3b42" stroke-width="1"/></pattern></defs>
            <rect width="1000" height="660" fill="#091115"/><rect width="1000" height="660" fill="url(#major-grid)"/>
            {#if regionCycle().length}{@const polygon=regionCycle().map(n=>{const p=point(n.position);return`${p.x},${p.y}`}).join(' ')}<polygon points={polygon} fill="#d85e2d22" stroke="#e16a3b" stroke-width="2"/>{/if}
            {#each spatial() as source}{@const a=point(source.position)}{#each source.links.map(Number) as targetIndex}{#if source.index<targetIndex&&edgeTarget(targetIndex)}{@const b=point(edgeTarget(targetIndex)!.position)}<line x1={a.x} y1={a.y} x2={b.x} y2={b.y} class="net-edge"/>{/if}{/each}{/each}
            {#each nodes().filter(n=>n.specialEdge?.length) as special}{@const from=edgeTarget(Number(special.specialEdge![0]))}{@const to=edgeTarget(Number(special.specialEdge![1]))}{#if from&&to}{@const a=point(from.position)}{@const b=point(to.position)}<line x1={a.x} y1={a.y} x2={b.x} y2={b.y} class="special-edge"/><g transform={`translate(${(a.x+b.x)/2} ${(a.y+b.y)/2})`}><rect x="-24" y="-9" width="48" height="18" rx="4" class="edge-badge"/><text text-anchor="middle" dominant-baseline="middle">{tags(special.record,special.tagStart)[0]?.tag??'link'}</text></g>{/if}{/each}
            {#each spatial() as node (node.id)}{@const p=point(node.position)}<g role="button" tabindex="0" aria-label={`Select and move ${node.key}`} class:selected-node={currentNode?.key===node.key} onclick={()=>selectedNode=node.key} onkeydown={e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();selectedNode=node.key}}} onpointerdown={e=>pointerDown(e,node)}><circle cx={p.x} cy={p.y} r="9"/><text x={p.x+13} y={p.y-11}>{node.key}</text></g>{/each}
            <text x="18" y="642" class="axis-label">X → · Z ↓ · future map underlay</text>
          </svg>
        </div>
        <aside class="property-panel">
          {#if currentNode}<fieldset disabled={isLocked(workspace,currentDoc)} class="editor-fields"><div class="property-title"><div><small>NODE</small><h3>{currentNode.key}</h3></div><button class="danger" onclick={()=>removeNode(currentNode)}>Delete</button></div>
            {#if currentNode.spatial}<div class="property-section"><h4>Position</h4>{#each ['X','Y','Z'] as axis,i}<label><span class="field-title">{axis} <small>(record[1][{i}])</small></span><input type="number" step="any" value={scalar(currentNode.position[i])} oninput={e=>setPosition(currentNode,i,e.currentTarget.value)}></label>{/each}</div><div class="property-section"><h4>Connections</h4><label><span class="field-title">Node indices <small>(record[2])</small></span><input value={currentNode.links.join(', ')} onchange={e=>setLinks(currentNode,e.currentTarget.value)}></label></div>{/if}
            <div class="property-section"><h4>Optional tags</h4>{#each tags(currentNode.record,currentNode.tagStart) as tag}<label><span class="field-title">{tag.tag} <small>(record[{tag.at}])</small><button type="button" class="inline-danger" onclick={()=>removeTag(currentNode,tag.at)}>Remove</button></span><textarea class="json-value" onchange={e=>setTag(currentNode,tag.at,e.currentTarget.value)}>{JSON.stringify(tag.value)}</textarea></label>{/each}{#if !tags(currentNode.record,currentNode.tagStart).length}<p>No optional node tags.</p>{/if}<div class="inline-editor"><input placeholder="Tag name, e.g. valve" bind:value={newTag}><button type="button" disabled={!newTag.trim()} onclick={()=>addTag(currentNode)}>Add tag</button></div>{#if tagError}<p class="field-error">{tagError}</p>{/if}</div>
            <details><summary>Raw node</summary><pre>{JSON.stringify(currentNode.record,null,2)}</pre></details>
          </fieldset>{:else}<div class="empty">Select a spatial or special node.</div>{/if}
        </aside>
      </div>
    {:else}<div class="empty-state"><h2>No networks or regions loaded</h2><p>The selected Reader layers contain no `net_*.zrd` or `reg_*.zrd` documents.</p></div>{/if}
  </section>
</div>
