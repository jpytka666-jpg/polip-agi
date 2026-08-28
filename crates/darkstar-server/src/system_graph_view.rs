//! Embedded live System Graph web view for Darkstar.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 22:50:00
//! REASON FOR CREATION: Provide an n8n-like browser view where a human can inspect the architecture and watch a real Darkstar run move through its nodes.
//! MECHANICS: Serves one self-contained HTML document. The viewer loads the authenticated architecture snapshot, starts an explicit demo run, then reads its authenticated SSE stream with Fetch and highlights nodes as execution events arrive.
//! SYSTEM PART: Darkstar Server / System Graph UI
//! ARCHITECTURE FUNCTION: Human-facing execution inspector; read-only with respect to tools and security policy.
//! DEPENDENCIES/LINKS: darkstar-core::system_graph, /v1/system-graph, /v1/runs/start, /v1/runs/{run_id}/events; browser Fetch API and SVG.
//! TECH STACK: Rust 2024 + embedded HTML/CSS/vanilla JavaScript; selected to keep the headless container small and avoid a second frontend build system at this stage.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-system-graph
//! ==========================================

pub const SYSTEM_GRAPH_HTML: &str = r##"<!doctype html>
<html lang="pl">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Darkstar Control Room</title>
<style>
:root{color-scheme:dark;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}*{box-sizing:border-box}body{margin:0;background:#05070c;color:#edf4ff;overflow:hidden}header{height:68px;display:grid;grid-template-columns:auto auto 1fr auto auto auto;gap:10px;align-items:center;padding:0 18px;border-bottom:1px solid #1b2a3d;background:linear-gradient(180deg,#0d1522 0%,#09111b 100%);box-shadow:0 10px 28px #0008}header strong{font-size:16px;letter-spacing:.08em;text-transform:uppercase;color:#f4f8ff}header .mode{padding:7px 11px;border:1px solid #28405a;border-radius:999px;background:#0b1726;color:#9fc2de;font-size:11px;font-weight:700}.connection{display:flex;align-items:center;gap:7px;font-size:11px;color:#7690a9}.dot{width:8px;height:8px;border-radius:50%;background:#55d68b;box-shadow:0 0 10px #55d68b}.search{width:100%;max-width:390px;justify-self:center}input,button{font:inherit;background:#0d1827;color:#edf4ff;border:1px solid #29415c;border-radius:10px;padding:9px 12px}input::placeholder{color:#668099}input:focus,button:focus{outline:2px solid #4fc3ff55;outline-offset:1px}button{cursor:pointer;transition:transform .15s ease,background .15s ease,border-color .15s ease,box-shadow .15s ease}button:hover:not(:disabled){background:#132338;border-color:#3a5f7d;box-shadow:0 0 18px #2aa9ff14;transform:translateY(-1px)}button.primary{background:linear-gradient(135deg,#1677b8,#0f5f99);border-color:#3798d0;font-weight:700}.danger{border-color:#713d49}.danger:hover:not(:disabled){background:#31161f;border-color:#a85d6e}.success-btn{border-color:#2d735a}.success-btn:hover:not(:disabled){background:#102a20;border-color:#48a77f}button:disabled{opacity:.4;cursor:default}main{display:grid;grid-template-columns:minmax(0,1fr) 360px;grid-template-rows:minmax(0,1fr) 168px;height:calc(100vh - 68px)}#canvas{position:relative;overflow:hidden;background:radial-gradient(circle at 48% 42%,#12253a 0%,#0a121d 38%,#05070c 78%);border-right:1px solid #172638}#canvas:before{content:"";position:absolute;inset:0;background-image:linear-gradient(#ffffff06 1px,transparent 1px),linear-gradient(90deg,#ffffff06 1px,transparent 1px);background-size:32px 32px;pointer-events:none}svg{position:relative;z-index:1;width:100%;height:100%;touch-action:none}.edge{stroke:#35556c;stroke-width:1.5;opacity:.65}.edge.live{stroke:#64dbff;stroke-width:4;filter:drop-shadow(0 0 9px #45cfff);opacity:1}.node{cursor:pointer}.node circle{stroke:#d8eaff;stroke-width:1.5;fill:#1e4056}.node.repository circle{fill:#1d6f91}.node.runtime circle{fill:#705321}.node.file circle{fill:#284b61}.node.running circle{fill:#a47822;filter:drop-shadow(0 0 14px #f5c65d)}.node.success circle,.node.allow circle{fill:#2f7e5d;filter:drop-shadow(0 0 12px #62e3a5)}.node.selected circle{stroke:#ffffff;stroke-width:3;filter:drop-shadow(0 0 14px #ffffff55)}.node text{fill:#eef6ff;font-size:11px;font-weight:600;paint-order:stroke;stroke:#05070c;stroke-width:3px;pointer-events:none}aside{grid-column:2;grid-row:1;background:linear-gradient(180deg,#09111c,#07101a);border-left:1px solid #1a293b;padding:16px;overflow:auto}aside h1{font-size:11px;letter-spacing:.14em;text-transform:uppercase;color:#7290aa;margin:0 0 10px}.panel{background:#0b1624;border:1px solid #1c3147;border-radius:14px;padding:13px;margin-bottom:12px;box-shadow:0 10px 25px #0003}.runbar{font-size:11px;color:#a5bdd2;padding:0 0 10px;word-break:break-word}.muted{color:#7f99b1;font-size:12px}.state-line{display:flex;justify-content:space-between;gap:8px;align-items:center;font-size:12px}.status-chip{display:inline-flex;align-items:center;gap:6px;border:1px solid #28435e;border-radius:999px;padding:4px 8px;font-size:11px;font-weight:700;text-transform:uppercase}.status-chip.ready{border-color:#34785f;color:#71e6ad}.status-chip.running{border-color:#8e6a28;color:#ffd36f}.status-chip.blocked{border-color:#7d4051;color:#ff9db0}.status-chip.failed{border-color:#8f4d42;color:#ffae99}.inspector-grid{display:grid;grid-template-columns:1fr 1fr;gap:8px}.metric{padding:8px;border:1px solid #1d3348;border-radius:9px;background:#091522}.metric b{display:block;font-size:10px;color:#66839c;text-transform:uppercase;letter-spacing:.08em;margin-bottom:3px}.metric span{font-size:12px;color:#d9e8f6}.pill{display:inline-block;border:1px solid #29415f;border-radius:999px;padding:3px 8px;margin:2px;font-size:10px;color:#9fb8cd;background:#0a1522}.actions{display:grid;grid-template-columns:1fr 1fr;gap:8px}.actions button:last-child{grid-column:1/-1}.log{margin:5px 0;padding:8px 9px;border:1px solid #182c40;border-left:3px solid #2f6f8e;border-radius:8px;background:#091522;font-size:11px}.log b{color:#dbeeff}.legend{font-size:10px;color:#6c879e;line-height:1.5}pre{white-space:pre-wrap;word-break:break-word;color:#bfd0df;font-size:11px;line-height:1.45}#timeline{grid-column:1/3;grid-row:2;background:#07101a;border-top:1px solid #1a2a3d;padding:12px 16px;overflow:auto}.timeline-head{display:flex;align-items:center;justify-content:space-between;margin-bottom:8px}.timeline-title{font-size:11px;letter-spacing:.14em;text-transform:uppercase;color:#7895ad}.timeline-list{display:flex;gap:10px;overflow:auto;padding-bottom:4px}.timeline-item{min-width:190px;padding:9px 10px;background:#0b1724;border:1px solid #193048;border-radius:10px}.timeline-item b{display:block;font-size:11px;color:#dcecff}.timeline-item span{display:block;font-size:10px;color:#7692aa;margin-top:3px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
</style></head>
<body>
<header><strong>Darkstar</strong><span class="mode">CONTROL ROOM</span><span id="viewMode" class="mode">System Graph</span><div class="connection"><span class="dot"></span><span id="connection">READY</span></div><input class="search" id="q" placeholder="Search nodes, providers, repositories..."><input id="token" type="password" placeholder="API token"><button id="load" class="primary">Load Graph</button></header>
<main><div id="canvas"><div style="position:absolute;top:14px;left:16px;z-index:2"><span class="mode">Live Run Graph</span></div><svg id="svg" viewBox="0 0 1200 800"><g id="world"></g></svg></div><aside><div class="panel"><div class="runbar" id="run">NO ACTIVE RUN</div><div class="state-line"><span class="muted">CONTROL STATUS</span><span id="statusChip" class="status-chip">DISCONNECTED</span></div><div id="status" class="muted" style="margin-top:8px">Authenticate to load the system graph.</div></div><div class="panel"><h1>Inspector</h1><div id="details"><div class="muted">Select a node to inspect it.</div></div></div><div class="panel"><h1>Actions</h1><div class="actions"><button id="start" class="success-btn" disabled>Start</button><button id="stop" class="danger" disabled>Stop</button><button id="restart" disabled>Restart</button><button id="open" disabled>Open Resource</button></div></div><div class="panel"><h1>Live Events</h1><div id="log"></div><div class="legend">Drag = pan · Wheel = zoom · Click node = inspect · Live edges show the current execution path.</div></div></aside><section id="timeline"><div class="timeline-head"><span class="timeline-title">Live Timeline</span><span class="muted" id="timelineState">Waiting for events</span></div><div class="timeline-list" id="timelineList"></div></section></main>
<script>
const svg=document.getElementById('svg');
const world=document.getElementById('world');
const status=document.getElementById('status');
const statusChip=document.getElementById('statusChip');
const details=document.getElementById('details');
const log=document.getElementById('log');
const timelineList=document.getElementById('timelineList');
const timelineState=document.getElementById('timelineState');
const runLabel=document.getElementById('run');
const tokenEl=document.getElementById('token');
const connection=document.getElementById('connection');
const startBtn=document.getElementById('start');
const stopBtn=document.getElementById('stop');
const restartBtn=document.getElementById('restart');
const openBtn=document.getElementById('open');

let data=null;
let scale=1;
let tx=0;
let ty=0;
let drag=null;
let currentRun=null;
let liveNode=null;
let selectedNode=null;
let selectedById=null;
let sessionId=null;
let sessionToken=null;
let sessionCapabilities=[];

const esc=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));

