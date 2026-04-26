/**
 * Steam Launcher - Frontend JavaScript
 * Обработка взаимодействия с API и управление UI
 */

// ============================================
// Глобальные переменные
// ============================================
let logs = [];

// ============================================
// Утилиты
// ============================================

/**
 * Показывает toast уведомление
 */
function showToast(message, type = 'info') {
    const container = document.getElementById('toastContainer');
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;

    const icons = {
        success: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>',
        error: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>',
        warning: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>',
        info: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>'
    };

    toast.innerHTML = `
        <span class="toast-icon">${icons[type]}</span>
        <span class="toast-message">${message}</span>
    `;

    container.appendChild(toast);

    // Удаляем через 4 секунды
    setTimeout(() => {
        toast.classList.add('hiding');
        setTimeout(() => toast.remove(), 300);
    }, 4000);
}

/**
 * Добавляет/удаляет класс loading у кнопки
 */
function setButtonLoading(button, loading) {
    if (loading) {
        button.classList.add('loading');
        button.disabled = true;
    } else {
        button.classList.remove('loading');
        button.disabled = false;
    }
}

/**
 * Форматирует текущее время для логов
 */
function formatTime() {
    const now = new Date();
    return now.toLocaleTimeString('ru-RU', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit'
    });
}

// ============================================
// Логирование
// ============================================

function addLog(message, type = 'info') {
    const timestamp = formatTime();
    logs.push({ timestamp, message, type });

    const container = document.getElementById('logsContainer');
    
    // Очищаем placeholder если это первая запись
    if (logs.length === 1) {
        container.innerHTML = '';
    }

    const logEntry = document.createElement('div');
    logEntry.className = `log-entry ${type}`;
    logEntry.innerHTML = `
        <span class="log-time">${timestamp}</span>
        <span class="log-message">${message}</span>
    `;

    container.appendChild(logEntry);
    
    // Прокрутка вниз
    container.scrollTop = container.scrollHeight;

    // Сохраняем только последние 50 записей
    if (logs.length > 50) {
        logs.shift();
    }
}

function clearLogs() {
    logs = [];
    const container = document.getElementById('logsContainer');
    container.innerHTML = `
        <div class="log-entry info">
            <span class="log-time">--:--:--</span>
            <span class="log-message">Ожидание событий...</span>
        </div>
    `;
    showToast('Журнал очищен', 'info');
}

// ============================================
// API функции (Tauri invoke)
// ============================================

const { invoke } = window.__TAURI__.core;

async function launchApp() {
    return await invoke('launch_app');
}

async function getStatus() {
    return await invoke('get_status');
}

async function getLogs(lines = 50) {
    return await invoke('get_logs', { lines });
}

async function clearLogsApi() {
    return await invoke('clear_logs');
}

async function cleanStrings() {
    return await invoke('clean_strings');
}

async function cleanTracks() {
    return await invoke('clean_tracks');
}

async function simulateFolders() {
    return await invoke('simulate_folders');
}

async function cleanJavawMemory() {
    return await invoke('clean_javaw_memory');
}

async function getToolsStatus() {
    return await invoke('get_tools_status');
}

async function getGlobalCleanOptions() {
    return await invoke('get_global_clean_options');
}

async function runGlobalClean(options) {
    return await invoke('run_global_clean', { params: options });
}

// ============================================
// Обновление UI
// ============================================

function updateStatusIndicator(status) {
    const indicator = document.getElementById('statusIndicator');
    const statusText = indicator.querySelector('.status-text');

    indicator.className = 'status-indicator';

    switch (status) {
        case 'ready':
            indicator.classList.add('ready');
            statusText.textContent = 'Готов';
            break;
        case 'running':
            indicator.classList.add('running');
            statusText.textContent = 'Запуск...';
            break;
        case 'error':
            indicator.classList.add('error');
            statusText.textContent = 'Ошибка';
            break;
        default:
            statusText.textContent = 'Готов';
    }
}

