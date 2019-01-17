#pragma once

#include <iostream>
#include <Windows.h>

using InputCb = void (*)(int key, bool down);

struct Color {
    unsigned char r, g, b, z;
};

bool operator==(const Color& lhs, const Color& rhs);
bool operator!=(const Color& lhs, const Color& rhs);
std::ostream& operator<<(std::ostream& lhs, const Color& rhs);
std::istream& operator>>(std::istream& lhs, Color& rhs);

// get the primary screen size
void getscreensize(int &w, int &h);

// set the mouse position
bool setmouse(int x, int y);

// get the mouse position, true on success
bool getmouse(int &x, int &y);

// clicks with button, 1 left, 2 right; default 1
void click();
void click(int button);

// scrolls the mouse n times, positive forward (up), negative backward (down)
void scroll(int amount);

// types the given text
void type(const char *text);

// presses the given key, and how (0 tap, 1 down, 2 up)
void press(WORD vk, char action);

// is the given key down?
bool isdown(char key);

// steps the input state to update it
void stepinput();

// wait until the given key is pressed (down and released)
void waitpress(int key);

// wait until any key state changes and then is released, and return which
int waitinput();

// sets the input callback, for possible keys see:
// https://docs.microsoft.com/en-gb/windows/desktop/inputdev/virtual-key-codes
void setinputcb(InputCb cb);

// get a pixel onscreen
Color getpixel(int x, int y);
