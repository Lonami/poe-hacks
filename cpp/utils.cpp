#include "utils.h"

#include <Psapi.h> // https://docs.microsoft.com/es-mx/windows/desktop/api/psapi/nf-psapi-enumprocesses

bool startswith(const char *what, const char *with) {
    while (*what && *with && *what == *with) {
        ++what;
        ++with;
    }
    return !*with;
}

bool getprocname(DWORD processID, char *name, DWORD namelen) {
    HANDLE process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                                 FALSE, processID);
    if (process) {
        HMODULE mod;
        DWORD cbNeeded;
        if (EnumProcessModules(process, &mod, sizeof(mod), &cbNeeded)) {
            GetModuleBaseName(process, mod, name, namelen);
            CloseHandle(process);
            return true;
        }
    }

    CloseHandle(process);
    return false;
}

DWORD findproc(const char *startwith, char *name, DWORD namelen) {
    DWORD got;
    DWORD pids[1024];
    if (EnumProcesses(pids, sizeof(pids), &got)) {
        for (int i = (int)(got / sizeof(DWORD)); i-- != 0;) {
            if (getprocname(pids[i], name, namelen)) {
                if (startswith(name, startwith)) {
                    return pids[i];
                }
            }
        }
    }
    return 0;
}
