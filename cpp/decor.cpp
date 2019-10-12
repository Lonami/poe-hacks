#include "decor.h"

#include "input.h"

bool Decor::check() {
    return screen::get(this->p1) == this->c1
        && screen::get(this->p2) == this->c2;
}
