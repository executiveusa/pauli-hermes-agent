/**
 * HERMES — Lightweight Orchestrator for The Pauli Effect
 * -------------------------------------------------------
 * The dispatcher. Receives missions (web, Telegram, API), routes them
 * to the right specialist agent, monitors progress, reports to Bambú.
 *
 * This is a pragmatic orchestrator — it doesn't need the full Hermes
 * codebase to function. It uses the same runtime pattern as the other
 * agents, plus mission routing logic and optional Telegram integration.
 *
 * Runs on port 4800. The full Nous Research Hermes can replace this later.
 */
const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 4800;
const PAULI_ROOT = process.env.PAULI_ROOT || '/opt/pauli-effect';
const MISSIONS_DIR = path.join(PAULI_ROOT, 'missions');
const RUNS_DIR = path.join(PAULI_ROOT, 'runs');
const ADMIN_TOKEN = process.env.ADMIN_TOKEN || 'pauli-effect-bambu-2026';
const TELEGRAM_BOT_TOKEN = process.env.TELEGRAM_BOT_TOKEN || '';
const TELEGRAM_CHAT_ID = process.env.TELEGRAM_CHAT_ID || '';

// Agent routing table
const AGENTS = {
  'cosmos-pi': { name: 'Cosmos', url: 'http://127.0.0.1:4717', role: 'Engineering Lead' },
  'tars': { name: 'TARS', url: 'http://127.0.0.1:4321', role: 'Builder' },
  'cosmos-brain': { name: 'Cosmos-II', url: 'http://127.0.0.1:4719', role: 'Brain Keeper' }
};

const startTime = Date.now();
let logs = [];

function log(msg) {
  const entry = `[${new Date().toISOString()}] ${msg}`;
  logs.push(entry);
  if (logs.length > 200) logs.shift();
  console.log(entry);
}

// ── Mission routing ─────────────────────────────────────────────
function routeMission(missionText) {
  const text = missionText.toLowerCase();
  if (/build|create|make|deploy|launch|website|app|landing|page/.test(text)) return 'tars';
  if (/brain|knowledge|organize|graph|node|edge|library|shelf|document/.test(text)) return 'cosmos-brain';
  if (/code|architect|engineer|debug|fix|review|design|database|api/.test(text)) return 'cosmos-pi';
  return 'cosmos-pi'; // default to engineering lead
}

async function dispatchToAgent(agentSlug, mission) {
  const agent = AGENTS[agentSlug];
  if (!agent) throw new Error(`Unknown agent: ${agentSlug}`);

  try {
    const resp = await fetch(`${agent.url}/mission`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        mission: mission.mission,
        context: mission.context || '',
        priority: mission.priority || 'normal',
        from: 'hermes',
        timestamp: new Date().toISOString()
      })
    });
    const data = await resp.json();
    return data;
  } catch (err) {
    return { error: `Agent ${agent.name} unreachable: ${err.message}` };
  }
}

async function sendTelegram(text) {
  if (!TELEGRAM_BOT_TOKEN || !TELEGRAM_CHAT_ID) return;
  try {
    await fetch(`https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendMessage`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ chat_id: TELEGRAM_CHAT_ID, text, parse_mode: 'Markdown' })
    });
  } catch (err) {
    log(`Telegram error: ${err.message}`);
  }
}

function saveMission(mission, routing, result) {
  const id = `mission-${Date.now()}`;
  const record = {
    id,
    mission: mission.mission,
    context: mission.context,
    priority: mission.priority,
    routed_to: routing.agent,
    agent_name: AGENTS[routing.agent]?.name,
    result,
    created_at: new Date().toISOString()
  };

  // Save to missions/
  fs.mkdirSync(MISSIONS_DIR, { recursive: true });
  fs.writeFileSync(
    path.join(MISSIONS_DIR, `${id}.json`),
    JSON.stringify(record, null, 2)
  );

  // Also save to runs/
  fs.mkdirSync(RUNS_DIR, { recursive: true });
  fs.writeFileSync(
    path.join(RUNS_DIR, `${id}.json`),
    JSON.stringify(record, null, 2)
  );

  return record;
}