function stateClass(value){
    const state=String(value||'').toLowerCase();
    if(state.includes('run'))return 'running';
    if(state.includes('block'))return 'blocked';
    if(state.includes('fail'))return 'failed';
    if(state.includes('success')||state.includes('allow'))return 'ready';
    return 'ready';
}

function layout(d){
    const groups={};
    d.nodes.forEach(n=>(groups[n.system||'Other']??=[]).push(n));
    const systems=Object.entries(groups);
    const pos={};
    const gap=systems.length>1?Math.max(360,920/(systems.length-1)):0;

    systems.forEach(([system,nodes],i)=>{
        const x=170+i*gap;
        nodes.forEach((n,j)=>{
            pos[n.id]={x,y:115+j*118};
        });
    });

    return pos;
}

function nodeCard(n,p){
    const g=document.createElementNS('http://www.w3.org/2000/svg','g');
    g.setAttribute('class',`node-card ${n.kind||'other'} ${stateClass(n.status)}`);
    g.dataset.id=n.id;
    g.setAttribute('transform',`translate(${p.x-125},${p.y-42})`);

    const rect=document.createElementNS('http://www.w3.org/2000/svg','rect');
    rect.setAttribute('x','0');
    rect.setAttribute('y','0');
    rect.setAttribute('width','250');
    rect.setAttribute('height','84');
    rect.setAttribute('rx','14');
    rect.setAttribute('class','node-surface');
    g.appendChild(rect);

    const accent=document.createElementNS('http://www.w3.org/2000/svg','rect');
    accent.setAttribute('x','0');
    accent.setAttribute('y','0');
    accent.setAttribute('width','6');
    accent.setAttribute('height','84');
    accent.setAttribute('rx','3');
    accent.setAttribute('class','node-accent');
    g.appendChild(accent);

    const title=document.createElementNS('http://www.w3.org/2000/svg','text');
    title.setAttribute('x','20');
    title.setAttribute('y','28');
    title.setAttribute('class','node-title');
    title.textContent=n.name;
    g.appendChild(title);

    const type=document.createElementNS('http://www.w3.org/2000/svg','text');
    type.setAttribute('x','20');
    type.setAttribute('y','47');
    type.setAttribute('class','node-meta');
    type.textContent=String(n.kind||'module').toUpperCase();
    g.appendChild(type);

    const state=document.createElementNS('http://www.w3.org/2000/svg','text');
    state.setAttribute('x','20');
    state.setAttribute('y','66');
    state.setAttribute('class','node-state');
    state.textContent=`STATE · ${n.status||'unknown'}`;
    g.appendChild(state);

    if(n.health){
        const health=document.createElementNS('http://www.w3.org/2000/svg','text');
        health.setAttribute('x','184');
        health.setAttribute('y','47');
        health.setAttribute('class','node-health');
        health.textContent=`HEALTH · ${n.health}`;
        g.appendChild(health);
    }

    g.onclick=()=>inspect(n);
    return g;
}