function updateFileStatus(exists, size) {
    const banner = document.getElementById('fileStatusBanner');
    const text = document.getElementById('fileStatusText');
    
    banner.style.display = 'flex';
    banner.className = 'file-status-sidebar';
    
    if (exists) {
        const sizeKB = (size / 1024).toFixed(2);
        text.textContent = `Microsoft.Ink.dll найден (${sizeKB} KB)`;
        banner.classList.add('success');
    } else {
        text.textContent = 'Microsoft.Ink.dll не найден. Будет загружен при запуске.';
        banner.classList.add('warning');
    }
}

function updateLaunchStatus(message, type = 'info') {
    const statusEl = document.getElementById('launchStatus');
    statusEl.className = 'launch-status';
    
    if (type !== 'info') {
        statusEl.classList.add(type);
    }
    
    statusEl.querySelector('.status-message').textContent = message;
}

// ============================================
// Инструменты - Чистка строк
// ============================================

function resetStepStatus(stepId) {
    const step = document.getElementById(stepId);
    const statusEl = document.getElementById(stepId + 'Status');
    
    step.classList.remove('active', 'completed', 'failed');
    statusEl.innerHTML = '<span class="status-badge pending">Ожидание</span>';
}

function setStepRunning(stepId) {
    const step = document.getElementById(stepId);
    const statusEl = document.getElementById(stepId + 'Status');
    
    step.classList.add('active');
    step.classList.remove('completed', 'failed');
    statusEl.innerHTML = '<span class="status-badge running">Выполнение</span>';
}

function setStepCompleted(stepId) {
    const step = document.getElementById(stepId);
    const statusEl = document.getElementById(stepId + 'Status');
    
    step.classList.add('completed');
    step.classList.remove('active', 'failed');
    statusEl.innerHTML = '<span class="status-badge success">Готово</span>';
}

function setStepFailed(stepId) {
    const step = document.getElementById(stepId);
    const statusEl = document.getElementById(stepId + 'Status');
    
    step.classList.add('failed');
    step.classList.remove('active', 'completed');
    statusEl.innerHTML = '<span class="status-badge error">Ошибка</span>';
}

function showToolResult(resultId, message, type = 'info') {
    const resultEl = document.getElementById(resultId);
    resultEl.style.display = 'block';
    resultEl.className = `tool-result ${type}`;
    resultEl.querySelector('.result-message').textContent = message;
}

// ============================================
// Инструменты - Прогресс бары
// ============================================

function updateProgress(toolName, progress, text) {
    const progressEl = document.getElementById(toolName + 'Progress');
    const fillEl = document.getElementById(toolName + 'ProgressFill');
    const textEl = document.getElementById(toolName + 'ProgressText');
    
    progressEl.style.display = 'block';
    
    if (progress > 0 && progress < 100) {
        progressEl.classList.add('running');
    } else {
        progressEl.classList.remove('running');
    }
    
    fillEl.style.width = progress + '%';
    textEl.textContent = text;
}

function hideProgress(toolName) {
    const progressEl = document.getElementById(toolName + 'Progress');
    progressEl.style.display = 'none';
}

// ============================================
// Обработчики событий
// ============================================

