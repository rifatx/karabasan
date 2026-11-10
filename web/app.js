import { Engine } from "./engine.js";

const $ = id => document.getElementById(id);
const terminal = $("terminal");
const transcript = $("transcript");
const answer = $("answer");
const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
const status = $("status");
const form = $("chat-form");
let engine;
let generation = 0;
let busy = true;
let finished = false;

const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
const bottom = () => { terminal.scrollTop = terminal.scrollHeight; };
function append(text, className = "") {
  const line = document.createElement("p");
  line.className = `line ${className}`;
  line.textContent = text;
  transcript.append(line);
  bottom();
  return line;
}
function lock(value) {
  busy = value;
  answer.disabled = value || finished;
}
async function render(turn, token) {
  lock(true);
  form.hidden = true;
  status.textContent = "karabasan yazıyor…";
  for (const data of turn.lines) {
    if (token !== generation) return;
    // Keep partial typewriter updates out of the live region. Announce once.
    const line = document.createElement("p");
    line.className = "line";
    line.setAttribute("aria-hidden", "true");
    transcript.append(line);
    if (reducedMotion.matches) line.textContent = data.text;
    else {
      for (const c of data.text) {
        if (token !== generation) return;
        line.textContent += c;
        bottom();
        if (!reducedMotion.matches) await wait(12);
      }
      if (!reducedMotion.matches) await wait(data.delay_ms);
    }
    line.remove();
    append(data.text);
  }
  if (token !== generation) return;
  finished = turn.finished;
  $("prompt").textContent = turn.prompt;
  form.hidden = finished;
  status.textContent = finished ? "Program sonlandı. Yeniden başlayabilirsin." : "cevabını bekliyor";
  lock(false);
  bottom();
  if (!finished) answer.focus({ preventScroll: true });
}
function restart() {
  generation++;
  transcript.replaceChildren();
  answer.value = "";
  finished = false;
  const params = new URLSearchParams(location.search);
  const chosen = params.get("seed");
  const seed = chosen !== null && /^\d+$/.test(chosen)
    ? Number(chosen) >>> 0 : crypto.getRandomValues(new Uint32Array(1))[0];
  render(engine.start(seed), generation).catch(fail);
}
function fail(error) {
  generation++;
  lock(true);
  status.textContent = "Yükleme / çalışma hatası";
  append(error.message, "system");
}
form.addEventListener("submit", event => {
  event.preventDefault();
  if (busy || finished) return;
  const input = answer.value;
  append(`> ${input}`, "user");
  answer.value = "";
  try { render(engine.submit(input), generation).catch(fail); }
  catch (error) { fail(error); }
});
document.addEventListener("keydown", event => {
  if (event.key === "F2" && engine) {
    event.preventDefault();
    restart();
  }
});
terminal.addEventListener("click", () => {
  if (!busy && !finished && !window.getSelection()?.toString()) answer.focus();
});
try {
  engine = await Engine.load();
  restart();
} catch (error) { fail(error); }
