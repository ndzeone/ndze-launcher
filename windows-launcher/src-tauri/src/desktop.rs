#[tauri::command(async)]
pub fn defender_scan(path: String) -> Result<String, String> {
    #[cfg(not(windows))]
    {
        let _ = path;
        Err("Microsoft Defender доступен в Windows.".into())
    }
    #[cfg(windows)]
    {
        use std::{os::windows::process::CommandExt, path::Path, process::Command};
        let file = Path::new(&path);
        if !file.is_absolute() || !file.is_file() {
            return Err("Выберите существующий файл.".into());
        }
        let root = std::env::var_os("ProgramW6432")
            .or_else(|| std::env::var_os("ProgramFiles"))
            .ok_or("Не найдена папка Program Files")?;
        let tool = Path::new(&root).join(r"Windows Defender\MpCmdRun.exe");
        let result = Command::new(tool)
            .args(["-Scan", "-ScanType", "3", "-File"])
            .arg(file)
            .arg("-DisableRemediation")
            .creation_flags(0x08000000)
            .status()
            .map_err(|e| format!("Не удалось запустить Microsoft Defender: {e}"))?;
        match result.code() {
            Some(0) => Ok("Defender завершил проверку. Угроз не обнаружено.".into()),
            Some(2) => Err("Обнаружена угроза или проверка не завершилась. Откройте журнал защиты Windows.".into()),
            _ => Err(format!("Ошибка Microsoft Defender: {result}")),
        }
    }
}

#[derive(serde::Serialize)]
pub struct Track {
    title: String,
    artist: String,
    playing: bool,
}

#[tauri::command(async)]
pub fn spotify(action: String) -> Result<Option<Track>, String> {
    #[cfg(not(windows))]
    {
        let _ = action;
        Err("Синхронизация доступна в Windows.".into())
    }
    #[cfg(windows)]
    media(&action).map_err(|e| e.to_string())
}

#[cfg(windows)]
fn media(action: &str) -> windows::core::Result<Option<Track>> {
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    };
    let sessions = Manager::RequestAsync()?.get()?.GetSessions()?;
    for index in 0..sessions.Size()? {
        let session = sessions.GetAt(index)?;
        let id = session.SourceAppUserModelId()?.to_string().to_ascii_lowercase();
        if !(id == "spotify.exe" || id == "spotify" || id.starts_with("spotifyab.spotifymusic_")) {
            continue;
        }
        let ok = match action {
            "status" => true,
            "toggle" => session.TryTogglePlayPauseAsync()?.get()?,
            "next" => session.TrySkipNextAsync()?.get()?,
            "previous" => session.TrySkipPreviousAsync()?.get()?,
            _ => return Err(windows::core::Error::from_hresult(windows::core::HRESULT(0x80070057u32 as i32))),
        };
        if !ok {
            return Err(windows::core::Error::new(windows::core::HRESULT(0x80004005u32 as i32), "Spotify отклонил команду"));
        }
        let info = session.TryGetMediaPropertiesAsync()?.get()?;
        return Ok(Some(Track {
            title: info.Title()?.to_string(),
            artist: info.Artist()?.to_string(),
            playing: session.GetPlaybackInfo()?.PlaybackStatus()? == Status::Playing,
        }));
    }
    Ok(None)
}
