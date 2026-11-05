//! Mix B (stream) engine.
//!
//! Captures the sound of each app separately (Windows process loopback),
//! multiplies it by the gain the UI computed, sums everything and plays the
//! result on the stream device (normally a virtual cable that OBS records).
//!
//! Threads:
//! - supervisor: reads the latest params every 250 ms, starts/stops captures,
//!   picks the stream device from the priority list (fallback), restarts render.
//! - one capture thread per app: writes samples into that app's ring.
//! - render thread: event driven, pulls from the rings, mixes, limits, writes.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use wasapi::*;

const SR: usize = 48_000;
const CH: usize = 2;
const PRIME: usize = SR * CH * 30 / 1000; // start playing an app after 30 ms buffered
const MAX_RING: usize = SR * CH * 150 / 1000; // never hold more than 150 ms
const TRIM_TO: usize = SR * CH * 50 / 1000; // when too full, drop back to 50 ms

type Res<T> = Result<T, Box<dyn Error>>;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct AppGain {
    pub pid: u32,
    pub gain: f32,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Params {
    pub enabled: bool,
    /// Stream device ids in priority order (first = preferred).
    pub outputs: Vec<String>,
    pub apps: Vec<AppGain>,
}

#[derive(Serialize, Clone, Default, Debug)]
pub struct Status {
    /// off | starting | active | fallback | no_device | error
    pub state: String,
    pub device: String,
    pub device_id: String,
    pub peak_l: f32,
    pub peak_r: f32,
    pub captures: usize,
    pub error: String,
}

struct Ring {
    buf: VecDeque<f32>,
    primed: bool,
}

struct Capture {
    stop: Arc<AtomicBool>,
    dead: Arc<AtomicBool>,
    gain: Arc<AtomicU32>,
    ring: Arc<Mutex<Ring>>,
}

struct Shared {
    params: Mutex<Params>,
    status: Mutex<Status>,
    captures: Mutex<HashMap<u32, Capture>>,
    peak_l: AtomicU32,
    peak_r: AtomicU32,
}

pub struct Engine {
    shared: Arc<Shared>,
}

impl Engine {
    pub fn start() -> Self {
        let shared = Arc::new(Shared {
            params: Mutex::new(Params::default()),
            status: Mutex::new(Status { state: "off".into(), ..Default::default() }),
            captures: Mutex::new(HashMap::new()),
            peak_l: AtomicU32::new(0),
            peak_r: AtomicU32::new(0),
        });
        let sh = shared.clone();
        thread::Builder::new()
            .name("stream-supervisor".into())
            .spawn(move || supervisor(sh))
            .expect("spawn supervisor");
        Engine { shared }
    }

    /// Store new params and return the current status (peaks since last call).
    pub fn update(&self, p: Params) -> Status {
        if let Ok(mut g) = self.shared.params.lock() {
            *g = p;
        }
        let mut st = self.shared.status.lock().map(|s| s.clone()).unwrap_or_default();
        st.peak_l = f32::from_bits(self.shared.peak_l.swap(0, Ordering::Relaxed));
        st.peak_r = f32::from_bits(self.shared.peak_r.swap(0, Ordering::Relaxed));
        st
    }
}

fn set_status(sh: &Shared, f: impl FnOnce(&mut Status)) {
    if let Ok(mut s) = sh.status.lock() {
        f(&mut s);
    }
}

struct RenderHandle {
    id: String,
    stop: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
}

fn supervisor(sh: Arc<Shared>) {
    let _ = initialize_mta();
    let failed_err: Arc<Mutex<Option<(String, String)>>> = Arc::new(Mutex::new(None));
    let mut render: Option<RenderHandle> = None;
    let mut bad_devices: HashMap<String, Instant> = HashMap::new();
    let mut bad_pids: HashMap<u32, Instant> = HashMap::new();
    let capture_err: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    loop {
        let p = sh.params.lock().map(|g| g.clone()).unwrap_or_default();

        // ---- captures ----
        let wanted: HashMap<u32, f32> = if p.enabled {
            p.apps.iter().map(|a| (a.pid, a.gain.max(0.0))).collect()
        } else {
            HashMap::new()
        };
        bad_pids.retain(|_, t| t.elapsed() < Duration::from_secs(5));
        let mut n_caps = 0;
        if let Ok(mut caps) = sh.captures.lock() {
            caps.retain(|pid, c| {
                let dead = c.dead.load(Ordering::Relaxed);
                if dead {
                    bad_pids.insert(*pid, Instant::now());
                }
                let keep = wanted.contains_key(pid) && !dead;
                if !keep {
                    c.stop.store(true, Ordering::Relaxed);
                }
                keep
            });
            for (pid, gain) in &wanted {
                if let Some(c) = caps.get(pid) {
                    c.gain.store(gain.to_bits(), Ordering::Relaxed);
                } else if *pid != 0 && !bad_pids.contains_key(pid) {
                    let c = Capture {
                        stop: Arc::new(AtomicBool::new(false)),
                        dead: Arc::new(AtomicBool::new(false)),
                        gain: Arc::new(AtomicU32::new(gain.to_bits())),
                        ring: Arc::new(Mutex::new(Ring { buf: VecDeque::with_capacity(MAX_RING), primed: false })),
                    };
                    let (pid2, stop, dead, ring, err) =
                        (*pid, c.stop.clone(), c.dead.clone(), c.ring.clone(), capture_err.clone());
                    let _ = thread::Builder::new().name(format!("capture-{pid}")).spawn(move || {
                        if let Err(e) = capture_loop(pid2, &ring, &stop) {
                            if let Ok(mut m) = err.lock() {
                                *m = format!("Could not capture app {pid2}: {e}");
                            }
                        }
                        dead.store(true, Ordering::Relaxed);
                    });
                    caps.insert(*pid, c);
                }
            }
            n_caps = caps.len();
        }

        // ---- render device (priority list with fallback) ----
        if let Some((id, e)) = failed_err.lock().ok().and_then(|mut g| g.take()) {
            bad_devices.insert(id, Instant::now());
            set_status(&sh, |s| s.error = e);
        }
        // a failed device is retried after 10 s, so a returning device gets picked again
        bad_devices.retain(|_, t| t.elapsed() < Duration::from_secs(10));
        let alive = render.as_ref().map(|r| !r.done.load(Ordering::Relaxed)).unwrap_or(false);

        let target = if p.enabled {
            p.outputs.iter().find(|id| !bad_devices.contains_key(*id)).cloned()
        } else {
            None
        };
        match target {
            None => {
                if let Some(r) = render.take() {
                    r.stop.store(true, Ordering::Relaxed);
                }
                let state = if !p.enabled { "off" } else { "no_device" };
                set_status(&sh, |s| {
                    s.state = state.into();
                    s.device.clear();
                    s.device_id.clear();
                });
            }
            Some(id) => {
                let same = render.as_ref().map(|r| r.id == id).unwrap_or(false);
                if !same || !alive {
                    if let Some(r) = render.take() {
                        r.stop.store(true, Ordering::Relaxed);
                    }
                    let stop = Arc::new(AtomicBool::new(false));
                    let done = Arc::new(AtomicBool::new(false));
                    let (sh2, id2, stop2, done2, fe) = (sh.clone(), id.clone(), stop.clone(), done.clone(), failed_err.clone());
                    set_status(&sh, |s| s.state = "starting".into());
                    let _ = thread::Builder::new().name("stream-render".into()).spawn(move || {
                        if let Err(e) = render_loop(&id2, &sh2, &stop2) {
                            if !stop2.load(Ordering::Relaxed) {
                                if let Ok(mut g) = fe.lock() {
                                    *g = Some((id2.clone(), format!("Stream device failed: {e}")));
                                }
                            }
                        }
                        done2.store(true, Ordering::Relaxed);
                    });
                    render = Some(RenderHandle { id: id.clone(), stop, done });
                }
                let primary = p.outputs.first().map(|f| *f == id).unwrap_or(false);
                set_status(&sh, |s| {
                    if s.state != "starting" || alive {
                        s.state = if primary { "active".into() } else { "fallback".into() };
                    }
                    s.device_id = id.clone();
                });
            }
        }

        let cap_err = capture_err.lock().map(|m| m.clone()).unwrap_or_default();
        set_status(&sh, |s| {
            s.captures = n_caps;
            if !cap_err.is_empty() && s.error.is_empty() {
                s.error = cap_err.clone();
            }
        });
        thread::sleep(Duration::from_millis(250));
        // errors are shown for one round, then cleared unless they happen again
        set_status(&sh, |s| s.error.clear());
        if let Ok(mut m) = capture_err.lock() {
            m.clear();
        }
    }
}

fn format() -> WaveFormat {
    WaveFormat::new(32, 32, &SampleType::Float, SR, CH, None)
}

fn capture_loop(pid: u32, ring: &Arc<Mutex<Ring>>, stop: &AtomicBool) -> Res<()> {
    let _ = initialize_mta();
    let mut ac = AudioClient::new_application_loopback_client(pid, true)?;
    let mode = StreamMode::EventsShared { autoconvert: true, buffer_duration_hns: 0 };
    ac.initialize_client(&format(), &Direction::Capture, &mode)?;
    let ev = ac.set_get_eventhandle()?;
    let cc = ac.get_audiocaptureclient()?;
    ac.start_stream()?;
    let mut bytes = vec![0u8; SR * CH * 4];
    while !stop.load(Ordering::Relaxed) {
        let _ = ev.wait_for_event(200); // silence = no events; just loop
        loop {
            let frames = cc.get_next_packet_size()?.unwrap_or(0) as usize;
            if frames == 0 {
                break;
            }
            let need = frames * CH * 4;
            if bytes.len() < need {
                bytes.resize(need, 0);
            }
            let (got, _info) = cc.read_from_device(&mut bytes[..need])?;
            let n = got as usize * CH;
            if let Ok(mut r) = ring.lock() {
                for i in 0..n {
                    let b = &bytes[i * 4..i * 4 + 4];
                    r.buf.push_back(f32::from_le_bytes([b[0], b[1], b[2], b[3]]));
                }
                if r.buf.len() > MAX_RING {
                    let drop = r.buf.len() - TRIM_TO;
                    r.buf.drain(..drop);
                }
            }
        }
    }
    let _ = ac.stop_stream();
    Ok(())
}

fn find_device(id: &str) -> Res<Device> {
    let en = DeviceEnumerator::new()?;
    let col = en.get_device_collection(&Direction::Render)?;
    for d in &col {
        let d = d?;
        if d.get_id()? == id {
            return Ok(d);
        }
    }
    Err("device not found".into())
}

/// Soft limiter: transparent below 0.9, gently saturates above, never exceeds 1.0.
fn limit(x: f32) -> f32 {
    let a = x.abs();
    if a <= 0.9 {
        x
    } else {
        x.signum() * (0.9 + 0.1 * ((a - 0.9) / 0.1).tanh())
    }
}

fn render_loop(id: &str, sh: &Arc<Shared>, stop: &AtomicBool) -> Res<()> {
    let _ = initialize_mta();
    let dev = find_device(id)?;
    let name = dev.get_friendlyname().unwrap_or_else(|_| "Stream device".into());
    set_status(sh, |s| s.device = name);
    let mut ac = dev.get_iaudioclient()?;
    let (def_period, _) = ac.get_device_period()?;
    let mode = StreamMode::EventsShared { autoconvert: true, buffer_duration_hns: def_period };
    ac.initialize_client(&format(), &Direction::Render, &mode)?;
    let ev = ac.set_get_eventhandle()?;
    let rc = ac.get_audiorenderclient()?;
    let mut mix: Vec<f32> = Vec::new();
    let mut bytes: Vec<u8> = Vec::new();

    // start with one buffer of silence
    let first = ac.get_available_space_in_frames()? as usize;
    rc.write_to_device(first, &vec![0u8; first * CH * 4], None)?;
    ac.start_stream()?;

    while !stop.load(Ordering::Relaxed) {
        if ev.wait_for_event(1000).is_err() {
            return Err("device stopped responding".into());
        }
        let frames = ac.get_available_space_in_frames()? as usize;
        if frames == 0 {
            continue;
        }
        let n = frames * CH;
        mix.clear();
        mix.resize(n, 0.0);
        if let Ok(caps) = sh.captures.lock() {
            for c in caps.values() {
                let g = f32::from_bits(c.gain.load(Ordering::Relaxed));
                if let Ok(mut r) = c.ring.lock() {
                    if !r.primed {
                        if r.buf.len() >= PRIME {
                            r.primed = true;
                        } else {
                            continue;
                        }
                    }
                    let take = n.min(r.buf.len());
                    for (i, s) in r.buf.drain(..take).enumerate() {
                        mix[i] += s * g;
                    }
                    if take < n {
                        r.primed = false; // underrun: wait for a fresh 30 ms
                    }
                }
            }
        }
        let (mut pl, mut pr) = (0f32, 0f32);
        bytes.clear();
        bytes.reserve(n * 4);
        for (i, s) in mix.iter().enumerate() {
            let y = limit(*s);
            if i % 2 == 0 { pl = pl.max(y.abs()) } else { pr = pr.max(y.abs()) }
            bytes.extend_from_slice(&y.to_le_bytes());
        }
        // non-negative floats keep their order as u32 bits, so fetch_max works
        sh.peak_l.fetch_max(pl.to_bits(), Ordering::Relaxed);
        sh.peak_r.fetch_max(pr.to_bits(), Ordering::Relaxed);
        rc.write_to_device(frames, &bytes, None)?;
    }
    let _ = ac.stop_stream();
    Ok(())
}
