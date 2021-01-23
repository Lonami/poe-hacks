#include <Windows.h>
#include <stdio.h>

#include "utils.h"
#include "input.h"

int width, height, midx, midy;

// this loads screen size and sets width/height/midx/midy
void setup() {
    screen::size(width, height);
    midx = width / 1.9; // slightly to the right to workaround memu quirks
    midy = height / 2;
}

// called on all input, mouse too
void oninput(int key, bool down) {
    switch (key) {
    case VK_RBUTTON:
        kbd::press('E', down);
        if (down) {
            mouse::set(midx, midy);
        }
        break;
    case VK_MBUTTON:
        kbd::press('Q', down);
        if (down) {
            mouse::set(midx, midy);
        }
        break;
    }
}

volatile bool running = true;

// to know when to finish (not quite necessary but graceul shutdown)
BOOL WINAPI oninterrupt(_In_ DWORD type) {
    if (running) {
        running = false;
        return true;
    }
    return false;
}

// this runs the program forever listening for input
int main() {
    setup();
    win::setcb(oninput);
    SetConsoleCtrlHandler(oninterrupt, true);

    printf("size: %d x %d\n", width, height);
    printf("program now running\n");
    while (running) {
        Sleep(10);
        win::step();
    }    

    printf("graceful shutdown\n");
    return 0;
}
