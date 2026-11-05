// Engine: meters (one rAF loop writing straight to the DOM), demo signal,
// and — inside the Windows app — real per-app audio sessions via Tauri.
import { S, dbToPos, lin, source, mixOf, assignedSource, appVol, appMuted, isVirtual, save } from './state.svelte.js';

export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
let invoke = null;
if (isTauri) import('@tauri-apps/api/core').then((m) => { invoke = m.invoke; startLive(); });

/* ---------- meter registry ---------- */
const meters = new Map(); // key -> {el, bars, horizontal, st}
export function registerMeter(key, el, horizontal = false) {
  if (!el) return;
  const bars = [...el.querySelectorAll('.bar')].map((b) => ({ cover: b.querySelector('.cover'), peak: b.querySelector('.peak') }));
  const m = { el, bars, horizontal, st: bars.map(() => ({ v: -100, p: -100, t: 0 })), clip: false };
  meters.set(key, m);
  return () => { if (meters.get(key) === m) meters.delete(key); };
}
export function resetPeaks() {
  for (const m of meters.values()) {
    m.st.forEach((s) => { s.p = -100; s.t = 0; });
    m.clip = false; m.el.removeAttribute('data-clip');
  }
}

/* ---------- demo apps + signal ---------- */
export const DEMO_APPS = [
  { pid: 101, name: 'Steam' },
  { pid: 102, name: 'Discord' },
  { pid: 103, name: 'Spotify' },
  { pid: 104, name: 'Chrome' },
  { pid: 105, name: 'OBS Studio' },
  { pid: 0, name: 'System sounds' },
];
const talk = {};
function talking(id, t) {
  const s = (talk[id] ??= { on: false, next: 0 });
  if (t > s.next) { s.on = !s.on; s.next = t + (s.on ? 0.6 + Math.random() * 2.2 : 0.4 + Math.random() * 1.8); }
  return s.on;
}
const rnd = () => Math.random() * 2 - 1;
function demoAppDb(name, t) {
  const k = name.toLowerCase();
  if (k === 'discord') return talking('discord', t) ? -18 + rnd() * 5 : -100;
  if (k === 'spotify') { const beat = Math.exp(-((t * 2.1) % 1) * 5); return -17 + beat * 7 + rnd() * 2; }
  if (k === 'chrome') return talking('chrome', t) ? -22 + rnd() * 4 : -100;
  if (k === 'steam') return -16 + 3 * Math.sin(t * 1.7) + rnd() * 3 + (Math.random() < 0.01 ? 6 : 0);
  if (k === 'obs studio') return -100;
  if (k === 'system sounds') return Math.random() < 0.004 ? -8 : -100;
  return -40 + rnd() * 6;
}

/* ---------- live (Windows) ---------- */
const livePeaks = new Map(); // pid -> peak (linear 0..1)
const written = new Map();   // pid -> last volume written
async function startLive() {
  S.rt.live = true;
  refreshOutputs();
  setInterval(refreshOutputs, 5000);
  const poll = async () => {
    const t0 = performance.now();
    try {
      const list = await invoke('list_sessions');
      S.rt.pollMs = Math.round(performance.now() - t0);
      livePeaks.clear();
      for (const s of list) livePeaks.set(s.pid, s.peak);
      const sig = list.map((s) => s.pid + ':' + s.name).join('|');
      if (sig !== lastSig) {
        lastSig = sig;
        S.rt.sessions = list.map((s) => ({ pid: s.pid, name: pretty(s.name) }));
      }
      S.rt.error = '';
      applyVolumes();
      updateStream();
    } catch (e) {
      S.rt.error = String(e);
    }
    setTimeout(poll, 80);
  };
  poll();
}
let lastSig = '';
const pretty = (n) => (n && n[0] === n[0].toLowerCase() ? n[0].toUpperCase() + n.slice(1) : n);

export async function refreshOutputs() {
  if (!invoke) return;
  try {
    S.rt.outputs = await invoke('list_outputs');
    // First run: use any virtual cable as the stream output.
    if (!S.devices.stream.length) {
      const v = S.rt.outputs.filter((o) => isVirtual(o.name)).map((o) => o.id);
      if (v.length) { S.devices.stream = v; save(); }
    }
  } catch (e) { S.rt.error = String(e); }
}

