const { invoke } = window.__TAURI__.core;
let logs = [];
let scanData = null; // хранит последние результаты сканирования

// ── Audio ──
const _ac = new (window.AudioContext || window.webkitAudioContext)();

function _playTone(freq, type, duration, vol = 0.08) {
    try {
        const o = _ac.createOscillator();
        const g = _ac.createGain();
        o.connect(g); g.connect(_ac.destination);
        o.type = type; o.frequency.value = freq;
        g.gain.setValueAtTime(vol, _ac.currentTime);
        g.gain.exponentialRampToValueAtTime(0.0001, _ac.currentTime + duration);
        o.start(_ac.currentTime);
        o.stop(_ac.currentTime + duration);
    } catch {}
}

const sfx = {
    nav:     () => { if (settings.sounds) _playTone(880, 'sine', 0.08, 0.06); },
    success: () => { if (settings.sounds) { _playTone(660, 'sine', 0.12, 0.07); setTimeout(() => _playTone(880, 'sine', 0.15, 0.06), 80); } },
    error:   () => { if (settings.sounds) _playTone(220, 'sawtooth', 0.18, 0.05); },
    boot:    () => { if (settings.sounds) { _playTone(440, 'sine', 0.15, 0.05); setTimeout(() => _playTone(550, 'sine', 0.15, 0.05), 120); setTimeout(() => _playTone(660, 'sine', 0.25, 0.07), 240); } },
    click:   () => { if (settings.sounds) _playTone(1200, 'sine', 0.06, 0.04); },
};

// ── Splash ──
async function runSplash() {
    const splash = document.getElementById('splashScreen');
    const fill   = document.getElementById('splashBarFill');
    const status = document.getElementById('splashStatus');
    const app    = document.getElementById('appRoot');

    const steps = [
        [15,  'загрузка модулей...'],
        [40,  'подключение к системе...'],
        [70,  'проверка компонентов...'],
        [90,  'почти готово...'],
        [100, 'готово'],
    ];

    sfx.boot();

    for (const [pct, msg] of steps) {
        fill.style.width = pct + '%';
        status.textContent = msg;
        await new Promise(r => setTimeout(r, pct === 100 ? 200 : 280 + Math.random() * 120));
    }

    await new Promise(r => setTimeout(r, 300));
    splash.classList.add('hiding');
    app.style.transition = 'opacity .4s ease';
    app.style.opacity = '1';
    setTimeout(() => splash.remove(), 450);
}

// ── Odometer ──
function animateOdo(el, from, to, duration = 600) {
    const start = performance.now();
    const update = (now) => {
        const p = Math.min((now - start) / duration, 1);
        const ease = 1 - Math.pow(1 - p, 3);
        const val = Math.round(from + (to - from) * ease);
        el.textContent = val.toLocaleString('ru-RU');
        if (p < 1) requestAnimationFrame(update);
    };
    requestAnimationFrame(update);
}

// ── Scan ring ──
function setScanRing(pct) {
    const circ = 94.25; // 2π * 15
    const fill = document.getElementById('scanRingFill');
    if (!fill) return;
    fill.style.strokeDashoffset = circ - (circ * pct / 100);
}

// ── Settings ──
const settings = {
    sounds: true,
    animations: true,
    compact: false,
    confirm: false,
    autoScan: false,
    logLimit: 50,
};

function loadSettings() {
    try {
        const saved = JSON.parse(localStorage.getItem('ec_settings') || '{}');
        Object.assign(settings, saved);
    } catch {}
    applySettings();
}

function saveSettings() {
    localStorage.setItem('ec_settings', JSON.stringify(settings));
}

