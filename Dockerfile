# Multi-stage build for efficiency
FROM public.ecr.aws/lambda/provided:al2023 as builder

# Install build dependencies
RUN dnf install -y gcc gcc-c++ && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# First, build the engine library
COPY pokemon-adventure /var/pokemon-adventure
WORKDIR /var/pokemon-adventure
RUN cargo build --release --lib

# Then copy API source and build it
COPY pokemon-adventure-api /var/task
WORKDIR /var/task

# Update Cargo.toml to use the built library path
RUN sed -i 's|pokemon_adventure = { path = "../pokemon-adventure", package = "pokemon-adventure" }|pokemon_adventure = { path = "/var/pokemon-adventure", package = "pokemon-adventure" }|' Cargo.toml

# Build the API application
RUN cargo build --release

# Final stage
FROM public.ecr.aws/lambda/provided:al2023

# Copy the built binary
COPY --from=builder /var/task/target/release/pokemon-adventure-api ${LAMBDA_RUNTIME_DIR}/bootstrap

# Ensure it's executable
RUN chmod +x ${LAMBDA_RUNTIME_DIR}/bootstrap

CMD ["bootstrap"]