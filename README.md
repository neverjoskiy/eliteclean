<p align="center">
  <img src="https://allwebs.ru/images/2026/03/31/ea5d79c0d75018713d74acd181ba86f4.png" alt="PopUp Cleaner" width="100%" />
</p>

<h1 align="center">✦ PopUp Cleaner</h1>

<p align="center">
  <b>Скрытый запуск с эмуляцией Steam окружения</b><br/>
  <sup>FastAPI • pywebview • USN Clean • Browser History Wipe</sup>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Python-3.10+-3776AB?style=for-the-badge&logo=python&logoColor=white" alt="Python 3.10+" />
  <img src="https://img.shields.io/badge/FastAPI-0.104+-009688?style=for-the-badge&logo=fastapi&logoColor=white" alt="FastAPI" />
  <img src="https://img.shields.io/badge/PyInstaller-6.16-1F8476?style=for-the-badge&logo=python&logoColor=white" alt="PyInstaller" />
  <img src="https://img.shields.io/badge/License-AGPLv3-blue?style=for-the-badge" alt="AGPL-3.0" />
</p>

<br/>
---

<br/>

## ◈ О проекте

**PopUp Cleaner** — это приложение для скрытого запуска с эмуляцией Steam окружения, оснащённое веб-интерфейсом в тёмных тонах и набором инструментов для очистки следов системы.

### Ключевые особенности

| Особенность | Описание |
|:---|:---|
| 🎨 **Dark Web UI** | Современный glassmorphism интерфейс |
| 🔒 **Скрытый запуск** | Запуск приложений без видимых окон |
| 🌐 **Steam Emulation** | Эмуляция SteamAppId и SteamGameId |
| 🧹 **USN Clean** | Очистка и пересоздание USN журнала |
| 🗑️ **Trace Removal** | ShellBag, Prefetch, Minidump, Jump Lists |
| 📦 **Portable EXE** | Работает из одного файла |

<br/>

---

<br/>

## ◈ Функционал

### Инструменты

| Инструмент | Описание |
|:---|:---|
| **Чистка строк** | Очистка и пересоздание USN журнала |
| **Очистка следов** | Удаление ShellBag, Explorer, Prefetch, Minidump |
| **Симуляция папок** | Запуск simulate.exe для симуляции активности |
| **Глобальная очистка** | Комплексная очистка с выбором компонентов |

#### Глобальная очистка включает:

- ☑️ **Event Log** — логи Windows (Security, System, Application)
- ☑️ **$MFT** — Master File Table (Prefetch)
- ☑️ **Amcache** — следы запуска программ
- ☑️ **Jump Lists** — последние документы
- ☑️ **Recent Files** — история открытых файлов
- ☑️ **Browser History** — история браузеров (Chrome, Firefox, Edge)
- ☑️ **USN Journal** — журнал изменений NTFS
- ☑️ **Temp Files** — временные файлы

<br/>

---

<br/>

## ◈ Установка

### Вариант 1: Готовая сборка

1. Скачайте последний релиз из раздела [Releases](https://github.com/neverjoskiy/popup/releases)
2. Распакуйте архив в любую папку
3. Запустите `SteamLauncher.exe`

### Вариант 2: Сборка из исходников

#### Требования

| Компонент | Версия |
|:---|:---|
| **Python** | 3.10+ |
| **pip** | последний |
| **PyInstaller** | 6.16+ |

#### Установка зависимостей

```bash
pip install -r requirements.txt
```

#### Сборка

```bash
pyinstaller --onefile --windowed ^
  --name "SteamLauncher" ^
  --icon=steam.ico ^
  --add-data "static;static" ^
  --add-data "templates;templates" ^
  --add-data "scripts;scripts" ^
  --hidden-import=uvicorn ^
  --hidden-import=fastapi ^
  --hidden-import=webview ^
  --hidden-import=tkinter ^
  --hidden-import=aiofiles ^
  main.py
```

**Результат:** `dist/SteamLauncher.exe`

<br/>

---

<br/>

## ◈ Использование

### Запуск

```bash
# Из исходников
python main.py

# Или готовый exe
SteamLauncher.exe
```

### Аргументы командной строки

| Аргумент | Описание | По умолчанию |
|:---|:---|:---|
| `--host` | Хост сервера | `127.0.0.1` |
| `--port` | Порт сервера | `8765` |

### Инструкция по запуску Doomsday

1. Нажмите кнопку **ℹ** рядом с кнопкой "Запустить"
2. Следуйте инструкции:
   - Выберите "Advanced Inject"
   - Выберите процесс Minecraft
   - Выберите способ "jvmti"
   - Закройте программу после инжекта

> 💡 Используйте **VPN** или **Zapret** если функции не появляются

<br/>

---

<br/>

## ◈ Структура проекта

```
web/
│
├── main.py                 # Главный файл (FastAPI + pywebview)
├── requirements.txt        # Зависимости
├── steam.ico              # Иконка
│
├── scripts/               # Скрипты инструментов
│   ├── вирус.bat
│   ├── не вирус.bat
│   ├── винлокер.bat
│   └── simulate.exe
│
├── templates/             # HTML шаблоны
│   └── index.html
│
├── static/                # Статические файлы
│   ├── css/
│   │   └── styles.css
│   └── js/
│       └── app.js
│
└── logs/                  # Логи (автоматически)
    └── app.log
```

<br/>

---

<br/>

## ◈ Технологии

| Категория | Стек |
|:---|:---|
| **Backend** | FastAPI, Uvicorn |
| **Frontend** | HTML5, CSS3, Vanilla JS |
| **Desktop** | pywebview |
| **Сборка** | PyInstaller |
| **Дизайн** | Glassmorphism Dark Theme |

<br/>

---

<br/>

## ◈ Предупреждение

> Приложение предназначено **только для образовательных целей**.  
> Авторы не несут ответственности за неправильное использование.  
> Используйте на свой страх и риск.

<br/>

---

<br/>

## ◈ Лицензия

MIT License — подробности в файле [LICENSE](LICENSE)

<br/>

---

<br/>

## ◈ Вклад

1. Fork репозиторий
2. Создайте ветку (`git checkout -b feature/AmazingFeature`)
3. Commit изменения (`git commit -m 'Add AmazingFeature'`)
4. Push в ветку (`git push origin feature/AmazingFeature`)
5. Откройте Pull Request

<br/>

---

<br/>

## ◈ Контакты

| Платформа | Ссылка |
|:---|:---|
| **GitHub** | [@neverjoskiy](https://github.com/neverjoskiy) |
| **Telegram** | [@bioneverr](https://t.me/bioneverr) |

<br/>

---

<br/>

<p align="center">
  <sub>✦ PopUp Cleaner — Чисто. Скрыто. Надёжно ✦</sub><br/>
  <sub>Python • FastAPI • pywebview • Glassmorphism UI</sub>
</p>
