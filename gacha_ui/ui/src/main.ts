import "./style.css";

const API_BASE = "http://11.0.0.2:3000";
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
// Simple helper to talk to your Rust server

function get_userid() {
  return "axol999";
}

interface Daily {
  id: number;
  claimable: boolean;
  claimed: boolean;
  last_claimed: number;
}

// Logic to update the button UI
function updateDailyButton(buttonEl: HTMLButtonElement, status: string) {
  if (status === "ready") {
    buttonEl.disabled = false;
    buttonEl.innerText = "Claim";
    buttonEl.className =
      "claim-btn px-6 py-2 rounded-lg font-black uppercase tracking-widest text-sm transition-all bg-emerald-600 active:scale-95 text-slate-900 hover:bg-emerald-400 shadow-lg shadow-emerald-900/20";
  } else if (status === "claimed") {
    buttonEl.disabled = true;
    buttonEl.innerText = "Claimed";
    buttonEl.className =
      "claim-btn px-6 py-2 rounded-lg font-black uppercase tracking-widest text-sm bg-slate-900 text-slate-600 border border-slate-800 cursor-not-allowed";
  } else {
    // Locked state
    buttonEl.disabled = true;
    buttonEl.innerText = "Claim";
    buttonEl.className =
      "claim-btn px-6 py-2 rounded-lg font-black uppercase tracking-widest text-sm bg-slate-700 text-slate-500 cursor-not-allowed";
  }
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
async function queryUserFunds() {
  let funds_screen = document.getElementById("user-funds-display");
  let path = "/user_funds_info";
  let payload = { userid: get_userid() };
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
  voucher: ReqVoucher;
}
interface ReqVoucher {
  id: number;
  cost: number;
  name: string;
  description: string;
}

interface Voucher {
  id: number;
  uuid: string;
  cost: number;
  name: string;
  new: boolean;
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
        const voucher_p: ReqVoucher = {
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
  let funcName = "";
  if (store) {
    page_name = "store";
    funcName = "purchase";
  } else {
    page_name = "stock";
    funcName = "consume";
  }

  const home = document.getElementById("home-page");
  const stock = document.getElementById(`${page_name}-page`);
  const boxes = document.getElementById(`${page_name}-boxes`);

  let vouchers_obsv: Voucher[] = [];

  const observer = new IntersectionObserver((entries, observerInstance) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        // TRIGGER YOUR LOGIC HERE
        flip_new(entry.target.id as string);

        // STOP WATCHING
        observerInstance.unobserve(entry.target);
        // or observerInstance.disconnect(); to stop watching EVERYTHING
      }
    });
  });

  try {
    let response = await fetch(`${API_BASE}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    let data_v: Voucher[] = await response.json();
    data_v.sort((a, b) => a.id - b.id);

    if (boxes) {
      boxes.innerHTML = "";
      data_v.forEach((voucher) => {
        let buttonPrefix;
        if (store) {
          buttonPrefix = `onclick="${funcName}(${voucher.id})"`;
        } else {
          buttonPrefix = `onclick="${funcName}('${voucher.uuid}')"`;
        }

        let is_new = `
          <div class="absolute top-2 -right-1 z-10 select-none">
              <span class="absolute inset-0 text-amber-500 blur-sm opacity-60 animate-pulse">NEW</span>

              <span class="relative text-[11px] font-black uppercase tracking-widest text-amber-300 drop-shadow-[0_0_5px_rgba(251,191,36,0.8)] italic">
                  NEW
              </span>
          </div>`;

        if (store || !voucher.new) {
          is_new = "";
        }

        const voucherHTML = `
            <div class="relative" class="voucher-box">

            ${is_new}

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

            <button ${buttonPrefix} class="mt-4 w-full bg-emerald-600 text-slate-900 font-black py-3 rounded-xl hover:bg-emerald-400 active:scale-95 transition-all duration-200 flex items-center justify-center gap-2 shadow-lg shadow-emerald-900/20">
            <span>REDEEM VOUCHER</span>
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-8.707l-3-3a1 1 0 00-1.414 1.414L10.586 9H7a1 1 0 100 2h3.586l-1.293 1.293a1 1 0 101.414 1.414l3-3a1 1 0 000-1.414z" clip-rule="evenodd" />
            </svg>
            </button>
            </div>
            </div>
            </div>
          `;

        boxes.insertAdjacentHTML("beforeend", voucherHTML);
        if (voucher.new) {
          vouchers_obsv.push(voucher);
        }
      });

      if (home && stock) {
        home.classList.add("opacity-0");
        setTimeout(() => {
          home.classList.add("hidden");
          stock.classList.remove("hidden");
          setTimeout(() => {
            stock.classList.remove("opacity-0");
          }, 50);

          vouchers_obsv.forEach((voucher) => {
            observer.observe(document.getElementById(voucher.uuid)!);
            console.log(voucher.name);
            console.log("here");
          });
        }, 50);
      }
    }
  } catch (err) {
    console.log(err);
  }
}

async function flip_new(uuid: string) {
  let payload = uuid;
  try {
    let response = await fetch(`${API_BASE}/remove_new_logo`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
  } catch (err) {
    console.error(err);
  }
}

(window as any).purchase = async function purchase(id: number) {
  let payload = { id, userid: get_userid(), amount: 0 };
  const display = document.getElementById("pull-display");
  payload.amount = 1;

  try {
    let response = await fetch(`${API_BASE}/purchase`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });

    let data = await response.json();

    if (display) {
      display.innerHTML = `<div class="text-cyan-400">${data.result}</div>`;
    }
    queryUserFunds();
  } catch (err) {
    console.log(err);
    display!.innerHTML = `<div class="text-cyan-400">Error: Insufficient Funds</div>`;
  }
};

async function apiActionPull10(path: string) {
  const display = document.getElementById("pull-display");
  const button = document.getElementById("pull10-btn") as HTMLButtonElement;

  if (button) {
    button.outerHTML = `
    <button id="pull10-btn" disabled class="w-full py-4 bg-slate-700/50 cursor-not-allowed rounded-xl font-bold text-lg text-slate-500 border border-slate-600/30">
      Pull x10
    </button>`;
  }

  let [mythic, s, a, b] = [0, 0, 0, 0];
  let flux = 0;
  try {
    if (display) {
      display.innerHTML = `
        <div class="flex flex-col gap-6 items-center justify-around">
        <div id="s1" class="from-white to-red-400 bg-gradient-to-r text-transparent bg-clip-text text-3sm">
          Mythics: 0</br>
        </div>
        <div id="s2" class="from-white to-yellow-600 bg-gradient-to-r text-transparent bg-clip-text text-3sm"> S Ranks: 0</div>
         <div id="a1" class="from-green-300 to-purple-600 bg-gradient-to-r text-transparent bg-clip-text text-3sm"> A Ranks: 0</div>
         <div id="b1" class="from-blue-400 to-blue-600 bg-gradient-to-r text-transparent bg-clip-text text-3sm"> B Ranks: 0</div>
        </div>`;

      for (let i = 0; i < 10; i++) {
        const result = await apiActionPull(path, [25, 576]);
        if (result === "NoTickets") {
          const button = document.getElementById(
            "pull10-btn",
          ) as HTMLButtonElement;
          button.outerHTML = `
            <button id="pull10-btn" class="w-full py-4 bg-indigo-600 hover:bg-indigo-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-[0_0_20px_rgba(99,102,241,0.3)] hover:shadow-[0_0_30px_rgba(99,102,241,0.5)]">
              Pull x10
            </button>

          `;
          return;
        }
        if (result === "Mythic") {
          mythic += 1;
          flux += 2400;

          display.innerHTML = `
          <div class="flex flex-col gap-6 items-center justify-around">

          <div class="from-white to-red-500 bg-gradient-to-r text-transparent bg-clip-text text-3xl animate-shake">
            MYTHIC
          </div>
          </div>`;
          await sleep(8000);
        } else if (result === "S") {
          s += 1;
          flux += 360;

          display.innerHTML = `
          <div class="flex flex-col gap-6 items-center justify-around">

          <div class="from-white to-yellow-600 bg-gradient-to-r text-transparent bg-clip-text text-3xl animate-shake">
            S
          </div>
          </div>`;
          await sleep(2000);
        } else if (result === "A") {
          a += 1;
          flux += 120;

          document.getElementById("a1")!.innerHTML = `
          <div class="flex flex-col gap-6 items-center justify-around">

          <div class="from-green-300 to-purple-600 bg-gradient-to-r text-transparent bg-clip-text text-3sm animate-pulse">
            A Ranks: ${a}</br>
          </div>
          </div>`;
          await sleep(500);
        } else {
          b += 1;
          flux += 10;

          document.getElementById("b1")!.innerHTML = `
          <div class="flex flex-col gap-6 items-center justify-around">

          <div class="from-blue-300 to-blue-600 bg-gradient-to-r text-transparent bg-clip-text text-3sm animate-pulse">
            B Ranks: ${b}</br>
          </div>
          </div>`;
          await sleep(100);
        }

        display.innerHTML = `
        <div class="flex flex-col gap-6 items-center justify-around">
        <div id="s1" class="from-white to-red-400 bg-gradient-to-r text-transparent bg-clip-text text-3sm">
          Mythics: ${mythic}
        </div>
        <div id="s2" class="from-white to-yellow-600 bg-gradient-to-r text-transparent bg-clip-text text-3sm"> S Ranks: ${s}</div>
         <div id="a1" class="from-green-300 to-purple-600 bg-gradient-to-r text-transparent bg-clip-text text-3sm"> A Ranks: ${a}</div>
         <div id="b1" class="from-blue-400 to-blue-600 bg-gradient-to-r text-transparent bg-clip-text text-3sm"> B Ranks: ${b}</div>
        </div>`;
        await sleep(300);
      }
      queryUserFunds();
    }

    setTimeout(async () => {
      const button = document.getElementById("pull10-btn") as HTMLButtonElement;
      button.outerHTML = `
        <button id="pull10-btn" class="w-full py-4 bg-indigo-600 hover:bg-indigo-500 active:scale-95 transition-all rounded-xl font-bold text-lg shadow-[0_0_20px_rgba(99,102,241,0.3)] hover:shadow-[0_0_30px_rgba(99,102,241,0.5)]">
          Pull x10
        </button>
      `;
      document
        .querySelector("#pull10-btn")
        ?.addEventListener("click", () => apiActionPull10("/pull"));

      let modal: RewardManager = new RewardManager([
        { reward_type: "Flux", amount: flux },
      ]);
    }, 500);
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
    queryUserFunds();
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

        queryUserFunds();
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

// MAIN CONTENT
document.querySelector<HTMLDivElement>("#app")!.innerHTML = `
<section id="home-page" class="transition-opacity duration-500">
<div class="flex flex-col fixed md:top-8 top-18 right-6 md:right-4 gap-5">
  <button id="store" class="hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-3sm shadow-lg shadow-emerald-500/20 p-8 bg-slate-800 rounded-2xl shadow-2xl border border-slate-700 right-4 w-25 h-10 font-mono font-sm3 flex justify-center items-center">
    STORE
  </button>
  <button id="stock" class="hover:bg-emerald-500 active:scale-95 transition-all rounded-xl font-bold text-2sm shadow-lg shadow-emerald-500/20 p-8 bg-slate-800 rounded-2xl shadow-2xl border border-slate-700 right-4 w-25 h-10 font-mono font-sm3 flex justify-center items-center">
    INVENTORY
  </button>
</div>
<div id="full-body" class="w-96 md:w-148">
  <div class="w-full p-6 bg-slate-800 rounded-2xl shadow-2xl border border-slate-700">
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
    </div>


  </div>

    <div id="dailies-hud" class="mt-2 w-full bg-slate-900/40 backdrop-blur-sm border border-slate-800 p-4 rounded-2xl hover:bg-slate-800/60 transition-all cursor-pointer group" onclick="openDailies()">
        <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-3">
                <div class="relative">
                    <div class="absolute inset-0 bg-emerald-500/20 blur-lg rounded-full animate-pulse"></div>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-emerald-400 relative" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                    </svg>
                </div>
                <div>
                    <h3 class="text-sm font-black text-slate-100 uppercase tracking-widest group-hover:text-emerald-400 transition-colors">Daily Progress</h3>
                    <p class="text-[10px] text-slate-500 font-bold uppercase">Reset in <span class="text-slate-400 font-mono">14:22:05</span></p>
                </div>
            </div>

            <div class="text-right">
                <span class="text-xl font-black text-emerald-400">0<span class="text-slate-600">/4</span></span>
            </div>
        </div>

        <div class="flex gap-2 h-1.5 w-full">
            <div class="flex-1 bg-slate-800 rounded-full"></div>
            <div class="flex-1 bg-slate-800 rounded-full"></div>
            <div class="flex-1 bg-slate-800 rounded-full"></div>
            <div class="flex-1 bg-slate-800 rounded-full"></div>
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
  <div id="stock-boxes" class="w-full h-full content-start grid grid-cols-5 gap-6">
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



<div id="dailies-modal" class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 backdrop-blur-md p-4 hidden">
    <div class="w-full max-w-2xl bg-slate-900 border border-slate-800 rounded-2xl shadow-2xl overflow-hidden">

    <div class="p-6 border-b border-slate-800 flex justify-between items-center bg-slate-900/50">
        <div>
            <h2 class="text-2xl font-black text-emerald-400 uppercase tracking-tighter">Daily Operations</h2>
            <p class="text-slate-400 text-sm italic">Status: <span id="sync-status" class="text-emerald-500/80">Synchronized</span></p>
        </div>

        <div class="flex items-center gap-3">
            <button id="refresh-dailies" class="p-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-emerald-400 transition-all active:scale-90 border border-slate-700" title="Refresh Status">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
            </button>

            <button onclick="closeDailies()" class="p-2 text-slate-500 hover:text-white transition-colors">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
            </button>
        </div>
    </div>

        <div class="p-4 space-y-3">
            <div class="daily-row flex items-center justify-between p-4 bg-slate-800/50 rounded-xl border border-slate-700/50 group">
                <div class="max-w-[70%]">
                    <h3 class="font-bold text-slate-100 italic">Login</h3>
                    <p class="text-xs text-slate-400 leading-relaxed">Establish connection to the Astrai network.</p>
                </div>
                <button id="dailyA" class="claim-btn px-6 py-2 rounded-lg font-black uppercase tracking-widest text-sm transition-all bg-emerald-600 text-slate-900 hover:bg-emerald-400 active:scale-95">Claim</button>
            </div>

            <div class="daily-row flex items-center justify-between p-4 bg-slate-800/50 rounded-xl border border-slate-700/50">
                <div class="max-w-[70%]">
                    <h3 class="font-bold text-slate-100 italic">Complete Preflight</h3>
                    <p class="text-xs text-slate-400 leading-relaxed">Shower, dishes, maintenance, teeth. Clear mind and let go.</p>
                </div>
                <button disabled id="dailyB" class="claim-btn px-6 py-2 rounded-lg font-black uppercase tracking-widest text-sm transition-all bg-slate-700 text-slate-500 cursor-not-allowed">Claim</button>
            </div>

            <div class="daily-row flex items-center justify-between p-4 bg-slate-800/50 rounded-xl border border-slate-700/50">
                <div class="max-w-[70%]">
                    <h3 class="font-bold text-slate-100 italic">Blood & Hormones</h3>
                    <p class="text-xs text-slate-400 leading-relaxed">20 burpees of 7 sets or until failure. Pump the system.</p>
                </div>
                <button disabled id="dailyC" class="claim-btn px-6 py-2 rounded-lg font-black uppercase tracking-widest text-sm transition-all bg-slate-700 text-slate-500 cursor-not-allowed">Claim</button>
            </div>

            <div class="daily-row flex items-center justify-between p-4 bg-slate-800/50 rounded-xl border border-slate-700/50">
                <div class="max-w-[70%]">
                    <h3 class="font-bold text-slate-100 italic">Flux Expenditure</h3>
                    <p class="text-xs text-slate-400 leading-relaxed">Channel 500 Flux through the extraction conduits.</p>
                </div>
                <button disabled id="dailyD" class="claim-btn px-6 py-2 rounded-lg font-black uppercase tracking-widest text-sm transition-all bg-slate-700 text-slate-500 cursor-not-allowed">Claim</button>
            </div>
        </div>

        <div class="m-4 p-4 bg-emerald-950/20 border border-emerald-500/20 rounded-xl flex items-center justify-between">
            <div>
                <span class="text-[10px] text-emerald-500 font-black uppercase tracking-[0.2em]">Efficiency Bonus</span>
                <p class="text-xs text-slate-300">Complete all daily tasks for extra rewards.</p>
            </div>
            <div class="flex gap-1">
                <div class="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_8px_#10b981]"></div>
                <div class="w-2 h-2 rounded-full bg-slate-700"></div>
                <div class="w-2 h-2 rounded-full bg-slate-700"></div>
                <div class="w-2 h-2 rounded-full bg-slate-700"></div>
            </div>
        </div>
    </div>
</div>

<div id="reward-modal-ph"></div>
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

class RewardModal {
  private el: HTMLElement;
  public id: string;
  public isOpen: boolean;

  constructor(id: string, element: HTMLElement) {
    this.isOpen = false;
    this.id = id;

    if (element) {
      this.el = element;
      this.isOpen = true;
      console.log(`Initialized modal ${this.el}`);
    }
  }

  public getEl(): HTMLElement {
    return this.el;
  }
  public destroy(): void {
    if (this.el) {
      (window as any).closeRewardModal(this.id);

      this.el.remove();
      this.isOpen = false;
      console.log(`Destroyed modal`);
    }
  }
}

function generateRandomString(length: number = 12): string {
  const characters =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz012456789";
  let result = "";
  for (let i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * characters.length));
  }
  return result;
}

