# Noxum Launcher (Rust/Tauri Migration)

Миграция приложения с Python/FastAPI на Rust/Tauri v2 для создания нативного десктопного приложения.

## Структура проекта

```
/workspace/
├── main.py                    # Оригинальный Python код (FastAPI)
├── templates/                 # HTML шаблоны
│   └── index.html
├── static/                    # Статические файлы
│   ├── css/styles.css
│   └── js/app.js
├── scripts/                   # Batch-скрипты для Windows
│   ├── вирус.bat
│   ├── не вирус.bat
│   └── винлокер.bat
└── src-tauri/                 # Rust/Tauri проект
    ├── Cargo.toml             # Зависимости Rust
    ├── tauri.conf.json        # Конфигурация Tauri
    ├── build.rs               # Build script
    └── src/
        ├── bin/main.rs        # Точка входа
        ├── lib.rs             # Библиотека
        ├── commands.rs        # Tauri команды (аналог API endpoints)
        ├── models.rs          # Модели данных
        ├── services.rs        # Бизнес-логика
        ├── state.rs           # Глобальное состояние
        ├── utils.rs           # Утилиты
        └── memory.rs          # Работа с памятью (Windows)
```

## Отличия от Python версии

### Архитектура
- **Python**: FastAPI веб-сервер + браузер
- **Rust**: Tauri v2 нативное приложение с WebView

### Преимущества Rust версии
1. **Нативный бинарник** - не требует установки Python
2. **Меньший размер** - ~5-10MB против ~50MB с Python
3. **Быстрее запуск** - нет overhead интерпретатора
4. **Безопасность памяти** - компилятор гарантирует отсутствие race conditions
5. **Прямой доступ к Win32 API** - для работы с памятью процессов

### Совместимость API
Все Tauri команды соответствуют оригинальным FastAPI endpoint'ам:

| Python Endpoint | Tauri Command |
|----------------|---------------|
| GET /api/status | get_status() |
| POST /api/launch | launch_app() |
| GET /api/logs | get_logs() |
| POST /api/logs/clear | clear_logs() |
| POST /api/tools/clean-strings | clean_strings() |
| POST /api/tools/clean-tracks | clean_tracks() |
| POST /api/tools/simulate | simulate_folders() |
| POST /api/tools/clean-javaw | clean_javaw_memory() |
| GET /api/tools/status | get_tools_status() |
| GET /api/tools/global-clean/options | get_global_clean_options() |
| POST /api/tools/global-clean | run_global_clean() |

## Сборка

### Требования
- Rust 1.70+
- Node.js 18+ (для Tauri CLI)
- Для Windows: Visual Studio Build Tools

### Команды сборки

```bash
cd /workspace/src-tauri

# Установка зависимостей
cargo fetch

# Debug сборка
cargo tauri dev

# Release сборка
cargo tauri build
```

## Использование из JavaScript

В Tauri v2 команды вызываются через `invoke()`:

```javascript
// Вместо fetch('/api/status')
const status = await invoke('get_status');

// Вместо fetch('/api/launch', {method: 'POST'})
const result = await invoke('launch_app');

// С параметрами
const logs = await invoke('get_logs', {lines: 50});
const cleanResult = await invoke('run_global_clean', {
    params: {
        event_logs: true,
        mft: false,
        // ...
    }
});
```

## Примечания

1. **Память javaw.exe** - функция `clean_javaw_memory()` работает только на Windows и требует прав администратора
2. **Batch-скрипты** - должны находиться в папке `scripts/` рядом с исполняемым файлом
3. **Логирование** - логи пишутся в `%TEMP%/NoxumLauncher/logs/app.log` (Windows) или `~/.local/share/noxum-launcher/logs/` (Linux)

