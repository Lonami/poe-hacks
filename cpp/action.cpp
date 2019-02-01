#include "action.h"

Action::Action():
    enabled(false) {
}

bool Action::check() {
    // TODO GetTickCount, last delay
    return true;
}

std::ostream& operator<<(std::ostream& lhs, const Action& rhs) {
    lhs << rhs.enabled << '\n'
        << rhs.flask << '\n'
        << rhs.delay << '\n'
        << rhs.skill << '\n'
        << rhs.point << '\n'
        << rhs.color << '\n'
        << rhs.desc << '\n';
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
