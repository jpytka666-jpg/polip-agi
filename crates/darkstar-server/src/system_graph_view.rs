//! Embedded System Graph web view for Darkstar.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 22:20:00
//! REASON FOR CREATION: Provide a MindMap-like interactive architecture viewer without adding a frontend build system or external runtime dependency.
//! MECHANICS: Serves one self-contained HTML document that requests the authenticated architecture snapshot from /v1/system-graph and renders it as interactive SVG.
//! SYSTEM PART: Darkstar Server / System Graph UI
//! ARCHITECTURE FUNCTION: Human-facing inspector for architecture nodes and relationships; it is read-only and does not execute tools.
//! DEPENDENCIES/LINKS: darkstar-core::system_graph and /v1/system-graph; browser Fetch API and SVG only.
//! TECH STACK: Rust 2024 + embedded HTML/CSS/vanilla JavaScript; selected to keep the headless container small and deterministic.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-system-graph
//! ==========================================

pub const SYSTEM_GRAPH_HTML: &str = r##"<!doctype html>
<html lang="pl">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Darkstar System Graph</title>
<style>
:root{color-scheme:dark;font-family:system-ui,-apple-system,Segoe UI,sans-serif}body{margin:0;background:#050812;color:#e9f2ff;overflow:hidden}header{height:58px;display:flex;gap:8px;align-items:center;padding:0 14px;border-bottom:1px solid #203049;background:#09111f}input,button{background:#0d1829;color:#e9f2ff;border:1px solid #29415f;border-radius:8px;padding:8px 10px}input{min-width:230px}button{cursor:pointer}main{display:grid;grid-template-columns:1fr 320px;height:calc(100vh - 59px)}#canvas{position:relative;overflow:hidden;background:radial-gradient(circle at 50% 40%,#10243a 0,#070c16 45%,#03050a 100%)}svg{width:100%;height:100%;touch-action:none}.edge{stroke:#3b6b92;stroke-width:1.3;opacity:.7}.node{cursor:pointer}.node circle{stroke:#d6ecff;stroke-width:1.5}.node text{fill:#eef7ff;font-size:11px;pointer-events:none}.repo circle{r:19;fill:#1f7a9e}.file circle{r:12;fill:#274f6c}.runtime circle{r:16;fill:#7b5a25}.node.active circle{filter:drop-shadow(0 0 7px #3ca7d9)}.node.selected circle{stroke-width:3;filter:drop-shadow(0 0 10px #fff)}aside{background:#08101d;border-left:1px solid #203049;padding:14px;overflow:auto}h1{font-size:16px;margin:0 0 10px}.muted{color:#7e97af;font-size:12px}pre{white-space:pre-wrap;word-break:break-word;color:#b9cce0;font-size:12px}.pill{display:inline-block;border:1px solid #29415f;border-radius:999px;padding:2px 7px;margin:2px;font-size:11px}.hint{margin-top:14px;color:#7892aa;font-size:11px}
</style></head>
<body>
<header><strong>Darkstar / System Graph</strong><input id="token" type="password" placeholder="DARKSTAR_API_TOKEN"><input id="q" placeholder="szukaj"><button id="load">Wczytaj</button><button id="reset">Reset</button></header>
<main><div id="canvas"><svg id="svg" viewBox="0 0 1200 800"><g id="world"></g></svg></div><aside><h1>Inspektor</h1><div id="status" class="muted">Podaj token i kliknij „Wczytaj”.</div><div id="details"></div><div class="hint">Przeciągaj mapę, używaj kółka do zoomu, klikaj węzły.</div></aside></main>
<script>
const svg=document.getElementById('svg'),world=document.getElementById('world'),status=document.getElementById('status'),details=document.getElementById('details');let data=null,scale=1,tx=0,ty=0,drag=null;
const esc=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
function layout(d){const groups={};d.nodes.forEach(n=>(groups[n.system||'Other']??=[]).push(n));const systems=Object.entries(groups);const gap=systems.length>1?900/(systems.length-1):0,pos={};systems.forEach(([system,nodes],i)=>{const x=150+i*gap;nodes.sort((a,b)=>(a.last_modified_at||'').localeCompare(b.last_modified_at||''));const step=Math.max(48,560/Math.max(nodes.length,1));nodes.forEach((n,j)=>pos[n.id]={x,y:100+j*step});});return pos}
function render(d){world.innerHTML='';const pos=layout(d),byId=Object.fromEntries(d.nodes.map(n=>[n.id,n]));d.edges.forEach(e=>{const a=pos[e.from],b=pos[e.to];if(!a||!b)return;const line=document.createElementNS('http://www.w3.org/2000/svg','line');line.setAttribute('x1',a.x);line.setAttribute('y1',a.y);line.setAttribute('x2',b.x);line.setAttribute('y2',b.y);line.setAttribute('class','edge');world.appendChild(line)});d.nodes.forEach(n=>{const p=pos[n.id],g=document.createElementNS('http://www.w3.org/2000/svg','g');g.setAttribute('class',`node ${n.kind} ${n.status}`);g.setAttribute('transform',`translate(${p.x},${p.y})`);const c=document.createElementNS('http://www.w3.org/2000/svg','circle');g.appendChild(c);const t=document.createElementNS('http://www.w3.org/2000/svg','text');t.setAttribute('x',n.kind==='file'?18:25);t.setAttribute('y',4);t.textContent=n.name;g.appendChild(t);g.onclick=()=>inspect(n,byId);world.appendChild(g)});applySearch();status.textContent=`${d.snapshot_id} · ${d.nodes.length} węzłów · ${d.edges.length} relacji`}
function applySearch(){const q=document.getElementById('q').value.trim().toLowerCase();if(!data)return;[...world.querySelectorAll('.node')].forEach((el,i)=>{const n=data.nodes[i];el.style.display=!q||JSON.stringify(n).toLowerCase().includes(q)?'':'none'})}
function inspect(n,byId){world.querySelectorAll('.selected').forEach(x=>x.classList.remove('selected'));const el=[...world.querySelectorAll('.node')].find(x=>x.querySelector('text')?.textContent===n.name);if(el)el.classList.add('selected');const rel=data.edges.filter(e=>e.from===n.id||e.to===n.id);details.innerHTML=`<div class="pill">${esc(n.kind)}</div><div class="pill">${esc(n.status)}</div><h1>${esc(n.name)}</h1><pre>${esc(JSON.stringify(n,null,2))}</pre><h1>Relacje</h1>${rel.map(e=>{const other=byId[e.from===n.id?e.to:e.from];return `<div class="muted">${esc(e.kind)} → ${esc(other?.name||'?')}</div>`}).join('')}`}
async function load(){const token=document.getElementById('token').value;status.textContent='Ładowanie...';try{const r=await fetch('/v1/system-graph',{headers:{Authorization:`Bearer ${token}`}});if(!r.ok)throw new Error(`HTTP ${r.status}`);data=await r.json();render(data)}catch(e){status.textContent=`Błąd: ${e.message}`}}
document.getElementById('load').onclick=load;document.getElementById('q').oninput=applySearch;document.getElementById('reset').onclick=()=>{scale=1;tx=0;ty=0;world.setAttribute('transform','translate(0 0) scale(1)')};svg.addEventListener('wheel',e=>{e.preventDefault();scale=Math.max(.35,Math.min(4,scale*(e.deltaY<0?1.1:.9)));world.setAttribute('transform',`translate(${tx} ${ty}) scale(${scale})`)},{passive:false});svg.addEventListener('pointerdown',e=>{drag={x:e.clientX,y:e.clientY,tx,ty};svg.setPointerCapture(e.pointerId)});svg.addEventListener('pointermove',e=>{if(!drag)return;tx=drag.tx+(e.clientX-drag.x);ty=drag.ty+(e.clientY-drag.y);world.setAttribute('transform',`translate(${tx} ${ty}) scale(${scale})`)});svg.addEventListener('pointerup',()=>drag=null);
</script></body></html>"##;
