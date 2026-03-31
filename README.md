# 🚀 PopUp Cleaner

**PopUp Cleaner** — это приложение для скрытого запуска doomsday с эмуляцией Steam окружения, оснащённое веб-интерфейсом в тёмных тонах и набором инструментов для очистки следов системы.

![Python](https://img.shields.io/badge/Python-3.10+-3776AB?style=for-the-badge&logo=python&logoColor=white)
![FastAPI](https://img.shields.io/badge/FastAPI-0.104+-009688?style=for-the-badge&logo=fastapi&logoColor=white)
![PyInstaller](https://img.shields.io/badge/PyInstaller-6.16-1F8476?style=for-the-badge&logo=python&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)

---

## ✨ Особенности

- 🎨 **Красивый веб-интерфейс** — современный дизайн в тёмных тонах
- 🔒 **Скрытый запуск** — запуск приложений без видимых окон
- 🛠️ **Набор инструментов** — очистка следов системы, USN журнала, браузеров
- 📦 **Портативность** — работает из одного .exe файла
- 🌐 **Веб-доступ** — управление через браузер или встроенное окно

---

## 📸 Скриншоты

### Главная страница
![Главная](https://allwebs.ru/images/2026/03/31/ea5d79c0d75018713d74acd181ba86f4.png)

### Инструменты
![Инструменты](https://via.placeholder.com/800x450/18181b/3b82f6?text=Tools+-+Global+Clean)

### Глобальная очистка
![Очистка](https://via.placeholder.com/800x450/18181b/3b82f6?text=Global+Clean+Modal](https://allwebs.ru/images/2026/03/31/9fab007b04432f34dd11d6e708519d25.png)](https://allwebs.ru/images/2026/03/31/7b7fae5fc25de99ed4c97e31a4a4570b.png)

---

## 🔧 Функционал

### Запуск приложения
- Скрытый запуск целевого приложения
- Эмуляция Steam окружения (SteamAppId, SteamGameId)
- Автоматическая загрузка Microsoft.Ink.dll если отсутствует
- Инструкция по использованию Doomsday

### Инструменты

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
3. Запустите `SteamLauncher.exe`

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

Готовый файл: `dist/SteamLauncher.exe`

---

## 🚀 Использование

### Запуск

```bash
# Из исходников
python main.py

# Или готовый exe
SteamLauncher.exe
```

### Аргументы командной строки

```bash
python main.py --host 127.0.0.1 --port 8765
```

| Аргумент | Описание | По умолчанию |
|----------|----------|--------------|
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

---

## 📁 Структура проекта

```
web/
├── main.py                 # Главный файл
├── requirements.txt        # Зависимости
├── steam.ico              # Иконка
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
- **Дизайн:** Glassmorphism Dark Theme

---

## ⚠️ Предупреждение

> Приложение предназначено **только для образовательных целей**.  
> Авторы не несут ответственности за неправильное использование.  
> Используйте на свой страх и риск.

---

## 📝 Лицензия

MIT License — см. файл [LICENSE](LICENSE) для деталей.

---

## 🤝 Вклад

1. Fork репозиторий
2. Создайте ветку (`git checkout -b feature/AmazingFeature`)
3. Commit изменения (`git commit -m 'Add AmazingFeature'`)
4. Push в ветку (`git push origin feature/AmazingFeature`)
5. Откройте Pull Request

---

## 📧 Контакты

- **GitHub:** [@neverjoskiy](https://github.com/neverjoskiy)
- **Telegram:** [@bioneverr](https://t.me/bioneverr)

---

## ⭐ Поддержка

Если проект был полезен — поставьте звезду! ⭐

[![Star History](https://api.star-history.com/svg?repos=neverjoskiy/steam-launcher&type=Date)](https://star-history.com/#neverjoskiy/steam-launcher&Date)

---

<div align="center">

**Made with ❤️ by neverjoskiy**

</div>
