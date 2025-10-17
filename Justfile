default: check

build:
    cargo build

run:
    cargo run

test:
    cargo test

clean:
    cargo clean

fmt:
    cargo fmt

clippy:
    cargo clippy --all-targets --all-features -- -W clippy::all -W clippy::pedantic

check:
    just fmt
    just clippy
    just test
