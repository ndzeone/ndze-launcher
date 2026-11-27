<div align="center">

# NDZE

**Gaming utilities for Windows and a flexible AI client for iOS**

![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)
![Swift](https://img.shields.io/badge/Swift-SwiftUI-F05138?logo=swift&logoColor=white)

[Website](https://ndzeone.github.io/ndze-launcher/) · [Releases](https://github.com/ndzeone/ndze-launcher/releases) · [Report an issue](https://github.com/ndzeone/ndze-launcher/issues) · [Русская документация](#русская-документация)

</div>

## About

NDZE is an independently maintained, early-stage software project combining a Windows gaming launcher, an iOS client for user-configured AI services, and a project website. The goal is to make game management, transparent system configuration and access to AI APIs more convenient.

| Component | Purpose | Technology |
| --- | --- | --- |
| [Windows launcher](windows-launcher) | Game library and Windows 11 optimization | Rust, Tauri 2, React 19, TypeScript, Vite |
| [iOS app](ios-app) | AI chat client for custom API endpoints | SwiftUI, SwiftData, Keychain |
| [Website](website) | Project website | React, TypeScript, Vite |

## Windows launcher

- Detect Steam and Epic Games libraries and add executables manually.
- Manage favorites, playtime, launch counts and per-game profiles.
- Configure GPU preferences, power plans and process priorities.
- Inspect hardware, startup applications and temporary files.
- Restore saved Windows tweak settings when toggles are disabled.

Performance improvements vary by hardware and workload. The launcher is designed for Windows 11; Linux UI development uses stubs for Windows-specific functionality.

## NDZE AI for iOS

- OpenAI-compatible Chat Completions, Anthropic Messages and configurable custom API formats.
- Multiple server connections, streamed responses, Markdown and local chat history.
- API keys stored in the device Keychain.
- iOS 17+; a compatible iPhone is required.

Users supply their own API endpoints and credentials. API access and any associated charges are separate from the app.

## Getting started

Windows development requires Rust, Node.js 22+, Visual Studio C++ Build Tools and WebView2.

```bash
git clone https://github.com/ndzeone/ndze-launcher.git
cd ndze-launcher/windows-launcher
npm ci
npm run tauri dev
```

Build with `npm run tauri build`. For iOS development, use macOS, Xcode and XcodeGen. See the detailed Russian documentation below for build, signing and release instructions.

## Security and development

The project aims to minimize elevated privileges, preserve reversible settings and protect API credentials. These are design goals and documented behaviors, not a guarantee of security. Please avoid sharing credentials or sensitive diagnostics in public issues.

## Roadmap

Planned priorities:

- Improve automated tests and build reliability.
- Review dependencies, privileged operations and credential handling.
- Improve documentation and accessibility.
- Gather feedback and prepare stable releases.

## Feedback and licensing

Bug reports and suggestions are welcome through [GitHub Issues](https://github.com/ndzeone/ndze-launcher/issues). Include your platform, application version and steps to reproduce, without secrets or private information.

The repository is publicly visible, but an open-source license has not yet been selected. Public visibility alone does not grant permission to redistribute or modify the code.

---

## Русская документация

**Сайт:** https://ndzeone.github.io/ndze-launcher/ · **Скачать:** [последний релиз](https://github.com/ndzeone/ndze-launcher/releases/latest)

| Папка | Что это | Стек |
|---|---|---|
| [`windows-launcher/`](windows-launcher) | Лаунчер-оптимизатор игр для Windows 11 | Tauri 2 · Rust · React 19 · TypeScript · Vite |
| [`ios-app/`](ios-app) | Клиент для своих ИИ по API-ключу (iPhone 11 и новее, iOS 17+) | SwiftUI · SwiftData · Keychain |
| [`website/`](website) | Сайт проекта на GitHub Pages | React · TypeScript · Vite |

### Как выпустить новую версию

1. Поднять версию в `windows-launcher/package.json`, `src-tauri/Cargo.toml` и `src-tauri/tauri.conf.json`.
2. Написать заметки в `.github/releases/vX.Y.Z.md`.
3. Запушить тег — workflow `Release` соберёт установщики для Windows и `.ipa` и опубликует релиз:

```bash
git tag v0.3.0 && git push origin v0.3.0
```

Сайт пересобирается сам при изменениях в `website/` и всегда ведёт на последний релиз.

---

## Windows-лаунчер

### Что умеет

**Библиотека**
- Автоматически находит игры **Steam** (все библиотеки из `libraryfolders.vdf`) и **Epic Games** (манифесты `.item`), можно добавить любой `.exe` вручную.
- Время в игре, число запусков, избранное, фильтры по магазину, сортировка, баннер «Продолжить игру».
- Для каждой игры — профиль:
  - **Высокопроизводительный GPU** — пишет `GpuPreference=2` в `HKCU\Software\Microsoft\DirectX\UserGpuPreferences` (то же, что «Параметры → Дисплей → Графика»).
  - **Схема питания на время игры** — переключает на «Высокую»/«Максимальную производительность», после выхода из игры возвращает прежнюю.
  - **Приоритет процесса** — `SetPriorityClass` после того, как процесс игры появился.
  - **Закрытие фоновых процессов** перед запуском (системные процессы, Steam и Epic защищены от закрытия).
  - **Отключение FSO** — флаг `DISABLEDXMAXIMIZEDWINDOWEDMODE` (как галочка на вкладке «Совместимость»).
- Лаунчер отслеживает процесс игры по пути к exe — так работают приоритет и возврат схемы питания.

**Оптимизация Windows** — только настройки с реальным эффектом:

| Твик | Эффект | Права | Перезапуск |
|---|---|---|---|
| Игровой режим | заметный | — | — |
| Отключить фоновую запись Game DVR | заметный | — | — |
| Оптимизации для игр в окне (flip model) | заметный | — | — |
| HAGS (аппаратное планирование GPU) | заметный | UAC | перезагрузка |
| Отключить Power Throttling | небольшой | UAC | перезагрузка |
| Приоритет MMCSS для игр | небольшой | UAC | перезагрузка |
| Отключить акселерацию мыши | на аим, не на FPS | — | перевход |

Перед изменением исходные значения сохраняются в `%APPDATA%\one.ndze.launcher\config.json`, выключение переключателя возвращает их обратно.

**Мой ПК** — железо по блокам: CPU (ядра, частота, кэш, нагрузка по потокам), видеокарта (VRAM, драйвер, DLSS/FSR/Reflex), планки RAM (слот, частота, производитель), SSD/HDD, мониторы с переключением частоты, материнская плата, ОС. Данные — из DXGI, WinAPI (`EnumDisplaySettings`) и WMI.

**Конфигурация под ваш ПК** — эвристическая оценка компонентов (не бенчмарк), поиск узкого места, настройки графики под конкретную связку CPU/GPU/монитор и список улучшений: монитор не на максимальной частоте, одноканальная память, выключенный XMP/EXPO, игры на HDD, мало места на C:. Рекомендованные твики применяются одной кнопкой.

**Автозагрузка** — как в Диспетчере задач: флаг `StartupApproved\Run`, программа не удаляется.

**Очистка** — `%TEMP%`, кэш шейдеров NVIDIA/AMD/DirectX, дампы падений; занятые файлы пропускаются.

**Схемы питания** — переключение, Ultimate Performance без повторного создания копий, удаление дубликатов.

### Как устроены права администратора

Сам лаунчер работает **без прав администратора**. Когда нужен `HKLM`, он один раз показывает окно UAC и запускает `reg.exe` с повышенными правами только для этих ключей. Почему так:
- игры, запущенные из лаунчера, не наследуют права администратора (иначе ломаются оверлеи Steam/Discord и сохранения);
- в скрипт для `reg.exe` попадают только значения из встроенного каталога, проверенные по белому списку символов, — подменённый `config.json` не может выполнить команды от имени администратора.

### Сборка

Готовый установщик собирает GitHub Actions (`.github/workflows/windows-launcher.yml`): **Actions → Windows launcher → артефакт `ndze-launcher-windows`** (`.exe` NSIS и `.msi`).

Локально на Windows нужны [Rust](https://rustup.rs), Node.js 22+, Visual Studio Build Tools (C++), WebView2 (в Windows 11 уже есть):

```bash
cd windows-launcher
npm ci
npm run tauri dev
```

```bash
npm run tauri build
```

Разработка интерфейса на CachyOS тоже работает (`npm run tauri dev`), но Windows-функции там отключены (заглушки). Проверить, что Rust-часть компилируется под Windows, можно прямо с Linux:

```bash
rustup target add x86_64-pc-windows-msvc
```

```bash
cd windows-launcher/src-tauri && cargo clippy --target x86_64-pc-windows-msvc -- -D warnings
```

### Подводные камни

- **Античиты** (EAC, BattlEye, Vanguard) часто запрещают менять приоритет защищённого процесса — лаунчер покажет понятную ошибку, остальной профиль работает.
- Для Steam-игр exe выбирается автоматически (самый большой файл в папке игры). Если игра запускается через отдельный лаунчер (например, Rockstar/Ubisoft), укажите в профиле **реальный** exe игры.
- **FSO** в Windows 11 обычно лучше не трогать: современная flip-модель даёт ту же задержку, что и эксклюзивный fullscreen.
- **Ultimate Performance** на ноутбуке от батареи заметно сокращает время работы.

---

## iOS-приложение

### Что умеет

- Несколько подключений: название (например, «mac os»), адрес сервера, API-ключ.
- Ключ хранится в **Keychain** (`AfterFirstUnlockThisDeviceOnly` — не попадает в iCloud-бэкапы), в интерфейсе показывается маской `tc_live_****************1b33`.
- Форматы API:
  - **OpenAI-совместимый** — `POST /v1/chat/completions`, `Authorization: Bearer <ключ>`, потоковый ответ (SSE). Подходит для OpenRouter, vLLM, LM Studio, Ollama, LiteLLM и большинства своих серверов.
  - **Anthropic Messages** — `POST /v1/messages`, `x-api-key`.
  - **Свой формат** — путь, заголовок и префикс ключа, JSON-шаблон тела и путь к тексту ответа настраиваются в приложении.
- История чатов (SwiftData), стриминг ответа с кнопкой «Стоп», Markdown, копирование, выбор подключения прямо в чате, кнопка «Проверить подключение» с задержкой в мс.

### Настройка своего сервера

Если сервер не OpenAI-совместимый, выберите **«Свой формат» → «Дополнительно»**. Пример: сервер принимает `{"prompt": "...", "history": [...]}`, ключ в `X-API-Key`, отвечает `{"data": {"reply": "..."}}`:

| Поле | Значение |
|---|---|
| Путь | `/api/chat` |
| Заголовок ключа | `X-API-Key` |
| Префикс ключа | *(пусто)* |
| Тело запроса | `{"prompt": {{prompt}}, "history": {{messages}}}` |
| Путь к ответу | `data.reply` |

Плейсхолдеры подставляются как готовые JSON-значения (строки уже в кавычках, спецсимволы экранированы).

### Сборка и установка на iPhone

Swift-приложение собирается только на macOS с Xcode. Варианты:

**1. Без Mac (через CI + sideload).** GitHub Actions (`.github/workflows/ios-app.yml`) прогоняет тесты и собирает неподписанный `NDZEAI-unsigned.ipa` (**Actions → iOS app → артефакт**). Его можно поставить на iPhone через [Sideloadly](https://sideloadly.io) (Windows/macOS) или AltStore с бесплатным Apple ID. Ограничение Apple для бесплатного аккаунта: подпись действует 7 дней, потом нужно переподписать.

**2. С Mac.**

```bash
brew install xcodegen
```

```bash
cd ios-app && xcodegen generate && open NDZEAI.xcodeproj
```

В Xcode: *Signing & Capabilities → Team* → свой Apple ID, подключите iPhone, ⌘R.

### Подводные камни

- **HTTP без TLS** разрешён только для локальной сети (`NSAllowsLocalNetworking`). Для сервера в интернете нужен HTTPS.
- Если сервер не поддерживает SSE, выключите «Потоковый ответ» в разделе «Дополнительно».
- Ключ не синхронизируется между устройствами — так задумано.
