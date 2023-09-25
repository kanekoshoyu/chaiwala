# Use the local src to build binary and copy to docker image
# For local testing purpose

# NOTE Run this from the project root directory
# docker build . -t local-chaiwala -f ./.deploy/local.dockerfile
# docker run -p 80:1080 local-chaiwala:latest

# Use the official Rust image as the build environment
FROM rust:1.71 as build-env

# Set to Debian base for the Rust image
RUN apt-get update && apt-get install -y build-essential

# Metadata
LABEL maintainer="kanekoshoyu@gmail.com"

# The application directory
WORKDIR /usr/src

# Copy the source
COPY . /usr/src

# Build the Rust application
RUN cargo build --bin ws_broadcast --release

# Remove debug info 
RUN strip target/release/ws_broadcast

# Final stage (app)
FROM debian:bullseye-slim
WORKDIR /app

# Open application endpoints
EXPOSE 1080

# Copy the binary from the build stage
COPY --from=build-env /usr/src/target/release/chaiwala_service /app/

# Specify the command to run on container start
CMD ["/app/chaiwala_service"]