# Multi-stage Dockerfile for Production
# Builds both the Rust CLI and React dashboard in an optimized container

# Stage 1: Build Rust dependencies (cache layer)
FROM rust:1.70-slim as rust-deps
WORKDIR /build

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY packages/cli/Cargo.toml packages/cli/Cargo.lock ./packages/cli/
COPY packages/projects/Cargo.toml ./packages/projects/
COPY packages/tui/Cargo.toml ./packages/tui/
COPY packages/preview/Cargo.toml ./packages/preview/
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to cache dependencies
RUN mkdir -p packages/cli/src packages/projects/src packages/tui/src packages/preview/src && \
    echo "fn main() {}" > packages/cli/src/main.rs && \
    echo "fn main() {}" > packages/projects/src/lib.rs && \
    echo "fn main() {}" > packages/tui/src/lib.rs && \
    echo "fn main() {}" > packages/preview/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cd packages/cli && cargo build --release && rm -rf src target/release/deps/orkee*

# Stage 2: Build Rust application
FROM rust-deps as rust-builder

# Copy source code
COPY packages/ ./packages/

# Build the application
WORKDIR /build/packages/cli
RUN cargo build --release --bin orkee

# Stage 3: Build Node.js dependencies (cache layer)
FROM node:18-slim as node-deps
WORKDIR /build

# Install pnpm
RUN npm install -g pnpm@^10.15.1

# Copy package files
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY packages/dashboard/package.json ./packages/dashboard/
COPY turbo.json ./

# Install dependencies (this layer will be cached)
RUN pnpm install --frozen-lockfile

# Stage 4: Build dashboard
FROM node-deps as dashboard-builder

# Copy dashboard source
COPY packages/dashboard ./packages/dashboard

# Build dashboard
RUN pnpm build --filter=@orkee/dashboard

# Stage 5: Create final runtime image
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd -r orkee && useradd -r -g orkee -d /var/lib/orkee -s /bin/false orkee

# Create application directories
RUN mkdir -p /var/lib/orkee/{data,certs,logs} /usr/share/orkee && \
    chown -R orkee:orkee /var/lib/orkee /usr/share/orkee

# Copy built binaries and assets
COPY --from=rust-builder --chown=orkee:orkee /build/target/release/orkee /usr/local/bin/orkee
COPY --from=dashboard-builder --chown=orkee:orkee /build/packages/dashboard/dist /usr/share/orkee/dashboard

# Make binary executable
RUN chmod +x /usr/local/bin/orkee

# Copy entrypoint script
COPY deployment/docker/docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh && \
    chown orkee:orkee /usr/local/bin/docker-entrypoint.sh

# Switch to non-root user
USER orkee
WORKDIR /var/lib/orkee

# Expose ports
EXPOSE 4001 4000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD curl -f http://localhost:4001/api/health || exit 1

# Environment defaults
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    PORT=4001 \
    TLS_ENABLED=false \
    AUTO_GENERATE_CERT=true

# Use entrypoint script
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["orkee", "dashboard"]