// ── HTTP Server ─────────────────────────────────────────────────
const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://localhost:${PORT}`);

  function json(code, data) {
    res.writeHead(code, { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' });
    res.end(JSON.stringify(data, null, 2));
  }

  // ── Health ──
  if (url.pathname === '/health') {
    json(200, { status: 'ok', agent: 'Hermes', role: 'Orchestrator', uptime: Math.floor((Date.now() - startTime) / 1000) });
    return;
  }

  // ── Dispatch a mission (POST /dispatch) ──
  if (url.pathname === '/dispatch' && req.method === 'POST') {
    let body = '';
    req.on('data', c => body += c);
    req.on('end', async () => {
      try {
        const mission = JSON.parse(body);

        // Auth check (unless from dashboard with token)
        const auth = req.headers.authorization || '';
        const token = auth.replace('Bearer ', '');
        if (token !== ADMIN_TOKEN && !mission.skip_auth) {
          json(401, { error: 'Unauthorized' });
          return;
        }

        // Route the mission
        const agentSlug = mission.route_to || routeMission(mission.mission);
        log(`Mission received: "${mission.mission.slice(0, 60)}" → routing to ${AGENTS[agentSlug]?.name || agentSlug}`);

        // Dispatch
        const result = await dispatchToAgent(agentSlug, mission);
        const record = saveMission(mission, { agent: agentSlug }, result);

        // Report to Telegram
        const status = result.error ? 'FAILED' : 'COMPLETED';
        await sendTelegram(`*Hermes Report*\n\nMission: ${mission.mission.slice(0, 100)}\nRouted to: ${AGENTS[agentSlug]?.name}\nStatus: ${status}${result.response ? '\n\nResponse:\n' + result.response.slice(0, 500) : ''}`);

        log(`Mission ${status}: ${record.id}`);
        json(200, { ...record, routed_to: agentSlug });
      } catch (err) {
        log(`Dispatch error: ${err.message}`);
        json(400, { error: err.message });
      }
    });
    return;
  }

  // ── Get all missions ──
  if (url.pathname === '/missions') {
    try {
      const files = fs.readdirSync(MISSIONS_DIR).filter(f => f.endsWith('.json')).sort().reverse().slice(0, 20);
      const missions = files.map(f => JSON.parse(fs.readFileSync(path.join(MISSIONS_DIR, f), 'utf-8')));
      json(200, missions);
    } catch { json(200, []); }
    return;
  }

  // ── Web UI ──
  if (url.pathname === '/' || url.pathname === '/index.html') {
    res.writeHead(200, { 'Content-Type': 'text/html' });
    res.end(hermesUI());
    return;
  }

  json(404, { error: 'Not found' });
});

