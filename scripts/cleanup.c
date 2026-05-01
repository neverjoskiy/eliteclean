/*
 * cleanup.c — аналог "очистка места.bat"
 * Компиляция: gcc cleanup.c -o cleanup.exe -ladvapi32 -lshlwapi
 */

#define _WIN32_WINNT 0x0600
#include <windows.h>
#include <sddl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// ─── Вспомогательные функции ────────────────────────────────────────────────

static void run(const char *cmd) {
    STARTUPINFOA si = { .cb = sizeof(si) };
    PROCESS_INFORMATION pi;
    char buf[2048];
    snprintf(buf, sizeof(buf), "cmd.exe /C %s", cmd);
    if (CreateProcessA(NULL, buf, NULL, NULL, FALSE, CREATE_NO_WINDOW, NULL, NULL, &si, &pi)) {
        WaitForSingleObject(pi.hProcess, 10000);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }
}

static void reg_delete(const char *key, const char *flags) {
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "reg delete \"%s\" %s 2>nul", key, flags);
    run(cmd);
}

static void reg_add(const char *key) {
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "reg add \"%s\" /f 2>nul", key);
    run(cmd);
}

static void del_files(const char *path) {
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "del /f /q \"%s\" 2>nul", path);
    run(cmd);
}

// Удаление файлов по маске через FindFirstFile
static void del_mask(const char *dir, const char *mask) {
    char pattern[MAX_PATH];
    snprintf(pattern, sizeof(pattern), "%s\\%s", dir, mask);

    WIN32_FIND_DATAA fd;
    HANDLE h = FindFirstFileA(pattern, &fd);
    if (h == INVALID_HANDLE_VALUE) return;
    do {
        if (!(fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)) {
            char full[MAX_PATH];
            snprintf(full, sizeof(full), "%s\\%s", dir, fd.cFileName);
            DeleteFileA(full);
        }
    } while (FindNextFileA(h, &fd));
    FindClose(h);
}

// Проверка прав администратора
static int is_admin(void) {
    BOOL result = FALSE;
    PSID admins_group = NULL;
    SID_IDENTIFIER_AUTHORITY nt_authority = SECURITY_NT_AUTHORITY;
    if (AllocateAndInitializeSid(&nt_authority, 2,
            SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_ADMINS,
            0,0,0,0,0,0, &admins_group)) {
        CheckTokenMembership(NULL, admins_group, &result);
        FreeSid(admins_group);
    }
    return result;
}

// Получить SID текущего пользователя в виде строки
static int get_user_sid(char *out, DWORD out_size) {
    HANDLE token;
    if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &token))
        return 0;

    DWORD size = 0;
    GetTokenInformation(token, TokenUser, NULL, 0, &size);
    TOKEN_USER *tu = (TOKEN_USER *)malloc(size);
    if (!tu) { CloseHandle(token); return 0; }

    if (!GetTokenInformation(token, TokenUser, tu, size, &size)) {
        free(tu); CloseHandle(token); return 0;
    }

    LPSTR sid_str = NULL;
    ConvertSidToStringSidA(tu->User.Sid, &sid_str);
    strncpy(out, sid_str, out_size - 1);
    out[out_size - 1] = '\0';

    LocalFree(sid_str);
    free(tu);
    CloseHandle(token);
    return 1;
}

// ─── Блоки очистки ──────────────────────────────────────────────────────────

static void clean_shellbag(void) {
    printf("Очистка ShellBag - реестр\n");
    reg_delete("HKCU\\Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\Shell\\MuiCache", "/va /f");
    reg_delete("HKCU\\Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\Shell\\BagMRU", "/f");
    reg_delete("HKCU\\Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\Shell\\Bags", "/f");
    reg_delete("HKCU\\Software\\Microsoft\\Windows\\Shell\\BagMRU", "/f");
    reg_delete("HKCU\\Software\\Microsoft\\Windows\\Shell\\Bags", "/f");
    printf("Выполнено\n\n");
}

static void clean_explorer_mru(void) {
    printf("Очистка Explorer RunMRU - реестр\n");
    reg_delete("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\RunMRU", "/va /f");
    printf("Выполнено\n\n");
}

