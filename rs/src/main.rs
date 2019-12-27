use rshacks;

fn main() {
    let (w, h) = rshacks::input::screen::size().unwrap();
    let (x, y) = rshacks::input::mouse::get().unwrap();
    println!(
        "Your screen is {}x{}, and you have the mouse at ({}, {})",
        w, h, x, y
    );

    rshacks::input::mouse::click(rshacks::input::mouse::Button::Left);
    rshacks::input::keyboard::type_string("// Hello, World!");
}
