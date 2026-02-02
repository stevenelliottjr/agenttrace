#!/bin/bash
# AgentTrace One-Liner Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/stevenelliottjr/agenttrace/main/install.sh | bash
#
# Environment variables:
#   AGENTTRACE_HOME  - Installation directory (default: ~/.agenttrace)
#   AGENTTRACE_REPO  - Git repository URL (default: https://github.com/stevenelliottjr/agenttrace.git)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="${AGENTTRACE_HOME:-$HOME/.agenttrace}"
REPO_URL="${AGENTTRACE_REPO:-https://github.com/stevenelliottjr/agenttrace.git}"

# Print functions
print_banner() {
    echo -e "${CYAN}"
    echo "    _                    _  _____                    "
    echo "   / \   __ _  ___ _ __ | ||_   _| __ __ _  ___ ___ "
    echo "  / _ \ / _\` |/ _ \ '_ \| __|| || '__/ _\` |/ __/ _ \\"
    echo " / ___ \ (_| |  __/ | | | |_ | || | | (_| | (_|  __/"
    echo "/_/   \_\__, |\___|_| |_|\__||_||_|  \__,_|\___\___|"
    echo "        |___/                                        "
    echo -e "${NC}"
    echo -e "${BOLD}Datadog for AI Agents${NC}"
    echo ""
}

print_step() {
    echo -e "${BLUE}==>${NC} ${BOLD}$1${NC}"
}

print_success() {
    echo -e "${GREEN}==>${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}Warning:${NC} $1"
}

print_error() {
    echo -e "${RED}Error:${NC} $1" >&2
}

# Detect operating system
detect_os() {
    print_step "Detecting operating system..."

    OS="unknown"
    case "$(uname -s)" in
        Linux*)
            if grep -q Microsoft /proc/version 2>/dev/null; then
                OS="wsl"
                print_success "Detected: WSL2 (Windows Subsystem for Linux)"
            else
                OS="linux"
                print_success "Detected: Linux"
            fi
            ;;
        Darwin*)
            OS="macos"
            print_success "Detected: macOS"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            print_error "Windows native is not supported. Please use WSL2."
            echo "  Install WSL2: https://docs.microsoft.com/en-us/windows/wsl/install"
            exit 1
            ;;
        *)
            print_error "Unknown operating system: $(uname -s)"
            exit 1
            ;;
    esac
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_step "Checking prerequisites..."

    local missing=()

    # Check Docker
    if ! command_exists docker; then
        missing+=("docker")
    else
        print_success "Docker: $(docker --version | head -n1)"
    fi

    # Check Docker Compose (v2 style: docker compose)
    if docker compose version >/dev/null 2>&1; then
        print_success "Docker Compose: $(docker compose version --short)"
    elif command_exists docker-compose; then
        print_warning "Found legacy docker-compose. Recommend upgrading to Docker Compose V2."
        print_success "Docker Compose: $(docker-compose --version)"
    else
        missing+=("docker-compose")
    fi

    # Check Git
    if ! command_exists git; then
        missing+=("git")
    else
        print_success "Git: $(git --version)"
    fi

    # Check curl (for health checks)
    if ! command_exists curl; then
        missing+=("curl")
    fi

    # Report missing dependencies
    if [ ${#missing[@]} -ne 0 ]; then
        print_error "Missing required dependencies: ${missing[*]}"
        echo ""
        echo "Please install the missing dependencies:"

        if [[ " ${missing[*]} " =~ " docker " ]] || [[ " ${missing[*]} " =~ " docker-compose " ]]; then
            case "$OS" in
                macos)
                    echo "  Docker Desktop: https://docs.docker.com/desktop/install/mac-install/"
                    ;;
                linux)
                    echo "  Docker Engine: https://docs.docker.com/engine/install/"
                    ;;
                wsl)
                    echo "  Docker Desktop for Windows: https://docs.docker.com/desktop/install/windows-install/"
                    echo "  Enable WSL2 backend in Docker Desktop settings"
                    ;;
            esac
        fi

        if [[ " ${missing[*]} " =~ " git " ]]; then
            case "$OS" in
                macos)
                    echo "  Git: brew install git"
                    ;;
                linux|wsl)
                    echo "  Git: sudo apt install git (Ubuntu/Debian) or sudo yum install git (RHEL/CentOS)"
                    ;;
            esac
        fi

        if [[ " ${missing[*]} " =~ " curl " ]]; then
            case "$OS" in
                macos)
                    echo "  curl: brew install curl"
                    ;;
                linux|wsl)
                    echo "  curl: sudo apt install curl (Ubuntu/Debian)"
                    ;;
            esac
        fi

        exit 1
    fi

    # Check if Docker daemon is running
    if ! docker info >/dev/null 2>&1; then
        print_error "Docker daemon is not running."
        case "$OS" in
            macos)
                echo "  Please start Docker Desktop."
                ;;
            linux)
                echo "  Please start Docker: sudo systemctl start docker"
                ;;
            wsl)
                echo "  Please start Docker Desktop for Windows."
                ;;
        esac
        exit 1
    fi

    print_success "All prerequisites satisfied!"
}