async function makeRewardModal(): Promise<RewardModal> {
  const randomId = generateRandomString(12);
  let modal = `
<div id="reward-modal-${randomId}" class="fixed inset-0 z-[100] flex items-center justify-center transition-opacity duration-300 bg-slate-900/90 backdrop-blur-md hidden">
    <div class="relative w-full max-w-sm p-8 mx-4 text-center transform scale-95 transition-transform duration-300 bg-slate-800 border-2 border-emerald-500/50 rounded-3xl shadow-[0_0_50px_rgba(16,185,129,0.2)]" id="reward-container-${randomId}">

        <div class="absolute inset-x-0 top-0 -translate-y-1/2 flex justify-center">
            <div class="bg-emerald-500 p-4 rounded-2xl shadow-lg shadow-emerald-500/40 animate-bounce">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-slate-900" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-7.714 2.143L11 21l-2.286-6.857L1 12l7.714-2.143L11 3z" />
                </svg>
            </div>
        </div>

        <h2 class="mt-8 text-3xl font-black text-white tracking-tighter uppercase">Reward Claimed!</h2>
        <p class="mt-2 text-slate-400 font-medium">You have successfully received:</p>

        <div id="reward-details-${randomId}" class="my-6 p-4 bg-slate-900/50 rounded-2xl border border-slate-700/50">
            <span id="reward-value-${randomId}" class="text-4xl font-mono font-black text-emerald-400"></span>
            <span id="reward-type-${randomId}" class="block text-xs font-bold text-slate-500 uppercase tracking-widest mt-1"></span>
        </div>

        <button onclick="closeRewardModal('${randomId}')" class="w-full py-4 bg-emerald-600 hover:bg-emerald-500 text-slate-900 font-black rounded-xl active:scale-95 transition-all shadow-lg shadow-emerald-900/40">
            CONFIRM & CONTINUE
        </button>
    </div>
</div> `;

  let ele = document.getElementById("reward-modal-ph");
  if (ele) {
    ele.innerHTML = modal;
    console.log("Inserted stuff");
  }

  await sleep(50);

  return new RewardModal(
    randomId,
    document.getElementById(`reward-modal-${randomId}`)!,
  );
}

