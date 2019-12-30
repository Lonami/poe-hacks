use rshacks;

fn main() {
    dbg!(rshacks::proc::kill_network(rshacks::proc::find_proc("PathOfExile").unwrap()));
}
