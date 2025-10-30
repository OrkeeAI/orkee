#!/bin/bash
# ABOUTME: Pre-CI check script that runs all formatting, linting, clippy, and tests
# ABOUTME: Mirrors the GitHub Actions CI workflow to catch issues before pushing

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
FIX_MODE=false
SKIP_TESTS=false

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --fix)
      FIX_MODE=true
      shift
      ;;
    --no-tests)
      SKIP_TESTS=true
      shift
      ;;
    --help|-h)
      echo "Usage: ./check-ci.sh [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --fix         Auto-fix formatting and clippy warnings"
      echo "  --no-tests    Skip running tests (faster, useful for quick checks)"
      echo "  --help, -h    Show this help message"
      echo ""
      echo "Examples:"
      echo "  ./check-ci.sh              # Run all checks"
      echo "  ./check-ci.sh --fix        # Run all checks and auto-fix issues"
      echo "  ./check-ci.sh --no-tests   # Run checks but skip tests"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Run './check-ci.sh --help' for usage"
      exit 1
      ;;
  esac
done

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Orkee Pre-CI Check                  ║${NC}"
echo -e "${BLUE}║   (Mirrors GitHub test-web + test-rust)║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

if [ "$FIX_MODE" = true ]; then
  echo -e "${YELLOW}🔧 Fix mode enabled - will auto-fix issues${NC}"
fi

if [ "$SKIP_TESTS" = true ]; then
  echo -e "${YELLOW}⏭️  Skipping tests${NC}"
fi

echo ""

#####################################
# 1. Rust Formatting
#####################################
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}1/5 Checking Rust formatting...${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ "$FIX_MODE" = true ]; then
  cargo fmt --all
  echo -e "${GREEN}✓ Rust formatting applied${NC}"
else
  if cargo fmt --all -- --check; then
    echo -e "${GREEN}✓ Rust formatting OK${NC}"
  else
    echo -e "${RED}✗ Rust formatting failed${NC}"
    echo -e "${YELLOW}Tip: Run './check-ci.sh --fix' to auto-format${NC}"
    exit 1
  fi
fi
echo ""

#####################################
# 2. Rust Clippy
#####################################
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}2/5 Running Clippy (Rust linter)...${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

CLIPPY_PACKAGES=(
  "orkee-config"
)

# Note: orkee-projects and orkee-cli require database setup for sqlx macros
# Run manually with: DATABASE_URL=sqlite:orkee.db cargo test --package orkee-projects
# Run manually with: DATABASE_URL=sqlite:orkee.db cargo test --package orkee-cli

# Set DATABASE_URL for sqlx macros if not already set
if [ -z "$DATABASE_URL" ]; then
  export DATABASE_URL="sqlite::memory:"
fi

# SQLX_OFFLINE requires .sqlx directory with prepared queries
# For now, orkee-projects is excluded from CI checks

if [ "$FIX_MODE" = true ]; then
  for package in "${CLIPPY_PACKAGES[@]}"; do
    echo -e "${YELLOW}Fixing $package...${NC}"
    cargo clippy --package "$package" --fix --allow-dirty --allow-staged -- -D warnings || {
      echo -e "${RED}✗ Clippy fix failed for $package${NC}"
      exit 1
    }
  done
  echo -e "${GREEN}✓ Clippy fixes applied${NC}"
else
  for package in "${CLIPPY_PACKAGES[@]}"; do
    echo -e "${YELLOW}Checking $package...${NC}"
    cargo clippy --package "$package" -- -D warnings || {
      echo -e "${RED}✗ Clippy failed for $package${NC}"
      echo -e "${YELLOW}Tip: Run './check-ci.sh --fix' to auto-fix some issues${NC}"
      exit 1
    }
  done
  echo -e "${GREEN}✓ Clippy checks passed${NC}"
fi
echo ""

#####################################
# 3. Rust Tests
#####################################
if [ "$SKIP_TESTS" = false ]; then
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${BLUE}3/5 Running Rust tests...${NC}"
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

  for package in "${CLIPPY_PACKAGES[@]}"; do
    echo -e "${YELLOW}Testing $package...${NC}"
    cargo test --package "$package" || {
      echo -e "${RED}✗ Tests failed for $package${NC}"
      exit 1
    }
  done
  echo -e "${GREEN}✓ All Rust tests passed${NC}"
  echo ""
else
  echo -e "${YELLOW}⏭️  Skipping Rust tests${NC}"
  echo ""
fi

#####################################
# 4. Frontend Lint
#####################################
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}4/5 Linting dashboard (ESLint)...${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

cd packages/dashboard

if [ "$FIX_MODE" = true ]; then
  bun run lint --fix || {
    echo -e "${RED}✗ ESLint fix failed${NC}"
    exit 1
  }
  echo -e "${GREEN}✓ ESLint fixes applied${NC}"
else
  bun run lint || {
    echo -e "${RED}✗ ESLint failed${NC}"
    echo -e "${YELLOW}Tip: Run './check-ci.sh --fix' to auto-fix some issues${NC}"
    exit 1
  }
  echo -e "${GREEN}✓ ESLint passed${NC}"
fi

cd ../..
echo ""

#####################################
# 5. Frontend Build
#####################################
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}5/5 Building dashboard...${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

cd packages/dashboard
bun run build || {
  echo -e "${RED}✗ Dashboard build failed${NC}"
  exit 1
}
echo -e "${GREEN}✓ Dashboard build succeeded${NC}"

# Show bundle size
echo -e "${YELLOW}Bundle size:${NC}"
du -sh dist

cd ../..
echo ""

#####################################
# Summary
#####################################
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   ✓ All CI checks passed!             ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""

if [ "$FIX_MODE" = true ]; then
  echo -e "${YELLOW}🔧 Auto-fixes were applied. Review changes before committing.${NC}"
  echo ""
fi

echo -e "${BLUE}Next steps:${NC}"
echo -e "  • Review any changes made by auto-fix"
echo -e "  • Commit your changes: ${YELLOW}git add . && git commit${NC}"
echo -e "  • Push to GitHub: ${YELLOW}git push${NC}"
echo ""
