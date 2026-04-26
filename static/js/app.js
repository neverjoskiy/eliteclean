const { invoke } = window.__TAURI__.core;
let logs = [];

// ── Toast ──
function showToast(message, type = 'info') {
    const icons = {
        success: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>',
        error:   '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>',
        warning: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/></svg>',
        info:    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>'
    };
    const t = document.createElement('div');
    t.className = `toast ${type}`;
    t.innerHTML = `<span class="toast-icon">${icons[type]}</span><span class="toast-message">${message}</span>`;
    document.getElementById('toastContainer').appendChild(t);
    setTimeout(() => { t.classList.add('hiding'); setTimeout(() => t.remove(), 250); }, 3500);
}

// ── Logs ──
function addLog(message, type = 'info') {
    const ts = new Date().toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    logs.push({ ts, message, type });
    const c = document.getElementById('logsContainer');
    if (logs.length === 1) c.innerHTML = '';
    const el = document.createElement('div');
    el.className = `log-entry ${type}`;
    el.innerHTML = `<span class="lt">${ts}</span><span class="lm">${message}</span>`;
    c.appendChild(el);
    c.scrollTop = c.scrollHeight;
    if (logs.length > 50) logs.shift();
}

function clearLogs() {
    logs = [];
    document.getElementById('logsContainer').innerHTML =
        '<div class="log-entry info"><span class="lt">--:--:--</span><span class="lm">ожидание событий...</span></div>';
    showToast('журнал очищен', 'info');
}

// ── Status ──
function updateStatusIndicator(status) {
    const el = document.getElementById('statusIndicator');
    el.className = 'sdot ' + (status || 'ready');
}

function setLaunchStatus(message, type = '') {
    const el = document.getElementById('launchStatus');
    el.className = 'launch-status' + (type ? ' ' + type : '');
    el.textContent = message;
}

// ── Steps ──
function resetStep(id) {
    const s = document.getElementById(id);
    s.classList.remove('active', 'completed', 'failed');
    document.getElementById(id + 'Status').className = 'sb pending';
    document.getElementById(id + 'Status').textContent = '—';
}
function stepRunning(id) {
    const s = document.getElementById(id);
    s.classList.add('active'); s.classList.remove('completed', 'failed');
    const b = document.getElementById(id + 'Status');
    b.className = 'sb running'; b.textContent = '...';
}
function stepDone(id) {
    const s = document.getElementById(id);
    s.classList.add('completed'); s.classList.remove('active', 'failed');
    const b = document.getElementById(id + 'Status');
    b.className = 'sb success'; b.textContent = '✓';
}
function stepFail(id) {
    const s = document.getElementById(id);
    s.classList.add('failed'); s.classList.remove('active', 'completed');
    const b = document.getElementById(id + 'Status');
    b.className = 'sb error'; b.textContent = '✗';
}

// ── Progress ──
function showProgress(name, pct, text) {
    const el = document.getElementById(name + 'Progress');
    el.classList.remove('hidden');
    document.getElementById(name + 'ProgressFill').style.width = pct + '%';
    document.getElementById(name + 'ProgressText').textContent = text;
}
function hideProgress(name) {
    document.getElementById(name + 'Progress').classList.add('hidden');
}

// ── Tool Result ──
function showResult(id, message, type) {
    const el = document.getElementById(id);
    el.className = `tool-result ${type}`;
    el.querySelector('.result-message').textContent = message;
    el.classList.remove('hidden');
}
function hideResult(id) {
    document.getElementById(id).classList.add('hidden');
}

// ── Modal ──
function openModal(id) { document.getElementById(id).classList.remove('hidden'); }
function closeModal(id) { document.getElementById(id).classList.add('hidden'); }

// ── Button loading ──
function setBtnLoading(btn, on) {
    btn.classList.toggle('loading', on);
    btn.disabled = on;
}

