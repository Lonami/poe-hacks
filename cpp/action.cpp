#include "action.h"

#include <Windows.h>
#include <cstdio>

#include "input.h"

bool Action::check() {
    if (this->skill) {
        if (!kbd::down(this->skill)) {
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

void Action::print(FILE* out) {
    if (this->flask == 0) {
        fprintf(out, "logout ");
    } else {
        fprintf(out, "use flask %c ", this->flask);
    }

    if (this->delay == 0) {
        fprintf(out, "immediatly on ");
    } else {
        fprintf(out, "every %dms on ", this->delay);
    }

    if (this->skill != 0) {
        fprintf(out, "skill ");
        fprintf(out, (this->skill >= '0' ? "%c" : "%d"), this->skill);
    } else if (this->point.x < 200) {
        fprintf(out, "life change");
    } else {
        fprintf(out, "mana change");
    }

    if (this->desc.empty()) {
        fprintf(out, " (no description)");
    } else {
        fprintf(out, ": %s", this->desc.c_str());
    }
}

std::ostream& operator<<(std::ostream& lhs, const Action& rhs) {
    lhs << rhs.enabled << '\n'
        << rhs.flask << '\n'
        << rhs.delay << '\n'
        << rhs.skill << '\n'
        << rhs.point << '\n'
        << rhs.color << '\n'
        << rhs.desc;
    return lhs;
}

std::istream& operator>>(std::istream& lhs, Action& rhs) {
    lhs >> rhs.enabled
        >> rhs.flask
        >> rhs.delay
        >> rhs.skill
        >> rhs.point
        >> rhs.color;

    std::getline(lhs, rhs.desc); // end of line for color
    std::getline(lhs, rhs.desc); // actual description line
    return lhs;
}
