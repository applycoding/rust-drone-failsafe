import init, { WasmSim } from "./pkg/failsafe_wasm.js";

const SCENARIO = `\
[meta]
name = "browser"
duration_ms = 60000

[fence]
vertices = [[-500, -500], [-500, 500], [500, 500], [500, -500]]
min_alt_m = 5.0
max_alt_m = 120.0

[[events]]
at_ms = 0
kind = "reset"
`;

const canvas = document.getElementById("view");
const ctx = canvas.getContext("2d");
const statusEl = document.getElementById("status");

let sim;
const trail = [];
const controls = {
  gcs_cut: false,
  rc_cut: false,
  gps_kill: false,
  ekf_fault: false,
  drain: 0.01,
};

function draw(frame) {
  const w = canvas.width;
  const h = canvas.height;
  ctx.clearRect(0, 0, w, h);
  const scale = 0.35;
  const cx = w / 2;
  const cy = h / 2;
  ctx.strokeStyle = "#4a90d9";
  ctx.strokeRect(cx - 500 * scale, cy - 500 * scale, 1000 * scale, 1000 * scale);
  ctx.fillStyle = "#f5a623";
  ctx.beginPath();
  ctx.arc(cx, cy, 5, 0, Math.PI * 2);
  ctx.fill();
  trail.push({ n: frame.n, e: frame.e });
  if (trail.length > 200) trail.shift();
  ctx.strokeStyle = "#7ed321";
  ctx.beginPath();
  for (const p of trail) {
    const x = cx + p.e * scale;
    const y = cy - p.n * scale;
    ctx.lineTo(x, y);
  }
  ctx.stroke();
  ctx.fillStyle = "#fff";
  ctx.beginPath();
  ctx.arc(cx + frame.e * scale, cy - frame.n * scale, 6, 0, Math.PI * 2);
  ctx.fill();
}

function applyControls() {
  sim.set_controls(JSON.stringify(controls));
}

async function boot() {
  await init();
  sim = new WasmSim(SCENARIO);
  applyControls();

  document.getElementById("cut-gcs").onclick = () => {
    controls.gcs_cut = true;
    applyControls();
  };
  document.getElementById("cut-rc").onclick = () => {
    controls.rc_cut = true;
    applyControls();
  };
  document.getElementById("kill-gps").onclick = () => {
    controls.gps_kill = true;
    applyControls();
  };
  document.getElementById("ekf-fault").onclick = () => {
    controls.ekf_fault = true;
    applyControls();
  };
  document.getElementById("drain").oninput = (e) => {
    controls.drain = parseFloat(e.target.value);
    applyControls();
  };
  document.getElementById("reset").onclick = () => {
    controls.gcs_cut = false;
    controls.rc_cut = false;
    controls.gps_kill = false;
    controls.ekf_fault = false;
    sim = new WasmSim(SCENARIO);
    trail.length = 0;
    applyControls();
  };

  setInterval(() => {
    try {
      const json = sim.tick();
      const frame = JSON.parse(json);
      draw(frame);
      statusEl.textContent = `Action: ${frame.action} (${frame.reason}) disarm_eligible=${frame.disarm_eligible}`;
    } catch (err) {
      statusEl.textContent = `Tick error: ${err}`;
    }
  }, 50);
}

boot().catch((e) => {
  console.error(e);
  statusEl.textContent = `Load error: ${e}`;
});
