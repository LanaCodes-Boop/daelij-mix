// Central state: mixer (undoable, saved in scenes), apps, UI, settings.

export const NEG_INF = -100; // stored value for "-∞"

/* ---------- dB <-> fader position ---------- */
export function posToDb(p) {
  if (p <= 0.002) return NEG_INF;
  if (p >= 0.75) return ((p - 0.75) / 0.25) * 12;
  return -60 * Math.pow(1 - p / 0.75, 1.5);
}
export function dbToPos(db) {
  if (db <= -60) return 0;
  if (db >= 0) return Math.min(1, 0.75 + (db / 12) * 0.25);
  return 0.75 * (1 - Math.pow(-db / 60, 1 / 1.5));
}
export const fmtDb = (db) =>
  db <= -60 ? '−∞' : (db > 0.04 ? '+' : db < -0.04 ? '−' : '') + Math.abs(db).toFixed(1);
export const lin = (db) => (db <= -60 ? 0 : Math.pow(10, db / 20));
export const clampDb = (db) => Math.max(NEG_INF, Math.min(12, db));
export function parseDb(text) {
  const t = String(text).trim().replace('−', '-').replace(/db$/i, '').trim();
  if (/^-?(inf|∞)$/i.test(t)) return NEG_INF;
  const n = parseFloat(t);
  return Number.isFinite(n) ? clampDb(n) : null;
}

/* ---------- EQ ---------- */
export const EQ_PRESETS = {
  Flat: [0, 0, 0, 0, 0],
  'Voice clarity': [-5, -2, 1, 4, 2],
  'Bass boost': [6, 2, 0, 0, 1],
  Warm: [2, 2, 0, -2, -3],
  Bright: [0, -1, 0, 3, 5],
};
const BANDS = [
  { f: 80, type: 'low' },
  { f: 250, type: 'peak' },
  { f: 1000, type: 'peak' },
  { f: 4000, type: 'peak' },
  { f: 10000, type: 'high' },
];
export const bandsFor = (name) => BANDS.map((b, i) => ({ ...b, g: EQ_PRESETS[name][i], q: 1 }));

/* ---------- defaults ---------- */
const src = (id, name, color, preset, extra = {}) => ({
  id, name, alias: '', color, mute: false, solo: false, pan: 0, group: 0,
  preset, eq: bandsFor(preset),
  comp: { on: false, amount: 30 }, gate: { on: false, thr: -50 }, nr: { on: false, amount: 50 },
  ...extra,
});
export function defaultMix() {
  return {
    sources: [
      src('game', 'Game', '#5ee08a', 'Flat'),
      src('chat', 'Chat', '#4cc3ff', 'Voice clarity', { comp: { on: true, amount: 30 }, nr: { on: true, amount: 50 } }),
      src('media', 'Media', '#ff7a59', 'Warm'),
      src('aux', 'Aux', '#c38bff', 'Flat'),
    ],
    mixes: [
      { id: 'A', name: 'Mix A', alias: 'Headphones', color: '#8b9cff', gain: 0, mute: false },
      { id: 'B', name: 'Mix B', alias: 'Stream', color: '#ff5577', gain: 0, mute: false },
    ],
    sends: {
      game: { A: { on: true, gain: 0 }, B: { on: true, gain: -6 } },
      chat: { A: { on: true, gain: 0 }, B: { on: true, gain: -3 } },
      media: { A: { on: true, gain: -6 }, B: { on: true, gain: -14 } },
      aux: { A: { on: true, gain: -8 }, B: { on: false, gain: -10 } },
    },
  };
}

/* ---------- persistence ---------- */
const KEY = 'daelij-mix-v2';
function load() {
  try { return JSON.parse(localStorage.getItem(KEY)) || null; } catch { return null; }
}
const saved = load();

export const S = $state({
  mix: saved?.mix ?? defaultMix(),
  ui: {
    view: 'mixer', focus: 'A', selected: 'game',
    density: saved?.ui?.density ?? 'standard',
    inspector: saved?.ui?.inspector ?? true,
  },
  scenes: saved?.scenes ?? {
    current: 'Default',
    list: [
      { name: 'Default', data: JSON.stringify(defaultMix()) },
      { name: 'Gaming', data: JSON.stringify(gamingScene()) },
      { name: 'Streaming', data: JSON.stringify(streamingScene()) },
    ],
  },
  apps: saved?.apps ?? { assign: {}, vol: {}, mute: {} },
  devices: saved?.devices ?? { fallback: [] },
  settings: saved?.settings ?? { sr: 48000, buf: 256 },
  // runtime only (not saved)
  rt: {
    live: false, sessions: [], outputs: [], pollMs: 0, error: '',
    canUndo: false, canRedo: false, snap: '',
  },
});

