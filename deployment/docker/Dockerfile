# Multi-stage Dockerfile for Production
# Builds both the Rust CLI and React dashboard in an optimized container

# Stage 1: Build Rust dependencies (cache layer)
FROM rust:1.85-slim AS rust-deps
WORKDIR /build

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace and dependency manifests
COPY Cargo.toml Cargo.lock ./
COPY packages/cli/Cargo.toml ./packages/cli/
COPY packages/projects/Cargo.toml ./packages/projects/
COPY packages/tui/Cargo.toml ./packages/tui/
COPY packages/preview/Cargo.toml ./packages/preview/
COPY packages/api/Cargo.toml ./packages/api/
COPY packages/auth/Cargo.toml ./packages/auth/
COPY packages/core/Cargo.toml ./packages/core/
COPY packages/storage/Cargo.toml ./packages/storage/
COPY packages/security/Cargo.toml ./packages/security/
COPY packages/config/Cargo.toml ./packages/config/
COPY packages/cloud/Cargo.toml ./packages/cloud/
COPY packages/ai/Cargo.toml ./packages/ai/
COPY packages/context/Cargo.toml ./packages/context/
COPY packages/formatter/Cargo.toml ./packages/formatter/
COPY packages/git_utils/Cargo.toml ./packages/git_utils/
COPY packages/tags/Cargo.toml ./packages/tags/
COPY packages/settings/Cargo.toml ./packages/settings/
COPY packages/tasks/Cargo.toml ./packages/tasks/
COPY packages/models/Cargo.toml ./packages/models/
COPY packages/agents/Cargo.toml ./packages/agents/
COPY packages/sandbox/Cargo.toml ./packages/sandbox/
COPY packages/executions/Cargo.toml ./packages/executions/
COPY packages/ideate/Cargo.toml ./packages/ideate/
COPY packages/mcp-server/Cargo.toml ./packages/mcp-server/

# Create dummy source files to cache dependencies
RUN mkdir -p packages/cli/src/bin \
    packages/projects/src packages/tui/src packages/preview/src \
    packages/api/src packages/auth/src packages/core/src \
    packages/storage/src packages/security/src packages/config/src \
    packages/cloud/src packages/ai/src packages/context/src \
    packages/formatter/src packages/git_utils/src packages/tags/src \
    packages/settings/src packages/tasks/src packages/models/src \
    packages/agents/src packages/sandbox/src packages/executions/src \
    packages/ideate/src packages/mcp-server/src && \
    echo "fn main() {}" > packages/cli/src/bin/orkee.rs && \
    echo "fn main() {}" > packages/mcp-server/src/main.rs && \
    for pkg in projects tui preview api auth core storage security config \
               cloud ai context formatter git_utils tags settings tasks \
               models agents sandbox executions ideate; do \
        echo "" > packages/$pkg/src/lib.rs; \
    done

# Build dependencies (this layer will be cached)
RUN cargo build --release --bin orkee && rm -rf packages/*/src

# Stage 2: Build Rust application
FROM rust-deps AS rust-builder

# Copy source code
COPY packages/ ./packages/

# Build the application
RUN cargo build --release --bin orkee

# Stage 3: Build dashboard dependencies (cache layer)
FROM oven/bun:1-slim AS node-deps
WORKDIR /build

# Copy package files
COPY package.json bun.lock ./
COPY packages/dashboard/package.json ./packages/dashboard/
COPY turbo.json ./

# Install dependencies (this layer will be cached)
RUN bun install --frozen-lockfile

# Stage 4: Build dashboard
FROM node-deps AS dashboard-builder

# Copy dashboard source
COPY packages/dashboard ./packages/dashboard

# Build dashboard
RUN bun run build --filter=@orkee/dashboard

# Stage 5: Create final runtime image
FROM debian:bookworm-slim AS runtime

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