function applySettings() {
    document.getElementById('settingSounds').checked     = settings.sounds;
    document.getElementById('settingAnimations').checked = settings.animations;
    document.getElementById('settingCompact').checked    = settings.compact;
    document.getElementById('settingConfirm').checked    = settings.confirm;
    document.getElementById('settingAutoScan').checked   = settings.autoScan;
    document.getElementById('settingLogLimit').value     = settings.logLimit;
    document.body.classList.toggle('compact', settings.compact);
    document.body.classList.toggle('no-anim', !settings.animations);
}
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
    if (type === 'success') sfx.success();
    else if (type === 'error') sfx.error();
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
    funTime:         () => invoke('fun_time'),
    globalOptions:   () => invoke('get_global_clean_options'),
    globalClean:     (options) => invoke('run_global_clean', { params: { options } }),
    scanSystem:      () => invoke('scan_system'),
    cleanScan:       (ids) => invoke('clean_scan_results', { params: { ids } }),
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
    // твики
    getTweaks:       () => invoke('get_tweaks'),
    applyTweak:      (id) => invoke('apply_tweak', { id }),
    revertTweak:     (id) => invoke('revert_tweak', { id }),
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

    // Splash
    runSplash().then(() => { loadSettings(); init(); });

    // Звук на все кнопки действий
    document.addEventListener('click', e => {
        if (e.target.closest('.btn-tool, .btn-run, .hql-btn')) sfx.click();
    });

    // Tabs
    document.querySelectorAll('.nav-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            sfx.nav();
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

    // FunTime
    document.getElementById('funTimeBtn').addEventListener('click', async () => {
        const btn = document.getElementById('funTimeBtn');
        setBtnLoading(btn, true);
        hideResult('funTimeResult');
        document.getElementById('funTimeLines').classList.add('hidden');
        showProgress('funTime', 20, 'запуск 1fc.exe...');
        addLog('FunTime: запуск...', 'info');
        try {
            const r = await api.funTime();
            if (r.success) {
                showProgress('funTime', 100, 'завершено');
                showResult('funTimeResult', r.message, 'success');
                addLog('FunTime: ' + r.message, 'success');
                showToast('FunTime завершён', 'success');
                if (r.details && r.details.length) {
                    const el = document.getElementById('funTimeLines');
                    el.innerHTML = r.details.map(l => `<div>${l}</div>`).join('');
                    el.classList.remove('hidden');
                }
                setTimeout(() => hideProgress('funTime'), 2500);
            } else {
                showProgress('funTime', 100, 'ошибка');
                showResult('funTimeResult', r.message || 'ошибка', 'error');
                addLog('FunTime ошибка: ' + r.message, 'error');
                showToast(r.message, 'error');
            }
        } catch (e) {
            showProgress('funTime', 100, 'ошибка');
            showResult('funTimeResult', e.message, 'error');
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
        const options = Array.from(checked).map(cb => cb.value);
        setBtnLoading(btn, true);
        closeModal('globalCleanModal');
        hideResult('globalCleanResult');
        showProgress('globalClean', 0, 'запуск...');
        addLog('глобальная очистка...', 'info');
        try {
            const r = await api.globalClean(options);
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

    // ── Переходы со страницы home ──
    document.querySelectorAll('.home-section-goto').forEach(btn => {
        btn.addEventListener('click', () => {
            sfx.nav();
            const tab = btn.dataset.tab;
            document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelector(`.nav-btn[data-tab="${tab}"]`)?.classList.add('active');
            document.getElementById(tab)?.classList.add('active');
        });
    });

    // ── Быстрые кнопки на главной ──
    function hqRun(btnId, apiFn, label) {
        const btn = document.getElementById(btnId);
        const statusEl = document.getElementById(btnId + '-status');
        btn.disabled = true;
        if (statusEl) { statusEl.textContent = '…'; statusEl.className = 'hql-status run'; }
        addLog(`${label}...`, 'info');
        apiFn().then(r => {
            const ok = r.success;
            if (statusEl) { statusEl.textContent = ok ? '✓' : '✗'; statusEl.className = 'hql-status ' + (ok ? 'ok' : 'err'); }
            addLog(`${label}: ${r.message}`, ok ? 'success' : 'error');
            showToast(ok ? `${label} завершено` : r.message, ok ? 'success' : 'error');
        }).catch(e => {
            if (statusEl) { statusEl.textContent = '✗'; statusEl.className = 'hql-status err'; }
            showToast(e.message, 'error');
        }).finally(() => { btn.disabled = false; });
    }

    // инструменты
    document.getElementById('hq-cleanStrings').addEventListener('click', () => hqRun('hq-cleanStrings', api.cleanStrings, 'чистка строк'));
    document.getElementById('hq-cleanTracks').addEventListener('click', () => hqRun('hq-cleanTracks', api.cleanTracks, 'очистка следов'));
    document.getElementById('hq-cleanJavaw').addEventListener('click', () => hqRun('hq-cleanJavaw', api.cleanJavaw, 'javaw.exe'));
    document.getElementById('hq-globalClean').addEventListener('click', () => {
        // глобальная — открываем модалку
        document.getElementById('globalCleanBtn').click();
    });
    // сеть
    document.getElementById('hq-flushDns').addEventListener('click', () => hqRun('hq-flushDns', api.flushDns, 'DNS'));
    document.getElementById('hq-clearArp').addEventListener('click', () => hqRun('hq-clearArp', api.clearArp, 'ARP'));
    document.getElementById('hq-clearNetbios').addEventListener('click', () => hqRun('hq-clearNetbios', api.clearNetbios, 'NetBIOS'));
    document.getElementById('hq-resetNetwork').addEventListener('click', () => hqRun('hq-resetNetwork', api.resetNetwork, 'сброс сети'));
    // система
    document.getElementById('hq-cleanRegistry').addEventListener('click', () => hqRun('hq-cleanRegistry', api.cleanRegistry, 'реестр'));
    document.getElementById('hq-cleanDumps').addEventListener('click', () => hqRun('hq-cleanDumps', api.cleanDumps, 'дампы'));
    document.getElementById('hq-cleanWu').addEventListener('click', () => hqRun('hq-cleanWu', api.cleanWu, 'кэш WU'));
    document.getElementById('hq-cleanThumbs').addEventListener('click', () => hqRun('hq-cleanThumbs', api.cleanThumbs, 'thumbnails'));
    // приватность
    document.getElementById('hq-clearClipboard').addEventListener('click', () => hqRun('hq-clearClipboard', api.clearClipboard, 'буфер обмена'));
    document.getElementById('hq-cleanIconCache').addEventListener('click', () => hqRun('hq-cleanIconCache', api.cleanIconCache, 'иконки'));
    document.getElementById('hq-cleanSearch').addEventListener('click', () => hqRun('hq-cleanSearch', api.cleanSearch, 'поиск'));
    document.getElementById('hq-cleanRun').addEventListener('click', () => hqRun('hq-cleanRun', api.cleanRun, 'история запуска'));

    // ── Сканер ──
    function fmtSize(bytes) {
        if (bytes < 1024) return bytes + ' Б';
        if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' КБ';
        return (bytes / 1024 / 1024).toFixed(1) + ' МБ';
    }

    document.getElementById('scanBtn').addEventListener('click', async () => {
        const btn = document.getElementById('scanBtn');
        setBtnLoading(btn, true);
        btn.classList.add('scanning');
        setScanRing(0);
        document.getElementById('scanResults').classList.add('hidden');
        document.getElementById('scanCleanResult').classList.add('hidden');
        document.getElementById('scanProgress').classList.remove('hidden');
        document.getElementById('scanSubtitle').textContent = 'анализ...';

        // анимируем кольцо пока идёт сканирование
        let pct = 0;
        const ticker = setInterval(() => {
            pct = Math.min(pct + Math.random() * 12, 90);
            setScanRing(pct);
        }, 200);

        addLog('сканирование системы...', 'info');
        try {
            scanData = await api.scanSystem();
            clearInterval(ticker);
            setScanRing(100);
            btn.classList.remove('scanning');
            btn.classList.add('done');
            document.getElementById('scanProgressText').textContent = 'готово';

            // Рендерим результаты
            const total = scanData.total_size_bytes;
            const files = scanData.total_files;

            // Одометр для файлов и размера
            const sumEl = document.getElementById('scanSummary');
            sumEl.innerHTML = `<span class="scan-sum-files"><span id="odoFiles">0</span> файлов</span><span class="scan-sum-sep">·</span><span class="scan-sum-size"><span id="odoSize">0</span> МБ</span>`;
            animateOdo(document.getElementById('odoFiles'), 0, files, 700);
            animateOdo(document.getElementById('odoSize'), 0, Math.round(total / 1024 / 1024 * 10) / 10, 900);

            const cats = document.getElementById('scanCategories');
            cats.innerHTML = '';
            scanData.categories.forEach(cat => {
                const row = document.createElement('label');
                row.className = 'scan-cat-row';
                row.innerHTML = `
                    <input type="checkbox" class="scan-cat-cb" value="${cat.id}" ${cat.selected ? 'checked' : ''}>
                    <span class="scan-cat-name">${cat.name}</span>
                    <span class="scan-cat-desc">${cat.description}</span>
                    <span class="scan-cat-count">${cat.file_count} файл.</span>
                    <span class="scan-cat-size ${cat.size_bytes > 10*1024*1024 ? 'big' : ''}">${fmtSize(cat.size_bytes)}</span>
                `;
                cats.appendChild(row);
            });

            setTimeout(() => {
                document.getElementById('scanProgress').classList.add('hidden');
                document.getElementById('scanResults').classList.remove('hidden');
                btn.classList.remove('done');
                setScanRing(0);
            }, 600);

            document.getElementById('scanSubtitle').textContent = `найдено ${fmtSize(total)}`;
            addLog(`сканирование завершено: ${files} файлов, ${fmtSize(total)}`, 'success');
            showToast(`найдено ${fmtSize(total)}`, 'info');
        } catch (e) {
            clearInterval(ticker);
            setScanRing(0);
            btn.classList.remove('scanning', 'done');
            document.getElementById('scanProgress').classList.add('hidden');
            document.getElementById('scanSubtitle').textContent = 'ошибка сканирования';
            addLog('ошибка сканирования: ' + e.message, 'error');
            showToast(e.message, 'error');
        } finally { setBtnLoading(btn, false); }
    });

    // выбрать все
    document.getElementById('scanSelectAll').addEventListener('change', (e) => {
        document.querySelectorAll('.scan-cat-cb').forEach(cb => cb.checked = e.target.checked);
    });

    // очистить выбранное
    document.getElementById('scanCleanBtn').addEventListener('click', async () => {
        const checked = [...document.querySelectorAll('.scan-cat-cb:checked')].map(cb => cb.value);
        if (!checked.length) { showToast('ничего не выбрано', 'warning'); return; }
        const btn = document.getElementById('scanCleanBtn');
        setBtnLoading(btn, true);
        document.getElementById('scanCleanResult').classList.add('hidden');
        addLog(`очистка ${checked.length} категорий...`, 'info');
        try {
            const r = await api.cleanScan(checked);
            const msg = `удалено ${r.cleaned_files} файлов (${fmtSize(r.cleaned_bytes)})`;
            const el = document.getElementById('scanCleanResult');
            el.className = 'scan-clean-result success';
            el.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg> ${msg}`;
            el.classList.remove('hidden');
            document.getElementById('scanResults').classList.add('hidden');
            document.getElementById('scanSubtitle').textContent = 'очищено';
            addLog(msg, 'success');
            showToast(msg, 'success');
            scanData = null;
        } catch (e) {
            const el = document.getElementById('scanCleanResult');
            el.className = 'scan-clean-result error';
            el.textContent = e.message;
            el.classList.remove('hidden');
            showToast(e.message, 'error');
        } finally { setBtnLoading(btn, false); }
    });

    // ── Настройки ──
    document.getElementById('settingSounds').addEventListener('change', e => { settings.sounds = e.target.checked; saveSettings(); });
    document.getElementById('settingAnimations').addEventListener('change', e => { settings.animations = e.target.checked; saveSettings(); applySettings(); });
    document.getElementById('settingCompact').addEventListener('change', e => { settings.compact = e.target.checked; saveSettings(); applySettings(); });
    document.getElementById('settingConfirm').addEventListener('change', e => { settings.confirm = e.target.checked; saveSettings(); });
    document.getElementById('settingAutoScan').addEventListener('change', e => { settings.autoScan = e.target.checked; saveSettings(); });
    document.getElementById('settingLogLimit').addEventListener('change', e => { settings.logLimit = parseInt(e.target.value); saveSettings(); });

    // ── Твики ──
    async function loadTweaks() {
        const list = document.getElementById('tweaksList');
        list.innerHTML = '<div class="tweaks-loading">загрузка...</div>';
        try {
            const tweaks = await api.getTweaks();
            renderTweaks(tweaks);
        } catch (e) {
            list.innerHTML = `<div class="tweaks-loading">ошибка: ${e.message}</div>`;
        }
    }

    function renderTweaks(tweaks) {
        const list = document.getElementById('tweaksList');
        list.innerHTML = '';
        tweaks.forEach(tweak => {
            const card = document.createElement('div');
            card.className = `tweak-card${tweak.applied ? ' applied' : ''}${tweak.danger ? ' danger' : ''}`;
            card.dataset.id = tweak.id;
            card.innerHTML = `
                <div class="tweak-info">
                    <div class="tweak-name-row">
                        <span class="tweak-name">${tweak.name}</span>
                        ${tweak.danger ? '<span class="tweak-badge-danger">опасно</span>' : ''}
                    </div>
                    <span class="tweak-desc">${tweak.description}</span>
                </div>
                <span class="tweak-status ${tweak.applied ? 'on' : 'off'}">${tweak.applied ? 'вкл' : 'выкл'}</span>
                <button class="tweak-btn ${tweak.applied ? 'revert' : 'apply'}" data-id="${tweak.id}" data-applied="${tweak.applied}">
                    <span class="tweak-btn-text">${tweak.applied ? 'откат' : 'применить'}</span>
                </button>
            `;
            list.appendChild(card);
        });

        list.querySelectorAll('.tweak-btn').forEach(btn => {
            btn.addEventListener('click', async () => {
                const id = btn.dataset.id;
                const isApplied = btn.dataset.applied === 'true';
                btn.disabled = true;
                btn.classList.add('loading');
                addLog(`твик "${id}": ${isApplied ? 'откат' : 'применение'}...`, 'info');
                try {
                    const r = isApplied ? await api.revertTweak(id) : await api.applyTweak(id);
                    const type = r.success ? 'success' : 'error';
                    addLog(`твик "${id}": ${r.message}`, type);
                    showToast(r.message, type);
                    if (r.success) {
                        // обновляем карточку
                        const card = btn.closest('.tweak-card');
                        const statusEl = card.querySelector('.tweak-status');
                        card.classList.toggle('applied', r.applied);
                        statusEl.className = `tweak-status ${r.applied ? 'on' : 'off'}`;
                        statusEl.textContent = r.applied ? 'вкл' : 'выкл';
                        btn.className = `tweak-btn ${r.applied ? 'revert' : 'apply'}`;
                        btn.dataset.applied = r.applied;
                        btn.querySelector('.tweak-btn-text').textContent = r.applied ? 'откат' : 'применить';
                    }
                } catch (e) {
                    addLog(`твик "${id}" ошибка: ${e.message}`, 'error');
                    showToast(e.message, 'error');
                } finally {
                    btn.disabled = false;
                    btn.classList.remove('loading');
                }
            });
        });
    }

    // Загружаем твики при переходе на вкладку
    document.querySelector('.nav-btn[data-tab="tweaks"]').addEventListener('click', () => {
        loadTweaks();
    });

    document.getElementById('tweaksRefreshBtn').addEventListener('click', () => loadTweaks());

});