/* ---------- Mix B (stream) ---------- */
let streamPeak = [0, 0];
let lastStreamKey = '', lastStreamSent = 0;
/**
 * Gain per app for Mix B = app volume × source send to B × Mix B fader.
 * The capture hears the app after its Windows volume (set for Mix A), so that is divided out again.
 */
function streamGain(app) {
  const srcId = assignedSource(app.name);
  const s = source(srcId), send = S.mix.sends[srcId].B, b = mixOf('B');
  const anySolo = S.mix.sources.some((x) => x.solo);
  if (appMuted(app.name) || s.mute || !send.on || b.mute || (anySolo && !s.solo)) return 0;
  const g = appVol(app.name) * lin(send.gain + b.gain);
  const wv = written.get(app.pid) ?? 1;
  return wv > 0.02 ? Math.min(16, g / wv) : 0;
}
async function updateStream() {
  const present = new Set(S.rt.outputs.map((o) => o.id));
  const params = {
    enabled: !!S.settings.streamOn,
    outputs: S.devices.stream.filter((id) => present.has(id)),
    apps: S.rt.sessions.filter((a) => a.pid !== 0).map((a) => ({ pid: a.pid, gain: +streamGain(a).toFixed(4) })),
  };
  const key = JSON.stringify(params);
  const now = performance.now();
  if (key === lastStreamKey && now - lastStreamSent < 100) return;
  lastStreamKey = key; lastStreamSent = now;
  try {
    const st = await invoke('stream_update', { params });
    streamPeak = [Math.max(streamPeak[0], st.peak_l), Math.max(streamPeak[1], st.peak_r)];
    const r = S.rt.stream;
    if (r.state !== st.state) r.state = st.state;
    if (r.device !== st.device) r.device = st.device;
    if (r.device_id !== st.device_id) r.device_id = st.device_id;
    if (r.captures !== st.captures) r.captures = st.captures;
    if (st.error && r.error !== st.error) r.error = st.error;
    if (!st.error && r.error && st.state === 'active') r.error = '';
  } catch (e) {
    S.rt.stream.state = 'error'; S.rt.stream.error = String(e);
  }
}
/** Mix B is real when the engine is playing to a stream device. */
export const streamLive = () => S.rt.live && (S.rt.stream.state === 'active' || S.rt.stream.state === 'fallback');

/** Windows volume for an app = app volume × source send to Mix A × Mix A fader. */
function targetVolume(name) {
  const srcId = assignedSource(name);
  const s = source(srcId), send = S.mix.sends[srcId].A, a = mixOf('A');
  const anySolo = S.mix.sources.some((x) => x.solo);
  if (appMuted(name) || s.mute || !send.on || a.mute || (anySolo && !s.solo)) return 0;
  return Math.max(0, Math.min(1, appVol(name) * lin(send.gain + a.gain)));
}
function applyVolumes() {
  for (const app of S.rt.sessions) {
    const v = targetVolume(app.name);
    const prev = written.get(app.pid);
    if (prev === undefined || Math.abs(prev - v) > 0.002) {
      written.set(app.pid, v);
      invoke('set_session_volume', { pid: app.pid, volume: v }).catch(() => written.delete(app.pid));
    }
  }
}
export async function resetAppVolumes() {
  S.apps.vol = {}; S.apps.mute = {};
  written.clear();
  if (invoke) await invoke('restore_all').catch(() => {});
}

/** Apps currently known: live sessions or the demo list. */
export function currentApps() {
  return S.rt.live ? S.rt.sessions : DEMO_APPS;
}

/* ---------- levels ---------- */
const dbAdd = (arr) => {
  let p = 0;
  for (const d of arr) if (d > -100) p += Math.pow(10, d / 10);
  return p > 0 ? 10 * Math.log10(p) : -100;
};
function panDb(pan) {
  const th = ((pan + 100) / 200) * (Math.PI / 2);
  const l = Math.cos(th) * Math.SQRT2, r = Math.sin(th) * Math.SQRT2;
  return [l > 1e-4 ? 20 * Math.log10(l) : -100, r > 1e-4 ? 20 * Math.log10(r) : -100];
}

