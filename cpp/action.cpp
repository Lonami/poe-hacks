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
}

bool Action::check() {
    if (screen::get(this->point) == this->color) {
        return false;
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
    float percent;
    lhs >> kind
        >> key
        >> percent
        >> rhs.delay;

    // key should be uppercase
    if ('a' <= key && key <= 'z') {
        key += 'A' - 'a';
    }
    rhs.flask = (unsigned int)key;

    // figure out location with percent
    if (percent < 0) {
        fprintf(stderr, "percent too low (%f), using 0%%\n", percent);
        percent = 0;
    } else if (percent > 100) {
        fprintf(stderr, "percent too high (%f), using 100%%\n", percent);
        percent = 1;
    } else {
        percent /= 100;
    }

    // (M)ana or (L)ife
    if (kind == 'm' || kind == 'M') {
        rhs.set_point(percent, true);
        printf("loaded mana on key %c at %.1f%%\n", key, percent * 100);
    } else {
        if (kind != 'l' && kind != 'L') {
            fprintf(stderr, "unknown point type '%c', assuming L; must be L(ife) or M(ana)\n", kind);
        }
        rhs.set_point(percent, false);
        printf("loaded life on key %c at %.1f%%\n", key, percent * 100);
    }

    rhs.last_use = 0;
    return lhs;
}
