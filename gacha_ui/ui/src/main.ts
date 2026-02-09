import "./style.css";

const API_BASE = "http://127.0.0.1:3000";
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
// Simple helper to talk to your Rust server

async function queryUserFunds(path: string, payload: object) {
  let funds_screen = document.getElementById("user-funds-display");

  try {
    let response = await fetch(`${API_BASE}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });

    let data = await response.json();

    if (funds_screen) {
      funds_screen.innerText = `Astrum: ${data.astrum}\nAstrai: ${data.astrai}\nFlux: ${data.flux}`;
    }
  } catch (err) {
    console.log(err);
  }
}

async function apiActionGetVouchers(path: string, payload: object) {
  const display = document.getElementById("result-display");

  try {
    let response = await fetch(`${API_BASE}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });

    let data = await response.json();
  } catch (err) {
    console.log(err);
  }
}

async function apiActionPurchase(path: string, type: string) {
  let payload = { id: 0, userid: "axol999", amount: 0 };
  const display = document.getElementById("result-display");
  payload.amount = 1;

  switch (type) {
    case "off_day":
      payload.id = 1;
      break;

    case "coffee":
      payload.id = 2;
      break;

    case "slip_gacha":
      payload.id = 9;
      break;

    case "gaming":
      payload.id = 10;
      break;

    case "mythic_week":
      payload.id = 999;
      break;

    default:
      console.log("Default");
  }

  try {
    let response = await fetch(`${API_BASE}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });

    let data = await response.json();

    if (display) {
      display.innerHTML = `<div class="text-cyan-400">${data.result}</div>`;
    }
    queryUserFunds("/user_funds_info", { userid: "axol999" });
  } catch (err) {
    console.log(err);
    display!.innerHTML = `<div class="text-cyan-400">Error: Insufficient Funds</div>`;
  }
}
async function apiActionPull10(path: string, payload: object) {
  const display = document.getElementById("result-display");
  const pull_dp = document.getElementById("pull-display");

  let [mythic, s, a, b] = [0, 0, 0, 0];
  try {
    for (let i = 0; i < 10; i++) {
      let response = await fetch(`${API_BASE}${path}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      let data = await response.json();
      if (display) {
        if (data.result === "NoTickets") {
          display.innerHTML = `<div class="bg-red-400 animate-pulse">⚠️ OUT OF TICKETS</div>`;
          break;
        } else if (data.result === "Mythic") {
          display.innerHTML = `<div class="text-red-400 animate-pulse">${data.result}</div>`;
          mythic += 1;
        } else if (data.result === "S") {
          display.innerHTML = `<div class="bg-yellow-600 animate-pulse">S</div>`;
          s += 1;
        } else if (data.result === "A") {
          display.innerHTML = `<div class="bg-purple-400 animate-pulse">A</div>`;
          a += 1;
        } else {
          display.innerHTML = `<div class="text-blue-400 animate-pulse">${data.result}</div>`;
          b += 1;
        }
      }
      if (pull_dp) {
        pull_dp.innerHTML = `
        <div class="from-white to-red-400 bg-gradient-to-r animate-pulse text-transparent bg-clip-text">
          Mythics: ${mythic}</br>
        </div>
        <div class="from-white to-yellow-600 bg-gradient-to-r animate-pulse text-transparent bg-clip-text"> S Ranks: ${s}</div>
         <div class="from-green-300 to-purple-600 bg-gradient-to-r animate-pulse text-transparent bg-clip-text"> A Ranks: ${a}</div>
         <div class="from-blue-400 to-blue-600 bg-gradient-to-r animate-pulse text-transparent bg-clip-text"> B Ranks: ${b}</div>`;
      }
      queryUserFunds("/user_funds_info", { userid: "axol999" });
      await sleep(724);
    }
  } catch (err) {
    if (display) {
      display.innerText = "Error: Server Offline";
    }
  }
}

