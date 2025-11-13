default: check

build:
    cargo build

run:
    cargo run

test:
    cargo test

test-integration:
    # Run integration tests specifically
    cargo test --test playwright_test

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

# Coverage commands
coverage:
    cargo llvm-cov

coverage-report:
    cargo llvm-cov report --text

coverage-html:
    cargo llvm-cov --html --open
