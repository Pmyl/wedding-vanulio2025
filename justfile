build:
    vercel build

dev: build
    vercel dev

templates:
    cargo run --manifest-path templating/Cargo.toml
