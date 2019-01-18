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
    getscreensize(width, height);

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
    setinputcb(oninput);
    SetConsoleCtrlHandler(oninterrupt, true);

    int key;
    int lifeflask;
    int lastheal = 0;
    Point decop, lifep, midlifep;
    Color decoc, lifec, midlifec;
    std::ifstream keyfile("poe.key");
    if (keyfile) {
        keyfile >> targetkey
                >> lifeflask
                >> decop >> decoc
                >> lifep >> lifec
                >> midlifep >> midlifec
                ;

        keyfile.close();
    } else {
        printf("-- no key file detected, running first time setup --\n"
               "press the key to use for logout\n");
        while ((targetkey = waitinput()) < 0x07); // repeat on mouse input

        printf("press the key with the healing flask\n");
        while ((lifeflask = waitinput()) < 0x07); // repeat on mouse input

        printf("right click on mid-life to auto-heal\n");
        waitpress(VK_RBUTTON);
        midlifep = getmouse();
        midlifec = getpixel(midlifep);

        printf("right click on low life to auto-dc\n");
        waitpress(VK_RBUTTON);
        lifep = getmouse();
        lifec = getpixel(lifep);

        // lock mouse in the y axis: if taskbar covers life -> cover deco
        printf("right click on the life decoration\n");
        while (!pressed(VK_RBUTTON)) {
            Sleep(10);
            setmouse(getmouse().x, lifep.y);
        }
        decop = getmouse();
        decoc = getpixel(decop);

        std::ofstream savekey("poe.key");
        savekey << targetkey << '\n'
                << lifeflask << '\n'
                << decop << ' ' << decoc << '\n'
                << lifep << ' ' << lifec << '\n'
                << midlifep << ' ' << midlifec << '\n'
                ;

        savekey.close();
    }

    printf("using key %d, checking life at (%d, %d)\n"
           "delete poe.key and re-run to change this\n",
           targetkey, lifep.x, lifep.y);

    while (running) {
        Sleep(10);
        stepinput();
        if (getpixel(lifep) != lifec && getpixel(decop) == decoc) {
            printf("low life!\n");
            if (logout()) {
                Sleep(100); // don't spam logout if it worked
            }
        }
        if (getpixel(midlifep) != midlifec
                && getpixel(decop) == decoc
                && GetTickCount() - lastheal > 1000) {
            printf("mid life, healing!\n");
            press(lifeflask, 0);
            lastheal = GetTickCount();
        }
    }

    printf("graceful shutdown\n");
    cleanup();
    return 0;
}
