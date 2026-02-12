import "./style.css";

const API_BASE = "http://11.0.0.2:3000";
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
// Simple helper to talk to your Rust server

function get_userid() {
  return "axol999";
}
interface ConsumeRequest {
  userid: string;
  uuid: string;
}

(window as any).consume = async function consume(uuid: string) {
  const voucher = document.getElementById(uuid);

  let path = "/consume";
  let payload: ConsumeRequest = { userid: get_userid(), uuid };
  const response = await fetch(`${API_BASE}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  const data = await response.json();

  console.log(data);

  if (voucher) {
    voucher.outerHTML = "";
  }
};

async function goHome(store: boolean) {
  let home = document.getElementById("home-page");
  let stock = document.getElementById("stock-page");

  if (store) {
    stock = document.getElementById("store-page");
  }

  if (home && stock) {
    stock.classList.add("opacity-0");
    setTimeout(() => {
      stock.classList.add("hidden");
      home.classList.remove("hidden");
      setTimeout(() => {
        home.classList.remove("opacity-0");
      }, 50);
    }, 50);
  }
}
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

interface CreateRequest {
  userid: string;
  voucher: Voucher;
}
interface Voucher {
  id: number;
  cost: number;
  name: string;
  description: string;
}

async function apiActionCreateVoucher(path: string) {
  const modal_div = document.getElementById("modaldiv");
  if (modal_div) {
    modal_div.classList.add(
      "hidden",
      "opacity-0",
      "transition-all",
      "duration-500",
    );
    modal_div.innerHTML = modaldivcontent;
    setTimeout(() => {
      modal_div.classList.remove("hidden", "opacity-0");
    }, 50);
  }

  const modal = document.getElementById("create-modal");

  const formElem = document.getElementById(
    "create-voucher-form",
  ) as HTMLFormElement;

  if (modal) {
    formElem.addEventListener("submit", async (e) => {
      e.preventDefault();

      try {
        const formData = new FormData(formElem);
        const voucher_p: Voucher = {
          id: Number(formData.get("id")),
          cost: Number(formData.get("cost")),
          name: formData.get("name") as string,
          description: formData.get("description") as string,
        };
        let payload: CreateRequest = {
          userid: get_userid(),
          voucher: voucher_p,
        };
        const response = await fetch(`${API_BASE}${path}`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });

        if (response.ok) {
          await sleep(750);
          (window as any).closeModal();
        } else {
          switch_svg();
        }
      } catch (err) {
        console.log(err);
        switch_svg();
      }
    });
  }
}
async function switch_svg() {
  const btn = document.getElementById("init-btn");
  let btn_span;
  if (btn) {
    btn_span = btn.querySelector("span");
  }

  if (btn && btn_span) {
    btn_span.innerText = "INVALID INPUT";
    btn.classList.remove("hover:bg-emerald-400", "bg-emerald-600");

    let svg = btn.querySelector("path");
    let d =
      "M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z";
    let d_valid =
      "M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z";
    if (svg) {
      svg.setAttribute("d", d);
    }

    btn.classList.add(
      "animate-shake",
      "duration-100",
      "bg-red-500",
      "transition-all",
    );

    setTimeout(() => {
      svg?.setAttribute("d", d_valid);
      btn.classList.remove("animate-shake", "bg-red-500", "transition-all");

      btn.classList.add("bg-emerald-600", "hover:bg-emerald-400");
      btn_span.innerText = "INITIALIZE VOUCHER";
    }, 1200);
  }
}

(window as any).closeModal = function closeModal() {
  document.getElementById("create-modal")?.remove();
};

async function apiActionGetTemplates(path: string, payload: object) {
  apiActionGetVouchers(path, true, payload);
}

async function apiActionGetVouchers(
  path: string,
  store: boolean,
  payload: object,
) {
  let page_name = "";
  if (store) {
    page_name = "store-page";
  } else {
    page_name = "stock-page";
  }

  const home = document.getElementById("home-page");
  const stock = document.getElementById(page_name);
  const boxes = document.getElementById("store-boxes");

  try {
    let response = await fetch(`${API_BASE}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    let data_v: Voucher[] = await response.json();
    console.log("Here!");
    if (boxes) {
      boxes.innerHTML = "";
      data_v.forEach((voucher) => {
        boxes.innerHTML =
          boxes.innerHTML +
          `
<div id="${voucher.uuid}" class="group relative flex flex-col bg-slate-700/50 backdrop-blur-sm rounded-2xl shadow-lg hover:shadow-emerald-500/10 hover:border-emerald-500/50 transition-all duration-300 h-[400px] border border-slate-600 overflow-hidden">
<div class="h-2.5 w-full bg-emerald-500 shadow-[0_0_15px_rgba(16,185,129,0.3)]"></div>
<div class="p-6 flex-1 flex flex-col">
<div class="flex justify-between items-start mb-4">
<h2 class="text-xl font-bold text-slate-100 tracking-tight leading-tight uppercase group-hover:text-emerald-400 transition-colors">
${voucher.name}
</h2>
<span class="bg-slate-800 text-slate-400 text-[9px] font-mono py-1 px-2 rounded border border-slate-600">
#${voucher.id}
</span>
</div>

<div class="mb-4">
<span class="text-4xl font-black text-emerald-400 font-mono">${voucher.cost}</span>
<span class="text-xs font-bold text-slate-500 ml-1 uppercase tracking-widest">credits</span>
</div>

<p class="text-slate-400 leading-relaxed text-sm flex-1 overflow-hidden italic">
"${voucher.description}"
</p>

<div class="mt-4 pt-4 border-t border-slate-600/50">
<p class="text-[10px] text-slate-500 font-mono truncate opacity-60">
UUID: ${voucher.uuid}
</p>
</div>

<button onclick="consume('${voucher.uuid}')" class="mt-4 w-full bg-emerald-600 text-slate-900 font-black py-3 rounded-xl hover:bg-emerald-400 active:scale-95 transition-all duration-200 flex items-center justify-center gap-2 shadow-lg shadow-emerald-900/20">
<span>REDEEM VOUCHER</span>
<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-8.707l-3-3a1 1 0 00-1.414 1.414L10.586 9H7a1 1 0 100 2h3.586l-1.293 1.293a1 1 0 101.414 1.414l3-3a1 1 0 000-1.414z" clip-rule="evenodd" />
</svg>
</button>
</div>
</div>
`;
        console.log(voucher.name);
      });

      if (home && stock) {
        home.classList.add("opacity-0");
        setTimeout(() => {
          home.classList.add("hidden");
          stock.classList.remove("hidden");
          setTimeout(() => {
            stock.classList.remove("opacity-0");
          }, 50);
        }, 50);
      }
    }
  } catch (err) {
    console.log(err);
  }
}