function hermesUI() {
  return `<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Hermes — Orchestrator | The Pauli Effect</title>
<style>
:root{--bg:#fff;--surface:#f7f7f8;--ink:#0a0a0a;--muted:#6e6e73;--border:rgba(0,0,0,.1);--gold:#F5A617;--green:#16a34a}
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',system-ui,sans-serif;background:var(--bg);color:var(--ink);min-height:100vh}
header{padding:20px;border-bottom:1px solid var(--border);display:flex;align-items:center;gap:12px;max-width:800px;margin:0 auto}
.icon{width:44px;height:44px;border-radius:12px;background:var(--ink);color:var(--gold);display:flex;align-items:center;justify-content:center;font-weight:700;font-size:1.4rem}
h1{font-size:1.1rem} .role{font-size:.8rem;color:var(--muted)}
.status{margin-left:auto;display:flex;align-items:center;gap:6px;font-size:.75rem;color:var(--green)}
.dot{width:8px;height:8px;border-radius:50%;background:var(--green);animation:pulse 2s infinite}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:.5}}
main{max-width:800px;margin:0 auto;padding:20px}
.section-title{font-size:.7rem;text-transform:uppercase;letter-spacing:.06em;color:var(--muted);margin-bottom:12px;margin-top:24px}
.dispatch-box{background:var(--surface);border:1px solid var(--border);border-radius:12px;padding:16px;margin-bottom:16px}
textarea{width:100%;padding:12px;border:1px solid var(--border);border-radius:8px;font-family:inherit;font-size:.9rem;resize:vertical;outline:none;margin-bottom:8px}
textarea:focus{border-color:var(--ink)}
select{padding:8px 12px;border:1px solid var(--border);border-radius:8px;font-family:inherit;font-size:.85rem;background:#fff;margin-right:8px}
button{background:var(--ink);color:#fff;border:none;border-radius:8px;padding:10px 20px;cursor:pointer;font-size:.85rem}
button:hover{opacity:.85}
button:disabled{opacity:.4}
.result{background:#0a0a0a;color:#e0e0e0;border-radius:8px;padding:14px;font-family:monospace;font-size:.8rem;margin-top:12px;white-space:pre-wrap;display:none}
.result.show{display:block}
.agent-status{display:flex;gap:12px;flex-wrap:wrap}
.agent-chip{background:var(--surface);border:1px solid var(--border);border-radius:8px;padding:8px 14px;font-size:.8rem;display:flex;align-items:center;gap:6px}
.agent-dot{width:6px;height:6px;border-radius:50%;background:var(--green)}
.agent-dot.off{background:#ccc}
</style></head><body>
<header><div class="icon">H</div><div><h1>Hermes</h1><div class="role">Orchestrator · The Pauli Effect</div></div><div class="status"><span class="dot"></span> Online</div></header>
<main>
<div class="section-title">Agent Fleet Status</div>
<div class="agent-status" id="fleet"></div>
<div class="section-title">Dispatch a Mission</div>
<div class="dispatch-box">
<textarea id="mission" rows="3" placeholder="Describe the mission... e.g. 'Build a landing page for a Seattle coffee shop'"></textarea>
<div style="display:flex;gap:8px;align-items:center">
<select id="route"><option value="">Auto-route</option><option value="tars">TARS (Builder)</option><option value="cosmos-pi">Cosmos (Engineering)</option><option value="cosmos-brain">Cosmos-II (Brain)</option></select>
<button id="dispatch-btn" onclick="dispatch()">Dispatch Mission</button>
</div>
<div class="result" id="result"></div>
</div>
<div class="section-title">Recent Missions</div>
<div id="history" style="font-size:.85rem"></div>
</main>
<script>
async function loadFleet(){
try{
var r=await fetch('/health');var d=await r.json();
document.getElementById('fleet').innerHTML=
'<div class="agent-chip"><span class="agent-dot"></span>Hermes (self)</div>'+
[{'name':'TARS','port':4321,'slug':'tars'},{'name':'Cosmos','port':4717,'slug':'cosmos-pi'},{'name':'Cosmos-II','port':4719,'slug':'cosmos-brain'}].map(async function(a){
var ar=await fetch('http://31.220.58.212:'+a.port+'/health');var ad=await ar.json();
return '<div class="agent-chip"><span class="agent-dot'+(ad.status==='ok'?'':' off')+'"></span>'+a.name+'</div>';
}).join('');
}catch(e){}
}
async function loadHistory(){
try{
var r=await fetch('/missions');var d=await r.json();
document.getElementById('history').innerHTML=d.length?d.map(function(m){return '<div style="padding:8px 0;border-bottom:1px solid var(--border)"><strong>'+m.routed_to+'</strong> — '+m.mission.slice(0,80)+(m.mission.length>80?'...':'')+'<br><span style="color:var(--muted);font-size:.75rem">'+new Date(m.created_at).toLocaleString()+'</span></div>'}).join(''):'<div style="color:var(--muted);padding:16px">No missions dispatched yet.</div>';
}catch(e){}
}
async function dispatch(){
var m=document.getElementById('mission').value.trim();if(!m)return;
var route=document.getElementById('route').value;
var btn=document.getElementById('dispatch-btn');btn.disabled=true;btn.textContent='Dispatching...';
document.getElementById('result').classList.add('show');document.getElementById('result').textContent='Routing mission...';
try{
var r=await fetch('/dispatch',{method:'POST',headers:{'Content-Type':'application/json','Authorization':'Bearer ${ADMIN_TOKEN}'},body:JSON.stringify({mission:m,route_to:route||undefined})});
var d=await r.json();
document.getElementById('result').textContent=JSON.stringify(d,null,2);
document.getElementById('mission').value='';
loadHistory();
}catch(e){document.getElementById('result').textContent='Error: '+e.message;}
btn.disabled=false;btn.textContent='Dispatch Mission';
}
loadFleet();loadHistory();
</script>
</body></html>`;
}

server.listen(PORT, '0.0.0.0', () => {
  log('Hermes orchestrator running on port ' + PORT);
  log('Web UI: http://localhost:' + PORT);
  log('Telegram: ' + (TELEGRAM_BOT_TOKEN ? 'configured' : 'not configured'));
});
