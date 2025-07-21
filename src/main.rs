use tutorial8_depth::core::event_loop::run;

fn main() {
    pollster::block_on(run());
}