async function apiActionPurchase(path: string, type: string) {
  let payload = { id: 0, userid: get_userid(), amount: 0 };
  const display = document.getElementById("result-display");
  payload.amount = 1;

  switch (type) {
    case "off_day":
      payload.id = 1;
      break;

    case "coffee":
      payload.id = 3;
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
    queryUserFunds("/user_funds_info", { userid: get_userid() });
  } catch (err) {
    console.log(err);
    display!.innerHTML = `<div class="text-cyan-400">Error: Insufficient Funds</div>`;
  }
}

async function apiActionPull10(path: string) {
  const display = document.getElementById("pull-display");

  let [mythic, s, a, b] = [0, 0, 0, 0];
  try {
    if (display) {
      display.innerText = "";
      for (let i = 0; i < 10; i++) {
        const result = await apiActionPull(path, [25, 576]);
        if (result === "NoTickets") {
          return;
        }
        if (result === "Mythic") {
          mythic += 1;
        } else if (result === "S") {
          s += 1;
        } else if (result === "A") {
          a += 1;
        } else {
          b += 1;
        }

        display.innerHTML = `
        <div class="flex flex-col gap-6 items-center justify-around">
        <div class="from-white to-red-400 bg-gradient-to-r animate-pulse text-transparent bg-clip-text text-sm3">
          Mythics: ${mythic}</br>
        </div>
        <div class="from-white to-yellow-600 bg-gradient-to-r text-transparent animate-pulse bg-clip-text text-sm3"> S Ranks: ${s}</div>
         <div class="from-green-300 to-purple-600 bg-gradient-to-r text-transparent animate-pulse bg-clip-text text-sm3"> A Ranks: ${a}</div>
         <div class="from-blue-400 to-blue-600 bg-gradient-to-r text-transparent animate-pulse bg-clip-text text-sm3"> B Ranks: ${b}</div>
        </div>`;
      }
      queryUserFunds("/user_funds_info", { userid: get_userid() });
    }
  } catch (err) {
    if (display) {
      display.innerText = "Error: Server Offline";
    }
  }
}