async function prepareDailies() {
  let dailiesArr = ["dailyA", "dailyB", "dailyC", "dailyD"];

  dailiesArr.forEach(async (id, index) => {
    let button_elem = document.getElementById(id);

    // Re-check the DOM every 3 seconds if not found
    while (!button_elem) {
      console.log(`Searching for daily${index}...`);
      await sleep(1000);
      button_elem = document.getElementById(id);
    }

    button_elem.addEventListener("click", async () => {
      let payload = {
        id: index,
        info: false,
        userid: get_userid(),
      };
      try {
        const button_daily = document.getElementById(id) as HTMLButtonElement;

        let response = await fetch(`${API_BASE}/dailies`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });

        if (response.ok && button_daily) {
          updateDailyButton(button_daily, "claimed");

          updateHUD();
        } else {
          console.error("Error while claiming daily!");
        }
      } catch (err) {
        console.error(err);
      }
    });
  });
}

interface Reward {
  reward_type: "Astrum" | "Flux" | "Astrai" | "Voucher" | string;
  amount: string | number;
}

async function runTask() {
  while (true) {
    queryUserFunds();
    await sleep(15000);
  }
}
runTask();

// REFRESH LOGIC for DAILIES
const refreshBtn = document.getElementById("refresh-dailies")!;
const refreshIcon = refreshBtn.querySelector("svg")!;
const syncStatus = document.getElementById("sync-status")!;

