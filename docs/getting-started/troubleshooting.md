---
sidebar_position: 4
title: Troubleshooting
---

# Troubleshooting

Common issues and solutions for Orkee installation and operation.

## Installation Issues

### Rust Installation Fails

**Problem**: Error installing Rust toolchain

**Solution**:
```bash
# Remove existing installation
rm -rf ~/.cargo ~/.rustup

# Reinstall Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload shell environment
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Build Fails with SSL Errors (Linux)

**Problem**: `error: failed to run custom build command for openssl-sys`

**Solution**:
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel

# Then rebuild
cd packages/cli
cargo build --release
```

### pnpm Command Not Found

**Problem**: `pnpm: command not found` after installation

**Solution**:
```bash
# Install pnpm globally
npm install -g pnpm

# Or use npm alternative
npm install

# Verify installation
pnpm --version
```

### Permission Denied During Installation

**Problem**: `EACCES: permission denied` when installing globally

**Solution**:
```bash
# Option 1: Use sudo (not recommended)
sudo npm install -g orkee

# Option 2: Configure npm to use user directory (recommended)
mkdir -p ~/.npm-global
npm config set prefix '~/.npm-global'
echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
source ~/.bashrc

# Then install without sudo
npm install -g orkee
```

## Runtime Issues

### Port Already in Use

**Problem**: `Error: Address already in use (os error 48)` or `EADDRINUSE`

**Symptoms**:
```
Error: Port 4001 is already in use
```

**Solution**:
```bash
# Find process using the port
lsof -i :4001
# or
netstat -an | grep 4001

# Kill the process
kill <PID>

# Or use a different port
orkee dashboard --api-port 8080

# Or set permanently
export ORKEE_API_PORT=8080
orkee dashboard
```

### Dashboard Won't Connect to API

**Problem**: Dashboard shows "Cannot connect to API server"

**Symptoms**:
- Dashboard loads but shows connection error
- Projects don't display
- Health check fails

**Solution**:

1. **Verify API is running**:
   ```bash
   curl http://localhost:4001/api/health
   # Should return: {"success":true,"data":{"status":"healthy"}}
   ```

2. **Check API port configuration**:
   ```bash
   # Verify which port API is using
   ps aux | grep orkee
   lsof -i -P | grep orkee
   ```

3. **Fix CORS issues**:
   ```bash
   # Set correct CORS origin
   export ORKEE_CORS_ORIGIN=http://localhost:5173
   orkee dashboard
   ```

4. **Check firewall** (Linux):
   ```bash
   # Allow port through firewall
   sudo ufw allow 4001/tcp
   sudo firewall-cmd --add-port=4001/tcp --permanent
   ```

### Database Errors

**Problem**: `Database is locked` or `unable to open database file`

**Solution**:

1. **Database locked**:
   ```bash
   # Stop all Orkee processes
   pkill orkee

   # Wait a moment
   sleep 2

   # Restart
   orkee dashboard
   ```

2. **Permission issues**:
   ```bash
   # Fix permissions
   chmod 644 ~/.orkee/orkee.db
   chown $USER ~/.orkee/orkee.db

   # Or for system installation
   sudo chown orkee:orkee /var/lib/orkee/data/orkee.db
   sudo chmod 644 /var/lib/orkee/data/orkee.db
   ```

3. **Corrupt database**:
   ```bash
   # Backup current database
   cp ~/.orkee/orkee.db ~/.orkee/orkee.db.backup

   # Try to repair
   sqlite3 ~/.orkee/orkee.db "PRAGMA integrity_check;"

   # If repair fails, restore from backup or start fresh
   rm ~/.orkee/orkee.db
   orkee dashboard  # Will create new database
   ```

### Certificate Errors

**Problem**: `SSL certificate verify failed` or `certificate has expired`

**Solution**:

1. **Self-signed certificate not trusted**:
   ```bash
   # macOS - Add to keychain
   sudo security add-trusted-cert -d -r trustRoot \
     -k /Library/Keychains/System.keychain \
     ~/.orkee/certs/cert.pem

   # Linux - Add to trusted certificates
   sudo cp ~/.orkee/certs/cert.pem /usr/local/share/ca-certificates/orkee.crt
   sudo update-ca-certificates

   # Or disable TLS for testing
   TLS_ENABLED=false orkee dashboard
   ```