function render(d){
    world.innerHTML='';
    const pos=layout(d);
    selectedById=Object.fromEntries(d.nodes.map(n=>[n.id,n]));

    d.edges.forEach(e=>{
        const a=pos[e.from],b=pos[e.to];
        if(!a||!b)return;

        const line=document.createElementNS('http://www.w3.org/2000/svg','line');
        line.setAttribute('x1',a.x+125);
        line.setAttribute('y1',a.y);
        line.setAttribute('x2',b.x-125);
        line.setAttribute('y2',b.y);
        line.setAttribute('class','edge');
        line.dataset.edge=`${e.from}->${e.to}`;
        world.appendChild(line);
    });

    d.nodes.forEach(n=>{
        world.appendChild(nodeCard(n,pos[n.id]));
    });

    applySearch();
    status.textContent=`${d.snapshot_id} · ${d.nodes.length} nodes · ${d.edges.length} relations`;
}

function applySearch(){
    const q=document.getElementById('q').value.trim().toLowerCase();

    world.querySelectorAll('.node-card').forEach(el=>{
        const n=data?.nodes.find(x=>x.id===el.dataset.id);
        el.style.display=!q||JSON.stringify(n).toLowerCase().includes(q)?'':'none';
    });
}

function setStatusChip(label,kind=''){
    statusChip.textContent=label;
    statusChip.className=`status-chip ${kind}`;
}

