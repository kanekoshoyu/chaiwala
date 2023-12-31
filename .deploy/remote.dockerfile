# Use the remote src to build binary and copy to docker image
# For continuous deployment purpose using GitHub CI

# Rust 2021 latest image
FROM rust:1.71

# Metadata
LABEL maintainer="kanekoshoyu@gmail.com" 

# The /app directory should act as the main application directory
WORKDIR /app

# Clone the repo
RUN git clone https://github.com/kanekoshoyu/chaiwala.git

# Build release
RUN cd chaiwala && cargo build --bin chaiwala_service --release

# Open application endpoints
EXPOSE 1080

# Run the binary
CMD ["./chaiwala/target/release/chaiwala_service"]