2. **Certificate expired**:
   ```bash
   # Check expiration
   openssl x509 -in ~/.orkee/certs/cert.pem -noout -dates

   # Regenerate (self-signed)
   openssl req -x509 -newkey rsa:4096 \
     -keyout ~/.orkee/certs/key.pem \
     -out ~/.orkee/certs/cert.pem \
     -days 365 -nodes \
     -subj "/CN=localhost"

   # Or let Orkee auto-generate
   AUTO_GENERATE_CERT=true orkee dashboard
   ```

## Performance Issues

### Slow Response Times

**Problem**: API requests take too long to respond

**Solution**:

1. **Check system resources**:
   ```bash
   # Monitor CPU and memory
   top
   htop

   # Check disk I/O
   iostat
   ```

2. **Optimize database**:
   ```bash
   # Vacuum database
   sqlite3 ~/.orkee/orkee.db "VACUUM;"

   # Analyze for optimization
   sqlite3 ~/.orkee/orkee.db "ANALYZE;"

   # Enable WAL mode
   sqlite3 ~/.orkee/orkee.db "PRAGMA journal_mode=WAL;"
   ```

3. **Increase rate limits** (if hitting them):
   ```bash
   export RATE_LIMIT_GLOBAL_RPM=100
   export RATE_LIMIT_BURST_SIZE=20
   orkee dashboard
   ```

4. **Check for disk space**:
   ```bash
   df -h
   du -sh ~/.orkee/*
   ```

### High Memory Usage

**Problem**: Orkee consuming excessive memory

**Solution**:

1. **Check memory usage**:
   ```bash
   ps aux | grep orkee
   # or
   top -p $(pgrep orkee)
   ```

2. **Reduce logging level**:
   ```bash
   RUST_LOG=warn orkee dashboard  # Instead of debug or trace
   ```

3. **Limit resource usage** (systemd):
   ```ini
   [Service]
   MemoryMax=512M
   MemoryHigh=400M
   ```

4. **Restart service periodically**:
   ```bash
   # Add to cron (restart daily at 3 AM)
   0 3 * * * systemctl restart orkee
   ```

## Network Issues

### Cannot Access from Other Machines

**Problem**: Orkee accessible from localhost but not from network

**Solution**:

1. **Bind to all interfaces**:
   ```bash
   # Instead of localhost (127.0.0.1)
   export ORKEE_HOST=0.0.0.0
   orkee dashboard
   ```

2. **Check firewall rules**:
   ```bash
   # Ubuntu/Debian
   sudo ufw allow 4001/tcp
   sudo ufw status

   # CentOS/RHEL
   sudo firewall-cmd --add-port=4001/tcp --permanent
   sudo firewall-cmd --reload
   ```

3. **Verify network binding**:
   ```bash
   netstat -tulpn | grep 4001
   # Should show 0.0.0.0:4001 not 127.0.0.1:4001
   ```

### CORS Errors in Browser

**Problem**: Browser console shows CORS policy errors

**Symptoms**:
```
Access to fetch at 'http://localhost:4001/api/projects' from origin 'http://localhost:5173'
has been blocked by CORS policy
```

**Solution**:

1. **Set correct CORS origin**:
   ```bash
   export ORKEE_CORS_ORIGIN=http://localhost:5173
   orkee dashboard
   ```

2. **Allow any localhost** (development only):
   ```bash
   export CORS_ALLOW_ANY_LOCALHOST=true
   orkee dashboard
   ```

3. **For production**, set specific domain:
   ```bash
   export ORKEE_CORS_ORIGIN=https://your-domain.com
   ```

## Cloud Sync Issues

### Cloud Sync Not Working

**Problem**: Projects not syncing to Orkee Cloud

**Solution**:

1. **Verify authentication**:
   ```bash
   orkee cloud status
   # Should show: Status: Enabled

   # If not authenticated
   orkee cloud login
   ```

2. **Check cloud features enabled**:
   ```bash
   # Ensure Orkee was built with cloud features
   orkee --version
   # Look for "cloud: enabled"

   # If not, rebuild with cloud
   cargo build --release --features cloud
   ```

3. **Manual sync**:
   ```bash
   # Force a sync
   orkee cloud sync

   # Check for errors
   orkee cloud status
   ```

4. **Check network connectivity**:
   ```bash
   # Test connection to Orkee Cloud
   curl https://api.orkee.ai/health

   # Check proxy settings if behind corporate firewall
   export HTTPS_PROXY=http://proxy.example.com:8080
   ```

### Cloud Authentication Fails

**Problem**: `Authentication failed` when logging in

**Solution**:

1. **Clear existing credentials**:
   ```bash
   rm ~/.orkee/auth.toml
   orkee cloud login
   ```