function inspect(n){
    selectedNode=n;
    world.querySelectorAll('.selected').forEach(x=>x.classList.remove('selected'));
    world.querySelector(`[data-id="${CSS.escape(n.id)}"]`)?.classList.add('selected');

    const rel=(data?.edges||[]).filter(e=>e.from===n.id||e.to===n.id);
    const capabilities=n.capabilities||n.allowed_capabilities||[];
    const resource=n.related_resource||n.resource||n.url||n.path||null;

    details.innerHTML=`
        <div class="state-line">
            <span class="muted">${esc(n.kind||'module')}</span>
            <span class="status-chip ${stateClass(n.status)}">${esc(n.status||'unknown')}</span>
        </div>
        <h1>${esc(n.name)}</h1>
        <div class="inspector-grid">
            <div class="metric"><b>Identity</b><span>${esc(n.id)}</span></div>
            <div class="metric"><b>Version</b><span>${esc(n.version||'—')}</span></div>
            <div class="metric"><b>Health</b><span>${esc(n.health||'—')}</span></div>
            <div class="metric"><b>System</b><span>${esc(n.system||'—')}</span></div>
        </div>
        <h1 style="margin-top:14px">Capabilities</h1>
        <div>${capabilities.length?capabilities.map(x=>`<span class="pill">${esc(x)}</span>`).join(''):'<span class="muted">None reported</span>'}</div>
        <h1 style="margin-top:14px">Dependencies</h1>
        <div>${(n.dependencies||[]).length?(n.dependencies||[]).map(x=>`<div class="muted">${esc(x)}</div>`).join(''):'<span class="muted">None reported</span>'}</div>
        <h1 style="margin-top:14px">Relations</h1>
        <div>${rel.length?rel.map(e=>{
            const other=selectedById?.[e.from===n.id?e.to:e.from];
            return `<div class="muted">${esc(e.kind)} → ${esc(other?.name||'?')}</div>`;
        }).join(''):'<span class="muted">None</span>'}</div>
        <h1 style="margin-top:14px">Resource</h1>
        <div class="muted">${esc(resource||'No related resource')}</div>
    `;

    refreshActions();
}

