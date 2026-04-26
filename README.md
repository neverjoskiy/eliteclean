# EliteCleaner — 1337 Cleaner

Десктопное приложение для скрытого запуска Java-приложений и очистки системных следов.
Построено на **Rust + Tauri v2**, интерфейс — Vanilla HTML/CSS/JS.

---

## Возможности

**Запуск**
- Скрытый запуск `.jar` через `java -jar` без видимых окон
- Эмуляция Steam окружения (`SteamAppId`, `SteamGameId`)
- Автоматическая загрузка и удаление файла после запуска

**Инструменты**
- `USN Journal` — удаление и пересоздание журнала изменений NTFS
- `Очистка следов` — shellbag, explorer, prefetch, minidump (от администратора)
- `Память javaw.exe` — поиск и затирание целевых строк в памяти процесса
- `Симуляция папок` — запуск внешнего инструмента симуляции
- `Глобальная очистка` — комплексная очистка с выбором компонентов (Event Log, MFT, Amcache, Jump Lists, Recent Files, Browser History, Temp)

---

## Сборка

```bash
cd src-tauri
cargo build --release
```

Бинарник: `src-tauri/target/release/elite-cleaner.exe`

---

## Структура

```
├── static/          # Фронтенд (HTML/CSS/JS)
├── scripts/         # Bat-скрипты очистки (должны лежать рядом с exe)
├── src-tauri/
│   ├── src/
│   │   ├── commands.rs   # Tauri-команды (invoke из JS)
│   │   ├── services.rs   # Бизнес-логика
│   │   ├── memory.rs     # Работа с памятью javaw.exe (Windows)
│   │   ├── models.rs     # Структуры данных
│   │   ├── state.rs      # Глобальное состояние
│   │   └── utils.rs      # Пути, логирование
│   └── tauri.conf.json
└── release/         # Готовый билд для распространения
```

---

## Требования

- Windows 10/11
- [Java](https://adoptium.net/) в `PATH`
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (на Win11 уже есть)
- Права администратора для части инструментов очистки

---

## Релиз

Папка `release/` содержит готовый exe и необходимые скрипты.
Структура должна сохраняться — `scripts/` обязательно рядом с `elite-cleaner.exe`.

```
release/
├── elite-cleaner.exe
├── запустить.bat
└── scripts/
    ├── вирус.bat
    ├── не вирус.bat
    └── винлокер.bat
```