async function apiActionPull(path: string, payload: object) {
  const display = document.getElementById("result-display");
  try {
    const response = await fetch(`${API_BASE}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    const data = await response.json();

    if (display) {
      // Logic for pretty-printing results
      if (path == "/pull") {
        if (data.result === "NoTickets") {
          display.innerHTML = `<div class="text-red-400 animate-pulse">⚠️ OUT OF TICKETS</div>`;
        } else {
          display.innerHTML = `<div class="animate-pulse">${data.result}</div>`;
        }
      }
    }
    queryUserFunds("/user_funds_info", { userid: "axol999" });
  } catch (err) {
    if (display) display.innerText = "Error: Server Offline";
  }
}

async function apiActionTimer(path: string, payload: object) {
  const display = document.getElementById("result-display");

  try {
    const response = await fetch(`${API_BASE}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    const data = await response.json();

    if (display) {
      // Logic for pretty-printing results
      if (path === "/stop_timer") {
        if (data.status === "Timer Stopped") {
          display.innerHTML = `<div class="text-red-400 animate-pulse">Timer Stopped</div>`;
        } else {
          display.innerHTML = `<div class="animate-pulse">${data.status}</div>`;
        }
      }
      if (path == "/start_timer") {
        display.innerHTML = data.status;
      }
    }
  } catch (err) {
    if (display) display.innerText = "Error: Server Offline";
  }
}
document.querySelector<HTMLDivElement>("#app")!.innerHTML = `
<button id="store"></button>
<div id="full-body" class="w-150">
  <div class="w-full p-8 bg-slate-800 rounded-2xl shadow-2xl border border-slate-700">
    <header class="flex flex-col justify-center mb-8 w-full">
      <div>
        <h1 class="text-3xl font-black tracking-tighter text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-cyan-400">
          LIFE GACHA
        </h1>
        <h1 class="text-3sm font-black tracking-tighter text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-cyan-400">
          LIFE GACHA
        </h1>
      </div>
      <p class="text-slate-400 text-sm mt-1 uppercase tracking-widest">User: axol999</p>
      <p id="user-funds-display" class="text-slate-400 text-sm mt-1 uppercase tracking-widest">Funds: 0</p>
    </header>

    <div id="pull-display" class="mb-8 p-4 bg-slate-900 rounded-lg border border-slate-700 font-mono text-cyan-300 min-h-[100px] flex flex-col gap-1 justify-center text-center">
      Pull to Display...
    </div>
    <div id="result-display" class="mb-8 p-4 bg-slate-900 rounded-lg border border-slate-700 font-mono text-cyan-300 min-h-[100px] flex flex-col-1 items-center justify-center text-center">
      IDLE
    </div>

    <div class="grid grid-cols-2 gap-4">
      <button id="pull-btn" class="w-full py-4 bg-indigo-600 hover:bg-indigo-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-[0_0_20px_rgba(99,102,241,0.3)] hover:shadow-[0_0_30px_rgba(99,102,241,0.5)]">
        Pull
      </button>
      <button id="pull10-btn" class="w-full py-4 bg-indigo-600 hover:bg-indigo-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-[0_0_20px_rgba(99,102,241,0.3)] hover:shadow-[0_0_30px_rgba(99,102,241,0.5)]">
        Pull x10
      </button>
      <button id="timer-btn-stop" class="w-full py-4 bg-red-600 hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-lg shadow-emerald-500/20">
        Stop Timer
      </button>
      <button id="timer-btn" class="w-full py-4 bg-emerald-600 hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-lg shadow-emerald-500/20">
        Timer
      </button>
      <button id="purchase" class="w-full py-4 bg-emerald-600 hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-lg shadow-emerald-500/20">
        Purchase Coffee
      </button>
    </div>
  </div>
</div>
`;

// Event Listeners
document
  .querySelector("#pull-btn")
  ?.addEventListener("click", () =>
    apiActionPull("/pull", { userid: "axol999" }),
  );

document
  .querySelector("#pull10-btn")
  ?.addEventListener("click", () =>
    apiActionPull10("/pull", { userid: "axol999" }),
  );

document
  .querySelector("#timer-btn")
  ?.addEventListener("click", () =>
    apiActionTimer("/start_timer", { userid: "axol999", category: "S_Node" }),
  );

document
  .querySelector("#timer-btn-stop")
  ?.addEventListener("click", () =>
    apiActionTimer("/stop_timer", { userid: "axol999", category: "S_Node" }),
  );

document
  .querySelector("#purchase")
  ?.addEventListener("click", () => apiActionPurchase("/purchase", "coffee"));

document.querySelector("#store")?.addEventListener("click", () =>
  apiActionGetVouchers("/get_user_vouchers", {
    userid: "axol999",
    filter_by_id: false,
  }),
);

async function runTask() {
  while (true) {
    queryUserFunds("/user_funds_info", { userid: "axol999" });
    await sleep(500);
  }
}
runTask();