function refreshActions(){
    const caps=new Set(sessionCapabilities);
    const hasSelection=!!selectedNode;
    const state=String(selectedNode?.status||'').toLowerCase();

    startBtn.disabled=!hasSelection||!caps.has('module.start')||!(state.includes('offline')||state.includes('failed'));
    stopBtn.disabled=!hasSelection||!caps.has('module.stop')||!(state.includes('ready')||state.includes('running'));
    restartBtn.disabled=!hasSelection||!caps.has('module.restart')||!state.includes('running');

    const resource=selectedNode?.related_resource||selectedNode?.resource||selectedNode?.url||selectedNode?.path;
    openBtn.disabled=!resource;
}

async function ensureSession(token){
    if(sessionId&&sessionToken===token)return sessionId;

    const response=await fetch('/v1/sessions',{
        method:'POST',
        headers:{
            Authorization:`Bearer ${token}`,
            'Content-Type':'application/json'
        },
        body:JSON.stringify({
            principal_id:'operator',
            principal_kind:'human',
            owner_id:'control-room',
            source:'browser-control-room',
            capabilities:['module.start','module.stop','module.restart']
        })
    });

    if(!response.ok)throw new Error(`Session HTTP ${response.status}`);

    const json=await response.json();
    sessionId=json.session.session_id;
    sessionToken=token;
    sessionCapabilities=json.session.capabilities||[];
    refreshActions();
    return sessionId;
}

function addTimeline(title,nodeId,state,message,requestId=null){
    const item=document.createElement('div');
    item.className='timeline-item';
    item.innerHTML=`
        <b>${esc(title)}</b>
        <span>${esc(nodeId||'—')} · ${esc(state||'—')}</span>
        <span>${esc(message||'')}</span>
        ${requestId?`<span>${esc(requestId)}</span>`:''}
    `;
    timelineList.prepend(item);
    timelineState.textContent=`${timelineList.children.length} events`;
}

async function loadGraph(){
    const token=tokenEl.value.trim();
    if(!token){
        status.textContent='API token required.';
        setStatusChip('DISCONNECTED','blocked');
        return;
    }

    try{
        connection.textContent='CONNECTING';
        setStatusChip('CONNECTING','running');

        await ensureSession(token);

        const response=await fetch('/v1/system-graph',{
            headers:{Authorization:`Bearer ${token}`}
        });

        if(!response.ok)throw new Error(`Graph HTTP ${response.status}`);

        data=await response.json();
        render(data);

        connection.textContent='CONNECTED';
        setStatusChip('CONNECTED','ready');
        status.textContent='System graph loaded.';
        inspect(data.nodes[0]);
    }catch(error){
        connection.textContent='ERROR';
        setStatusChip('ERROR','failed');
        status.textContent=`Error: ${error.message}`;
    }
}

