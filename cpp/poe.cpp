#include <Windows.h>
#include <stdio.h>
#include <fstream>
#include <vector>
#include <conio.h>

#include <Iphlpapi.h> // https://docs.microsoft.com/en-us/windows/desktop/api/iphlpapi/nf-iphlpapi-getextendedtcptable

#include "utils.h"
#include "input.h"
#include "color.h"
#include "point.h"
#include "action.h"
#include "settings.h"

// used declarations for iphlpapi.dll
HMODULE iphlpapi;

// https://docs.microsoft.com/en-us/windows/desktop/api/iphlpapi/nf-iphlpapi-getextendedtcptable
using GetExtendedTcpTablePtr = DWORD (*)(
    PVOID pTcpTable,
    PDWORD pdwSize,
    BOOL bOrder,
    ULONG ulAf,
    TCP_TABLE_CLASS TableClass,
    ULONG Reserved
);
GetExtendedTcpTablePtr gettable;

// https://docs.microsoft.com/en-us/windows/desktop/api/iphlpapi/nf-iphlpapi-settcpentry
using SetTcpEntryPtr = DWORD (*)(
    PMIB_TCPROW pTcpRow
);
SetTcpEntryPtr setentry;

// this loads screen size and sets width/height, load some functions
void setup() {
    iphlpapi = LoadLibrary("iphlpapi.dll");
    gettable = (GetExtendedTcpTablePtr)
        GetProcAddress(iphlpapi, "GetExtendedTcpTable");

    setentry = (SetTcpEntryPtr)
        GetProcAddress(iphlpapi, "SetTcpEntry");
}

// this cleans memory
void cleanup() {
    FreeLibrary(iphlpapi);
}

// if things go wrong, unlikely, do this
void portlogout(const char *name) {
    char command[] = "cports.exe /close * * * * PathOfExile_x64Steam.exe";
    //                01234567890123456789012345678901234567890123456789
    //                00000000001111111111222222222233333333334444444444
    char *dest = &command[26];
    while (*name) {
        *dest++ = *name++;
    }
    *dest = 0;
    if (system(command)) {
        fprintf(stderr, "portlogout err: cports.exe not found!\n");
    }
}

bool logout() {
    long start = GetTickCount();
    char name[MAX_PATH];

    DWORD poe = findproc("PathOfExile", name, sizeof(name));
    if (!poe) {
        fprintf(stderr, "logout err: could not find poe! is it running?\n");
        return false;
    }

    DWORD res;
    DWORD size = 0;
    gettable(NULL, &size, false, AF_INET, TCP_TABLE_OWNER_PID_ALL, 0);
    MIB_TCPTABLE_OWNER_PID *table = (MIB_TCPTABLE_OWNER_PID*)malloc(size);
    if ((res = gettable(table, &size, false,
                        AF_INET, TCP_TABLE_OWNER_PID_ALL, 0)) != NO_ERROR) {
        fprintf(stderr, "logout err: could not get tcptable! code %lu\n", res);
        free(table);
        return false;
    }

    int ok = 0;
    for (int i = 0; i < table->dwNumEntries; ++i) {
        MIB_TCPROW_OWNER_PID old = table->table[i];
        if (old.dwOwningPid == poe) {
            MIB_TCPROW row;
            row.dwState = 12; // magic number to terminate
            row.dwLocalAddr = old.dwLocalAddr;
            row.dwLocalPort = old.dwLocalPort;
            row.dwRemoteAddr = old.dwRemoteAddr;
            row.dwRemotePort = old.dwRemotePort;
            if (setentry(&row) != NO_ERROR) {
                portlogout(name);
            }
            ++ok;
        }
    }

    free(table);
    if (ok == 0) {
        fprintf(stderr, "logout err: didn't close any connection! "
                        "are you in the login screen?\n");
        return false;
    }

    long end = GetTickCount();
    printf("logout success: took %ldms for %d conns for pid %lu\n",
           end - start, ok, poe);

    return true;
}

void oninput(int key, bool down) {
    if (down) {
        if (key == settings::logout_key) {
            logout();
        }
    }
}

volatile bool running = true;
BOOL WINAPI oninterrupt(_In_ DWORD type) {
    if (running) {
        running = false;
        return true;
    }
    return false;
}

int main() {
    unsigned char ch1, ch2;
    setup();
    input::setcb(oninput);
    SetConsoleCtrlHandler(oninterrupt, true);

    if (!settings::load()) {
        printf("NOTE: first run, right click on two decorations\n");
        settings::decor.grab(VK_RBUTTON);
        settings::menu();
    }

    int count = 0;
    for (auto&& action: settings::actions) {
        if (action.enabled) {
            ++count;
        }
    }

    printf("press enter on this window to enter the settings menu\n");
    printf("loaded %d actions, %d enabled:\n",
            settings::actions.size(), count);
    for (auto&& action: settings::actions) {
        if (action.enabled) {
            printf("* ");
            action.print(stdout);
            putchar('\n');
        }
    }

    while (running) {
        Sleep(10);

        // sadly this needs to run first
        if (_kbhit()) {
            ch1 = _getch();
            ch2 = _getch();
            // return (enter) or arrow, right
            if (ch1 == '\r' || (ch1 == 0xe0 && ch2 == 77)) {
                printf("entering config menu, checks are now NOT running\n");
                settings::menu();
                cmd::cls();
                printf("exiting config menu, checks are now running\n");
            }
        }

        input::step();
        if (!settings::decor.check()) {
            continue; // don't check anything if decoration is not there
        }

        for (auto&& action: settings::actions) {
            // check deco twice to avoid false positives
            if (action.enabled && action.check() && settings::decor.check()) {
                printf("! running ");
                action.print(stdout);
                putchar('\n');
                if (action.flask) {
                    kbd::tap(action.flask);
                } else if (logout()) {
                    Sleep(100); // don't spam logout if it worked
                    break;
                }
            }
        }
    }

    printf("graceful shutdown\n");
    cleanup();
    return 0;
}