function setupEventListeners() {
    // Переключение вкладок из sidebar
    document.querySelectorAll('.nav-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const tabId = btn.dataset.tab;

            // Убираем активный класс у всех кнопок и контента
            document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));

            // Добавляем активный класс текущим
            btn.classList.add('active');
            document.getElementById(tabId).classList.add('active');
        });
    });

    // Кнопка запуска приложения
    document.getElementById('launchBtn').addEventListener('click', async () => {
        const btn = document.getElementById('launchBtn');
        
        setButtonLoading(btn, true);
        updateStatusIndicator('running');
        updateLaunchStatus('Выполняется запуск...', 'info');
        addLog('Инициализация запуска приложения', 'info');

        try {
            const result = await launchApp();

            if (result.success) {
                updateLaunchStatus('Приложение успешно запущено', 'success');
                updateStatusIndicator('ready');
                addLog('Приложение запущено успешно', 'success');
                showToast('Приложение запущено', 'success');
            } else {
                updateLaunchStatus(result.message || 'Ошибка запуска', 'error');
                updateStatusIndicator('error');
                addLog(`Ошибка запуска: ${result.message}`, 'error');
                showToast('Ошибка при запуске: ' + result.message, 'error');
            }

        } catch (error) {
            updateLaunchStatus('Ошибка соединения с сервером', 'error');
            updateStatusIndicator('error');
            addLog(`Критическая ошибка: ${error.message}`, 'error');
            showToast('Ошибка соединения: ' + error.message, 'error');
        } finally {
            setButtonLoading(btn, false);
        }
    });

    // Кнопка инструкции
    document.getElementById('instructionBtn').addEventListener('click', () => {
        document.getElementById('instructionModal').style.display = 'flex';
    });

    // Закрытие модального окна инструкции
    document.getElementById('instructionCloseBtn').addEventListener('click', () => {
        document.getElementById('instructionModal').style.display = 'none';
    });

    document.getElementById('instructionOkBtn').addEventListener('click', () => {
        document.getElementById('instructionModal').style.display = 'none';
    });

    // Закрытие по клику вне окна
    document.getElementById('instructionModal').addEventListener('click', (e) => {
        if (e.target.id === 'instructionModal') {
            e.target.style.display = 'none';
        }
    });

    // Кнопка очистки логов
    document.getElementById('clearLogsBtn').addEventListener('click', async () => {
        try {
            await clearLogsApi();
            clearLogs();
        } catch (error) {
            // Очищаем локально даже если API недоступно
            clearLogs();
        }
    });

    // Кнопка "Чистка строк"
    document.getElementById('cleanStringsBtn').addEventListener('click', async () => {
        const btn = document.getElementById('cleanStringsBtn');
        
        setButtonLoading(btn, true);
        document.getElementById('cleanStringsResult').style.display = 'none';
        
        // Сброс статусов шагов
        resetStepStatus('cleanStringsStep1');
        resetStepStatus('cleanStringsStep2');
        
        addLog('Запуск чистки строк', 'info');

        try {
            // Шаг 1: Удаление журнала USN
            setStepRunning('cleanStringsStep1');
            addLog('Шаг 1: Удаление журнала USN...', 'info');
            
            const result = await cleanStrings();
            
            if (result.success) {
                setStepCompleted('cleanStringsStep1');
                
                // Шаг 2: Создание журнала USN
                setStepRunning('cleanStringsStep2');
                addLog('Шаг 2: Создание журнала USN...', 'info');
                
                // Имитация задержки для второго шага
                await new Promise(resolve => setTimeout(resolve, 1000));
                
                setStepCompleted('cleanStringsStep2');
                
                showToolResult('cleanStringsResult', 'Очистка строк успешно завершена', 'success');
                addLog('Чистка строк завершена успешно', 'success');
                showToast('Очистка строк завершена', 'success');
            } else {
                setStepFailed('cleanStringsStep1');
                showToolResult('cleanStringsResult', result.message || 'Ошибка при выполнении', 'error');
                addLog(`Ошибка чистки: ${result.message}`, 'error');
                showToast('Ошибка: ' + result.message, 'error');
            }

        } catch (error) {
            setStepFailed('cleanStringsStep1');
            showToolResult('cleanStringsResult', 'Ошибка соединения с сервером', 'error');
            addLog(`Критическая ошибка чистки: ${error.message}`, 'error');
            showToast('Ошибка соединения: ' + error.message, 'error');
        } finally {
            setButtonLoading(btn, false);
        }
    });

    // Кнопка "Очистка следов"
    document.getElementById('cleanTracksBtn').addEventListener('click', async () => {
        const btn = document.getElementById('cleanTracksBtn');
        
        setButtonLoading(btn, true);
        document.getElementById('cleanTracksResult').style.display = 'none';
        updateProgress('cleanTracks', 10, 'Запуск...');
        
        addLog('Запуск очистки следов', 'info');

        try {
            const result = await cleanTracks();
            
            if (result.success) {
                updateProgress('cleanTracks', 100, 'Завершено');
                showToolResult('cleanTracksResult', 'Очистка следов выполнена', 'success');
                addLog('Очистка следов завершена', 'success');
                showToast('Очистка следов завершена', 'success');
                
                setTimeout(() => hideProgress('cleanTracks'), 3000);
            } else {
                updateProgress('cleanTracks', 100, 'Ошибка');
                showToolResult('cleanTracksResult', result.message || 'Ошибка при выполнении', 'error');
                addLog(`Ошибка очистки: ${result.message}`, 'error');
                showToast('Ошибка: ' + result.message, 'error');
            }

        } catch (error) {
            updateProgress('cleanTracks', 100, 'Ошибка');
            showToolResult('cleanTracksResult', 'Ошибка соединения с сервером', 'error');
            addLog(`Критическая ошибка очистки: ${error.message}`, 'error');
            showToast('Ошибка соединения: ' + error.message, 'error');
        } finally {
            setButtonLoading(btn, false);
        }
    });

    // Кнопка "Симуляция открытия папок"
    document.getElementById('simulateBtn').addEventListener('click', async () => {
        const btn = document.getElementById('simulateBtn');
        
        setButtonLoading(btn, true);
        document.getElementById('simulateResult').style.display = 'none';
        updateProgress('simulate', 50, 'Запуск...');
        
        addLog('Запуск симуляции открытия папок', 'info');

        try {
            const result = await simulateFolders();
            
            if (result.success) {
                updateProgress('simulate', 100, 'Запущено');
                showToolResult('simulateResult', 'Симуляция запущена', 'success');
                addLog('Симуляция запущена успешно', 'success');
                showToast('Симуляция запущена', 'success');
                
                setTimeout(() => hideProgress('simulate'), 3000);
            } else {
                updateProgress('simulate', 100, 'Ошибка');
                showToolResult('simulateResult', result.message || 'Ошибка при выполнении', 'error');
                addLog(`Ошибка симуляции: ${result.message}`, 'error');
                showToast('Ошибка: ' + result.message, 'error');
            }

        } catch (error) {
            updateProgress('simulate', 100, 'Ошибка');
            showToolResult('simulateResult', 'Ошибка соединения с сервером', 'error');
            addLog(`Критическая ошибка симуляции: ${error.message}`, 'error');
            showToast('Ошибка соединения: ' + error.message, 'error');
        } finally {
            setButtonLoading(btn, false);
        }
    });

    // Очистка памяти javaw.exe
    document.getElementById('cleanJavawBtn').addEventListener('click', async () => {
        const btn = document.getElementById('cleanJavawBtn');

        setButtonLoading(btn, true);
        document.getElementById('cleanJavawResult').style.display = 'none';
        updateProgress('cleanJavaw', 10, 'Подключение...');

        addLog('Запуск очистки памяти javaw.exe', 'info');

        try {
            const result = await cleanJavawMemory();

            if (result.success) {
                updateProgress('cleanJavaw', 100, 'Завершено');
                const msg = result.message || `Удалено ${result.cleared_count} совпадений`;
                showToolResult('cleanJavawResult', msg, 'success');
                addLog(`Очистка javaw завершена: ${msg}`, 'success');
                showToast('Очистка памяти javaw.exe завершена', 'success');

                setTimeout(() => hideProgress('cleanJavaw'), 3000);
            } else {
                updateProgress('cleanJavaw', 100, 'Ошибка');
                showToolResult('cleanJavawResult', result.message || 'Ошибка при выполнении', 'error');
                addLog(`Ошибка очистки javaw: ${result.message}`, 'error');
                showToast('Ошибка: ' + result.message, 'error');
            }

        } catch (error) {
            updateProgress('cleanJavaw', 100, 'Ошибка');
            showToolResult('cleanJavawResult', 'Ошибка соединения с сервером', 'error');
            addLog(`Критическая ошибка очистки javaw: ${error.message}`, 'error');
            showToast('Ошибка соединения: ' + error.message, 'error');
        } finally {
            setButtonLoading(btn, false);
        }
    });

    // Глобальная очистка - открытие модального окна
    document.getElementById('globalCleanBtn').addEventListener('click', async () => {
        const modal = document.getElementById('globalCleanModal');
        const optionsContainer = document.getElementById('cleanOptions');
        
        try {
            const data = await getGlobalCleanOptions();
            optionsContainer.innerHTML = '';
            
            for (const [key, option] of Object.entries(data.options)) {
                const optionEl = document.createElement('label');
                optionEl.className = 'clean-option';
                optionEl.innerHTML = `
                    <input type="checkbox" value="${key}" id="opt_${key}">
                    <div class="clean-option-label">
                        <div class="clean-option-name">${option.name}</div>
                        <div class="clean-option-desc">${option.description}</div>
                    </div>
                `;
                optionsContainer.appendChild(optionEl);
            }
            
            modal.style.display = 'flex';
        } catch (error) {
            showToast('Ошибка загрузки опций: ' + error.message, 'error');
        }
    });

    // Закрытие модального окна
    document.getElementById('modalCloseBtn').addEventListener('click', () => {
        document.getElementById('globalCleanModal').style.display = 'none';
    });

    document.getElementById('modalCancelBtn').addEventListener('click', () => {
        document.getElementById('globalCleanModal').style.display = 'none';
    });

    // Закрытие по клику вне окна
    document.getElementById('globalCleanModal').addEventListener('click', (e) => {
        if (e.target.id === 'globalCleanModal') {
            e.target.style.display = 'none';
        }
    });

    // Запуск глобальной очистки
    document.getElementById('modalStartBtn').addEventListener('click', async () => {
        const btn = document.getElementById('modalStartBtn');
        const checkboxes = document.querySelectorAll('#cleanOptions input[type="checkbox"]:checked');
        
        if (checkboxes.length === 0) {
            showToast('Выберите хотя бы один компонент', 'warning');
            return;
        }
        
        const options = {};
        checkboxes.forEach(cb => {
            options[cb.value] = true;
        });
        
        setButtonLoading(btn, true);
        document.getElementById('globalCleanModal').style.display = 'none';
        document.getElementById('globalCleanResult').style.display = 'none';
        updateProgress('globalClean', 0, 'Запуск...');
        
        addLog('Запуск глобальной очистки', 'info');

        try {
            const result = await runGlobalClean(options);
            
            if (result.success) {
                updateProgress('globalClean', 100, `Завершено: ${result.completed}/${result.total}`);
                showToolResult('globalCleanResult', `Очистка завершена: ${result.completed}/${result.total} успешно`, 'success');
                addLog(`Глобальная очистка: ${result.completed}/${result.total} успешно`, 'success');
                showToast(`Очистка завершена: ${result.completed}/${result.total}`, 'success');
                
                // Показать детали
                let details = '';
                for (const [key, res] of Object.entries(result.results)) {
                    details += `${res.success ? '✓' : '✗'} ${key}: ${res.message}\n`;
                }
                console.log(details);
                
                setTimeout(() => hideProgress('globalClean'), 5000);
            } else {
                updateProgress('globalClean', 100, 'Ошибка');
                showToolResult('globalCleanResult', result.message || 'Ошибка при выполнении', 'error');
                showToast('Ошибка: ' + result.message, 'error');
            }

        } catch (error) {
            updateProgress('globalClean', 100, 'Ошибка');
            showToolResult('globalCleanResult', 'Ошибка соединения с сервером', 'error');
            showToast('Ошибка соединения: ' + error.message, 'error');
        } finally {
            setButtonLoading(btn, false);
        }
    });
}

// Загрузка при старте
async function init() {
    try {
        // Получаем текущий статус
        const status = await getStatus();
        updateStatusIndicator(status.status || 'ready');
        updateFileStatus(status.file_exists, status.file_size);
        
        // Загружаем логи если есть
        try {
            const logsData = await getLogs();
            if (logsData.logs && logsData.logs.length > 0) {
                const container = document.getElementById('logsContainer');
                container.innerHTML = '';
                logsData.logs.forEach(log => {
                    if (log.message) {
                        addLog(log.message, log.type || 'info');
                    }
                });
            }
        } catch (e) {
            // Логи не загрузились - не критично
        }

        addLog('Веб-интерфейс инициализирован', 'info');
        
    } catch (error) {
        console.error('Init error:', error);
        addLog('Ошибка инициализации: ' + error.message, 'error');
    }
}

// Запуск после загрузки DOM
document.addEventListener('DOMContentLoaded', () => {
    setupEventListeners();
    init();
});
