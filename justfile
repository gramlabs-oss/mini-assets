format:
    cargo fmt

run +args='':
    cargo run -- {{args}}

install:
    cargo install --path .

lint:
    cargo clippy

clean:
    cargo clean
