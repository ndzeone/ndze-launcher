import { useEffect, useState } from "react";
import {
  ArrowRight, BatteryCharging, ChevronDown, CircleCheck, Clock, Cpu, Download, Gamepad2, KeyRound, Lock,
  Menu, MessageSquare, Monitor, Rocket, ShieldCheck, Smartphone, Sparkles, Trash2, Undo2, X, Zap,
  type LucideIcon,
} from "lucide-react";
import { LATEST_URL, RELEASES_URL, REPO, formatSize, useLatestRelease } from "./release";

const BASE = import.meta.env.BASE_URL;

const FEATURES: { icon: LucideIcon; title: string; text: string }[] = [
  { icon: Gamepad2, title: "Библиотека игр", text: "Steam и Epic Games находятся сами, любую игру можно добавить по .exe. Избранное, фильтры и время в игре." },
  { icon: Sparkles, title: "Конфигурация под ваш ПК", text: "Оценивает процессор, видеокарту, память и диски, находит узкое место и подбирает настройки графики." },
  { icon: Zap, title: "Профиль для каждой игры", text: "Дискретная видеокарта, приоритет процесса, схема питания на время игры и закрытие лишних программ." },
  { icon: Monitor, title: "Проверка монитора", text: "Замечает, что 144-герцовый монитор работает на 60 Гц, и переключает частоту одной кнопкой." },
  { icon: Cpu, title: "Мой ПК", text: "CPU, видеокарта с драйвером, планки памяти, SSD/HDD, мониторы и материнская плата — всё по блокам." },
  { icon: BatteryCharging, title: "Схемы питания", text: "Ultimate Performance одной кнопкой, автоматический возврат после игры и удаление дубликатов." },
  { icon: Rocket, title: "Автозагрузка", text: "Отключает программы в автозапуске так же, как Диспетчер задач, — без удаления самих программ." },
  { icon: Trash2, title: "Очистка", text: "Временные файлы, кэш шейдеров NVIDIA/AMD/DirectX и дампы падений с точным размером." },
];

const SCREENS = [
  { id: "library", label: "Библиотека" },
  { id: "config", label: "Конфигурация" },
  { id: "pc", label: "Мой ПК" },
  { id: "tweaks", label: "Оптимизация" },
  { id: "startup", label: "Автозагрузка" },
  { id: "cleanup", label: "Очистка" },
] as const;

const FAQ = [
  {
    q: "Windows пишет «Система Windows защитила ваш компьютер». Это нормально?",
    a: "Да. Установщик не подписан платным сертификатом, поэтому SmartScreen его не знает. Нажмите «Подробнее» → «Выполнить в любом случае». Исходный код открыт — его можно проверить или собрать самостоятельно.",
  },
  {
    q: "Не забанят ли меня в онлайн-играх?",
    a: "Лаунчер не внедряется в игры и не читает их память: он только меняет настройки Windows и запускает игру как обычный ярлык. Некоторые античиты запрещают менять приоритет своего процесса — тогда лаунчер просто сообщит об этом.",
  },
  {
    q: "Можно ли откатить изменения?",
    a: "Да. Перед каждым твиком сохраняется исходное значение реестра, выключение переключателя возвращает его. Автозагрузка и схемы питания меняются штатными механизмами Windows.",
  },
  {
    q: "Зачем запрос прав администратора?",
    a: "Лаунчер работает с правами обычного пользователя. Окно UAC появляется, только когда нужно изменить системный раздел реестра (например, HAGS). Игры при этом не запускаются от администратора — так не ломаются оверлеи Steam и Discord.",
  },
  {
    q: "Насколько вырастет FPS?",
    a: "Честно: зависит от ПК. Больше всего дают исправление частоты монитора, двухканальная память, XMP, правильный апскейлер и схема питания. Лаунчер показывает именно эти вещи и не обещает «+200% FPS».",
  },
];

function useScrolled() {
  const [scrolled, setScrolled] = useState(false);
  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 12);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);
  return scrolled;
}

function DownloadButton({ large }: { large?: boolean }) {
  const release = useLatestRelease();
  const href = release?.installer?.url ?? LATEST_URL;
  return (
    <a className={`btn primary ${large ? "lg" : ""}`} href={href}>
      <Download size={large ? 20 : 17} />
      Скачать для Windows
      {release?.installer && <span className="btn-meta">v{release.version} · {formatSize(release.installer.size)}</span>}
    </a>
  );
}

function Nav() {
  const scrolled = useScrolled();
  const [open, setOpen] = useState(false);
  const links = [
    ["#features", "Возможности"],
    ["#config", "Под ваш ПК"],
    ["#screens", "Скриншоты"],
    ["#ios", "iPhone"],
    ["#faq", "FAQ"],
  ];
  return (
    <header className={`nav ${scrolled ? "scrolled" : ""}`}>
      <div className="container nav-inner">
        <a className="logo" href="#top">
          <span className="logo-mark"><Zap size={17} fill="currentColor" strokeWidth={0} /></span>
          NDZE
        </a>
        <nav className={`nav-links ${open ? "open" : ""}`} onClick={() => setOpen(false)}>
          {links.map(([href, label]) => (
            <a key={href} href={href}>{label}</a>
          ))}
          <a href={`https://github.com/${REPO}`} target="_blank" rel="noreferrer">GitHub</a>
        </nav>
        <div className="nav-cta">
          <a className="btn primary sm" href={LATEST_URL}>
            <Download size={15} /> Скачать
          </a>
          <button className="menu-btn" onClick={() => setOpen((v) => !v)} aria-label="Меню" aria-expanded={open}>
            {open ? <X size={20} /> : <Menu size={20} />}
          </button>
        </div>
      </div>
    </header>
  );
}

