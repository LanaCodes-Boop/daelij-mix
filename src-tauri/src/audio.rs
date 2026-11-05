//! Per-app audio sessions on the default Windows output (Core Audio / WASAPI).
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Session {
    pub pid: u32,
    pub name: String,
    pub peak: f32,
    pub volume: f32,
}

#[derive(Serialize, Clone)]
pub struct Output {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[cfg(windows)]
mod imp {
    use super::*;
    use windows::core::{Interface, PWSTR};
    use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::*;
    use windows::Win32::System::Threading::*;

    fn init() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
    }

    fn enumerator() -> windows::core::Result<IMMDeviceEnumerator> {
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
    }

    fn process_name(pid: u32) -> String {
        if pid == 0 {
            return "System sounds".into();
        }
        unsafe {
            let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
                return format!("Process {pid}");
            };
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let r = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len);
            let _ = CloseHandle(h);
            if r.is_err() {
                return format!("Process {pid}");
            }
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            let file = path.rsplit('\\').next().unwrap_or(&path).to_string();
            let lower = file.to_lowercase();
            if lower.ends_with(".exe") {
                file[..file.len() - 4].to_string()
            } else {
                file
            }
        }
    }

    /// Calls `f` for every live audio session on the default render device.
    fn for_each_session(mut f: impl FnMut(u32, &IAudioSessionControl2)) -> windows::core::Result<()> {
        unsafe {
            let en = enumerator()?;
            let dev = en.GetDefaultAudioEndpoint(eRender, eConsole)?;
            let mgr: IAudioSessionManager2 = dev.Activate(CLSCTX_ALL, None)?;
            let list = mgr.GetSessionEnumerator()?;
            let n = list.GetCount()?;
            for i in 0..n {
                let Ok(ctl) = list.GetSession(i) else { continue };
                if let Ok(state) = ctl.GetState() {
                    if state == AudioSessionStateExpired {
                        continue;
                    }
                }
                let Ok(ctl2) = ctl.cast::<IAudioSessionControl2>() else { continue };
                let pid = ctl2.GetProcessId().unwrap_or(0);
                f(pid, &ctl2);
            }
            Ok(())
        }
    }

    pub fn list_sessions() -> Result<Vec<Session>, String> {
        init();
        let mut out: Vec<Session> = Vec::new();
        for_each_session(|pid, ctl| unsafe {
            let peak = ctl
                .cast::<IAudioMeterInformation>()
                .and_then(|m| m.GetPeakValue())
                .unwrap_or(0.0);
            let volume = ctl
                .cast::<ISimpleAudioVolume>()
                .and_then(|v| v.GetMasterVolume())
                .unwrap_or(1.0);
            if let Some(s) = out.iter_mut().find(|s| s.pid == pid) {
                s.peak = s.peak.max(peak);
            } else {
                out.push(Session { pid, name: process_name(pid), peak, volume });
            }
        })
        .map_err(|e| e.to_string())?;
        Ok(out)
    }

    pub fn set_session_volume(pid: u32, volume: f32) -> Result<(), String> {
        init();
        let v = volume.clamp(0.0, 1.0);
        for_each_session(|p, ctl| unsafe {
            if p == pid {
                if let Ok(sv) = ctl.cast::<ISimpleAudioVolume>() {
                    let _ = sv.SetMasterVolume(v, std::ptr::null());
                }
            }
        })
        .map_err(|e| e.to_string())
    }

    pub fn restore_all() -> Result<(), String> {
        init();
        for_each_session(|_, ctl| unsafe {
            if let Ok(sv) = ctl.cast::<ISimpleAudioVolume>() {
                let _ = sv.SetMasterVolume(1.0, std::ptr::null());
            }
        })
        .map_err(|e| e.to_string())
    }

    fn device_id(dev: &IMMDevice) -> Option<String> {
        unsafe {
            let p = dev.GetId().ok()?;
            let s = p.to_string().ok();
            CoTaskMemFree(Some(p.0 as *const _));
            s
        }
    }

    pub fn list_outputs() -> Result<Vec<Output>, String> {
        init();
        unsafe {
            let en = enumerator().map_err(|e| e.to_string())?;
            let def = en
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .ok()
                .and_then(|d| device_id(&d))
                .unwrap_or_default();
            let col = en
                .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
                .map_err(|e| e.to_string())?;
            let n = col.GetCount().map_err(|e| e.to_string())?;
            let mut out = Vec::new();
            for i in 0..n {
                let Ok(dev) = col.Item(i) else { continue };
                let Some(id) = device_id(&dev) else { continue };
                let name = dev
                    .OpenPropertyStore(STGM_READ)
                    .and_then(|st| st.GetValue(&PKEY_Device_FriendlyName))
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| "Audio device".into());
                out.push(Output { is_default: id == def, id, name });
            }
            Ok(out)
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;
    const MSG: &str = "Audio sessions are only available on Windows";
    pub fn list_sessions() -> Result<Vec<Session>, String> { Err(MSG.into()) }
    pub fn set_session_volume(_pid: u32, _volume: f32) -> Result<(), String> { Err(MSG.into()) }
    pub fn restore_all() -> Result<(), String> { Ok(()) }
    pub fn list_outputs() -> Result<Vec<Output>, String> { Err(MSG.into()) }
}

pub use imp::*;
