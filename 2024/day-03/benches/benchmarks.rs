use day_03::*;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn part1() {
    part1::process(divan::black_box(include_str!("../input1.txt",))).unwrap();
}

#[divan::bench(args = ["while", "many_till"])]
fn part2(parser: &str) {
    part2::process2(
        divan::black_box(include_str!("../input2.txt",)),
        parser.into(),
    )
    .unwrap();
}
