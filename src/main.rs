mod core;

fn main() {
    let mem = vec![4, 1, 2, 3, 4];
    let prog = vec![
        0, 0,
        10,
        5,
        0, 0,
        0, 1,

        4,
        10,
        3,
        7,
        2,
        8,
        6, 8,

        2,
        -0x7f
    ];
    let mut core = core::Core::new();
    core.next(&prog, &mem);

    while core.cmd != core::Command::NOP {
        core.next(&prog, &mem);
    }
}
