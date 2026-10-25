#[path = "/Users/builduser/Git/esp32-c6/projects/wireless-explorer/firmware/src/buffer.rs"]
mod buffer;
fn main() {
    let queue = buffer::Buffer::<u32, 1>::new(0);
    queue.push_with(|slot| *slot = 1);
    let escaped = queue.pop_with(|slot| slot);
    println!("{escaped:?}");
}
