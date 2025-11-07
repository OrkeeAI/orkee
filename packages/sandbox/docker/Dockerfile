FROM oven/bun:1

# Install system dependencies including build tools for native modules
RUN apt-get update && apt-get install -y \
    bash \
    curl \
    wget \
    git \
    python3 \
    python3-pip \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set up Bun environment
ENV BUN_INSTALL="/home/.bun"
ENV PATH="$BUN_INSTALL/bin:$PATH"

# Install coding agents globally with pinned versions
RUN bun add -g @anthropic-ai/claude-code@1.2.0 || echo "Claude CLI install failed"
RUN bun add -g @openai/codex@0.8.0 || echo "Codex CLI install failed"
RUN bun add -g @google/gemini-cli@1.0.0 || echo "Gemini CLI install failed"
RUN bun add -g opencode-ai@1.0.0 || echo "OpenCode CLI install failed"
RUN bun add -g @vibe-kit/grok-cli@1.0.0 || echo "Grok CLI install failed"

# Install vibekit CLI globally with pinned version
RUN bun add -g vibekit@1.0.0 || echo "Vibekit install failed"

# Create non-root user for running the sandbox
RUN useradd -m -s /bin/bash sandbox \
    && mkdir -p /workspace \
    && chown -R sandbox:sandbox /workspace /home/.bun

# Switch to non-root user
USER sandbox

WORKDIR /workspace

# Update PATH for non-root user
ENV PATH="/home/sandbox/.local/bin:$BUN_INSTALL/bin:$PATH"

# Expose common local development ports
EXPOSE 3001
EXPOSE 8080

# Add healthcheck to monitor container health
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD pgrep -u sandbox tail || exit 1

# Keep container running for persistent use
CMD ["tail", "-f", "/dev/null"]