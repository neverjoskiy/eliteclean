# 🛡️ NoxumClean

**NoxumClean** — это инструмент для очистки следов активности в Windows с веб-интерфейсом в тёмных тонах.

![Python](https://img.shields.io/badge/Python-3.10+-3776AB?style=for-the-badge&logo=python&logoColor=white)
![FastAPI](https://img.shields.io/badge/FastAPI-0.104+-009688?style=for-the-badge&logo=fastapi&logoColor=white)
![PyInstaller](https://img.shields.io/badge/PyInstaller-6.16-1F8476?style=for-the-badge&logo=python&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)

---

## ✨ Особенности

- 🎨 **Красивый веб-интерфейс** — современный дизайн в тёмных тонах
- 🛠️ **Набор инструментов** — очистка следов системы, USN журнала, браузеров
- 📦 **Портативность** — работает из одного .exe файла
- 🌐 **Веб-доступ** — управление через браузер или встроенное окно

---

## 🔧 Инструменты

| Инструмент | Описание |
|------------|----------|
| **Чистка строк** | Очистка и пересоздание USN журнала |
| **Очистка следов** | Удаление ShellBag, Explorer, Prefetch, Minidump |
| **Симуляция папок** | Запуск simulate.exe для симуляции активности |
| **Глобальная очистка** | Комплексная очистка с выбором компонентов |

#### Глобальная очистка включает:
- ☑️ Event Log — логи Windows (Security, System, Application)
- ☑️ $MFT — Master File Table (Prefetch)
- ☑️ Amcache — следы запуска программ
- ☑️ Jump Lists — последние документы
- ☑️ Recent Files — история открытых файлов
- ☑️ Browser History — история браузеров (Chrome, Firefox, Edge)
- ☑️ USN Journal — журнал изменений NTFS
- ☑️ Temp Files — временные файлы

---

## 📥 Установка

### Вариант 1: Готовая сборка

1. Скачайте последний релиз из раздела [Releases](https://github.com/neverjoskiy/popup/releases)
2. Распакуйте архив в любую папку
3. Запустите `NoxumClean.exe`

### Вариант 2: Сборка из исходников

#### Требования
- Python 3.10+
- pip

#### Установка зависимостей

```bash
pip install -r requirements.txt
```

#### Сборка через PyInstaller

```bash
pyinstaller --onefile --windowed ^
  --name "NoxumClean" ^
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

Готовый файл: `dist/NoxumClean.exe`

---

## 🚀 Использование

### Запуск

```bash
# Из исходников
python main.py

# Или готовый exe
NoxumClean.exe
```

### Аргументы командной строки

```bash
python main.py --host 127.0.0.1 --port 8765
```

| Аргумент | Описание | По умолчанию |
|----------|----------|--------------|
| `--host` | Хост сервера | `127.0.0.1` |
| `--port` | Порт сервера | `8765` |

---

## 📁 Структура проекта

```
web/
├── main.py                 # Главный файл
├── requirements.txt        # Зависимости
├── scripts/               # Скрипты инструментов
│   ├── вирус.bat
│   ├── не вирус.bat
│   ├── винлокер.bat
│   └── simulate.exe
├── templates/             # HTML шаблоны
│   └── index.html
├── static/                # Статические файлы
│   ├── css/
│   │   └── styles.css
│   └── js/
│       └── app.js
└── logs/                  # Логи (создаётся автоматически)
    └── app.log
```

---

## 🛠️ Технологии

- **Backend:** FastAPI, Uvicorn
- **Frontend:** HTML5, CSS3, Vanilla JavaScript
- **Desktop:** pywebview
- **Сборка:** PyInstaller
- **Дизайн:** Dark Minimalist Theme

---

## ⚠️ Предупреждение

> Приложение требует **прав администратора** для работы.
> 
> Используйте **на свой страх и риск**. Авторы не несут ответственности за неправильное использование.

---

## 📝 Лицензия

MIT License — см. файл [LICENSE](LICENSE) для деталей.

---

## 📧 Контакты

- **GitHub:** [@neverjoskiy](https://github.com/neverjoskiy)
- **Telegram:** [@neverjoskiy](https://t.me/neverjoskiy)

---

<div align="center">

**Made with ❤️ by neverjoskiy**

</div>
