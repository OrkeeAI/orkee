#!/bin/bash
set -e

# Docker entrypoint script for Orkee
# Handles initialization, certificate management, and graceful shutdown

# Default values
DEFAULT_PORT=${PORT:-4001}
DEFAULT_TLS_ENABLED=${TLS_ENABLED:-false}
DEFAULT_AUTO_GENERATE_CERT=${AUTO_GENERATE_CERT:-true}

# Colors for logging
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_debug() {
    if [[ "${RUST_LOG}" == "debug" || "${DEBUG}" == "true" ]]; then
        echo -e "${BLUE}[DEBUG]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
    fi
}

# Signal handlers for graceful shutdown
shutdown() {
    log_info "Received shutdown signal, stopping Orkee gracefully..."
    if [[ -n "$ORKEE_PID" ]]; then
        kill -TERM "$ORKEE_PID" 2>/dev/null || true
        wait "$ORKEE_PID" 2>/dev/null || true
    fi
    log_info "Orkee stopped"
    exit 0
}

# Set up signal handlers
trap shutdown SIGTERM SIGINT SIGQUIT

# Initialize directories
init_directories() {
    log_info "Initializing application directories..."
    
    # Ensure directories exist with correct permissions
    mkdir -p /var/lib/orkee/{data,certs,logs}
    
    # Set permissions (only if we have write access)
    if [[ -w /var/lib/orkee ]]; then
        chmod 755 /var/lib/orkee
        chmod 755 /var/lib/orkee/{data,logs}
        chmod 700 /var/lib/orkee/certs  # Restrict certificate access
        log_debug "Directory permissions set"
    else
        log_warn "Cannot set directory permissions (read-only filesystem?)"
    fi
}

# Validate configuration
validate_config() {
    log_info "Validating configuration..."
    
    # Check port is valid
    if ! [[ "$DEFAULT_PORT" =~ ^[0-9]+$ ]] || [ "$DEFAULT_PORT" -lt 1 ] || [ "$DEFAULT_PORT" -gt 65535 ]; then
        log_error "Invalid port number: $DEFAULT_PORT"
        exit 1
    fi
    
    # Check TLS configuration if enabled
    if [[ "$DEFAULT_TLS_ENABLED" == "true" ]]; then
        if [[ "$DEFAULT_AUTO_GENERATE_CERT" == "false" ]]; then
            # Check if custom certificates exist
            if [[ ! -f "${TLS_CERT_PATH:-/var/lib/orkee/certs/cert.pem}" ]] || [[ ! -f "${TLS_KEY_PATH:-/var/lib/orkee/certs/key.pem}" ]]; then
                log_error "TLS enabled but certificate files not found. Set AUTO_GENERATE_CERT=true or provide certificate files."
                exit 1
            fi
            log_info "Using custom TLS certificates"
        else
            log_info "TLS enabled with auto-generated certificates"
        fi
    fi
    
    log_info "Configuration validation passed"
}

# Check system health
health_check() {
    log_debug "Performing system health check..."
    
    # Check disk space
    local disk_usage
    disk_usage=$(df /var/lib/orkee 2>/dev/null | awk 'NR==2 {print $5}' | sed 's/%//')
    if [[ -n "$disk_usage" ]] && [[ "$disk_usage" -gt 90 ]]; then
        log_warn "Disk usage is high: ${disk_usage}%"
    fi
    
    # Check if ports are available (only if not already bound)
    if ! netstat -ln 2>/dev/null | grep -q ":${DEFAULT_PORT} "; then
        log_debug "Port $DEFAULT_PORT is available"
    else
        log_warn "Port $DEFAULT_PORT may already be in use"
    fi
    
    # Check memory
    local mem_available
    mem_available=$(free -m 2>/dev/null | awk 'NR==2{printf "%.1f", $7/$2*100}')
    if [[ -n "$mem_available" ]]; then
        log_debug "Available memory: ${mem_available}%"
    fi
}

# Wait for dependencies (if any)
wait_for_dependencies() {
    # If DATABASE_URL is set, wait for database
    if [[ -n "$DATABASE_URL" ]]; then
        log_info "Waiting for database connection..."
        # Add database connection check logic here if needed
    fi
    
    # If REDIS_URL is set, wait for Redis
    if [[ -n "$REDIS_URL" ]]; then
        log_info "Waiting for Redis connection..."
        # Add Redis connection check logic here if needed
    fi
    
    # Add other dependency checks as needed
}

# Print startup banner
print_banner() {
    cat << 'EOF'
     ██████  ██████  ██   ██ ███████ ███████ 
    ██    ██ ██   ██ ██  ██  ██      ██      
    ██    ██ ██████  █████   █████   █████   
    ██    ██ ██   ██ ██  ██  ██      ██      
     ██████  ██   ██ ██   ██ ███████ ███████ 
                                              
    AI Agent Orchestration Platform
EOF
    echo ""
    log_info "Starting Orkee v$(orkee --version 2>/dev/null | cut -d' ' -f2 || echo 'unknown')"
    log_info "Configuration:"
    log_info "  Port: $DEFAULT_PORT"
    log_info "  TLS: $DEFAULT_TLS_ENABLED"
    log_info "  Auto-generate certificates: $DEFAULT_AUTO_GENERATE_CERT"
    log_info "  Working directory: $(pwd)"
    log_info "  User: $(whoami)"
    echo ""
}

# Main execution
main() {
    # Print banner
    print_banner
    
    # Initialize
    init_directories
    validate_config
    health_check
    wait_for_dependencies
    
    # If no arguments provided, show help
    if [[ $# -eq 0 ]]; then
        log_info "No command specified, running default: orkee dashboard"
        set -- "orkee" "dashboard"
    fi
    
    # If first argument is 'orkee', run it directly
    if [[ "$1" == "orkee" ]]; then
        log_info "Starting Orkee with arguments: ${*}"
        
        # Export environment variables
        export PORT="$DEFAULT_PORT"
        export TLS_ENABLED="$DEFAULT_TLS_ENABLED"
        export AUTO_GENERATE_CERT="$DEFAULT_AUTO_GENERATE_CERT"
        
        # Start Orkee in background so we can handle signals
        "$@" &
        ORKEE_PID=$!
        
        log_info "Orkee started with PID $ORKEE_PID"
        log_info "Server will be available at:"
        if [[ "$DEFAULT_TLS_ENABLED" == "true" ]]; then
            log_info "  HTTPS: https://localhost:$DEFAULT_PORT"
            log_info "  HTTP (redirect): http://localhost:$((DEFAULT_PORT - 1))"
        else
            log_info "  HTTP: http://localhost:$DEFAULT_PORT"
        fi
        
        # Wait for the process
        wait $ORKEE_PID
        exit_code=$?
        
        if [[ $exit_code -ne 0 ]]; then
            log_error "Orkee exited with code $exit_code"
        else
            log_info "Orkee exited cleanly"
        fi
        
        exit $exit_code
    else
        # Run other commands directly
        log_info "Executing command: $*"
        exec "$@"
    fi
}

# Run main function with all arguments
main "$@"