#include "action.h"

#include <Windows.h>

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
        } else {
            this->last_use = GetTickCount();
        }
    }

    return true;
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

    std::getline(lhs, rhs.desc);
    return lhs;
}