async function moduleAction(command){
    const token=tokenEl.value.trim();
    if(!token||!sessionId||!selectedNode)return;

    const button=command==='start'?startBtn:command==='stop'?stopBtn:restartBtn;
    const previousText=button.textContent;
    button.disabled=true;
    button.textContent='Sending…';

    try{
        const response=await fetch(`/v1/modules/${encodeURIComponent(selectedNode.id)}/actions`,{
            method:'POST',
            headers:{
                Authorization:`Bearer ${token}`,
                'Content-Type':'application/json'
            },
            body:JSON.stringify({
                session_id:sessionId,
                command,
                reason:`Control Room operator requested ${command}`
            })
        });

        const json=await response.json();

        if(!response.ok){
            throw new Error(json.error||`HTTP ${response.status}`);
        }

        currentRun={id:json.request_id,previousNode:selectedNode.id};
        runLabel.textContent=`REQUEST ${json.request_id}`;
        status.textContent=`${command.toUpperCase()} authorized`;
        setStatusChip('AUTHORIZED','ready');

        addTimeline(
            `${command.toUpperCase()} · AUTHORIZED`,
            json.module_id,
            json.status,
            json.reason,
            json.request_id
        );
    }catch(error){
        status.textContent=`Action failed: ${error.message}`;
        setStatusChip('ACTION ERROR','failed');
        addTimeline(`${command.toUpperCase()} · ERROR`,selectedNode.id,'failed',error.message);
    }finally{
        button.textContent=previousText;
        refreshActions();
    }
}

async function startDemoRun(){
    const token=tokenEl.value.trim();
    if(!token)return;

    if(!data)await loadGraph();
    if(!data)return;

    const runId=crypto.randomUUID();
    currentRun={id:runId,previousNode:null};
    runLabel.textContent=`RUN ${runId}`;
    timelineState.textContent='Streaming';

    const streamPromise=streamRun(runId,token);

    try{
        const response=await fetch('/v1/runs/start',{
            method:'POST',
            headers:{
                Authorization:`Bearer ${token}`,
                'Content-Type':'application/json'
            },
            body:JSON.stringify({run_id:runId})
        });

        if(!response.ok)throw new Error(`HTTP ${response.status}`);
        await streamPromise;
    }catch(error){
        status.textContent=`Run error: ${error.message}`;
        addTimeline('RUN · ERROR','run','failed',error.message);
    }
}

async function streamRun(runId,token){
    const response=await fetch(`/v1/runs/${runId}/events`,{
        headers:{Authorization:`Bearer ${token}`}
    });

    if(!response.ok)throw new Error(`SSE HTTP ${response.status}`);

    const reader=response.body.getReader();
    const decoder=new TextDecoder();
    let buffer='';

    while(true){
        const {value,done}=await reader.read();
        if(done)break;

        buffer+=decoder.decode(value,{stream:true});
        const chunks=buffer.split('\n\n');
        buffer=chunks.pop()||'';

        for(const chunk of chunks){
            const line=chunk.split('\n').find(x=>x.startsWith('data: '));
            if(!line)continue;

            const event=JSON.parse(line.slice(6));
            handleEvent(event);
        }
    }
}

function handleEvent(event){
    setLiveNode(event.node_id,event.status);
    addTimeline(
        `${String(event.event_type||'EVENT').toUpperCase()} · ${String(event.status||'')}`,
        event.node_id,
        event.status,
        event.message,
        event.run_id
    );

    status.textContent=`${event.event_type} · ${event.status}`;
    timelineState.textContent=`RUN ${event.run_id}`;

    if(event.status==='success'||event.status==='completed'){
        runLabel.textContent=`RUN ${event.run_id} · COMPLETE`;
        setStatusChip('COMPLETE','ready');
    }
}

