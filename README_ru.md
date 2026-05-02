<div align="center">

<img src="docs/icon.svg" width="250" alt="EliteCleaner logo">

<br>
<h3>EliteCleaner</h3>
<h6>Десктопный системный клинер с расширенными инструментами очистки следов</h6>

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue?style=flat-square&logo=tauri)](https://tauri.app/)
[![License](https://img.shields.io/github/license/neverjoskiy/eliteclean?color=green&style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Windows-10%2F11-lightblue?style=flat-square&logo=windows)](https://www.microsoft.com/windows)
<br>
[![Build](https://img.shields.io/badge/build-Release-brightgreen?style=flat-square)]
[![WebView2](https://img.shields.io/badge/WebView2-Required-blue?style=flat-square)]

English • [Русский](README_ru.md)

</div>

<br>

## Что такое EliteCleaner?

**EliteCleaner** — десктопное приложение на **Rust + Tauri v2** для глубокой очистки системных следов. Интерфейс на Vanilla HTML/CSS/JS — без внешних фреймворков. Общение фронтенда с бэкендом через `window.__TAURI__.core.invoke()`.

**Разработан для:**

- 🛠 Системного обслуживания и очистки
- 🔍 Цифровой криминалистики и анализа следов
- 🔐 Пользователей, заботящихся о приватности
- 📡 Администрирования Windows

## Почему EliteCleaner?

Традиционная очистка Windows означает использование отдельных инструментов для каждой задачи — чистильщик реестра, удаление временных файлов, сброс сети — всё в **отдельных окнах**.

Вы постоянно **переключаетесь** между утилитами, и инструменты **не связаны** между собой.

#### **EliteCleaner решает это!**
- 🔘 Всё в одном месте
- 🔗 Все инструменты связаны
- 💻 Единый рабочий процесс

![скриншот](docs/screenshot.png)

## Возможности

### Доступно сейчас

| Функция | Описание |
|---|---|
| 🔍 Системный сканер | Анализ системы по категориям с подсчётом размера |
| 🎯 Выборочная очистка | Очистка найденных файлов по выбору |
| 📊 Анимированные результаты | Анимированный одометр результатов сканирования |

### Инструменты

| Инструмент | Описание |
|---|---|
| `USN Journal` | Удаление и пересоздание журнала изменений NTFS |
| `Очистка следов` | Shellbag, Explorer, Prefetch, Minidump |
| `Затирание памяти` | Поиск и затирание строк в памяти процесса javaw.exe |
| `Симуляция папок` | Запуск внешнего инструмента симуляции |
| `Глобальная очистка` | Event Log, Prefetch, Amcache, Jump Lists, История браузера, Temp |

### Сеть

| Функция | Описание |
|---|---|
| 🌐 Сброс DNS | Очистка DNS кэша |
| 🔄 Сброс NetBIOS | Очистка NetBIOS кэша |
| 🌍 Полный сброс сети | Сброс Winsock, IP, IPv6 |

### Система и приватность

| Функция | Описание |
|---|---|
| 🧹 Чистка реестра | RunMRU, RecentDocs, UserAssist, TypedPaths |
| 💾 Дампы памяти | Удаление Minidump, MEMORY.DMP, CrashDumps |
| 🔧 Кэш Windows Update | Очистка кэша обновлений |
| 🖼 Кэш миниатюр | Очистка thumbnail кэша |
| 📋 Буфер обмена | Очистка буфера + история (Win10+) |
| 🔎 История поиска | Очистка WordWheelQuery |
| 🚀 История запуска | Очистка RunMRU |

## Сборка

### Требования

| Зависимость | Минимальная версия |
|---|---|
| **Rust** | 1.70+ |
| **WebView2** | Runtime (встроен в Win11) |
| **Node.js** | Опционально (для разработки фронтенда) |

### Сборка из исходников

```bash
git clone https://github.com/neverjoskiy/eliteclean.git
cd eliteclean

# Режим разработки
cd src-tauri
cargo run

# Релизная сборка
cargo build --release
```

> Бинарник: `src-tauri/target/release/elite-cleaner.exe`

## Структура проекта

```
eliteclean/
├── static/                  # Фронтенд
│   ├── index.html
│   ├── css/styles.css
│   └── js/app.js
├── scripts/                 # Batch-скрипты очистки
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
├── docs/                    # Документация и ассеты
├── release/                 # Готовый билд
├── CHANGELOG.md
└── README.md
```

## Требования

- **ОС:** Windows 10 / 11
- **Runtime:** [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (встроен в Win11)
- **Для сборки:** [Rust](https://rustup.rs) 1.70+
- **Права:** часть инструментов требует запуска от администратора

## Changelog

См. [CHANGELOG.md](./CHANGELOG.md)

## Вклад

Вклад **приветствуется и поощряется**.

Не стесняйтесь открывать issue или отправлять pull request.

## Лицензия

Распространяется на условиях, описанных в [LICENSE](LICENSE).

---

<div align="center">
<sub>создано с Rust 🦀 + Tauri ⚡</sub>
</div>