# Setup repository
setup_repo() {
    print_step "Setting up AgentTrace..."

    if [ -d "$INSTALL_DIR" ]; then
        print_warning "Installation directory exists: $INSTALL_DIR"
        echo "  Updating existing installation..."
        cd "$INSTALL_DIR"
        git fetch origin
        git reset --hard origin/main
    else
        echo "  Cloning to $INSTALL_DIR..."
        git clone "$REPO_URL" "$INSTALL_DIR"
        cd "$INSTALL_DIR"
    fi

    print_success "Repository ready!"
}

# Build and start services
start_services() {
    print_step "Building and starting services..."
    echo "  This may take a few minutes on first run..."
    echo ""

    cd "$INSTALL_DIR"

    # Use docker compose (v2) if available, fallback to docker-compose
    if docker compose version >/dev/null 2>&1; then
        COMPOSE_CMD="docker compose"
    else
        COMPOSE_CMD="docker-compose"
    fi

    # Build and start
    $COMPOSE_CMD up -d --build

    print_success "Services started!"
}

# Wait for services to be healthy
wait_for_healthy() {
    print_step "Waiting for services to be healthy..."

    local max_attempts=60
    local attempt=0

    # Wait for TimescaleDB
    echo -n "  TimescaleDB: "
    while [ $attempt -lt $max_attempts ]; do
        if docker exec agenttrace-db pg_isready -U agenttrace >/dev/null 2>&1; then
            echo -e "${GREEN}ready${NC}"
            break
        fi
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    if [ $attempt -eq $max_attempts ]; then
        echo -e "${RED}timeout${NC}"
        print_error "TimescaleDB failed to start"
        return 1
    fi

    # Wait for Redis
    attempt=0
    echo -n "  Redis: "
    while [ $attempt -lt $max_attempts ]; do
        if docker exec agenttrace-redis redis-cli ping >/dev/null 2>&1; then
            echo -e "${GREEN}ready${NC}"
            break
        fi
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    if [ $attempt -eq $max_attempts ]; then
        echo -e "${RED}timeout${NC}"
        print_error "Redis failed to start"
        return 1
    fi

    # Wait for Collector
    attempt=0
    echo -n "  Collector: "
    while [ $attempt -lt $max_attempts ]; do
        if curl -sf http://localhost:8080/health >/dev/null 2>&1; then
            echo -e "${GREEN}ready${NC}"
            break
        fi
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    if [ $attempt -eq $max_attempts ]; then
        echo -e "${RED}timeout${NC}"
        print_error "Collector failed to start"
        echo "  Check logs: docker logs agenttrace-collector"
        return 1
    fi

    # Wait for Dashboard
    attempt=0
    echo -n "  Dashboard: "
    while [ $attempt -lt $max_attempts ]; do
        if curl -sf http://localhost:3000 >/dev/null 2>&1; then
            echo -e "${GREEN}ready${NC}"
            break
        fi
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    if [ $attempt -eq $max_attempts ]; then
        echo -e "${RED}timeout${NC}"
        print_error "Dashboard failed to start"
        echo "  Check logs: docker logs agenttrace-dashboard"
        return 1
    fi

    print_success "All services healthy!"
}

# Print success message
print_success_message() {
    echo ""
    echo -e "${GREEN}${BOLD}AgentTrace is running!${NC}"
    echo ""
    echo -e "${BOLD}Services:${NC}"
    echo -e "  Dashboard:   ${CYAN}http://localhost:3000${NC}"
    echo -e "  API:         ${CYAN}http://localhost:8080${NC}"
    echo -e "  gRPC:        ${CYAN}localhost:4317${NC}"
    echo -e "  UDP:         ${CYAN}localhost:4318${NC}"
    echo ""
    echo -e "${BOLD}Quick Start:${NC}"
    echo "  pip install agenttrace"
    echo ""
    echo -e "${BOLD}Example:${NC}"
    cat << 'EOF'
  from agenttrace import AgentTrace

  tracer = AgentTrace()

  with tracer.trace("my-agent") as trace:
      with trace.span("llm-call", span_type="llm") as span:
          # Your LLM call here
          span.set_attribute("model", "gpt-4")
          span.set_attribute("tokens.input", 100)
          span.set_attribute("tokens.output", 50)
EOF
    echo ""
    echo -e "${BOLD}Management:${NC}"
    echo "  cd $INSTALL_DIR"
    echo "  make up       # Start services"
    echo "  make down     # Stop services"
    echo "  make logs     # View logs"
    echo ""
}

# Main function
main() {
    print_banner

    detect_os
    check_prerequisites
    setup_repo
    start_services
    wait_for_healthy
    print_success_message
}

# Run main
main "$@"
