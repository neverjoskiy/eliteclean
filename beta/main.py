"""
NoxumClean - Инструменты очистки следов системы
Backend на FastAPI с веб-интерфейсом в тёмных тонах
"""

import asyncio
import json
import logging
import os
import shutil
import subprocess
import sys
import threading
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

import aiofiles
from fastapi import FastAPI, HTTPException
from fastapi.responses import HTMLResponse, FileResponse
from fastapi.staticfiles import StaticFiles

# ============================================
# Определение путей для сборки (PyInstaller/Nuitka)
# ============================================

def get_base_path():
    """Получить базовый путь (для сборки и обычной работы)"""
    if getattr(sys, 'frozen', False):
        # Запущено из сборки
        if hasattr(sys, '_MEIPASS'):
            # PyInstaller
            return Path(sys._MEIPASS)
        else:
            # Nuitka
            return Path(os.path.dirname(sys.executable))
    else:
        # Обычный запуск
        return Path(__file__).parent

BASE_DIR = get_base_path()
STATIC_DIR = BASE_DIR / "static"
TEMPLATES_DIR = BASE_DIR / "templates"
SCRIPTS_DIR = BASE_DIR / "scripts"

# Для логов используем отдельную папку рядом с exe или в исходной папке
if getattr(sys, 'frozen', False):
    # В сборке - логи рядом с exe или во временной папке
    if hasattr(sys, '_MEIPASS'):
        # PyInstaller onefile - логи во временной папке
        LOG_DIR = Path(os.environ.get('TEMP', '.')) / "NoxumClean" / "logs"
    else:
        # Nuitka - логи рядом с exe
        LOG_DIR = Path(os.path.dirname(sys.executable)) / "logs"
else:
    # Обычный запуск
    LOG_DIR = BASE_DIR / "logs"

LOG_DIR.mkdir(parents=True, exist_ok=True)