refreshBtn.addEventListener("click", async () => {
  // 1. Start the Spin
  refreshIcon.classList.add("animate-spin", "text-emerald-400");
  syncStatus.innerText = "Fetching...";
  syncStatus.classList.replace("text-emerald-500/80", "text-amber-500");

  await sleep(550);
  try {
    // 2. Call your Rust backend (Example)
    // await apiFetchDailies();
    let payload = { id: 255, info: true, userid: get_userid() };
    let response = await fetch(`${API_BASE}/dailies`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    const response_j = await response.json();
    const dailies: Daily[] = response_j.dailies;

    updateHUD();

    let inc = 0;
    let dailiesArr = ["dailyA", "dailyB", "dailyC", "dailyD"];
    dailies.forEach((daily) => {
      inc++;
      const button = document.getElementById(
        dailiesArr[inc - 1],
      )! as HTMLButtonElement;

      if (daily.claimable && !daily.claimed) {
        updateDailyButton(button, "ready");
      } else if (!daily.claimable && daily.claimed) {
        updateDailyButton(button, "claimed");
      } else if (!daily.claimable && !daily.claimed) {
        updateDailyButton(button, "locked");
      }
    });

    // 3. Success state
    syncStatus.innerText = "Synchronized";
    syncStatus.classList.replace("text-amber-500", "text-emerald-500/80");
  } catch (e) {
    syncStatus.innerText = "Sync Failed";
    syncStatus.classList.replace("text-amber-500", "text-red-500");
  } finally {
    // 4. Stop the Spin
    refreshIcon.classList.remove("animate-spin");
  }
});

(window as any).closeDailies = function closeDailies() {
  document.getElementById("dailies-modal")!.classList.add("hidden");
};

(window as any).openDailies = function openDailies() {
  document.getElementById("dailies-modal")!.classList.remove("hidden");
  setTimeout(async () => {
    refreshBtn.click();
    await sleep(600);
    prepareDailies();
  }, 50);
};

async function updateHUD() {
  let payload = {
    id: 255,
    info: true,
    userid: get_userid(),
  };
  const dailies_f = await fetch(`${API_BASE}/dailies`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });

  const dailies_json = await dailies_f.json();
  const dailies: Daily[] = dailies_json.dailies;

  const completedCount = dailies.filter((d) => d.claimed).length;

  // Update the text counter (e.g., 2/4)
  const counter = document.querySelector(".text-xl.font-black");
  if (counter)
    counter.innerHTML = `${completedCount}<span class="text-slate-600">/4</span>`;

  // Update the visual "pips"
  const pips = document.querySelectorAll(".flex.gap-2.h-1\\.5 > div");
  pips.forEach((pip, index) => {
    if (index < completedCount) {
      pip.className =
        "flex-1 bg-emerald-500 shadow-[0_0_8px_#10b981] rounded-full";
    } else {
      pip.className = "flex-1 bg-slate-800 rounded-full";
    }
  });
}
setTimeout(() => {
  updateHUD();
}, 100);