async function apiActionPull(path: string, delay: number[]) {
  const display = document.getElementById("pull-display-single");

  let payload = { userid: get_userid() };
  try {
    const response = await fetch(`${API_BASE}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    const data = await response.json();

    const [delay_l, delay_r] = delay;
    if (display) {
      // Logic for pretty-printing results
      if (path == "/pull") {
        if (data.result === "NoTickets") {
          display.innerHTML = `<div class="text-red-400 animate-pulse">⚠️ OUT OF ASTRUM</div>`;
        } else {
          display.innerHTML = `
          <div id="shuffler" class="text-3xl font-black text-white"></div>
          <div id="sss" class="from-white to-red-400 bg-gradient-to-r animate-shake text-transparent bg-clip-text text-3xl font-black"></div>
          <div id="s" class="from-white to-yellow-600 bg-gradient-to-r text-transparent animate-pulse bg-clip-text text-3xl font-black"></div>
          <div id="a" class="from-green-300 to-purple-600 bg-gradient-to-r text-transparent animate-pulse bg-clip-text text-3xl font-black"></div>
          <div id="b" class="from-blue-400 to-blue-600 bg-gradient-to-r text-transparent animate-pulse bg-clip-text text-3xl font-black"></div>`;

          const shuffler = document.getElementById("shuffler")!;

          // 2. The Shuffle Animation
          for (let i = 0; i < 15; i++) {
            // We update the SHUFFLER, not the display container
            if (i % 2 == 0) shuffler.innerText = "A";
            else if (i % 3 == 0) shuffler.innerText = "B";
            else shuffler.innerText = "S";

            await sleep(delay_l);
          }
          shuffler.innerText = "";

          switch (data.result) {
            case "Mythic":
              document.getElementById("sss")!.innerText = "MYTHIC SSS";
              break;

            case "S":
              document.getElementById("s")!.innerText = "S";
              break;

            case "A":
              document.getElementById("a")!.innerText = "A";
              break;

            case "B":
              document.getElementById("b")!.innerText = "B";
              break;

            default:
              console.error("error");
          }

          await sleep(delay_r);
        }
      }
    }
    queryUserFunds("/user_funds_info", { userid: get_userid() });
    return data.result;
  } catch (err) {
    if (display) display.innerText = "Error: Server Offline";
    console.log(err);
  }
}

interface TimerRequest {
  userid: string;
  category: string;
}
async function apiActionTimer(path: string, payload: TimerRequest) {
  const display = document.getElementById("pull-display");
  const button_dp = document.getElementById("timer-btn");

  try {
    if (display) {
      // Logic for pretty-printing results
      if (path === "/stop_timer") {
        const response = await fetch(`${API_BASE}${path}`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });
        const data = await response.json();

        if (data.status === "Timer Stopped") {
          display.innerHTML = `<div class="text-cyan-300 animate-pulse">Earned: ${data.reward}</div>`;
          button_dp!.outerHTML = `
          <button id="timer-btn" class="w-full py-4 bg-emerald-600 hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-lg shadow-emerald-500/20">
            Timer
          </button>`;
          document.querySelector("#timer-btn")?.addEventListener("click", () =>
            apiActionTimer("/start_timer", {
              userid: get_userid(),
              category: "SNode",
            }),
          );
        } else {
          display.innerHTML = `<div class="animate-pulse">${data.status}</div>`;
        }

        queryUserFunds("/user_funds_info", { userid: get_userid() });
      }
      if (path === "/start_timer") {
        button_dp!.outerHTML = `
        <div id="timer-btn" class="flex justify-around items-center gap-1 w-full py-4 bg-slate-700 transition-all rounded-xl font-bold text-lg shadow-lg shadow-slate-500/20">
          <button id="btn-SNode" class="flex justify-center ml-6 w-12 bg-yellow-600 hover:bg-yellow-500 active:scale-95 transition-all rounded-sm font-bold text-lg">S</button>
          <button id="btn-ANode" class="flex justify-center w-12 bg-indigo-600 hover:bg-indigo-500 active:scale-95 transition-all rounded-sm font-bold text-lg">A</button>
          <button id="btn-BNode" class="flex justify-center mr-6 w-12 bg-[#4682B4] hover:bg-[#5A9BD5] active:scale-95 transition-all rounded-sm font-bold text-lg">B</button>
        </div>`;

        document.querySelector("#btn-SNode")?.addEventListener("click", () => {
          apiActionTimer("/timer_node", {
            userid: get_userid(),
            category: "SNode",
          });
        });
        document.querySelector("#btn-ANode")?.addEventListener("click", () => {
          apiActionTimer("/timer_node", {
            userid: get_userid(),
            category: "ANode",
          });
        });
        document.querySelector("#btn-BNode")?.addEventListener("click", () => {
          apiActionTimer("/timer_node", {
            userid: get_userid(),
            category: "BNode",
          });
        });
      }
      if (path === "/timer_node") {
        path = "/start_timer";
        const response = await fetch(`${API_BASE}${path}`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });
        const data = await response.json();

        button_dp!.outerHTML = `
        <button id="timer-btn" class="w-full py-4 bg-emerald-600 hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-lg shadow-emerald-500/20">
          Timing: ${data.category}
        </button>`;
        display.innerText = data.status;

        document.querySelector("#timer-btn")?.addEventListener("click", () =>
          apiActionTimer("/start_timer", {
            userid: get_userid(),
            category: "SNode",
          }),
        );
      }
    }
  } catch (err) {
    if (display) display.innerText = "Error: Server Offline";
  }
}

const modaldivcontent = `
    <div id="create-modal" class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-md">

    <div class="relative flex flex-col bg-slate-700/90 backdrop-blur-xl rounded-2xl shadow-2xl w-full max-w-lg border border-slate-600 overflow-hidden">

        <div class="h-2.5 w-full bg-emerald-500 shadow-[0_0_15px_rgba(16,185,129,0.3)]"></div>

        <div class="p-8 flex flex-col gap-6">
            <div class="flex justify-between items-center">
                <h2 class="text-2xl font-black text-slate-100 uppercase tracking-tighter">Forge New Voucher</h2>
                <button onclick="closeModal()" class="text-slate-400 hover:text-white transition-colors">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>

            <form id="create-voucher-form" class="space-y-4">
                <div>
                    <label class="block text-[10px] font-bold text-emerald-500 uppercase tracking-[0.2em] mb-1.5">Voucher Name</label>
                    <input type="text" name="name" placeholder="e.g. Mythic Core"
                        class="w-full bg-slate-800 border border-slate-600 rounded-xl px-4 py-3 text-slate-100 placeholder:text-slate-500 focus:outline-none focus:border-emerald-500 transition-colors">
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-[10px] font-bold text-emerald-500 uppercase tracking-[0.2em] mb-1.5">Credit Cost</label>
                        <input type="number" name="cost" placeholder="1000"
                            class="w-full bg-slate-800 border border-slate-600 rounded-xl px-4 py-3 text-emerald-400 font-mono focus:outline-none focus:border-emerald-500 transition-colors">
                    </div>
                    <div>
                        <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em] mb-1.5">Registry ID</label>
                        <input type="number" name="id" placeholder="99"
                            class="w-full bg-slate-800 border border-slate-600 rounded-xl px-4 py-3 text-slate-400 font-mono focus:outline-none focus:border-slate-500 transition-colors">
                    </div>
                </div>

                <div>
                    <label class="block text-[10px] font-bold text-emerald-500 uppercase tracking-[0.2em] mb-1.5">Description</label>
                    <textarea name="description" rows="3" placeholder="Define the utility of this artifact..."
                        class="w-full bg-slate-800 border border-slate-600 rounded-xl px-4 py-3 text-slate-300 italic focus:outline-none focus:border-emerald-500 transition-colors resize-none"></textarea>
                </div>

                <button id="init-btn" type="submit" class="text-3sm mt-4 w-full bg-emerald-600 text-slate-900 font-black py-4 rounded-xl hover:bg-emerald-400 active:scale-[0.98] transition-all duration-200 flex justify-center items-center shadow-lg shadow-emerald-900/40 uppercase tracking-wider">
                    <div class="flex justify-center mx-4">
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
                      </svg>
                    </div>

                    <div class="flex justify-center mr-4 w-28">
                      <span id="createForm-btn">Initialize Voucher</span>
                    </div>
                </button>
            </form>
        </div>
    </div>
</div>
`;
document.querySelector<HTMLDivElement>("#app")!.innerHTML = `
<section id="home-page" class="transition-opacity duration-500">
<div class="flex flex-col fixed top-8 right-4 gap-5">
  <button id="store" class="hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-3sm shadow-lg shadow-emerald-500/20 p-8 bg-slate-800 rounded-2xl shadow-2xl border border-slate-700 right-4 w-25 h-10 font-mono font-sm3 flex justify-center items-center">
    STORE
  </button>
  <button id="stock" class="hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-2sm shadow-lg shadow-emerald-500/20 p-8 bg-slate-800 rounded-2xl shadow-2xl border border-slate-700 right-4 w-25 h-10 font-mono font-sm3 flex justify-center items-center">
    INVENTORY
  </button>
</div>
<div id="full-body" class="w-150">
  <div class="w-full p-8 bg-slate-800 rounded-2xl shadow-2xl border border-slate-700">
    <header class="flex flex-col justify-center mb-8 w-full">
      <div>
        <h1 class="text-3xl font-black tracking-tighter text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-cyan-400">
          LIFE GACHA
        </h1>
        <h1 class="text-3sm font-black tracking-tighter text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-cyan-400">
          Axol
        </h1>
      </div>
      <p id="user-funds-display" class="text-slate-400 text-sm mt-1 uppercase tracking-widest">Funds: 0</p>
    </header>

    <div id="result-display" class="m-2 mb-8 p-5 bg-slate-900 rounded-lg border border-slate-700 font-mono text-cyan-300 h-80 grid grid-cols-[70%_30%] items-center justify-around text-center">

    <div id="pull-display" class="m-8 ml-0 p-1 bg-slate-900 rounded-lg border border-slate-700 font-mono text-cyan-300 h-full flex items-center justify-center text-center">
      Pull to Display...
    </div>
    <div id="pull-display-single" class="m-8 p-1 bg-slate-900 rounded-lg font-mono h-full text-cyan-300 flex justify-center items-center text-center">
      IDLE
    </div>

    </div>

    <div class="grid grid-cols-2 gap-4">
      <button id="pull-btn" class="w-full py-4 bg-indigo-600 hover:bg-indigo-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-[0_0_20px_rgba(99,102,241,0.3)] hover:shadow-[0_0_30px_rgba(99,102,241,0.5)]">
        Pull
      </button>
      <button id="pull10-btn" class="w-full py-4 bg-indigo-600 hover:bg-indigo-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-[0_0_20px_rgba(99,102,241,0.3)] hover:shadow-[0_0_30px_rgba(99,102,241,0.5)]">
        Pull x10
      </button>
      <button id="timer-btn-stop" class="w-full py-4 bg-red-600 hover:bg-red-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-lg shadow-emerald-500/20">
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
</section>



<section id="stock-page" class="w-[calc(100vw-600px)] h-screen hidden opacity-0 transition-opacity duration-200">
<div class="flex items-center justify-between mb-8 p-2 border-b border-slate-700">
    <div class="flex items-center gap-3 m-6">
        <div class="w-3 h-8 bg-emerald-500 rounded-full"></div>
        <h1 class="text-2xl font-black text-white tracking-widest font-mono">INVENTORY</h1>
    </div>
<div class="flex flex-col fixed top-8 right-4 gap-5">
  <button id="home" class="hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-3sm shadow-lg shadow-emerald-500/20 p-8 bg-slate-800 rounded-2xl shadow-2xl border border-slate-700 right-4 w-25 h-10 font-mono font-sm3 flex justify-center items-center">
    HOME
  </button>
</div>
    <span class="text-slate-400 font-mono text-xs uppercase tracking-tighter">
        Verified Vouchers Only
    </span>
</div>
  <div id="boxes" class="w-full h-full content-start grid grid-cols-5 gap-6">
  </div>
</section>

<section id="store-page" class="w-[calc(100vw-600px)] h-screen hidden opacity-0 transition-opacity duration-200">
<div class="flex items-center justify-between mb-8 p-2 border-b border-slate-700">
    <div class="flex items-center gap-3 m-6">
        <div class="w-3 h-8 bg-emerald-500 rounded-full"></div>
        <h1 class="text-2xl font-black text-white tracking-widest font-mono">STORE</h1>
    </div>
</div>
<div class="flex flex-col fixed top-8 right-4 gap-5">
      <button id="home-from-store" class="hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-3sm shadow-lg shadow-emerald-500/20 p-8 bg-slate-800 rounded-2xl shadow-2xl border border-slate-700 right-4 w-25 h-10 font-mono font-sm3 flex justify-center items-center">
        HOME
      </button>
      <button id="create" class="mt-4 hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-3sm shadow-lg shadow-emerald-500/20 p-8 bg-slate-800 rounded-2xl shadow-2xl border border-slate-700 right-4 w-25 h-10 font-mono font-sm3 flex justify-center items-center">
        CREATE
      </button>
</div>
  <div id="store-boxes" class="w-full h-full content-start grid grid-cols-5 gap-6">
  </div>
</section>




<div id="modaldiv"></div>
`;

// Event Listeners
document
  .querySelector("#pull-btn")
  ?.addEventListener("click", () => apiActionPull("/pull", [30, 750]));

document
  .querySelector("#pull10-btn")
  ?.addEventListener("click", () => apiActionPull10("/pull"));

document
  .querySelector("#timer-btn")
  ?.addEventListener("click", () =>
    apiActionTimer("/start_timer", { userid: get_userid(), category: "SNode" }),
  );

document
  .querySelector("#timer-btn-stop")
  ?.addEventListener("click", () =>
    apiActionTimer("/stop_timer", { userid: get_userid(), category: "SNode" }),
  );

document
  .querySelector("#purchase")
  ?.addEventListener("click", () => apiActionPurchase("/purchase", "coffee"));

document.querySelector("#stock")?.addEventListener("click", () =>
  apiActionGetVouchers("/get_user_vouchers", false, {
    userid: get_userid(),
    filter_by_id: 0,
    request_all: true,
    store: false,
  }),
);
document.querySelector("#store")?.addEventListener("click", () => {
  apiActionGetTemplates("/get_user_vouchers", {
    userid: get_userid(),
    filter_by_id: 0,
    request_all: true,
    store: true,
  });
});

document.querySelector("#create")?.addEventListener("click", () => {
  apiActionCreateVoucher("/create");
});

document.querySelector("#home")?.addEventListener("click", () => {
  goHome(false);
});
document.querySelector("#home-from-store")?.addEventListener("click", () => {
  goHome(true);
});

async function runTask() {
  while (true) {
    queryUserFunds("/user_funds_info", { userid: get_userid() });
    await sleep(2000);
  }
}
runTask();
