format:
    cargo fmt

run +args='':
    cargo run -- {{args}}

install:
    cargo install --path .

linting:
    cargo clippy

clean:
    cargo clean