function Window({ src, alt }: { src: string; alt: string }) {
  return (
    <div className="window">
      <div className="window-bar">
        <span /><span /><span />
        <em>NDZE Launcher</em>
      </div>
      <img src={src} alt={alt} loading="lazy" width={1440} height={900} />
    </div>
  );
}

function Hero() {
  return (
    <section className="hero" id="top">
      <div className="container hero-inner">
        <div className="hero-copy">
          <span className="pill"><Sparkles size={14} /> Новое в 0.2 — конфигурация под ваш ПК</span>
          <h1>
            Больше FPS.
            <br />
            <span className="gradient">Без магии.</span>
          </h1>
          <p className="lead">
            Бесплатный лаунчер-оптимизатор для Windows 11. Он изучает ваше железо, находит узкое место и включает только те настройки,
            которые реально работают. Никаких «чистильщиков RAM» и обещаний +200% FPS.
          </p>
          <div className="hero-actions">
            <DownloadButton large />
            <a className="btn ghost lg" href="#features">
              Возможности <ArrowRight size={18} />
            </a>
          </div>
          <ul className="hero-notes">
            <li><CircleCheck size={15} /> Windows 10 / 11 · x64</li>
            <li><CircleCheck size={15} /> Открытый исходный код</li>
            <li><CircleCheck size={15} /> Всё откатывается</li>
          </ul>
        </div>
        <div className="hero-shot">
          <Window src={`${BASE}screens/library.webp`} alt="Библиотека игр в NDZE Launcher" />
        </div>
      </div>
    </section>
  );
}

function Stats() {
  const items = [
    ["7", "проверенных твиков Windows"],
    ["1", "окно UAC на все системные правки"],
    ["< 10 МБ", "установщик на Tauri + Rust"],
    ["0", "телеметрии и рекламы"],
  ];
  return (
    <section className="stats container">
      {items.map(([value, label]) => (
        <div key={label} className="stat">
          <strong>{value}</strong>
          <span>{label}</span>
        </div>
      ))}
    </section>
  );
}

function Features() {
  return (
    <section className="section container" id="features">
      <div className="section-head">
        <span className="eyebrow">Возможности</span>
        <h2>Всё для игр — в одном окне</h2>
        <p>Лаунчер не заменяет Steam: он запускает игры через них, а сам отвечает за настройки системы до, во время и после игры.</p>
      </div>
      <div className="features">
        {FEATURES.map(({ icon: Icon, title, text }) => (
          <article key={title} className="feature">
            <span className="feature-icon"><Icon size={22} /></span>
            <h3>{title}</h3>
            <p>{text}</p>
          </article>
        ))}
      </div>
    </section>
  );
}

