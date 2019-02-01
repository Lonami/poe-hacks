#include "input.h"

#include <cstring>
#include <Wingdi.h>


namespace screen {
// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getdesktopwindow
void size(int &w, int &h) {
    RECT desktop;
    static HWND hDesktop = GetDesktopWindow();
    GetWindowRect(hDesktop, &desktop);
    w = desktop.right;
    h = desktop.bottom;
}

// https://docs.microsoft.com/en-us/windows/desktop/api/wingdi/nf-wingdi-getpixel
Color get(Point p) {
    static HDC dc = GetDC(NULL);
    return { GetPixel(dc, p.x, p.y) };
}

// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-setwindowpos
bool stick() {
    HWND handle = GetConsoleWindow();
    if (handle) {
        return SetWindowPos(handle, HWND_TOPMOST, 0, 0, 600, 100,
                            SWP_DRAWFRAME | SWP_SHOWWINDOW);
    }
    return false;
}

bool unstick() {
    HWND handle = GetConsoleWindow();
    if (handle) {
        return SetWindowPos(handle, HWND_NOTOPMOST, 0, 0, 0, 0,
                            SWP_DRAWFRAME | SWP_NOMOVE
                             | SWP_NOSIZE | SWP_SHOWWINDOW);
    }
    return false;
}
} // screen

namespace cmd {
static const HANDLE console = GetStdHandle(STD_OUTPUT_HANDLE);

void cls() {
    COORD pos = { 0, 0 };
    Point dim = size();
    DWORD written;

    FillConsoleOutputCharacterA(console, ' ', dim.x * dim.y, pos, &written);
    SetConsoleCursorPosition(console, pos);
}

Point size() {
    CONSOLE_SCREEN_BUFFER_INFO screen;
    GetConsoleScreenBufferInfo(console, &screen);
    Point dim = { screen.dwSize.X, screen.dwSize.Y };
    return dim;
}

void set(int x, int y) {
   COORD pos = { (SHORT)x, (SHORT)y };
   SetConsoleCursorPosition(console, pos);
}

void set(Point p) {
   COORD pos = { (SHORT)p.x, (SHORT)p.y };
   SetConsoleCursorPosition(console, pos);
}
} // cmd

namespace mouse {
// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getcursorpos
Point get() {
    POINT point;
    if (GetCursorPos(&point)) {
        return Point { point.x, point.y };
    } else {
        return Point { -1, -1 };
    }
}

// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-setcursorpos
bool set(int x, int y) {
    return SetCursorPos(x, y);
}

bool set(Point p) {
    return SetCursorPos(p.x, p.y);
}

// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-sendinput
void click() {
    click(1);
}

void click(int button) {
    MOUSEINPUT mi = {
        0, 0, // dx, dy
        0,    // data
        0,    // action
        0     // time
    };
    INPUT input = { INPUT_MOUSE, mi };
    if (button == 2) {
        input.mi.dwFlags = MOUSEEVENTF_RIGHTDOWN;
        SendInput(1, &input, sizeof(input));
        input.mi.dwFlags = MOUSEEVENTF_RIGHTUP;
        SendInput(1, &input, sizeof(input));
    } else {
        input.mi.dwFlags = MOUSEEVENTF_LEFTDOWN;
        SendInput(1, &input, sizeof(input));
        input.mi.dwFlags = MOUSEEVENTF_LEFTUP;
        SendInput(1, &input, sizeof(input));
    }
}

void scroll(DWORD amount) {
    MOUSEINPUT mi = {
        0, 0,              // dx, dy
        120 * amount,      // amount
        MOUSEEVENTF_WHEEL, // action
        0                  // time
    };
    INPUT input = { INPUT_MOUSE, mi };
    SendInput(1, &input, sizeof(input));
}
} // mouse

namespace kbd {
// helper method to send input depending on the modifier
// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-vkkeyscana#return-value
// https://docs.microsoft.com/en-gb/windows/desktop/inputdev/virtual-key-codes
void _typemod(INPUT &input, const unsigned char mod) {
    if (mod & 1) {
        input.ki.wVk = VK_SHIFT;
        SendInput(1, &input, sizeof(input));
    }
    if (mod & 2) {
        input.ki.wVk = VK_CONTROL;
        SendInput(1, &input, sizeof(input));
    }
    if (mod & 4) {
        input.ki.wVk = VK_MENU;
        SendInput(1, &input, sizeof(input));
    }
}


void type(const char *text) {
    INPUT input = { INPUT_KEYBOARD };
    input.ki = {
        0, 0, 0, 0, 0 // vk, scan code, flags, time, extra
    };
    while (*text) {
        unsigned short res = VkKeyScanA(*text++);
        unsigned char mod = res >> 8;
        unsigned char key = res & 0xff;

        // down
        input.ki.dwFlags = 0;
        _typemod(input, mod);
        input.ki.wVk = key;
        SendInput(1, &input, sizeof(input));

        // up
        input.ki.dwFlags = KEYEVENTF_KEYUP;
        SendInput(1, &input, sizeof(input));
        _typemod(input, mod);
    }
}

void tap(WORD vk) {
    press(vk, true);
    press(vk, false);
}

void hold(WORD vk) {
    press(vk, true);
}

void release(WORD vk) {
    press(vk, false);
}

void press(WORD vk, bool down) {
    INPUT input = { INPUT_KEYBOARD };
    input.ki = {
        vk,                           // key
        0,                            // unicode scan
        down ? 0ul : KEYEVENTF_KEYUP, // flags
        0,                            //time
        0                             // extra
    };
    SendInput(1, &input, sizeof(input));
}

bool down(WORD vk) {
    return (GetKeyState(vk) & 0x80) != 0;
}

bool pressed(WORD vk) {
    static bool last_down = false;
    if (last_down) {
        if (!down(vk)) {
            last_down = false; // reset
            return true;
        }
    } else if (down(vk)) {
        last_down = true;
    }
    return false;
}
} // kbd

namespace input {
InputCb inputcb = nullptr;

void setcb(InputCb cb) {
    inputcb = cb;
}

void step() {
    static unsigned char kbd1[256]; // 1 = down, 0 = up
    static unsigned char kbd2[256];
    static unsigned char *before = kbd1;
    unsigned char *after = before == kbd1 ? kbd2 : kbd1;

    GetKeyState(0); // for some reason GetKeyboardState only works w/ this
    GetKeyboardState(after);
    
    if (inputcb != nullptr) {
        for (int i = 0; i < 256; ++i) {
            after[i] = after[i] & 0x80 ? 1 : 0;
            if (before[i] != after[i]) {
                inputcb(i, after[i]);
            }
        }
    }

    before = after;
}

void wait(WORD vk) {
    while ((GetKeyState(vk) & 0x80) == 0) Sleep(10);
    while ((GetKeyState(vk) & 0x80) != 0) Sleep(10);
}

WORD wait() {
    WORD vk;
    bool checking = true;
    unsigned char kbd1[256];
    unsigned char kbd2[256];

    GetKeyState(0);
    GetKeyboardState(kbd1);
    while (checking) {
        Sleep(10);
        GetKeyState(0);
        GetKeyboardState(kbd2);
        for (vk = 0; vk < 256; ++vk) {
            if (kbd1[vk] != kbd2[vk]) {
                checking = false;
                break;
            }
        }
    }
    while ((GetKeyState(vk) & 0x80) != 0) Sleep(10);
    return vk;
}
} // input