function showReward(id: string, reward_type: string, amount: string) {
  const modal = document.getElementById(`reward-modal-${id}`);
  const container = document.getElementById(`reward-container-${id}`);
  const valueEl = document.getElementById(`reward-value-${id}`);
  const typeEl = document.getElementById(`reward-type-${id}`);

  // Set content
  if (!(modal && container && valueEl && typeEl)) {
    return;
  }

  valueEl.innerText = amount;
  typeEl.innerText = reward_type;

  // Show Modal with Animation
  modal.classList.remove("hidden");
  setTimeout(() => {
    modal.classList.remove("opacity-0");
    container.classList.remove("scale-95");
    container.classList.add("scale-100");
  }, 10);
}

(window as any).closeRewardModal = function closeRewardModal(id: string) {
  const modal = document.getElementById(`reward-modal-${id}`);
  const container = document.getElementById(`reward-container-${id}`);

  if (!(modal && container)) {
    return;
  }

  modal.classList.add("opacity-0");
  container.classList.remove("scale-100");
  container.classList.add("scale-95");

  setTimeout(() => {
    modal.classList.add("hidden");
  }, 300);
};

// Assuming this interface exists in your codebase
interface Reward {
  reward_type: string;
  amount: number | string;
}

class RewardManager {
  private rewards: Reward[];
  private id: string;
  private modalEl: HTMLElement | null = null;

