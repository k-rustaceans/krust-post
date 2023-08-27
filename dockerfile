FROM rust:1.72

COPY ./ ./

RUN cargo build --release

# Run the binary
CMD ["cargo", "test"]
