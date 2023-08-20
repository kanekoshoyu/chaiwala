# Rust 2021 latest image
FROM rust:1.71

# Metadata
LABEL maintainer="kanekoshoyu@gmail.com" 

# The /app directory should act as the main application directory
WORKDIR /app

# Either clone the repo remotely
# RUN git clone https://github.com/kanekoshoyu/chaiwala.git
# WORKDIR /app/chaiwala

# Or copy the files locally
COPY ./ ./

# Build release
RUN cargo build --bin ws_broadcast --release

# Open application endpoints
EXPOSE 3000

# Run the binary
# CMD ["cargo", "run", "--release", "--bin", "ws_broadcast"]
CMD ["./target/release/ws_broadcast"]
