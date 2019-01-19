#include <Windows.h>
#include <stdio.h>
#include <fstream>

#include <Iphlpapi.h> // https://docs.microsoft.com/en-us/windows/desktop/api/iphlpapi/nf-iphlpapi-getextendedtcptable

#include "utils.h"
#include "input.h"
#include "color.h"

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

int main() {
    setup();
    input::setcb(oninput);
    SetConsoleCtrlHandler(oninterrupt, true);

    int key;
    int lifeflask;
    int manaflask;
    int lastheal = 0;
    int lastmana = 0;
    Point decop1, decop2, lifep, midlifep, manap;
    Color decoc1, decoc2, lifec, midlifec, manac, tmpc;
    std::ifstream keyfile("poe.key");
    if (keyfile) {
        keyfile >> targetkey
                >> lifeflask
                >> manaflask
                >> decop1 >> decoc1
                >> decop2 >> decoc2
                >> lifep >> lifec
                >> midlifep >> midlifec
                >> manap >> manac
                ;

        keyfile.close();
    } else {
        screen::stick();
        printf("-- no key file detected, running first time setup --\n");

        // autoheal
        printf("press the key with the healing flask\n");
        while ((lifeflask = input::wait()) < 0x07); // repeat on mouse input

        printf("right click on mid-life to auto-heal\n");
        input::wait(VK_RBUTTON);
        midlifep = mouse::get();
        midlifec = screen::get(midlifep);

        // automana
        printf("press the key with the mana flask\n");
        while ((manaflask = input::wait()) < 0x07); // repeat on mouse input

        printf("right click on low mana to auto-mana\n");
        input::wait(VK_RBUTTON);
        manap = mouse::get();
        manac = screen::get(manap);

        // auto/quick dc
        printf("press the key to use for logout\n");
        while ((targetkey = input::wait()) < 0x07); // repeat on mouse input

        printf("right click on low life to auto-dc\n");
        input::wait(VK_RBUTTON);
        lifep = mouse::get();
        lifec = screen::get(lifep);

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

        // double-check life/mana pixels bc sometimes they're black
        if ((tmpc = screen::get(lifep)) != lifec
                && screen::get(decop1) == decoc1
                && screen::get(decop2) == decoc2) {
            if (screen::get(lifep) != lifec) {
            printf("low life! (%d, %d, %d) != (%d, %d, %d)\n",
                   tmpc.r, tmpc.g, tmpc.b, lifec.r, lifec.g, lifec.b);
            if (logout()) {
                Sleep(100); // don't spam logout if it worked
            }
            } else {
                printf("first pixel was bad, but second not: (%d, %d, %d) != (%d, %d, %d)\n",
                   tmpc.r, tmpc.g, tmpc.b, lifec.r, lifec.g, lifec.b);
            }
        }
        if (screen::get(midlifep) != midlifec
                && screen::get(decop1) == decoc1
                && screen::get(decop2) == decoc2
                && GetTickCount() - lastheal > 100
                && (tmpc = screen::get(midlifep)) != midlifec) {
            printf("mid life, healing! (%d, %d, %d) != (%d, %d, %d)\n",
                   tmpc.r, tmpc.g, tmpc.b, midlifec.r, midlifec.g, midlifec.b);
            kbd::tap(lifeflask);
            lastheal = GetTickCount();
        }
        if (screen::get(manap) != manac
                && screen::get(decop1) == decoc1
                && screen::get(decop2) == decoc2
                && GetTickCount() - lastmana > 2000
                && (tmpc = screen::get(manap)) != manac) {
            printf("low mana, using flask! (%d, %d, %d) != (%d, %d, %d)\n",
                   tmpc.r, tmpc.g, tmpc.b, manac.r, manac.g, manac.b);
            kbd::tap(manaflask);
            lastmana = GetTickCount();
        }
    }

    printf("graceful shutdown\n");
    cleanup();
    return 0;
}