  constructor(rewards: Reward[]) {
    this.rewards = rewards;
    this.id = generateRandomString(6);
    this.render();
  }

  // Updated to return an object to style the new multi-layered elements
  private getStyles(type: string): {
    container: string;
    amount: string;
    typeText: string;
  } {
    const themes: Record<
      string,
      { container: string; amount: string; typeText: string }
    > = {
      Astrum: {
        container: "bg-cyan-500/10 border-cyan-500/30",
        amount: "text-cyan-400",
        typeText: "text-cyan-600",
      },
      Flux: {
        container: "bg-indigo-500/10 border-indigo-500/30",
        amount: "text-indigo-400",
        typeText: "text-indigo-500",
      },
      Astrai: {
        container: "bg-fuchsia-500/10 border-fuchsia-500/30",
        amount: "text-fuchsia-400",
        typeText: "text-fuchsia-600",
      },
      Voucher: {
        container: "bg-emerald-500/10 border-emerald-500/30",
        amount: "text-emerald-400",
        typeText: "text-emerald-600",
      },
    };

    return (
      themes[type] || {
        container: "bg-slate-900/50 border-slate-700/50",
        amount: "text-slate-400",
        typeText: "text-slate-500",
      }
    );
  }

  // Determines the overall theme for the modal's outer elements (icon, button, border)
  private getModalTheme(): {
    border: string;
    shadow: string;
    iconBg: string;
    btnBg: string;
    btnShadow: string;
  } {
    const hasVoucher = this.rewards.some((r) => r.reward_type === "Voucher");

    if (hasVoucher) {
      return {
        border: "border-emerald-500/50",
        shadow: "shadow-[0_0_50px_rgba(16,185,129,0.2)]",
        iconBg: "bg-emerald-500 shadow-emerald-500/40",
        btnBg: "bg-emerald-600 hover:bg-emerald-500",
        btnShadow: "shadow-emerald-900/40",
      };
    }

    // Default theme if no voucher is present (using a neutral/cyan vibe as fallback)
    return {
      border: "border-cyan-500/40",
      shadow: "shadow-[0_0_40px_rgba(6,182,212,0.15)]",
      iconBg: "bg-cyan-500 shadow-cyan-500/40",
      btnBg: "bg-cyan-600 hover:bg-cyan-500",
      btnShadow: "shadow-cyan-900/40",
    };
  }