// ── API ──
const api = {
    launch:          () => invoke('launch_app'),
    status:          () => invoke('get_status'),
    logs:            (n=50) => invoke('get_logs', { lines: n }),
    clearLogs:       () => invoke('clear_logs'),
    cleanStrings:    () => invoke('clean_strings'),
    cleanTracks:     () => invoke('clean_tracks'),
    simulate:        () => invoke('simulate_folders'),
    cleanJavaw:      () => invoke('clean_javaw_memory'),
    globalOptions:   () => invoke('get_global_clean_options'),
    globalClean:     (p) => invoke('run_global_clean', { params: p }),
    // сеть
    flushDns:        () => invoke('flush_dns'),
    resetNetwork:    () => invoke('reset_network'),
    clearArp:        () => invoke('clear_arp'),
    clearNetbios:    () => invoke('clear_netbios'),
    // система
    cleanRegistry:   () => invoke('clean_registry'),
    cleanDumps:      () => invoke('clean_dumps'),
    cleanWu:         () => invoke('clean_update_cache'),
    cleanThumbs:     () => invoke('clean_thumbnails'),
    // приватность
    clearClipboard:  () => invoke('clear_clipboard'),
    cleanIconCache:  () => invoke('clean_icon_cache'),
    cleanSearch:     () => invoke('clean_search_history'),
    cleanRun:        () => invoke('clean_run_history'),
};

// ── Init ──
async function init() {
    try {
        const s = await api.status();
        updateStatusIndicator(s.status || 'ready');
        try {
            const ld = await api.logs();
            if (ld.logs?.length) {
                document.getElementById('logsContainer').innerHTML = '';
                ld.logs.forEach(l => l.message && addLog(l.message, l.type || 'info'));
            }
        } catch {}
        addLog('интерфейс инициализирован', 'info');
    } catch (e) {
        addLog('ошибка инициализации: ' + e.message, 'error');
    }
}