function ConfigShowcase() {
  const points = [
    ["Найдёт узкое место", "Например, RTX 4070 с 4-ядерным i3: лаунчер скажет, что упор в процессор, и посоветует поднять графику и включить Frame Generation."],
    ["Подберёт настройки графики", "Пресет, текстуры по объёму видеопамяти, DLSS/FSR, Reflex/Anti-Lag и ограничение FPS под частоту монитора."],
    ["Проверит железо", "Одноканальная память, выключенный XMP, игры на HDD, монитор на 60 Гц вместо 144 Гц — и что с этим делать."],
    ["Применит одной кнопкой", "Рекомендованные твики включаются сразу, системные — через одно окно UAC."],
  ];
  return (
    <section className="section showcase" id="config">
      <div className="container showcase-inner">
        <div className="showcase-copy">
          <span className="eyebrow">Конфигурация под ваш ПК</span>
          <h2>Лаунчер знает ваше железо</h2>
          <div className="points">
            {points.map(([title, text], i) => (
              <div key={title} className="point">
                <span className="point-num">{i + 1}</span>
                <div>
                  <h3>{title}</h3>
                  <p>{text}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
        <Window src={`${BASE}screens/config.webp`} alt="Экран «Конфигурация под ваш ПК»" />
      </div>
    </section>
  );
}

function Screens() {
  const [active, setActive] = useState<(typeof SCREENS)[number]["id"]>("library");
  return (
    <section className="section container" id="screens">
      <div className="section-head">
        <span className="eyebrow">Скриншоты</span>
        <h2>Как это выглядит</h2>
      </div>
      <div className="tabs" role="tablist">
        {SCREENS.map((s) => (
          <button key={s.id} role="tab" aria-selected={active === s.id} className={active === s.id ? "active" : ""} onClick={() => setActive(s.id)}>
            {s.label}
          </button>
        ))}
      </div>
      <Window src={`${BASE}screens/${active}.webp`} alt={`Экран «${SCREENS.find((s) => s.id === active)?.label}»`} />
    </section>
  );
}

function Safety() {
  const items: { icon: LucideIcon; title: string; text: string }[] = [
    { icon: Undo2, title: "Всё обратимо", text: "Исходные значения реестра сохраняются перед изменением и возвращаются по одному переключателю." },
    { icon: ShieldCheck, title: "Минимум прав", text: "Права администратора запрашиваются только для системных ключей. Игры запускаются от вашего пользователя." },
    { icon: Lock, title: "Без внедрения в игры", text: "Никаких инъекций, оверлеев и чтения памяти — античитам не на что ругаться." },
  ];
  return (
    <section className="section container">
      <div className="section-head">
        <span className="eyebrow">Безопасность</span>
        <h2>Честная оптимизация</h2>
      </div>
      <div className="safety">
        {items.map(({ icon: Icon, title, text }) => (
          <article key={title} className="safety-item">
            <Icon size={24} />
            <h3>{title}</h3>
            <p>{text}</p>
          </article>
        ))}
      </div>
    </section>
  );
}

function Phone() {
  return (
    <div className="phone" aria-hidden>
      <div className="phone-notch" />
      <div className="phone-screen">
        <div className="phone-title">
          <strong>Сравни DLSS и FSR</strong>
          <span>mac os · llama-3 ▾</span>
        </div>
        <div className="bubble user">Что лучше для RTX 4070 в 1080p — DLSS или FSR?</div>
        <div className="bubble bot">
          Для RTX 4070 выбирайте <b>DLSS</b>: качество картинки выше, а в 1080p можно ставить <b>DLAA</b> — нативное разрешение со сглаживанием от DLSS.
        </div>
        <div className="bubble user">А Frame Generation включать?</div>
        <div className="bubble bot typing"><span /><span /><span /></div>
        <div className="phone-input">
          <span>Сообщение</span>
          <i />
        </div>
      </div>
    </div>
  );
}

function IOS() {
  const release = useLatestRelease();
  const items: { icon: LucideIcon; text: string }[] = [
    { icon: KeyRound, text: "Подключение по API-ключу: ключ хранится в Keychain и отображается маской" },
    { icon: MessageSquare, text: "OpenAI-совместимые серверы, Anthropic и свой формат JSON" },
    { icon: Sparkles, text: "Потоковые ответы, Markdown, история чатов" },
    { icon: Clock, text: "Проверка подключения с замером задержки" },
  ];
  return (
    <section className="section ios" id="ios">
      <div className="container ios-inner">
        <Phone />
        <div>
          <span className="eyebrow"><Smartphone size={14} /> NDZE AI для iPhone</span>
          <h2>Ваши ИИ в кармане</h2>
          <p className="lead">Клиент для своих ИИ-серверов на SwiftUI. Работает на iPhone 11 и новее (iOS 17+).</p>
          <ul className="checks">
            {items.map(({ icon: Icon, text }) => (
              <li key={text}><Icon size={18} /> {text}</li>
            ))}
          </ul>
          <div className="hero-actions">
            <a className="btn ghost" href={release?.ipa?.url ?? LATEST_URL}>
              <Download size={17} /> Скачать .ipa
            </a>
          </div>
          <p className="fine">
            Приложение не в App Store: .ipa устанавливается через Sideloadly или AltStore с вашим Apple ID. С бесплатным аккаунтом подпись действует 7 дней.
          </p>
        </div>
      </div>
    </section>
  );
}

function Faq() {
  return (
    <section className="section container narrow" id="faq">
      <div className="section-head">
        <span className="eyebrow">FAQ</span>
        <h2>Частые вопросы</h2>
      </div>
      <div className="faq">
        {FAQ.map(({ q, a }) => (
          <details key={q}>
            <summary>
              {q}
              <ChevronDown size={18} />
            </summary>
            <p>{a}</p>
          </details>
        ))}
      </div>
    </section>
  );
}

function Cta() {
  return (
    <section className="cta container">
      <div className="cta-card">
        <h2>Готовы выжать максимум из своего ПК?</h2>
        <p>Установка занимает меньше минуты. Удаляется как обычная программа.</p>
        <div className="hero-actions center">
          <DownloadButton large />
          <a className="btn ghost lg" href={RELEASES_URL}>Все версии</a>
        </div>
      </div>
    </section>
  );
}

export function App() {
  return (
    <>
      <div className="bg-glow" aria-hidden />
      <Nav />
      <main>
        <Hero />
        <Stats />
        <Features />
        <ConfigShowcase />
        <Screens />
        <Safety />
        <IOS />
        <Faq />
        <Cta />
      </main>
      <footer className="footer container">
        <span className="logo small">
          <span className="logo-mark"><Zap size={14} fill="currentColor" strokeWidth={0} /></span>
          NDZE
        </span>
        <span>Сделано на Tauri, Rust, React и SwiftUI · Открытый исходный код</span>
        <a href={`https://github.com/${REPO}`} target="_blank" rel="noreferrer">GitHub</a>
      </footer>
    </>
  );
}
