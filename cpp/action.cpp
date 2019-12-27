#include "action.h"

#include <Windows.h>
#include <cstdio>
#include <stdio.h>

#include "input.h"
#include "settings.h"

void Action::set_point(float percent, bool mana) {
    if (mana) {
        this->point.x = (int)(settings::width * MANA_X);
        this->point.y = (int)(settings::height * (MANA_Y0 + (MANA_Y1 - MANA_Y0) * percent));
    } else {
        this->point.x = (int)(settings::width * LIFE_X);
        this->point.y = (int)(settings::height * (LIFE_Y0 + (LIFE_Y1 - LIFE_Y0) * percent));
    }
    this->color = screen::get(this->point);
    this->skill_key = 0;
}

bool Action::check() {
    if (this->skill_key) {
        if (!kbd::down(this->skill_key)) {
            return false;
        }
    } else {
        if (screen::get(this->point) == this->color) {
            return false;
        }
    }

    if (this->delay) {
        if ((GetTickCount() - this->last_use) < this->delay) {
            return false;
        }
    }

    this->last_use = GetTickCount();
    return true;
}

std::istream& operator>>(std::istream& lhs, Action& rhs) {
    char kind, key;
    int value;
    float percent;
    lhs >> kind
        >> key
        >> value
        >> rhs.delay;

    // key should be uppercase
    if ('a' <= key && key <= 'z') {
        key += 'A' - 'a';
    }
    rhs.flask = (unsigned int)key;

    // figure out location with percent
    if (value < 0) {
        percent = 0;
    } else if (value > 100) {
        percent = 1;
    } else {
        percent = ((float)value) / 100.0f;
    }

    // (M)ana or (L)ife
    switch (kind) {
        case 'M':
        case 'm':
            rhs.set_point(percent, true);
            printf("loaded mana on key %c at %.1f%%\n", key, percent * 100);
            break;
        case 'l':
        case 'L':
            rhs.set_point(percent, false);
            printf("loaded life on key %c at %.1f%%\n", key, percent * 100);
            break;
        case 'k':
        case 'K':
            rhs.skill_key = value;
            printf("loaded flask on key %c when using %d\n", key, value);
            break;
        default:
            fprintf(stderr, "unknown point type '%c' won't do anything\n", kind);
            break;
    }

    rhs.last_use = 0;
    return lhs;
}