static void clean_comdlg32(void) {
    printf("Очистка OpenSave и LastVisited - реестр\n");
    reg_delete("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\ComDlg32\\FirstFolder", "/va /f");
    reg_delete("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\ComDlg32\\LastVisitedPidlMRU", "/va /f");
    reg_delete("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\ComDlg32\\LastVisitedPidlMRULegacy", "/va /f");
    reg_delete("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\ComDlg32\\OpenSavePidlMRU", "/f");
    reg_add("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\ComDlg32\\OpenSavePidlMRU");
    printf("Выполнено\n\n");
}

static void clean_userassist(void) {
    printf("Очистка UserAssist - реестр\n");
    reg_delete("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\UserAssist", "/f");
    reg_add("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\UserAssist");
    printf("Выполнено\n\n");
}

static void clean_appcompat_cache(void) {
    printf("Очистка AppCompatCache - реестр\n");
    reg_delete("HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\AppCompatCache", "/va /f");
    reg_delete("HKLM\\SYSTEM\\ControlSet001\\Control\\Session Manager\\AppCompatCache", "/va /f");
    printf("Выполнено\n\n");
}

static void clean_diagnosed_apps(void) {
    printf("Очистка DiagnosedApplications - реестр\n");
    reg_delete("HKLM\\SOFTWARE\\Microsoft\\RADAR\\HeapLeakDetection\\DiagnosedApplications", "/f");
    reg_add("HKLM\\SOFTWARE\\Microsoft\\RADAR\\HeapLeakDetection\\DiagnosedApplications");
    printf("Выполнено\n\n");
}

static void clean_search(const char *sid) {
    char key[512];
    printf("Очистка Search - реестр\n");
    snprintf(key, sizeof(key),
        "HKU\\%s\\Software\\Microsoft\\Windows\\CurrentVersion\\Search\\RecentApps", sid);
    reg_delete(key, "/f");
    reg_add(key);
    printf("Выполнено\n\n");
}

static void clean_bam(const char *sid) {
    char key[512];
    printf("Очистка Background Activity Moderator - реестр\n");
    snprintf(key, sizeof(key),
        "HKLM\\SYSTEM\\CurrentControlSet\\Services\\bam\\UserSettings\\%s", sid);
    reg_delete(key, "/va /f");
    snprintf(key, sizeof(key),
        "HKLM\\SYSTEM\\ControlSet001\\Services\\bam\\UserSettings\\%s", sid);
    reg_delete(key, "/va /f");
    printf("Выполнено\n\n");
}

static void clean_appcompat_flags(const char *sid, int mode) {
    char key[512];
    printf("Очистка AppCompatFlags - реестр\n");
    snprintf(key, sizeof(key),
        "HKU\\%s\\Software\\Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\Compatibility Assistant\\Store", sid);
    reg_delete(key, "/va /f");
    if (mode != 1) {
        snprintf(key, sizeof(key),
            "HKU\\%s\\Software\\Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\Layers", sid);
        reg_delete(key, "/va /f");
    }
    printf("Выполнено\n\n");
}

static void clean_mount_points(const char *sid) {
    char key[512];
    printf("Очистка MountedDevices - реестр\n");
    snprintf(key, sizeof(key),
        "HKU\\%s\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\MountPoints2", sid);
    reg_delete(key, "/f");
    reg_add(key);
    printf("Выполнено\n\n");
}

static void clean_recent(void) {
    char path[MAX_PATH];
    const char *appdata = getenv("APPDATA");
    printf("Очистка Recent - файловая система\n");
    snprintf(path, sizeof(path), "%s\\Microsoft\\Windows\\Recent\\*.*", appdata);
    del_files(path);
    snprintf(path, sizeof(path), "%s\\Microsoft\\Windows\\Recent\\CustomDestinations\\*.*", appdata);
    del_files(path);
    snprintf(path, sizeof(path), "%s\\Microsoft\\Windows\\Recent\\AutomaticDestinations\\*.*", appdata);
    del_files(path);
    printf("Выполнено\n\n");
}

static void clean_panther(void) {
    char path[MAX_PATH];
    const char *windir = getenv("SystemRoot");
    printf("Очистка Panther - файловая система\n");
    snprintf(path, sizeof(path), "%s\\Panther\\*.*", windir);
    del_files(path);
    printf("Выполнено\n\n");
}

static void clean_appcompat_files(void) {
    char path[MAX_PATH];
    const char *windir = getenv("SystemRoot");
    printf("Очистка AppCompat - файловая система\n");
    snprintf(path, sizeof(path), "%s\\appcompat\\Programs", windir);
    del_mask(path, "*.txt");
    del_mask(path, "*.xml");
    snprintf(path, sizeof(path), "%s\\appcompat\\Programs\\Install", windir);
    del_mask(path, "*.txt");
    del_mask(path, "*.xml");
    printf("Выполнено\n\n");
}

