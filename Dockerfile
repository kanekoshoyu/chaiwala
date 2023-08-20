# Rust 2021 latest image
FROM rust:1.71

# Metadata
LABEL maintainer="kanekoshoyu@gmail.com" 

# The /app directory should act as the main application directory
WORKDIR /app

# Clone the repo
RUN git clone https://github.com/kanekoshoyu/chaiwala.git

# Build release
RUN cd chaiwala && cargo build --bin ws_broadcast --release

# Open application endpoints
EXPOSE 3000

# Run the binary
# CMD ["cargo", "run", "--release", "--bin", "ws_broadcast"]
CMD ["./chaiwala/target/release/ws_broadcast"]
