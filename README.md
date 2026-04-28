<div align="center">

```
███████╗██╗     ██╗████████╗███████╗
██╔════╝██║     ██║╚══██╔══╝██╔════╝
█████╗  ██║     ██║   ██║   █████╗  
██╔══╝  ██║     ██║   ██║   ██╔══╝  
███████╗███████╗██║   ██║   ███████╗
╚══════╝╚══════╝╚═╝   ╚═╝   ╚══════╝
       C L E A N E R  v1.1
```

**Десктопный системный клинер с расширенными инструментами очистки следов**

![Rust](https://img.shields.io/badge/Rust-1.70+-orange?style=flat-square&logo=rust)
![Tauri](https://img.shields.io/badge/Tauri-v2-blue?style=flat-square&logo=tauri)
![Platform](https://img.shields.io/badge/Windows-10%2F11-lightblue?style=flat-square&logo=windows)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)

</div>

---

## О проекте

EliteCleaner — десктопное приложение на **Rust + Tauri v2** для глубокой очистки системных следов.
Интерфейс на Vanilla HTML/CSS/JS, без внешних фреймворков.
Общение фронтенда с бэкендом через `window.__TAURI__.core.invoke()`.

---

## Возможности

### Сканирование
- Анализ системы по категориям с подсчётом размера
- Выборочная очистка найденных файлов
- Анимированный одометр результатов

### Инструменты
| Инструмент | Описание |
|---|---|
| `USN Journal` | Удаление и пересоздание журнала изменений NTFS |
| `Очистка следов` | Shellbag, Explorer, Prefetch, Minidump |
| `Память javaw.exe` | Поиск и затирание строк в памяти процесса |
| `Симуляция папок` | Запуск внешнего инструмента симуляции |
| `Глобальная очистка` | Event Log, Prefetch, Amcache, Jump Lists, Recent Files, Browser History, Temp |

### Сеть
- Сброс DNS кэша
- Очистка ARP таблицы
- Сброс NetBIOS кэша
- Полный сброс сети (Winsock, IP, IPv6)

### Система
- Очистка реестра (RunMRU, RecentDocs, UserAssist, TypedPaths)
- Удаление дампов памяти (Minidump, MEMORY.DMP, CrashDumps)
- Очистка кэша Windows Update
- Очистка Thumbnail кэша

### Приватность
- Очистка буфера обмена + история (Win10+)
- Кэш иконок
- История поиска (WordWheelQuery)
- История запуска (RunMRU)

---

## Сборка и запуск

```bash
# Запуск в dev-режиме
cd src-tauri
cargo run

# Релизная сборка
cd src-tauri
cargo build --release
```

> Бинарник: `src-tauri/target/release/elite-cleaner.exe`

---

## Структура проекта

```
elitecleaner/
├── static/                  # Фронтенд
│   ├── index.html
│   ├── css/styles.css
│   └── js/app.js
├── scripts/                 # Bat-скрипты очистки
│   ├── вирус.bat            # Удаление USN журнала
│   ├── не вирус.bat         # Создание USN журнала
│   └── винлокер.bat         # Очистка следов (от администратора)
├── src-tauri/
│   └── src/
│       ├── bin/main.rs      # Точка входа, регистрация команд
│       ├── commands.rs      # Tauri-команды (invoke из JS)
│       ├── services.rs      # Бизнес-логика
│       ├── memory.rs        # Работа с памятью javaw.exe
│       ├── models.rs        # Структуры данных (serde)
│       ├── state.rs         # Глобальное состояние AppState
│       └── utils.rs         # Пути, логирование
├── release/                 # Готовый билд
├── CHANGELOG.md
└── README.md
```

---

## Требования

- **OS:** Windows 10 / 11
- **Runtime:** [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (на Win11 уже встроен)
- **Для сборки:** [Rust](https://rustup.rs) 1.70+
- **Права:** часть инструментов требует запуска от администратора

---

## Релиз

Папка `release/` содержит готовый exe и скрипты. Структура должна сохраняться:

```
release/
├── elite-cleaner.exe
└── scripts/
    ├── вирус.bat
    ├── не вирус.bat
    └── винлокер.bat
```

---

## Changelog

См. [CHANGELOG.md](./CHANGELOG.md)

---

<div align="center">
<sub>built with Rust 🦀 + Tauri ⚡</sub>
</div>