function setLiveNode(nodeId,statusValue){
    if(liveNode){
        liveNode.classList.remove('running','ready','selected');
    }

    liveNode=world.querySelector(`[data-id="${CSS.escape(nodeId)}"]`);
    if(!liveNode)return;

    const liveClass=stateClass(statusValue);
    liveNode.classList.add(liveClass,'selected');

    world.querySelectorAll('.edge.live').forEach(x=>x.classList.remove('live'));

    if(data&&currentRun){
        const prior=data.edges.find(e=>{
            return e.from===currentRun.previousNode&&e.to===nodeId;
        });

        if(prior){
            const edge=world.querySelector(`[data-edge="${CSS.escape(`${prior.from}->${prior.to}`)}"]`);
            edge?.classList.add('live');
        }
    }

    if(currentRun)currentRun.previousNode=nodeId;
}

function fitView(){
    if(!data)return;

    const nodes=[...world.querySelectorAll('.node-card')].filter(x=>x.style.display!=='none');
    if(!nodes.length)return;

    scale=1;
    tx=0;
    ty=0;
    world.setAttribute('transform','translate(0 0) scale(1)');
}

function resetView(){
    scale=1;
    tx=0;
    ty=0;
    currentRun=null;
    liveNode=null;
    selectedNode=null;

    world.setAttribute('transform','translate(0 0) scale(1)');
    world.querySelectorAll('.selected,.running,.ready,.allow').forEach(x=>{
        x.classList.remove('selected','running','ready','allow');
    });
    world.querySelectorAll('.edge.live').forEach(x=>x.classList.remove('live'));

    details.innerHTML='<div class="muted">Select a node to inspect it.</div>';
    log.innerHTML='';
    timelineList.innerHTML='';
    runLabel.textContent='NO ACTIVE RUN';
    timelineState.textContent='Waiting for events';
    status.textContent='View reset.';
    setStatusChip('CONNECTED','ready');
    refreshActions();
}

document.getElementById('load').onclick=loadGraph;
startBtn.onclick=()=>moduleAction('start');
stopBtn.onclick=()=>moduleAction('stop');
restartBtn.onclick=()=>moduleAction('restart');
openBtn.onclick=()=>{
    const resource=selectedNode?.related_resource||selectedNode?.resource||selectedNode?.url||selectedNode?.path;
    if(resource)window.open(resource,'_blank','noopener,noreferrer');
};
document.getElementById('q').oninput=applySearch;
document.getElementById('reset').onclick=resetView;

const fitButton=document.createElement('button');
fitButton.textContent='Fit';
fitButton.onclick=fitView;
document.querySelector('header').appendChild(fitButton);

const demoButton=document.createElement('button');
demoButton.textContent='Observe Run';
demoButton.onclick=startDemoRun;
document.querySelector('header').appendChild(demoButton);

svg.addEventListener('wheel',e=>{
    e.preventDefault();
    scale=Math.max(.35,Math.min(4,scale*(e.deltaY<0?1.1:.9)));
    world.setAttribute('transform',`translate(${tx} ${ty}) scale(${scale})`);
},{passive:false});

svg.addEventListener('pointerdown',e=>{
    drag={x:e.clientX,y:e.clientY,tx,ty};
    svg.setPointerCapture(e.pointerId);
});

svg.addEventListener('pointermove',e=>{
    if(!drag)return;
    tx=drag.tx+(e.clientX-drag.x);
    ty=drag.ty+(e.clientY-drag.y);
    world.setAttribute('transform',`translate(${tx} ${ty}) scale(${scale})`);
});

svg.addEventListener('pointerup',()=>drag=null);
</script></body></html>"##;

#[cfg(test)]
mod tests {
    use super::SYSTEM_GRAPH_HTML;

    #[test]
    fn control_room_contains_operator_ui_anchors() {
        for anchor in [
            "Darkstar Control Room",
            "System Graph",
            "Live Run Graph",
            "Inspector",
            "Live Timeline",
            "Start",
            "Stop",
            "Restart",
        ] {
            assert!(
                SYSTEM_GRAPH_HTML.contains(anchor),
                "missing Control Room UI anchor: {anchor}"
            );
        }
    }
}