function gamingScene() {
  const m = defaultMix();
  m.sends.game.A.gain = 2; m.sends.media.A.gain = -14; m.sends.chat.A.gain = -2;
  return m;
}
function streamingScene() {
  const m = defaultMix();
  m.sends.game.B.gain = -10; m.sends.media.B.gain = -18; m.sends.chat.B.gain = -4;
  return m;
}

let saveT;
export function save() {
  clearTimeout(saveT);
  saveT = setTimeout(() => {
    try {
      const s = $state.snapshot(S);
      delete s.rt;
      localStorage.setItem(KEY, JSON.stringify(s));
    } catch {}
  }, 250);
}

/* ---------- undo / redo ---------- */
const undoStack = [];
const redoStack = [];
S.rt.snap = JSON.stringify($state.snapshot(S.mix));

function refreshFlags() {
  S.rt.canUndo = undoStack.length > 0;
  S.rt.canRedo = redoStack.length > 0;
}
/** Call after a discrete change (pointer up, click, enter). */
export function commit() {
  const now = JSON.stringify($state.snapshot(S.mix));
  if (now !== S.rt.snap) {
    undoStack.push(S.rt.snap);
    if (undoStack.length > 200) undoStack.shift();
    redoStack.length = 0;
    S.rt.snap = now;
    refreshFlags();
  }
  save();
}
export function undo() {
  commit();
  if (!undoStack.length) return;
  redoStack.push(S.rt.snap);
  S.rt.snap = undoStack.pop();
  S.mix = JSON.parse(S.rt.snap);
  refreshFlags(); save();
}
export function redo() {
  if (!redoStack.length) return;
  undoStack.push(S.rt.snap);
  S.rt.snap = redoStack.pop();
  S.mix = JSON.parse(S.rt.snap);
  refreshFlags(); save();
}

/* ---------- scenes ---------- */
export function isDirty() {
  const sc = S.scenes.list.find((s) => s.name === S.scenes.current);
  return !sc || sc.data !== S.rt.snap;
}
export function loadScene(name) {
  const sc = S.scenes.list.find((s) => s.name === name);
  if (!sc) return;
  commit();
  S.scenes.current = name;
  S.mix = JSON.parse(sc.data);
  commit();
}
export function saveScene(name = S.scenes.current) {
  commit();
  const data = S.rt.snap;
  const sc = S.scenes.list.find((s) => s.name === name);
  if (sc) sc.data = data; else S.scenes.list.push({ name, data });
  S.scenes.current = name;
  save();
}

/* ---------- helpers ---------- */
export const source = (id) => S.mix.sources.find((s) => s.id === id);
export const mixOf = (id) => S.mix.mixes.find((m) => m.id === id);
export const label = (x) => x.alias || x.name;

/** Set a source's send gain in a mix; group members move by the same amount. */
export function setSendGain(srcId, mixId, db) {
  const s = source(srcId);
  const cur = S.mix.sends[srcId][mixId].gain;
  const delta = clampDb(db) - cur;
  S.mix.sends[srcId][mixId].gain = clampDb(db);
  if (s.group) {
    for (const o of S.mix.sources) {
      if (o.id !== srcId && o.group === s.group) {
        const sd = S.mix.sends[o.id][mixId];
        sd.gain = sd.gain <= -60 && delta > 0 ? -60 + delta : clampDb(sd.gain + delta);
      }
    }
  }
}

/* ---------- app assignment ---------- */
const CHAT = /discord|teams|zoom|skype|slack|whatsapp|telegram|signal|teamspeak|mumble/i;
const MEDIA = /spotify|chrome|firefox|msedge|edge|opera|brave|vlc|music|itunes|youtube|netflix|mpc|foobar|tidal|deezer|media/i;
const GAME = /steam|epic|riot|battle|game|valorant|fortnite|minecraft|league|cs2|dota|roblox|apex|gta|cod|overwatch|rocket/i;
export function assignedSource(name) {
  const k = name.toLowerCase();
  if (S.apps.assign[k]) return S.apps.assign[k];
  if (CHAT.test(k)) return 'chat';
  if (MEDIA.test(k)) return 'media';
  if (GAME.test(k)) return 'game';
  return 'aux';
}
export function assignApp(name, srcId) {
  S.apps.assign[name.toLowerCase()] = srcId;
  save();
}
export const appVol = (name) => S.apps.vol[name.toLowerCase()] ?? 1;
export const appMuted = (name) => !!S.apps.mute[name.toLowerCase()];