function computeLevels(t) {
  const lv = new Map();
  const apps = currentApps();
  const pre = { game: [], chat: [], media: [], aux: [] };
  for (const app of apps) {
    let db;
    if (S.rt.live) { const p = livePeaks.get(app.pid) ?? 0; db = p > 0.00001 ? 20 * Math.log10(p) : -100; }
    else db = demoAppDb(app.name, t);
    const vol = appMuted(app.name) ? 0 : appVol(app.name);
    const post = vol > 0 ? db + 20 * Math.log10(vol) : -100;
    lv.set('app:' + app.name, [post, post]);
    pre[assignedSource(app.name)]?.push(post);
  }
  const anySolo = S.mix.sources.some((x) => x.solo);
  const into = { A: [], B: [] };
  for (const s of S.mix.sources) {
    const p = dbAdd(pre[s.id]);
    const [pl, pr] = panDb(s.pan);
    for (const m of ['A', 'B']) {
      const send = S.mix.sends[s.id][m];
      const silent = s.mute || !send.on || (anySolo && !s.solo) || p <= -100;
      const v = silent ? [-100, -100] : [p + send.gain + pl + rnd() * 0.4, p + send.gain + pr + rnd() * 0.4];
      into[m].push(v);
      if (m === S.ui.focus) lv.set('src:' + s.id, v);
    }
  }
  for (const m of S.mix.mixes) {
    if (m.id === 'B' && streamLive()) {
      const d = (p) => (p > 0.00001 ? 20 * Math.log10(p) : -100);
      lv.set('mix:B', [d(streamPeak[0]), d(streamPeak[1])]);
      streamPeak = [0, 0];
      continue;
    }
    const g = m.mute ? -100 : m.gain;
    const L = dbAdd(into[m.id].map((v) => v[0])), R = dbAdd(into[m.id].map((v) => v[1]));
    lv.set('mix:' + m.id, [L > -100 ? L + g : -100, R > -100 ? R + g : -100]);
  }
  return lv;
}

/* ---------- render loop ---------- */
let last = performance.now(), frame = 0;
function tick(now) {
  const dt = Math.min(0.1, (now - last) / 1000);
  last = now; frame++;
  if (meters.size) {
    const lv = computeLevels(now / 1000);
    for (const [key, m] of meters) {
      const v = lv.get(key) ?? [-100, -100];
      m.st.forEach((s, i) => {
        const x = Math.min(12, v[i] ?? v[0]);
        s.v = x > s.v ? x : Math.max(x, s.v - 26 * dt); // fast attack, slow release
        if (s.v > s.p || now > s.t) { s.p = s.v; s.t = now + 1500; }
        const pos = dbToPos(s.v), pk = dbToPos(s.p);
        const b = m.bars[i];
        if (m.horizontal) { b.cover.style.width = (1 - pos) * 100 + '%'; b.peak.style.left = pk * 100 + '%'; }
        else { b.cover.style.height = (1 - pos) * 100 + '%'; b.peak.style.bottom = pk * 100 + '%'; }
        b.peak.style.opacity = s.p > -60 ? 1 : 0;
        if (s.v > 0 && !m.clip) { m.clip = true; m.el.setAttribute('data-clip', ''); }
      });
      if (frame % 30 === 0) m.el.setAttribute('aria-valuenow', Math.max(-60, m.st[0].v).toFixed(0));
    }
  }
  requestAnimationFrame(tick);
}
requestAnimationFrame(tick);

/* ---------- test tone ---------- */
export function testTone() {
  const AC = window.AudioContext || window.webkitAudioContext;
  if (!AC) return;
  const ac = new AC(), o = ac.createOscillator(), g = ac.createGain();
  o.frequency.value = 660; g.gain.value = 0;
  g.gain.linearRampToValueAtTime(0.2, ac.currentTime + 0.02);
  g.gain.linearRampToValueAtTime(0, ac.currentTime + 0.5);
  o.connect(g).connect(ac.destination); o.start(); o.stop(ac.currentTime + 0.55);
  setTimeout(() => ac.close(), 800);
}