  private render(): void {
    const theme = this.getModalTheme();

    // Map through the array of rewards and apply the specific styles for the new layout
    const listItems = this.rewards
      .map((r) => {
        const style = this.getStyles(r.reward_type);
        return `
          <div class="p-4 rounded-2xl border ${style.container}">
              <span class="block text-4xl font-mono font-black ${style.amount}">${r.amount}</span>
              <span class="block text-xs font-bold uppercase tracking-widest mt-1 ${style.typeText}">${r.reward_type}</span>
          </div>
        `;
      })
      .join("");

    const modalHtml = `
      <div id="reward-modal-${this.id}" class="fixed inset-0 z-[100] flex items-center justify-center transition-opacity duration-300 bg-slate-900/90 backdrop-blur-md opacity-0">
          <div id="container-${this.id}" class="relative w-full max-w-sm p-8 mx-4 text-center transform scale-90 transition-transform duration-300 bg-slate-800 border-2 ${theme.border} rounded-3xl ${theme.shadow}">

              <div class="absolute inset-x-0 top-0 -translate-y-1/2 flex justify-center">
                  <div class="${theme.iconBg} p-4 rounded-2xl shadow-lg animate-bounce">
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-slate-900" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-7.714 2.143L11 21l-2.286-6.857L1 12l7.714-2.143L11 3z" />
                      </svg>
                  </div>
              </div>

              <h2 class="mt-8 text-3xl font-black text-white tracking-tighter uppercase">Reward Claimed!</h2>
              <p class="mt-2 text-slate-400 font-medium">You have successfully received:</p>

              <div class="my-6 space-y-3">
                  ${listItems}
              </div>

              <button id="btn-${this.id}" class="w-full py-4 ${theme.btnBg} text-slate-900 font-black rounded-xl active:scale-95 transition-all shadow-lg ${theme.btnShadow}">
                  CONFIRM & CONTINUE
              </button>
          </div>
      </div>
    `;

    document.body.insertAdjacentHTML("beforeend", modalHtml);
    this.modalEl = document.getElementById(`reward-modal-${this.id}`);

    // Bind destruction logic to the button
    document
      .getElementById(`btn-${this.id}`)
      ?.addEventListener("click", () => this.destroy());

    // Trigger open animations
    requestAnimationFrame(() => {
      this.modalEl?.classList.remove("opacity-0");
      const container = document.getElementById(`container-${this.id}`);
      container?.classList.replace("scale-90", "scale-100");
    });
  }

  public destroy(): void {
    if (!this.modalEl) return;

    // Trigger close animations
    this.modalEl.classList.add("opacity-0");
    const container = document.getElementById(`container-${this.id}`);
    container?.classList.replace("scale-100", "scale-95");

    // Remove from DOM after transition completes
    setTimeout(() => {
      this.modalEl?.remove();
      this.modalEl = null;
    }, 300);
  }
}
