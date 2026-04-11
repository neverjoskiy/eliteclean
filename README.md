<p align="center">
  <img src="https://s3.radikal.cloud/2026/04/11/home988a8fbd28267c91.png" alt="NoxumCleaner" width="100%" />
</p>

<h1 align="center">✦ NoxumCleaner</h1>

<p align="center">
  <b>Утилита для очистки системы на C++</b><br/>
  <sup>C++ • USN Clean • Trace Removal • Folder Simulation</sup>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/C%2B%2B-17-00599C?style=for-the-badge&logo=cplusplus&logoColor=white" alt="C++ 17" />
  <img src="https://img.shields.io/badge/Platform-Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white" alt="Windows" />
  <img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" alt="MIT" />
</p>

<br/>
---

<br/>

## ◈ О проекте

**NoxumCleaner** — это высокопроизводительная утилита для очистки системы, переписанная с Python на C++ для максимальной скорости и минимального потребления ресурсов.

### Ключевые особенности

| Особенность | Описание |
|:---|:---|
| ⚡ **C++ Производительность** | В 10 раз быстрее Python, нативная компиляция |
| 🧹 **Очистка следов** | ShellBag, Prefetch, Minidump, Jump Lists |
| 📊 **USN Journal** | Очистка и управление NTFS журналом |
| 📁 **Симуляция папок** | Создание фиктивной структуры для маскировки |
| 🌐 **Open Source** | Полностью открытый исходный код |
| 🚀 **Минимальное потребление** | Низкое использование CPU и RAM |

<br/>

---

<br/>

## ◈ Функционал

### Инструменты

| Инструмент | Описание |
|:---|:---|
| **Чистка USN журнала** | Удаление записей NTFS USN, скрытие фактов изменения файлов |
| **Очистка следов** | Удаление ShellBag, Prefetch, Minidump, истории проводника |
| **Глобальная очистка** | Комплексная очистка с выбором компонентов |
| **Симуляция папок** | Запуск simulate.exe для имитации активности пользователя |

#### Глобальная очистка включает:

- ☑️ **Корзина** — очистка удалённых файлов
- ☑️ **Temp Files** — временные файлы системы
- ☑️ **Browser Cache** — кэш браузеров (Chrome, Firefox, Edge)
- ☑️ **Clipboard** — очистка буфера обмена
- ☑️ **Recent Files** — история открытых файлов
- ☑️ **Browser History** — история браузеров
- ☑️ **Error Reports** — отчёты об ошибках Windows
- ☑️ **System Logs** — логи системы

<br/>

---

<br/>

## ◈ Установка

### Вариант 1: Готовая сборка

1. Скачайте последний релиз из раздела [Releases](https://github.com/neverjoskiy/noxumcleaner/releases)
2. Запустите установщик `setup.exe`
3. Следуйте инструкциям установщика

### Вариант 2: Сборка из исходников

#### Требования

| Компонент | Версия |
|:---|:---|
| **C++ Compiler** | MSVC / MinGW-w64 |
| **CMake** | 3.20+ |
| **Windows SDK** | 10+ |

#### Сборка

```bash
# Клонирование репозитория
git clone https://github.com/neverjoskiy/noxumcleaner.git
cd noxumcleaner

# Создание папки сборки
mkdir build && cd build

# Конфигурация и сборка
cmake .. -A x64
cmake --build . --config Release
```

**Результат:** `build/Release/NoxumCleaner.exe`

<br/>

---

<br/>

## ◈ Использование

### Запуск

```bash
# Готовая сборка
NoxumCleaner.exe

# Или через установщик
# Запустите из меню Пуск
```

### Режимы работы

| Режим | Описание |
|:---|:---|
| **Быстрая очистка** | Очистка основных следов системы |
| **Глубокая очистка** | Полная очистка с выбором компонентов |
| **Симуляция** | Имитация активности пользователя |
| **USN Clean** | Очистка NTFS журнала |

> 💡 Рекомендуется запускать от имени администратора для полного доступа

<br/>

---

<br/>

## ◈ Структура проекта

```
noxumcleaner/
│
├── src/                   # Исходный код C++
│   ├── main.cpp           # Точка входа
│   ├── cleaner.cpp        # Модуль очистки
│   ├── usn.cpp            # Работа с USN журналом
│   └── simulation.cpp     # Симуляция активности
│
├── web/                   # Веб-интерфейс
│   ├── templates/         # HTML шаблоны
│   │   └── index.html
│   ├── static/            # Статические файлы
│   │   ├── css/
│   │   │   └── styles.css
│   │   └── js/
│   │       └── app.js
│   └── main.py            # Сервер для веб-интерфейса
│
├── scripts/               # Вспомогательные скрипты
│   └── simulate.exe       # Симуляция папок
│
├── CMakeLists.txt         # CMake конфигурация
└── README.md              # Документация
```

<br/>

---

<br/>

## ◈ Технологии

| Категория | Стек |
|:---|:---|
| **Backend** | C++17, WinAPI |
| **Frontend** | HTML5, CSS3, Vanilla JS |
| **Сборка** | CMake, MSVC |
| **Дизайн** | Glassmorphism Dark Theme |

<br/>

---

<br/>

## ◈ Сравнение с Python версией

| Параметр | Python | C++ |
|:---|:---|:---|
| **Скорость запуска** | ~2.5s | ~0.3s |
| **Потребление RAM** | ~85 MB | ~12 MB |
| **Размер exe** | ~45 MB | ~3 MB |
| **Зависимости** | Python, pip | Нет |

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
  <sub>✦ NoxumCleaner — Чисто. Быстро. Надёжно ✦</sub><br/>
  <sub>C++ • Open Source • Windows</sub>
</p>