2. **Check token expiration**:
   ```bash
   # View auth file
   cat ~/.orkee/auth.toml

   # If expired, re-authenticate
   orkee cloud logout
   orkee cloud login
   ```

3. **Verify account status**:
   - Log in to https://cloud.orkee.ai
   - Check account status and subscription
   - Verify API access is enabled

## TUI Issues

### TUI Display Corrupted

**Problem**: Terminal UI shows garbled characters or incorrect layout

**Solution**:

1. **Reset terminal**:
   ```bash
   reset
   # or
   tput reset
   ```

2. **Check terminal capabilities**:
   ```bash
   echo $TERM
   # Should be xterm-256color or similar

   # If not, set it
   export TERM=xterm-256color
   orkee tui
   ```

3. **Use compatible terminal**:
   - Recommended: iTerm2 (macOS), GNOME Terminal (Linux), Windows Terminal (Windows)
   - Avoid: basic Terminal.app, CMD.exe

4. **Adjust terminal size**:
   ```bash
   # TUI requires minimum size
   # Resize terminal to at least 80x24 characters
   ```

### TUI Keyboard Not Responding

**Problem**: Key presses don't work in TUI

**Solution**:

1. **Check terminal mode**:
   ```bash
   # Make sure not in vi mode or other keybindings
   set +o vi  # Disable vi mode if enabled
   ```

2. **Try alternative keys**:
   - Arrow keys not working? Try h/j/k/l
   - Enter not working? Try Return or Ctrl+M
   - Quit not working? Try Ctrl+C

3. **Restart TUI**:
   ```bash
   # Exit and restart
   pkill orkee
   orkee tui
   ```

## Common Error Messages

### "Permission denied (os error 13)"

**Cause**: Insufficient permissions to access file/directory

**Fix**:
```bash
# Fix file permissions
chmod 644 ~/.orkee/orkee.db
chmod 755 ~/.orkee

# Or run with appropriate user
sudo -u orkee orkee dashboard
```

### "No such file or directory"

**Cause**: Missing configuration or data files

**Fix**:
```bash
# Create missing directories
mkdir -p ~/.orkee/{data,certs,logs}

# Reset configuration
orkee dashboard  # Will create defaults
```

### "Connection refused"

**Cause**: API server not running or wrong port

**Fix**:
```bash
# Verify API is running
curl http://localhost:4001/api/health

# Check process
ps aux | grep orkee

# Start if not running
orkee dashboard
```

### "Rate limit exceeded"

**Cause**: Too many requests in short time

**Fix**:
```bash
# Wait 60 seconds, or increase limits
export RATE_LIMIT_GLOBAL_RPM=100
export RATE_LIMIT_BURST_SIZE=20
orkee dashboard
```

## Getting Help

### Collecting Diagnostic Information

When reporting issues, include:

```bash
# System information
uname -a
cat /etc/os-release  # Linux
sw_vers  # macOS

# Orkee version
orkee --version

# Runtime logs
orkee dashboard --verbose
# or
journalctl -u orkee -n 100  # systemd
tail -100 ~/.orkee/logs/orkee.log

# Configuration (redact sensitive info)
cat ~/.orkee/production.env

# Process information
ps aux | grep orkee
lsof -i -P | grep orkee

# Network connectivity
curl -v http://localhost:4001/api/health
```

### Enable Debug Logging

```bash
# Maximum verbosity
export RUST_LOG=debug
orkee dashboard --verbose

# Specific module logging
export RUST_LOG=orkee::api=debug,orkee::server=trace
orkee dashboard

# Save logs to file
orkee dashboard 2>&1 | tee orkee-debug.log
```

### Support Channels

- **GitHub Issues**: https://github.com/OrkeeAI/orkee/issues
- **GitHub Discussions**: https://github.com/OrkeeAI/orkee/discussions
- **Documentation**: https://docs.orkee.ai
- **Email**: support@orkee.ai

### Before Reporting Issues

1. Check this troubleshooting guide
2. Search existing GitHub issues
3. Update to latest version: `npm update -g orkee`
4. Try with clean configuration: `mv ~/.orkee ~/.orkee.backup && orkee dashboard`
5. Collect diagnostic information (see above)

## Next Steps

- [Installation Guide](installation) - Reinstall or try different method
- [Quick Start](quick-start) - Get started with fresh installation
- [Configuration Guide](../configuration/environment-variables) - Detailed configuration
- [GitHub Issues](https://github.com/OrkeeAI/orkee/issues) - Report bugs