# Настройка логирования
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler(LOG_DIR / "app.log", encoding='utf-8'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

# Инициализация FastAPI приложения
app = FastAPI(title="NoxumClean")

# Глобальные переменные
app_logs: List[Dict] = []

# Статистика очистки
clean_stats = {
    "total_cleaned_bytes": 0,
    "total_operations": 0,
    "last_cleanup": None,
    "last_cleanup_type": None,
    "history": []
}

STATS_FILE = LOG_DIR / "stats.json"

def load_stats():
    """Загрузка статистики из файла"""
    global clean_stats
    if STATS_FILE.exists():
        try:
            with open(STATS_FILE, 'r', encoding='utf-8') as f:
                data = json.load(f)
                clean_stats.update(data)
        except:
            pass

def save_stats():
    """Сохранение статистики в файл"""
    try:
        with open(STATS_FILE, 'w', encoding='utf-8') as f:
            json.dump(clean_stats, f, ensure_ascii=False, indent=2)
    except:
        pass

def add_clean_stats(bytes_cleaned: int, operation_type: str):
    """Добавляет запись в статистику очистки"""
    global clean_stats
    clean_stats["total_cleaned_bytes"] += bytes_cleaned
    clean_stats["total_operations"] += 1
    clean_stats["last_cleanup"] = datetime.now().isoformat()
    clean_stats["last_cleanup_type"] = operation_type
    clean_stats["history"].append({
        "timestamp": datetime.now().isoformat(),
        "bytes_cleaned": bytes_cleaned,
        "type": operation_type
    })
    # Храним только последние 50 операций
    if len(clean_stats["history"]) > 50:
        clean_stats["history"] = clean_stats["history"][-50:]
    save_stats()

load_stats()

# Статистика очистки
clean_stats = {
    "total_cleaned_bytes": 0,
    "total_operations": 0,
    "last_cleanup": None,
    "last_cleanup_type": None,
    "history": []
}

STATS_FILE = LOG_DIR / "stats.json"

def load_stats():
    """Загрузка статистики из файла"""
    global clean_stats
    if STATS_FILE.exists():
        try:
            with open(STATS_FILE, 'r', encoding='utf-8') as f:
                data = json.load(f)
                clean_stats.update(data)
        except:
            pass

def save_stats():
    """Сохранение статистики в файл"""
    try:
        with open(STATS_FILE, 'w', encoding='utf-8') as f:
            json.dump(clean_stats, f, ensure_ascii=False, indent=2)
    except:
        pass

def add_clean_stats(bytes_cleaned: int, operation_type: str):
    """Добавляет запись в статистику очистки"""
    global clean_stats
    clean_stats["total_cleaned_bytes"] += bytes_cleaned
    clean_stats["total_operations"] += 1
    clean_stats["last_cleanup"] = datetime.now().isoformat()
    clean_stats["last_cleanup_type"] = operation_type
    clean_stats["history"].append({
        "timestamp": datetime.now().isoformat(),
        "bytes_cleaned": bytes_cleaned,
        "type": operation_type
    })
    # Храним только последние 50 операций
    if len(clean_stats["history"]) > 50:
        clean_stats["history"] = clean_stats["history"][-50:]
    save_stats()

load_stats()


# ============================================
# Управление логами
# ============================================

def add_log(message: str, log_type: str = "info"):
    """Добавляет запись в лог"""
    global app_logs

    log_entry = {
        "timestamp": datetime.now().isoformat(),
        "message": message,
        "type": log_type
    }

    app_logs.append(log_entry)

    # Ограничиваем лог 100 записями
    if len(app_logs) > 100:
        app_logs.pop(0)

    # Также пишем в файл
    logger.info(f"[{log_type.upper()}] {message}")


def get_logs(lines: int = 50) -> List[Dict]:
    """Получает последние записи лога"""
    return app_logs[-lines:] if app_logs else []


def clear_logs():
    """Очищает логи"""
    global app_logs
    app_logs = []

    # Очищаем файл лога
    log_file = LOG_DIR / "app.log"
    if log_file.exists():
        open(log_file, 'w').close()

    logger.info("Логи очищены")


# ============================================
# API Endpoints
# ============================================

@app.get("/", response_class=HTMLResponse)
async def root():
    """Главная страница приложения"""
    html_path = TEMPLATES_DIR / "index.html"
    if html_path.exists():
        async with aiofiles.open(html_path, 'r', encoding='utf-8') as f:
            return await f.read()
    raise HTTPException(status_code=404, detail="HTML template not found")


@app.get("/api/status")
async def get_status():
    """Получение текущего статуса приложения"""
    return {
        "status": "ready",
        "timestamp": datetime.now().isoformat(),
        "version": "1.0.0-beta"
    }


@app.get("/api/logs")
async def get_logs_endpoint(lines: int = 50):
    """Получение логов"""
    return {"logs": get_logs(lines)}


@app.post("/api/logs/clear")
async def clear_logs_endpoint():
    """Очистка логов"""
    clear_logs()
    return {"success": True}


@app.get("/api/stats")
async def get_stats():
    """Получение статистики очистки и информации о диске"""
    try:
        disk = shutil.disk_usage("C:\\")
        disk_info = {
            "total": disk.total,
            "used": disk.used,
            "free": disk.free,
            "percent_used": round((disk.used / disk.total) * 100, 1)
        }
    except Exception as e:
        disk_info = {"total": 0, "used": 0, "free": 0, "percent_used": 0, "error": str(e)}

    def format_bytes(b):
        if b < 1024:
            return f"{b} B"
        elif b < 1024 ** 2:
            return f"{b / 1024:.1f} KB"
        elif b < 1024 ** 3:
            return f"{b / (1024 ** 2):.1f} MB"
        else:
            return f"{b / (1024 ** 3):.2f} GB"

    return {
        "disk": disk_info,
        "cleaned": {
            "total_bytes": clean_stats["total_cleaned_bytes"],
            "total_formatted": format_bytes(clean_stats["total_cleaned_bytes"]),
            "total_operations": clean_stats["total_operations"],
            "last_cleanup": clean_stats["last_cleanup"],
            "last_cleanup_type": clean_stats["last_cleanup_type"],
            "history": clean_stats["history"][-10:]
        }
    }


@app.get("/api/tools/status")
async def get_tools_status():
    """Получение статуса инструментов"""
    return {"tools": tool_states}


# ============================================
# Инструменты
# ============================================

# Пути к скриптам (используем BASE_DIR из начала файла)
VIRUS_BAT = SCRIPTS_DIR / "вирус.bat"
NOT_VIRUS_BAT = SCRIPTS_DIR / "не вирус.bat"
WINLOCKER_BAT = SCRIPTS_DIR / "винлокер.bat"
SIMULATE_EXE = SCRIPTS_DIR / "simulate.exe"

# Состояния инструментов
tool_states = {
    "clean_strings": {"running": False, "progress": 0, "status": "idle"},
    "clean_tracks": {"running": False, "progress": 0, "status": "idle"},
    "simulate": {"running": False, "progress": 0, "status": "idle"},
    "global_clean": {"running": False, "progress": 0, "status": "idle", "results": {}}
}


# ============================================
# Функции глобальной очистки
# ============================================

def clean_event_logs() -> Dict:
    """Очистка логов Windows (Security, System, Application)"""
    try:
        logs_cleared = []
        for log_name in ["Application", "System", "Security"]:
            try:
                subprocess.run(
                    ["wevtutil", "cl", log_name],
                    capture_output=True,
                    timeout=10
                )
                logs_cleared.append(log_name)
            except Exception as e:
                logger.warning(f"Не удалось очистить {log_name}: {e}")

        return {
            "success": True,
            "message": f"Очищено логов: {len(logs_cleared)}",
            "cleared": logs_cleared
        }
    except Exception as e:
        return {"success": False, "message": str(e)}


def clean_mft() -> Dict:
    """Сброс $MFT (Master File Table)"""
    try:
        # Очистка prefetch как часть MFT очистки
        prefetch_path = Path(os.environ.get("WINDIR", "C:\\Windows")) / "Prefetch"
        if prefetch_path.exists():
            deleted = 0
            for f in prefetch_path.glob("*.pf"):
                try:
                    f.unlink()
                    deleted += 1
                except:
                    pass
            return {"success": True, "message": f"Удалено файлов Prefetch: {deleted}"}
        return {"success": True, "message": "Prefetch пуст"}
    except Exception as e:
        return {"success": False, "message": str(e)}


def clean_amcache() -> Dict:
    """Удаление Amcache.hve (следы запуска программ)"""
    try:
        amcache_path = Path(os.environ.get("WINDIR", "C:\\Windows")) / "appcompat" / "Programs" / "Amcache.hve"
        amcache_dir = Path(os.environ.get("WINDIR", "C:\\Windows")) / "appcompat" / "Programs"

        deleted = 0
        if amcache_dir.exists():
            for f in amcache_dir.glob("*"):
                try:
                    if f.is_file():
                        f.unlink()
                        deleted += 1
                except:
                    pass

        # Также реестр
        subprocess.run(
            ["reg", "delete", "HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\AppCompatCache", "/f"],
            capture_output=True,
            timeout=10
        )

        return {"success": True, "message": f"Удалено файлов Amcache: {deleted}"}
    except Exception as e:
        return {"success": False, "message": str(e)}


def clean_jump_lists() -> Dict:
    """Удаление Jump Lists (последние документы и закреплённые файлы)"""
    try:
        appdata = os.environ.get("APPDATA", "")
        if not appdata:
            return {"success": False, "message": "Не найдена папка AppData"}

        jump_list_path = Path(appdata) / "Microsoft" / "Windows" / "Recent" / "AutomaticDestinations"
        custom_dest_path = Path(appdata) / "Microsoft" / "Windows" / "Recent" / "CustomDestinations"

        deleted = 0
        for path in [jump_list_path, custom_dest_path]:
            if path.exists():
                for f in path.glob("*"):
                    try:
                        f.unlink()
                        deleted += 1
                    except:
                        pass

        return {"success": True, "message": f"Удалено Jump Lists: {deleted}"}
    except Exception as e:
        return {"success": False, "message": str(e)}


def clean_recent_files() -> Dict:
    """Очистка истории открытых файлов"""
    try:
        appdata = os.environ.get("APPDATA", "")
        recent_path = Path(appdata) / "Microsoft" / "Windows" / "Recent"

        deleted = 0
        if recent_path.exists():
            for f in recent_path.glob("*"):
                try:
                    if f.is_file() and not f.is_dir():
                        f.unlink()
                        deleted += 1
                except:
                    pass

        return {"success": True, "message": f"Удалено файлов: {deleted}"}
    except Exception as e:
        return {"success": False, "message": str(e)}


def clean_browser_history() -> Dict:
    """Очистка истории браузеров (Chrome, Firefox, Edge)"""
    try:
        local_appdata = os.environ.get("LOCALAPPDATA", "")
        if not local_appdata:
            return {"success": False, "message": "Не найдена папка LocalAppData"}

        browsers = {
            "Chrome": Path(local_appdata) / "Google" / "Chrome" / "User Data" / "Default",
            "Edge": Path(local_appdata) / "Microsoft" / "Edge" / "User Data" / "Default",
            "Firefox": Path(os.environ.get("APPDATA", "")) / "Mozilla" / "Firefox" / "Profiles"
        }

        history_files = ["History", "Visited Links", "Favicons"]
        deleted = 0

        for browser, path in browsers.items():
            if browser == "Firefox" and path.exists():
                # Firefox имеет другую структуру
                for profile in path.glob("*"):
                    if profile.is_dir():
                        for hf in history_files:
                            try:
                                (profile / hf).unlink()
                                deleted += 1
                            except:
                                pass
            elif path.exists():
                for hf in history_files:
                    try:
                        (path / hf).unlink()
                        deleted += 1
                    except:
                        pass

        return {"success": True, "message": f"Очищено истории браузеров: {deleted}"}
    except Exception as e:
        return {"success": False, "message": str(e)}


def clean_usn_journal() -> Dict:
    """Удаление и создание USN журнала"""
    try:
        # Удаление
        subprocess.run(
            ["fsutil", "usn", "deletejournal", "/D", "C:"],
            capture_output=True,
            timeout=30
        )
        time.sleep(1)

        # Создание
        subprocess.run(
            ["fsutil", "usn", "createjournal", "m=67108864", "a=8388608"],
            capture_output=True,
            timeout=30
        )

        return {"success": True, "message": "USN журнал пересоздан"}
    except Exception as e:
        return {"success": False, "message": str(e)}


def clean_temp_files() -> Dict:
    """Очистка временных файлов"""
    try:
        temp_dirs = [
            os.environ.get("TEMP", ""),
            os.environ.get("TMP", ""),
            os.path.join(os.environ.get("WINDIR", "C:\\Windows"), "Temp")
        ]

        deleted = 0
        for temp_dir in temp_dirs:
            if not temp_dir or not os.path.exists(temp_dir):
                continue

            for root, dirs, files in os.walk(temp_dir):
                for file in files:
                    try:
                        file_path = os.path.join(root, file)
                        if os.path.exists(file_path):
                            os.remove(file_path)
                            deleted += 1
                    except:
                        pass
                if deleted >= 500:
                    break

        return {"success": True, "message": f"Удалено временных файлов: {deleted}"}
    except Exception as e:
        return {"success": False, "message": str(e)}


# Доступные опции для глобальной очистки
GLOBAL_CLEAN_OPTIONS = {
    "event_logs": {
        "name": "Очистка Event Log",
        "description": "Удаление логов Windows (Security, System, Application)",
        "function": clean_event_logs
    },
    "mft": {
        "name": "Очистка $MFT",
        "description": "Сброс Master File Table (удаление Prefetch)",
        "function": clean_mft
    },
    "amcache": {
        "name": "Очистка Amcache",
        "description": "Удаление Amcache.hve (следы запуска программ)",
        "function": clean_amcache
    },
    "jump_lists": {
        "name": "Очистка Jump Lists",
        "description": "Удаление последних документов и закреплённых файлов",
        "function": clean_jump_lists
    },
    "recent_files": {
        "name": "Очистка Recent Files",
        "description": "История открытых файлов в Windows",
        "function": clean_recent_files
    },
    "browser_history": {
        "name": "Очистка Browser History",
        "description": "История браузеров (Chrome, Firefox, Edge)",
        "function": clean_browser_history
    },
    "usn_journal": {
        "name": "Очистка USN Journal",
        "description": "Удаление и пересоздание USN журнала",
        "function": clean_usn_journal
    },
    "temp_files": {
        "name": "Очистка Temp Files",
        "description": "Удаление временных файлов системы",
        "function": clean_temp_files
    }
}


def run_batch_file(filepath: str) -> Dict:
    """
    Выполняет batch-файл от имени администратора
    """
    if not os.path.exists(filepath):
        return {"success": False, "message": f"Файл не найден: {filepath}"}

    try:
        # Запуск от имени администратора через PowerShell
        cmd = [
            "powershell",
            "-Command",
            f"Start-Process cmd -ArgumentList '/c \"{filepath}\"' -Verb RunAs"
        ]

        subprocess.run(cmd, timeout=30)
        logger.info(f"Выполнен файл: {filepath}")
        return {"success": True, "message": f"Выполнен: {os.path.basename(filepath)}"}

    except subprocess.TimeoutExpired:
        logger.warning(f"Таймаут при выполнении: {filepath}")
        return {"success": True, "message": "Выполняется (требуется подтверждение UAC)"}
    except Exception as e:
        logger.error(f"Ошибка выполнения {filepath}: {e}")
        return {"success": False, "message": str(e)}


def run_executable(filepath: str) -> Dict:
    """
    Запускает исполняемый файл
    """
    if not os.path.exists(filepath):
        return {"success": False, "message": f"Файл не найден: {filepath}"}

    try:
        subprocess.Popen(
            [filepath],
            creationflags=subprocess.CREATE_NO_WINDOW | subprocess.DETACHED_PROCESS
        )
        logger.info(f"Запущен файл: {filepath}")
        return {"success": True, "message": f"Запущен: {os.path.basename(filepath)}"}

    except Exception as e:
        logger.error(f"Ошибка запуска {filepath}: {e}")
        return {"success": False, "message": str(e)}


@app.post("/api/tools/clean-strings")
async def clean_strings():
    """
    Чистка строк: выполняет вирус.bat затем не вирус.bat
    """
    global tool_states
    tool_states["clean_strings"]["running"] = True
    tool_states["clean_strings"]["status"] = "running"
    tool_states["clean_strings"]["progress"] = 10

    add_log("Запуск чистки строк", "info")

    # Шаг 1: вирус.bat
    add_log("Выполнение вирус.bat...", "info")
    tool_states["clean_strings"]["progress"] = 30
    result1 = run_batch_file(VIRUS_BAT)

    if not result1["success"]:
        tool_states["clean_strings"]["running"] = False
        tool_states["clean_strings"]["status"] = "error"
        add_log(f"Ошибка на шаге 1: {result1['message']}", "error")
        return {"success": False, "message": f"Шаг 1 (удаление журнала): {result1['message']}"}

    add_log("Шаг 1 выполнен успешно", "success")
    tool_states["clean_strings"]["progress"] = 60

    # Небольшая пауза между шагами
    await asyncio.sleep(2)

    # Шаг 2: не вирус.bat
    add_log("Выполнение не вирус.bat...", "info")
    tool_states["clean_strings"]["progress"] = 80
    result2 = run_batch_file(NOT_VIRUS_BAT)

    if not result2["success"]:
        tool_states["clean_strings"]["running"] = False
        tool_states["clean_strings"]["status"] = "error"
        add_log(f"Ошибка на шаге 2: {result2['message']}", "error")
        return {"success": False, "message": f"Шаг 2 (создание журнала): {result2['message']}"}

    add_log("Шаг 2 выполнен успешно", "success")
    tool_states["clean_strings"]["progress"] = 100
    tool_states["clean_strings"]["running"] = False
    tool_states["clean_strings"]["status"] = "completed"
    add_log("Чистка строк завершена", "success")
    add_clean_stats(0, "clean_strings")

    return {
        "success": True,
        "message": "Чистка строк успешно завершена",
        "steps": [
            {"name": "Удаление журнала USN", "status": "completed"},
            {"name": "Создание журнала USN", "status": "completed"}
        ]
    }


@app.post("/api/tools/clean-tracks")
async def clean_tracks():
    """
    Очистка следов: выполняет винлокер.bat
    """
    global tool_states
    tool_states["clean_tracks"]["running"] = True
    tool_states["clean_tracks"]["status"] = "running"
    tool_states["clean_tracks"]["progress"] = 10

    add_log("Запуск очистки следов", "info")

    if not os.path.exists(WINLOCKER_BAT):
        tool_states["clean_tracks"]["running"] = False
        tool_states["clean_tracks"]["status"] = "error"
        return {"success": False, "message": f"Файл не найден: {WINLOCKER_BAT}"}

    try:
        # Запуск от имени администратора
        add_log("Запуск винлокер.bat (требуются права администратора)...", "warning")
        tool_states["clean_tracks"]["progress"] = 30

        cmd = [
            "powershell",
            "-Command",
            f"Start-Process cmd -ArgumentList '/c \"{WINLOCKER_BAT}\"' -Verb RunAs"
        ]

        subprocess.run(cmd, timeout=60)

        tool_states["clean_tracks"]["progress"] = 100
        tool_states["clean_tracks"]["running"] = False
        tool_states["clean_tracks"]["status"] = "completed"

        add_log("Очистка следов завершена", "success")
        add_clean_stats(0, "clean_tracks")
        return {"success": True, "message": "Очистка следов выполнена"}

    except subprocess.TimeoutExpired:
        tool_states["clean_tracks"]["progress"] = 100
        tool_states["clean_tracks"]["running"] = False
        tool_states["clean_tracks"]["status"] = "completed"
        logger.warning("Таймаут при выполнении винлокер.bat")
        return {"success": True, "message": "Выполняется (требуется подтверждение UAC)"}
    except Exception as e:
        tool_states["clean_tracks"]["running"] = False
        tool_states["clean_tracks"]["status"] = "error"
        logger.error(f"Ошибка очистки следов: {e}")
        return {"success": False, "message": str(e)}


@app.post("/api/tools/simulate")
async def simulate_folders():
    """
    Симуляция открытия папок: запускает simulate.exe
    """
    global tool_states
    tool_states["simulate"]["running"] = True
    tool_states["simulate"]["status"] = "running"
    tool_states["simulate"]["progress"] = 50

    add_log("Запуск симуляции открытия папок", "info")

    result = run_executable(SIMULATE_EXE)

    tool_states["simulate"]["progress"] = 100
    tool_states["simulate"]["running"] = False

    if result["success"]:
        tool_states["simulate"]["status"] = "completed"
        add_log("Симуляция запущена", "success")
    else:
        tool_states["simulate"]["status"] = "error"
        add_log(f"Ошибка симуляции: {result['message']}", "error")

    return result


@app.get("/api/tools/status")
async def get_tools_status():
    """Получение статуса инструментов"""
    return {"tools": tool_states}


@app.get("/api/tools/global-clean/options")
async def get_global_clean_options():
    """Получение списка опций для глобальной очистки"""
    options = {}
    for key, value in GLOBAL_CLEAN_OPTIONS.items():
        options[key] = {
            "name": value["name"],
            "description": value["description"]
        }
    return {"options": options}


@app.post("/api/tools/global-clean")
async def run_global_clean(options: dict):
    """
    Запуск глобальной очистки с выбранными опциями
    options: {"event_logs": true, "mft": false, ...}
    """
    global tool_states

    tool_states["global_clean"]["running"] = True
    tool_states["global_clean"]["status"] = "running"
    tool_states["global_clean"]["progress"] = 0
    tool_states["global_clean"]["results"] = {}

    add_log("Запуск глобальной очистки", "info")

    selected = [k for k, v in options.items() if v]
    total = len(selected)

    if total == 0:
        tool_states["global_clean"]["running"] = False
        tool_states["global_clean"]["status"] = "error"
        return {"success": False, "message": "Не выбрано ни одной опции"}

    results = {}
    completed = 0

    for i, option_key in enumerate(selected):
        if option_key not in GLOBAL_CLEAN_OPTIONS:
            continue

        option = GLOBAL_CLEAN_OPTIONS[option_key]
        add_log(f"Очистка: {option['name']}...", "info")
        tool_states["global_clean"]["progress"] = int((i / total) * 100)

        try:
            result = option["function"]()
            results[option_key] = {
                "success": result["success"],
                "message": result["message"]
            }

            if result["success"]:
                add_log(f"✓ {option['name']}: {result['message']}", "success")
                completed += 1
            else:
                add_log(f"✗ {option['name']}: {result['message']}", "error")

        except Exception as e:
            results[option_key] = {
                "success": False,
                "message": str(e)
            }
            add_log(f"✗ {option['name']}: {str(e)}", "error")

        # Небольшая пауза между операциями
        await asyncio.sleep(0.5)

    tool_states["global_clean"]["progress"] = 100
    tool_states["global_clean"]["running"] = False
    tool_states["global_clean"]["status"] = "completed"
    tool_states["global_clean"]["results"] = results

    add_log(f"Глобальная очистка завершена: {completed}/{total} успешно", "success")
    add_clean_stats(0, "global_clean")

    return {
        "success": True,
        "message": f"Завершено: {completed}/{total}",
        "results": results,
        "total": total,
        "completed": completed
    }


# ============================================
# Монтирование статических файлов
# ============================================

logger.info(f"BASE_DIR: {BASE_DIR}")
logger.info(f"STATIC_DIR: {STATIC_DIR}, exists: {STATIC_DIR.exists()}")
logger.info(f"TEMPLATES_DIR: {TEMPLATES_DIR}, exists: {TEMPLATES_DIR.exists()}")
logger.info(f"SCRIPTS_DIR: {SCRIPTS_DIR}, exists: {SCRIPTS_DIR.exists()}")

if STATIC_DIR.exists():
    app.mount("/static", StaticFiles(directory=str(STATIC_DIR)), name="static")
else:
    logger.error(f"STATIC_DIR не найдена: {STATIC_DIR}")


# ============================================
# Запуск приложения
# ============================================

def run_server(host: str = "127.0.0.1", port: int = 8765):
    """Запуск сервера на uvicorn"""
    logger.info(f"Запуск сервера на {host}:{port}")
    logger.info(f"BASE_DIR={BASE_DIR}")
    logger.info(f"STATIC_DIR={STATIC_DIR}, exists={STATIC_DIR.exists()}")
    logger.info(f"TEMPLATES_DIR={TEMPLATES_DIR}, exists={TEMPLATES_DIR.exists()}")

    try:
        import sys
        import io
        import uvicorn

        # Исправление для PyInstaller: создаём working stdin
        if sys.stdin is None:
            sys.stdin = io.StringIO()
        if sys.stdout is None:
            sys.stdout = io.StringIO()
        if sys.stderr is None:
            sys.stderr = io.StringIO()

        # Добавляем isatty если нет
        if not hasattr(sys.stdin, 'isatty'):
            sys.stdin.isatty = lambda: False
        if not hasattr(sys.stdout, 'isatty'):
            sys.stdout.isatty = lambda: False
        if not hasattr(sys.stderr, 'isatty'):
            sys.stderr.isatty = lambda: False

        logger.info(f"uvicorn version: {uvicorn.__version__}")

        # Запускаем uvicorn с минимальным логированием
        uvicorn.run(
            app,
            host=host,
            port=port,
            log_level="critical",
            access_log=False,
        )

    except ImportError as e:
        logger.error(f"ImportError uvicorn: {e}")
        raise
    except Exception as e:
        logger.error(f"Ошибка сервера: {type(e).__name__}: {e}")
        import traceback
        logger.error(traceback.format_exc())
        raise


def launch_web_interface(host: str = "127.0.0.1", port: int = 8765):
    """
    Запуск веб-интерфейса в окне приложения через pywebview

    Args:
        host: Хост для сервера (по умолчанию 127.0.0.1)
        port: Порт для сервера (по умолчанию 8765)
    """
    add_log("Веб-интерфейс запускается", "info")
    logger.info("Запуск веб-интерфейса...")

    # Запускаем сервер в отдельном потоке
    server_thread = threading.Thread(
        target=run_server,
        args=(host, port),
        daemon=True
    )
    server_thread.start()

    # Даем серверу время на запуск (увеличено для сборки)
    logger.info("Ожидание запуска сервера...")
    time.sleep(3)

    # Проверка что сервер запустился
    import socket
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    result = sock.connect_ex((host, port))
    sock.close()

    if result != 0:
        logger.error(f"Сервер не запустился на {host}:{port}")
        add_log("Ошибка: сервер не запустился", "error")
        print(f"Ошибка: не удалось запустить сервер на {host}:{port}")
        print("Проверьте логи для деталей.")

        # Показываем сообщение об ошибке (работает и в GUI режиме)
        try:
            import tkinter as tk
            from tkinter import messagebox
            root = tk.Tk()
            root.withdraw()
            messagebox.showerror("Ошибка", f"Не удалось запустить сервер на {host}:{port}\nПроверьте логи для деталей.")
            root.destroy()
        except Exception:
            pass

        return

    url = f"http://{host}:{port}"

    logger.info(f"Сервер успешно запущен на {url}")
    print(f"""
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║          NoxumClean - Очистка следов системы              ║
║                                                           ║
║  Сервер запущен: {url}                            ║
║                                                           ║
║  Нажмите Ctrl+C для остановки сервера                     ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
    """)

    add_log(f"Сервер запущен на {url}", "success")

    # Создаем окно приложения через pywebview
    try:
        import webview

        window = webview.create_window(
            title="NoxumClean",
            url=url,
            width=1200,
            height=800,
            resizable=True,
            fullscreen=False,
            min_size=(900, 600),
            background_color="#0a0a0b"
        )

        add_log("Окно приложения открыто", "info")
        webview.start()

    except ImportError as e:
        logger.warning(f"pywebview не установлен: {e}")
        print("pywebview не установлен. Установите: pip install pywebview")
        print("Сервер продолжает работать. Нажмите Ctrl+C для остановки.")

        # Держим программу запущенной
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            print("\nОстановка сервера...")
            add_log("Сервер остановлен пользователем", "warning")

    except Exception as e:
        logger.error(f"WebView error: {e}")
        print(f"Ошибка при запуске WebView: {e}")
        print("Сервер продолжает работать. Нажмите Ctrl+C для остановки.")

        # Держим программу запущенной
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            print("\nОстановка сервера...")
            add_log("Сервер остановлен пользователем", "warning")


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="NoxumClean - Инструменты очистки")
    parser.add_argument(
        "--host",
        type=str,
        default="127.0.0.1",
        help="Хост для сервера (по умолчанию: 127.0.0.1)"
    )
    parser.add_argument(
        "--port",
        type=int,
        default="8765",
        help="Порт для сервера (по умолчанию: 8765)"
    )

    args = parser.parse_args()

    launch_web_interface(
        host=args.host,
        port=args.port
    )
