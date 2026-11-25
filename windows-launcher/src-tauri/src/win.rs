//! Thin safe wrappers over the Win32 calls the launcher needs.

use std::os::windows::process::CommandExt;
use std::process::Command;

use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, HANDLE};
use windows::Win32::Globalization::{MultiByteToWideChar, CP_OEMCP, MULTI_BYTE_TO_WIDE_CHAR_FLAGS};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
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
