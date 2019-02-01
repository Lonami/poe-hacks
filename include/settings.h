#pragma once

#include <vector>

#include "decor.h"
#include "action.h"

// we don't need n instances of settings, single global namespace
namespace settings {
    // some variables to determine screen size
    extern int width, height;

    // key that when hit will logout
    extern int logout_key;

    // decoration to check to make sure we're in game
    extern Decor decor;

    // actions loaded by the settings and used
    extern std::vector<Action> actions;

    // load global settings, true if things are defined well
    bool load();

    // save global settings
    void save();

    // enter the interactive menu
    void menu();
}
