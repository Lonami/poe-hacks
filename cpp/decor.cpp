#include "decor.h"

#include "input.h"

void Decor::grab(int wait_key) {
    input::wait(wait_key);
    this->p1 = mouse::get();
    this->c1 = screen::get(this->p1);

    input::wait(wait_key);
    this->p2 = mouse::get();
    this->c2 = screen::get(this->p2);
}

bool Decor::check() {
    return screen::get(this->p1) == this->c1
        && screen::get(this->p2) == this->c2;
}

std::ostream& operator<<(std::ostream& lhs, const Decor& rhs) {
    lhs << rhs.p1 << ' ' << rhs.c1 << ' '
        << rhs.p2 << ' ' << rhs.c2;
    return lhs;
}

std::istream& operator>>(std::istream& lhs, Decor& rhs) {
    lhs >> rhs.p1 >> rhs.c1
        >> rhs.p2 >> rhs.c2;
    return lhs;
}
