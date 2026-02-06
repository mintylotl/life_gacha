import './style.css'

const API_BASE = 'http://127.0.0.1:3000';

// Simple helper to talk to your Rust server

async function apiAction(path: string, payload: object) {
  const display = document.getElementById('result-display');

  try {
    const response = await fetch(`${API_BASE}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    const data = await response.json();

    if (display) {
      // Logic for pretty-printing results
      if (data.result === "NoTickets") {
        display.innerHTML = `<div class="text-red-400 animate-pulse">⚠️ OUT OF TICKETS</div>`;
      } else {
        display.innerText = data.result;
      }

      if (data.status != undefined) {
        display.innerText = data.status;

      }
    }
  } catch (err) {
    if (display) display.innerText = "Error: Server Offline";
  }
}

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <div class="max-w-md w-full p-8 bg-slate-800 rounded-2xl shadow-2xl border border-slate-700">
    <header class="text-center mb-8">
      <h1 class="text-3xl font-black tracking-tighter text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-cyan-400">
        LIFE GACHA
      </h1>
      <p class="text-slate-400 text-sm mt-1 uppercase tracking-widest">User: axol999</p>
    </header>

    <div id="result-display" class="mb-8 p-4 bg-slate-900 rounded-lg border border-slate-700 font-mono text-cyan-300 min-h-[100px] flex items-center justify-center text-center">
      Waiting for action...
    </div>

    <div class="grid grid-cols-1 gap-4">
      <button id="pull-btn" class="w-full py-4 bg-indigo-600 hover:bg-indigo-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-[0_0_20px_rgba(99,102,241,0.3)] hover:shadow-[0_0_30px_rgba(99,102,241,0.5)]">
        Execute Pull
      </button>

      <button id="timer-btn" class="w-full py-4 bg-emerald-600 hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-lg shadow-emerald-500/20">
        Start Study Timer
      </button>
    </div>
  </div>
`

// Event Listeners
document.querySelector('#pull-btn')?.addEventListener('click', () =>
  apiAction('/pull', { userid: 'axol999' })
);

document.querySelector('#timer-btn')?.addEventListener('click', () =>
  apiAction('/start_timer', { userid: 'axol999', category: 'S_Node' })
);
