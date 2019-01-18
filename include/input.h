#pragma once

#include <iostream>
#include <Windows.h>

#include "color.h"
#include "point.h"

using InputCb = void (*)(int key, bool down);

// get the primary screen size
void getscreensize(int &w, int &h);

// set the mouse position
bool setmouse(int x, int y);
bool setmouse(Point p);

// get the mouse position
Point getmouse();

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
bool isdown(int key);

// is the given key pressed? (down and up)
// NOTE the key should not change! it will
// detect down and then up on any key.
// NOTE it should be used in a loop until pressed
bool pressed(int key);

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
Color getpixel(Point p);

// sticks the window up in screen
bool stickwindow();

// unsticks the window from screen
bool unstickwindow();
