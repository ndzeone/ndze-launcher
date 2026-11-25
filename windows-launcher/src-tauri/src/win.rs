//! Thin safe wrappers over the Win32 calls the launcher needs.

use std::os::windows::process::CommandExt;
use std::process::Command;

use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, HANDLE};
use windows::Win32::Globalization::{MultiByteToWideChar, CP_OEMCP, MULTI_BYTE_TO_WIDE_CHAR_FLAGS};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
};
use windows::Win32::Graphics::Gdi::{
    ChangeDisplaySettingsExW, EnumDisplayDevicesW, EnumDisplaySettingsW, CDS_TEST,
    CDS_UPDATEREGISTRY, DEVMODEW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP,
    DISPLAY_DEVICE_PRIMARY_DEVICE, DISP_CHANGE_RESTART, DISP_CHANGE_SUCCESSFUL,
    DM_DISPLAYFREQUENCY, ENUM_CURRENT_SETTINGS, ENUM_DISPLAY_SETTINGS_MODE,
};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetExitCodeProcess, OpenProcess, OpenProcessToken, SetPriorityClass,
    WaitForSingleObject, ABOVE_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS, INFINITE,
    NORMAL_PRIORITY_CLASS, PROCESS_SET_INFORMATION,
};
use windows::Win32::UI::Shell::{
    ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
};
use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNORMAL};

use crate::error::{fail, AppError, AppResult};
use crate::hardware::DisplayInfo;
use crate::store::Priority;
use crate::system::GpuInfo;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

pub fn decode_oem(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    unsafe {
        let flags = MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0);
        let len = MultiByteToWideChar(CP_OEMCP, flags, bytes, None);
        if len <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        let mut buf = vec![0u16; len as usize];
        MultiByteToWideChar(CP_OEMCP, flags, bytes, Some(&mut buf));
        String::from_utf16_lossy(&buf)
    }
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut TOKEN_ELEVATION as *mut _),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}

/// ShellExecuteEx handles URIs (steam://), elevation manifests of games and
/// the "runas" verb, which plain CreateProcess does not.
pub fn shell_execute(
    verb: &str,
    file: &str,
    params: Option<&str>,
    dir: Option<&str>,
    show: bool,
    wait: bool,
) -> AppResult<Option<u32>> {
    let verb = HSTRING::from(verb);
    let file = HSTRING::from(file);
    let params = params.map(HSTRING::from);
    let dir = dir.map(HSTRING::from);
    let ptr = |h: &Option<HSTRING>| h.as_ref().map_or(PCWSTR::null(), |h| PCWSTR(h.as_ptr()));

    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: ptr(&params),
        lpDirectory: ptr(&dir),
        nShow: if show { SW_SHOWNORMAL.0 } else { SW_HIDE.0 },
        ..Default::default()
    };

    unsafe { ShellExecuteExW(&mut info) }.map_err(|e| {
        if e.code() == ERROR_CANCELLED.to_hresult() {
            AppError::Msg("Действие отменено в окне UAC".into())
        } else {
            AppError::Msg(format!("Не удалось запустить {file}: {}", e.message()))
        }
    })?;

    let process = info.hProcess;
    if process.is_invalid() {
        return Ok(None);
    }
    let code = if wait {
        unsafe {
            WaitForSingleObject(process, INFINITE);
            let mut code = 0u32;
            GetExitCodeProcess(process, &mut code).ok().map(|_| code)
        }
    } else {
        None
    };
    unsafe {
        let _ = CloseHandle(process);
    }
    Ok(code)
}

pub fn set_priority(pid: u32, priority: Priority) -> AppResult<()> {
    let class = match priority {
        Priority::Normal => NORMAL_PRIORITY_CLASS,
        Priority::AboveNormal => ABOVE_NORMAL_PRIORITY_CLASS,
        Priority::High => HIGH_PRIORITY_CLASS,
    };
    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid).or_else(|e| {
            fail(format!(
                "Нет доступа к процессу {pid} (возможно, его защищает античит): {}",
                e.message()
            ))
        })?;
        let result = SetPriorityClass(handle, class);
        let _ = CloseHandle(handle);
        result.or_else(|e| fail(format!("SetPriorityClass: {}", e.message())))
    }
}

pub fn gpus() -> Vec<GpuInfo> {
    let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for index in 0.. {
        let Ok(adapter) = (unsafe { factory.EnumAdapters1(index) }) else {
            break;
        };
        let Ok(desc) = (unsafe { adapter.GetDesc1() }) else {
            continue;
        };
        if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
            continue;
        }
        let end = desc
            .Description
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(desc.Description.len());
        result.push(GpuInfo {
            name: String::from_utf16_lossy(&desc.Description[..end])
                .trim()
                .to_owned(),
            vram: desc.DedicatedVideoMemory as u64,
        });
    }
    result
}

fn wide_to_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end]).trim().to_owned()
}

fn devmode() -> DEVMODEW {
    DEVMODEW {
        dmSize: size_of::<DEVMODEW>() as u16,
        ..Default::default()
    }
}