static void clean_prefetch(void) {
    char path[MAX_PATH];
    const char *windir = getenv("SystemRoot");
    printf("Очистка Prefetch - файловая система\n");
    snprintf(path, sizeof(path), "%s\\Prefetch", windir);
    const char *masks[] = {"*.pf","*.ini","*.7db","*.ebd","*.bin","*.db", NULL};
    for (int i = 0; masks[i]; i++)
        del_mask(path, masks[i]);
    snprintf(path, sizeof(path), "%s\\Prefetch\\ReadyBoot", windir);
    del_mask(path, "*.fx");
    printf("Выполнено\n\n");
}

static void clean_minidump(void) {
    char path[MAX_PATH];
    const char *windir = getenv("SystemRoot");
    printf("Очистка Minidump - файловая система\n");
    snprintf(path, sizeof(path), "%s\\Minidump\\*.*", windir);
    del_files(path);
    printf("Выполнено\n\n");
}

static void clean_event_logs(void) {
    printf("Очистка журналов событий Windows\n");
    run("wevtutil el > %TEMP%\\evtlogs.txt 2>nul && "
        "for /f \"tokens=*\" %G in (%TEMP%\\evtlogs.txt) do wevtutil cl \"%G\" 2>nul");
    // Более надёжный способ через PowerShell
    run("powershell -NoProfile -Command \""
        "Get-WinEvent -ListLog * -ErrorAction SilentlyContinue | "
        "ForEach-Object { wevtutil cl $_.LogName 2>$null }\"");
    printf("Выполнено\n\n");
}

// ─── main ────────────────────────────────────────────────────────────────────

int main(void) {
    // Кодировка консоли — UTF-8
    SetConsoleOutputCP(65001);
    SetConsoleCP(65001);

    // Цвет — зелёный текст
    HANDLE con = GetStdHandle(STD_OUTPUT_HANDLE);
    SetConsoleTextAttribute(con, FOREGROUND_GREEN);

    if (!is_admin()) {
        SetConsoleTextAttribute(con, FOREGROUND_RED | FOREGROUND_INTENSITY);
        printf("Необходимо запустить скрипт от имени администратора!\n\n");
        system("pause");
        return 1;
    }

    printf("\nВНИМАНИЕ!\n");
    printf("Рекомендуется закрыть все программы и открытые файлы,\n");
    printf("если они связаны с текстовым редактором, а после завершения — перезагрузиться.\n\n");

    printf("1 - Очистка основных следов в реестре\n");
    printf("2 - Очистка всех следов в реестре, файлы Prefetch и Minidump\n");
    printf("3 - Очистка всех следов, файлы Prefetch и журналы Windows\n");
    printf("Нажмите ENTER для выхода\n\n");
    printf("Выберите действие: ");

    char input[16] = {0};
    fgets(input, sizeof(input), stdin);
    int mode = atoi(input);

    if (mode < 1 || mode > 3) {
        printf("Выход.\n");
        return 0;
    }

    printf("\n");

    // Получаем SID пользователя
    char sid[256] = {0};
    if (!get_user_sid(sid, sizeof(sid))) {
        printf("Не удалось получить SID пользователя\n");
        system("pause");
        return 1;
    }

    // Режим 3 — сначала журналы событий
    if (mode == 3) clean_event_logs();

    // Общие для всех режимов
    clean_shellbag();
    clean_explorer_mru();
    clean_comdlg32();

    // Режимы 2 и 3
    if (mode != 1) clean_userassist();

    clean_appcompat_cache();
    clean_diagnosed_apps();
    clean_search(sid);
    clean_bam(sid);
    clean_appcompat_flags(sid, mode);
    clean_mount_points(sid);
    clean_recent();
    clean_panther();
    clean_appcompat_files();

    // Режимы 2 и 3
    if (mode != 1) {
        clean_prefetch();
        clean_minidump();
    }

    SetConsoleTextAttribute(con, FOREGROUND_GREEN | FOREGROUND_INTENSITY);
    printf("Готово! Рекомендуется перезагрузить компьютер.\n\n");
    SetConsoleTextAttribute(con, FOREGROUND_GREEN);

    system("pause");
    return 0;
}
