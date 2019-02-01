#include <Windows.h>
#include <stdio.h>
#include <fstream>
#include <vector>

#include <Iphlpapi.h> // https://docs.microsoft.com/en-us/windows/desktop/api/iphlpapi/nf-iphlpapi-getextendedtcptable

#include "utils.h"
#include "input.h"
#include "color.h"
#include "point.h"
#include "action.h"

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

// some variables to determine screen size
int width, height;

// this loads screen size and sets width/height, load some functions
void setup() {
    screen::size(width, height);

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

int targetkey = 0;
void oninput(int key, bool down) {
    if (down) {
        if (key == targetkey) {
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

Point decop1, decop2;
Color decoc1, decoc2;

bool checkdeco() {
    return screen::get(decop1) == decoc1
        && screen::get(decop2) == decoc2;
}


int main() {
    setup();
    input::setcb(oninput);
    SetConsoleCtrlHandler(oninterrupt, true);

    Action action;
    std::vector<Action> actions {};

    std::ifstream keyfile("poe.key");
    if (keyfile) {
        keyfile >> targetkey
                >> decop1 >> decoc1
                >> decop2 >> decoc2;

        for (int i = 0; i < 3; ++i) {
            keyfile >> action;
            actions.push_back(action);
        }

        keyfile.close();
    } else {
        screen::stick();
        printf("-- no key file detected, running first time setup --\n");

        // autoheal
        printf("press the key with the healing flask\n");
        while ((action.flask = input::wait()) < 0x07); // repeat on mouse input

        printf("right click on mid-life to auto-heal\n");
        input::wait(VK_RBUTTON);
        action.skill = 0; // point
        action.delay = 1000;
        action.point = mouse::get();
        action.color = screen::get(action.point);
        actions.push_back(action);

        // automana
        printf("press the key with the mana flask\n");
        while ((action.flask = input::wait()) < 0x07); // repeat on mouse input

        printf("right click on low mana to auto-mana\n");
        input::wait(VK_RBUTTON);
        action.skill = 0; // point
        action.delay = 2000;
        action.point = mouse::get();
        action.color = screen::get(action.point);
        actions.push_back(action);

        // auto/quick dc
        printf("press the key to use for logout\n");
        while ((targetkey = input::wait()) < 0x07); // repeat on mouse input

        printf("right click on low life to auto-dc\n");
        input::wait(VK_RBUTTON);

        action.skill = 0; // point
        action.flask = 0; // logout
        action.delay = 0;
        action.point = mouse::get();
        action.color = screen::get(action.point);
        actions.push_back(action);

        printf("right click on some left decoration\n");
        input::wait(VK_RBUTTON);
        decop1 = mouse::get();
        decoc1 = screen::get(decop1);

        printf("right click on some right decoration\n");
        input::wait(VK_RBUTTON);
        decop2 = mouse::get();
        decoc2 = screen::get(decop2);

        std::ofstream savekey("poe.key");
        savekey << targetkey << '\n'
                << lifeflask << '\n'
                << manaflask << '\n'
                << decop1 << ' ' << decoc1 << '\n'
                << decop2 << ' ' << decoc2 << '\n'
                << lifep << ' ' << lifec << '\n'
                << midlifep << ' ' << midlifec << '\n'
                << manap << ' ' << manac << '\n'
                ;

        savekey.close();
        screen::unstick();
    }

    printf("using key %d, checking life at (%d, %d)\n"
           "delete poe.key and re-run to change this\n",
           targetkey, lifep.x, lifep.y);

    while (running) {
        Sleep(10);
        input::step();
        if (!checkdeco()) {
            continue; // don't check anything if decoration is not there
        }

        // check deco twice to avoid false positives
        for (auto&& action: actions) {
            if (action.check() && checkdeco()) {
                printf("running action %s\n", action.desc.c_str());
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
