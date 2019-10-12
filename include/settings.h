#pragma once

#include <vector>

#include "decor.h"
#include "action.h"

#define LIFE_X 0.06
#define MANA_X 0.94

#define LIFE_Y1 0.813
#define LIFE_Y0 0.974
#define MANA_Y1 0.809
#define MANA_Y0 0.981

#define DECO_X0 0.004
#define DECO_Y0 0.880

#define DECO_X1 0.036
#define DECO_Y1 0.960

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
}