// ── Event Listeners ──
document.addEventListener('DOMContentLoaded', () => {

    // Tabs
    document.querySelectorAll('.nav-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            btn.classList.add('active');
            document.getElementById(btn.dataset.tab).classList.add('active');
        });
    });

    // Launch
    document.getElementById('launchBtn').addEventListener('click', async () => {
        const btn = document.getElementById('launchBtn');
        setBtnLoading(btn, true);
        updateStatusIndicator('running');
        setLaunchStatus('запуск...');
        addLog('запуск приложения', 'info');
        try {
            const r = await api.launch();
            if (r.success) {
                setLaunchStatus('запущено успешно', 'success');
                updateStatusIndicator('ready');
                addLog('приложение запущено', 'success');
                showToast('запущено', 'success');
            } else {
                setLaunchStatus(r.message || 'ошибка', 'error');
                updateStatusIndicator('error');
                addLog('ошибка: ' + r.message, 'error');
                showToast(r.message, 'error');
            }
        } catch (e) {
            setLaunchStatus('ошибка соединения', 'error');
            updateStatusIndicator('error');
            addLog('ошибка: ' + e.message, 'error');
            showToast(e.message, 'error');
        } finally { setBtnLoading(btn, false); }
    });

    // Instruction modal
    document.getElementById('instructionBtn').addEventListener('click', () => openModal('instructionModal'));
    document.getElementById('instructionCloseBtn').addEventListener('click', () => closeModal('instructionModal'));
    document.getElementById('instructionOkBtn').addEventListener('click', () => closeModal('instructionModal'));
    document.getElementById('instructionModal').addEventListener('click', e => { if (e.target === e.currentTarget) closeModal('instructionModal'); });

    // Clear logs
    document.getElementById('clearLogsBtn').addEventListener('click', async () => {
        try { await api.clearLogs(); } catch {}
        clearLogs();
    });

    // Clean Strings
    document.getElementById('cleanStringsBtn').addEventListener('click', async () => {
        const btn = document.getElementById('cleanStringsBtn');
        setBtnLoading(btn, true);
        hideResult('cleanStringsResult');
        resetStep('cleanStringsStep1'); resetStep('cleanStringsStep2');
        addLog('чистка строк...', 'info');
        try {
            stepRunning('cleanStringsStep1');
            const r = await api.cleanStrings();
            if (r.success) {
                stepDone('cleanStringsStep1');
                stepRunning('cleanStringsStep2');
                await new Promise(res => setTimeout(res, 800));
                stepDone('cleanStringsStep2');
                showResult('cleanStringsResult', 'чистка завершена', 'success');
                addLog('чистка строк завершена', 'success');
                showToast('чистка строк завершена', 'success');
            } else {
                stepFail('cleanStringsStep1');
                showResult('cleanStringsResult', r.message || 'ошибка', 'error');
                addLog('ошибка: ' + r.message, 'error');
                showToast(r.message, 'error');
            }
        } catch (e) {
            stepFail('cleanStringsStep1');
            showResult('cleanStringsResult', e.message, 'error');
            showToast(e.message, 'error');
        } finally { setBtnLoading(btn, false); }
    });

    // Clean Tracks
    document.getElementById('cleanTracksBtn').addEventListener('click', async () => {
        const btn = document.getElementById('cleanTracksBtn');
        setBtnLoading(btn, true);
        hideResult('cleanTracksResult');
        showProgress('cleanTracks', 10, 'запуск...');
        addLog('очистка следов...', 'info');
        try {
            const r = await api.cleanTracks();
            if (r.success) {
                showProgress('cleanTracks', 100, 'завершено');
                showResult('cleanTracksResult', 'очистка следов выполнена', 'success');
                addLog('очистка следов завершена', 'success');
                showToast('очистка следов завершена', 'success');
                setTimeout(() => hideProgress('cleanTracks'), 2500);
            } else {
                showProgress('cleanTracks', 100, 'ошибка');
                showResult('cleanTracksResult', r.message || 'ошибка', 'error');
                addLog('ошибка: ' + r.message, 'error');
                showToast(r.message, 'error');
            }
        } catch (e) {
            showProgress('cleanTracks', 100, 'ошибка');
            showResult('cleanTracksResult', e.message, 'error');
            showToast(e.message, 'error');
        } finally { setBtnLoading(btn, false); }
    });

    // Simulate
    document.getElementById('simulateBtn').addEventListener('click', async () => {
        const btn = document.getElementById('simulateBtn');
        setBtnLoading(btn, true);
        hideResult('simulateResult');
        showProgress('simulate', 50, 'запуск...');
        addLog('симуляция папок...', 'info');
        try {
            const r = await api.simulate();
            if (r.success) {
                showProgress('simulate', 100, 'запущено');
                showResult('simulateResult', 'симуляция запущена', 'success');
                addLog('симуляция запущена', 'success');
                showToast('симуляция запущена', 'success');
                setTimeout(() => hideProgress('simulate'), 2500);
            } else {
                showProgress('simulate', 100, 'ошибка');
                showResult('simulateResult', r.message || 'ошибка', 'error');
                addLog('ошибка: ' + r.message, 'error');
                showToast(r.message, 'error');
            }
        } catch (e) {
            showProgress('simulate', 100, 'ошибка');
            showResult('simulateResult', e.message, 'error');
            showToast(e.message, 'error');
        } finally { setBtnLoading(btn, false); }
    });

    // Clean Javaw
    document.getElementById('cleanJavawBtn').addEventListener('click', async () => {
        const btn = document.getElementById('cleanJavawBtn');
        setBtnLoading(btn, true);
        hideResult('cleanJavawResult');
        showProgress('cleanJavaw', 10, 'подключение...');
        addLog('очистка памяти javaw.exe...', 'info');
        try {
            const r = await api.cleanJavaw();
            if (r.success) {
                showProgress('cleanJavaw', 100, 'завершено');
                const msg = r.message || `удалено ${r.cleared_count} совпадений`;
                showResult('cleanJavawResult', msg, 'success');
                addLog('javaw: ' + msg, 'success');
                showToast('очистка javaw завершена', 'success');
                setTimeout(() => hideProgress('cleanJavaw'), 2500);
            } else {
                showProgress('cleanJavaw', 100, 'ошибка');
                showResult('cleanJavawResult', r.message || 'ошибка', 'error');
                addLog('ошибка javaw: ' + r.message, 'error');
                showToast(r.message, 'error');
            }
        } catch (e) {
            showProgress('cleanJavaw', 100, 'ошибка');
            showResult('cleanJavawResult', e.message, 'error');
            showToast(e.message, 'error');
        } finally { setBtnLoading(btn, false); }
    });

    // Global Clean — open modal
    document.getElementById('globalCleanBtn').addEventListener('click', async () => {
        try {
            const data = await api.globalOptions();
            const c = document.getElementById('cleanOptions');
            c.innerHTML = '';
            for (const [key, opt] of Object.entries(data.options)) {
                const el = document.createElement('label');
                el.className = 'clean-option';
                el.innerHTML = `<input type="checkbox" value="${key}"><div class="clean-option-label"><div class="clean-option-name">${opt.name}</div><div class="clean-option-desc">${opt.description}</div></div>`;
                c.appendChild(el);
            }
            openModal('globalCleanModal');
        } catch (e) { showToast('ошибка загрузки опций: ' + e.message, 'error'); }
    });

    document.getElementById('modalCloseBtn').addEventListener('click', () => closeModal('globalCleanModal'));
    document.getElementById('modalCancelBtn').addEventListener('click', () => closeModal('globalCleanModal'));
    document.getElementById('globalCleanModal').addEventListener('click', e => { if (e.target === e.currentTarget) closeModal('globalCleanModal'); });

    // Global Clean — run
    document.getElementById('modalStartBtn').addEventListener('click', async () => {
        const btn = document.getElementById('modalStartBtn');
        const checked = document.querySelectorAll('#cleanOptions input:checked');
        if (!checked.length) { showToast('выберите хотя бы один пункт', 'warning'); return; }
        const params = {};
        checked.forEach(cb => params[cb.value] = true);
        setBtnLoading(btn, true);
        closeModal('globalCleanModal');
        hideResult('globalCleanResult');
        showProgress('globalClean', 0, 'запуск...');
        addLog('глобальная очистка...', 'info');
        try {
            const r = await api.globalClean(params);
            if (r.success) {
                showProgress('globalClean', 100, `${r.completed}/${r.total}`);
                showResult('globalCleanResult', `завершено: ${r.completed}/${r.total}`, 'success');
                addLog(`глобальная очистка: ${r.completed}/${r.total}`, 'success');
                showToast(`очистка: ${r.completed}/${r.total}`, 'success');
                setTimeout(() => hideProgress('globalClean'), 4000);
            } else {
                showProgress('globalClean', 100, 'ошибка');
                showResult('globalCleanResult', r.message || 'ошибка', 'error');
                showToast(r.message, 'error');
            }
        } catch (e) {
            showProgress('globalClean', 100, 'ошибка');
            showResult('globalCleanResult', e.message, 'error');
            showToast(e.message, 'error');
        } finally { setBtnLoading(btn, false); }
    });

    // ── Helpers for new tabs ──
    function showDetails(id, items) {
        const el = document.getElementById(id);
        if (!el) return;
        el.innerHTML = items.map(d => `<div class="detail-item">${d}</div>`).join('');
        el.classList.remove('hidden');
    }

    async function runSimpleClean(btnId, resultId, apiFn, label, detailsId) {
        const btn = document.getElementById(btnId);
        setBtnLoading(btn, true);
        hideResult(resultId);
        if (detailsId) document.getElementById(detailsId)?.classList.add('hidden');
        addLog(`${label}...`, 'info');
        try {
            const r = await apiFn();
            const type = r.success ? 'success' : 'error';
            showResult(resultId, r.message, type);
            addLog(`${label}: ${r.message}`, type);
            showToast(r.success ? `${label} завершено` : r.message, type);
            if (detailsId && r.details?.length) showDetails(detailsId, r.details);
        } catch (e) {
            showResult(resultId, e.message, 'error');
            showToast(e.message, 'error');
        } finally { setBtnLoading(btn, false); }
    }

    async function runProgressClean(btnId, resultId, progressName, apiFn, label, detailsId) {
        const btn = document.getElementById(btnId);
        setBtnLoading(btn, true);
        hideResult(resultId);
        if (detailsId) document.getElementById(detailsId)?.classList.add('hidden');
        showProgress(progressName, 20, 'запуск...');
        addLog(`${label}...`, 'info');
        try {
            const r = await apiFn();
            const type = r.success ? 'success' : 'error';
            showProgress(progressName, 100, r.success ? 'завершено' : 'ошибка');
            showResult(resultId, r.message, type);
            addLog(`${label}: ${r.message}`, type);
            showToast(r.success ? `${label} завершено` : r.message, type);
            if (detailsId && r.details?.length) showDetails(detailsId, r.details);
            setTimeout(() => hideProgress(progressName), 2500);
        } catch (e) {
            showProgress(progressName, 100, 'ошибка');
            showResult(resultId, e.message, 'error');
            showToast(e.message, 'error');
        } finally { setBtnLoading(btn, false); }
    }

    // ── Сеть ──
    document.getElementById('flushDnsBtn').addEventListener('click', () =>
        runSimpleClean('flushDnsBtn', 'flushDnsResult', api.flushDns, 'сброс DNS'));

    document.getElementById('clearArpBtn').addEventListener('click', () =>
        runSimpleClean('clearArpBtn', 'clearArpResult', api.clearArp, 'очистка ARP'));

    document.getElementById('clearNetbiosBtn').addEventListener('click', () =>
        runSimpleClean('clearNetbiosBtn', 'clearNetbiosResult', api.clearNetbios, 'очистка NetBIOS'));

    document.getElementById('resetNetworkBtn').addEventListener('click', () =>
        runProgressClean('resetNetworkBtn', 'resetNetworkResult', 'resetNetwork', api.resetNetwork, 'сброс сети', 'resetNetworkDetails'));

    // ── Система ──
    document.getElementById('cleanRegistryBtn').addEventListener('click', () =>
        runSimpleClean('cleanRegistryBtn', 'cleanRegistryResult', api.cleanRegistry, 'очистка реестра', 'cleanRegistryDetails'));

    document.getElementById('cleanDumpsBtn').addEventListener('click', () =>
        runSimpleClean('cleanDumpsBtn', 'cleanDumpsResult', api.cleanDumps, 'очистка дампов'));

    document.getElementById('cleanWuBtn').addEventListener('click', () =>
        runProgressClean('cleanWuBtn', 'cleanWuResult', 'cleanWu', api.cleanWu, 'кэш обновлений'));

    document.getElementById('cleanThumbBtn').addEventListener('click', () =>
        runSimpleClean('cleanThumbBtn', 'cleanThumbResult', api.cleanThumbs, 'thumbnail кэш'));

    // ── Приватность ──
    document.getElementById('clearClipboardBtn').addEventListener('click', () =>
        runSimpleClean('clearClipboardBtn', 'clearClipboardResult', api.clearClipboard, 'буфер обмена'));

    document.getElementById('cleanIconBtn').addEventListener('click', () =>
        runSimpleClean('cleanIconBtn', 'cleanIconResult', api.cleanIconCache, 'кэш иконок'));

    document.getElementById('cleanSearchBtn').addEventListener('click', () =>
        runSimpleClean('cleanSearchBtn', 'cleanSearchResult', api.cleanSearch, 'история поиска'));

    document.getElementById('cleanRunBtn').addEventListener('click', () =>
        runSimpleClean('cleanRunBtn', 'cleanRunResult', api.cleanRun, 'история запуска'));

    init();
});