fn current_mode(device: &HSTRING) -> Option<DEVMODEW> {
    let mut mode = devmode();
    unsafe { EnumDisplaySettingsW(device, ENUM_CURRENT_SETTINGS, &mut mode) }
        .as_bool()
        .then_some(mode)
}

/// Refresh rates supported at the current resolution and colour depth.
fn supported_rates(device: &HSTRING, current: &DEVMODEW) -> Vec<u32> {
    let mut rates = Vec::new();
    for index in 0.. {
        let mut mode = devmode();
        if !unsafe { EnumDisplaySettingsW(device, ENUM_DISPLAY_SETTINGS_MODE(index), &mut mode) }
            .as_bool()
        {
            break;
        }
        if mode.dmPelsWidth == current.dmPelsWidth
            && mode.dmPelsHeight == current.dmPelsHeight
            && mode.dmBitsPerPel == current.dmBitsPerPel
            && !rates.contains(&mode.dmDisplayFrequency)
        {
            rates.push(mode.dmDisplayFrequency);
        }
    }
    rates.sort_unstable();
    rates
}

pub fn displays() -> Vec<DisplayInfo> {
    let mut result = Vec::new();
    for index in 0.. {
        let mut adapter = DISPLAY_DEVICEW {
            cb: size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        if !unsafe { EnumDisplayDevicesW(PCWSTR::null(), index, &mut adapter, 0) }.as_bool() {
            break;
        }
        if !adapter
            .StateFlags
            .contains(DISPLAY_DEVICE_ATTACHED_TO_DESKTOP)
        {
            continue;
        }
        let device = wide_to_string(&adapter.DeviceName);
        let device_w = HSTRING::from(device.as_str());
        let Some(current) = current_mode(&device_w) else {
            continue;
        };

        let mut monitor = DISPLAY_DEVICEW {
            cb: size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        let name = if unsafe { EnumDisplayDevicesW(&device_w, 0, &mut monitor, 0) }.as_bool() {
            wide_to_string(&monitor.DeviceString)
        } else {
            device.clone()
        };

        let rates = supported_rates(&device_w, &current);
        result.push(DisplayInfo {
            name,
            width: current.dmPelsWidth,
            height: current.dmPelsHeight,
            refresh: current.dmDisplayFrequency,
            max_refresh: rates.last().copied().unwrap_or(current.dmDisplayFrequency),
            rates,
            primary: adapter.StateFlags.contains(DISPLAY_DEVICE_PRIMARY_DEVICE),
            device,
        });
    }
    result
}

pub fn set_refresh_rate(device: &str, hz: u32) -> AppResult<()> {
    let device_w = HSTRING::from(device);
    let Some(mut mode) = current_mode(&device_w) else {
        return fail(format!("Монитор {device} не найден"));
    };
    if !supported_rates(&device_w, &mode).contains(&hz) {
        return fail(format!(
            "Монитор не поддерживает {hz} Гц при текущем разрешении"
        ));
    }
    mode.dmDisplayFrequency = hz;
    mode.dmFields = DM_DISPLAYFREQUENCY;
    unsafe {
        if ChangeDisplaySettingsExW(&device_w, Some(&mode), None, CDS_TEST, None)
            != DISP_CHANGE_SUCCESSFUL
        {
            return fail("Windows отклонил этот режим монитора");
        }
        match ChangeDisplaySettingsExW(&device_w, Some(&mode), None, CDS_UPDATEREGISTRY, None) {
            DISP_CHANGE_SUCCESSFUL => Ok(()),
            DISP_CHANGE_RESTART => fail("Частота сохранена, применится после перезагрузки"),
            other => fail(format!("Не удалось сменить частоту (код {})", other.0)),
        }
    }
}

/// Details that only WMI exposes (RAM modules, GPU driver, board, chassis),
/// fetched in one PowerShell call because it takes about a second.
pub fn cim_details() -> Option<serde_json::Value> {
    const SCRIPT: &str = "[Console]::OutputEncoding=[Text.Encoding]::UTF8;\
        $m=@(Get-CimInstance Win32_PhysicalMemory | Select-Object Capacity,Speed,ConfiguredClockSpeed,Manufacturer,PartNumber,SMBIOSMemoryType,DeviceLocator);\
        $g=@(Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion);\
        $b=Get-CimInstance Win32_BaseBoard | Select-Object Manufacturer,Product;\
        $os=Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,BuildNumber,OSArchitecture;\
        $c=Get-CimInstance Win32_Processor | Select-Object -First 1 Name,MaxClockSpeed,NumberOfCores,L2CacheSize,L3CacheSize;\
        $cs=Get-CimInstance Win32_ComputerSystem | Select-Object PCSystemType;\
        @{memory=$m;gpus=$g;board=$b;os=$os;cpu=$c;system=$cs} | ConvertTo-Json -Depth 4 -Compress";
    let output = hidden_command("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            SCRIPT,
        ])
        .output()
        .ok()?;
    let bytes = output
        .stdout
        .strip_prefix(&[0xEF, 0xBB, 0xBF])
        .unwrap_or(&output.stdout);
    serde_json::from_slice(bytes).ok()
}
