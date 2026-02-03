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

# Install curl
install_curl() {
    print_step "Installing curl..."
    case "$OS" in
        linux|wsl)
            if command_exists apt-get; then
                sudo apt-get update && sudo apt-get install -y curl
            elif command_exists yum; then
                sudo yum install -y curl
            elif command_exists dnf; then
                sudo dnf install -y curl
            else
                print_error "Could not detect package manager to install curl."
                exit 1
            fi
            ;;
        macos)
            if command_exists brew; then
                brew install curl
            else
                print_error "Homebrew is not installed. Please install curl manually."
                echo "  Install Homebrew: https://brew.sh"
                exit 1
            fi
            ;;
    esac

    if command_exists curl; then
        print_success "curl installed successfully."
    else
        print_error "Failed to install curl."
        exit 1
    fi
}

# Install git
install_git() {
    print_step "Installing git..."
    case "$OS" in
        linux|wsl)
            if command_exists apt-get; then
                sudo apt-get update && sudo apt-get install -y git
            elif command_exists yum; then
                sudo yum install -y git
            elif command_exists dnf; then
                sudo dnf install -y git
            else
                print_error "Could not detect package manager to install git."
                exit 1
            fi
            ;;
        macos)
            if command_exists brew; then
                brew install git
            else
                print_step "Attempting to install git via Xcode Command Line Tools..."
                xcode-select --install 2>/dev/null || true
                echo "  If prompted, please complete the Xcode CLI tools installation and re-run this script."
                exit 1
            fi
            ;;
    esac

    if command_exists git; then
        print_success "git installed successfully."
    else
        print_error "Failed to install git."
        exit 1
    fi
}

# Install Docker
install_docker() {
    print_step "Installing Docker..."
    case "$OS" in
        linux|wsl)
            echo "  Using Docker's official install script..."
            if command_exists curl; then
                curl -fsSL https://get.docker.com | sudo sh
            elif command_exists wget; then
                wget -qO- https://get.docker.com | sudo sh
            else
                print_error "Neither curl nor wget is available. Cannot download Docker install script."
                exit 1
            fi

            # Add current user to docker group so sudo is not required
            if ! groups "$USER" | grep -q '\bdocker\b'; then
                echo "  Adding $USER to the docker group..."
                sudo usermod -aG docker "$USER"
                print_warning "You may need to log out and back in for group changes to take effect."
            fi

            # Start Docker daemon
            if command_exists systemctl; then
                sudo systemctl start docker
                sudo systemctl enable docker
            elif command_exists service; then
                sudo service docker start
            fi
            ;;
        macos)
            if command_exists brew; then
                echo "  Installing Docker Desktop via Homebrew..."
                brew install --cask docker
                echo "  Please open Docker Desktop to complete setup, then re-run this script."
                exit 0
            else
                print_error "Docker Desktop for macOS requires a GUI installer."
                echo "  Download it from: https://docs.docker.com/desktop/install/mac-install/"
                echo "  Or install Homebrew first: https://brew.sh"
                exit 1
            fi
            ;;
    esac

    if command_exists docker; then
        print_success "Docker installed successfully: $(docker --version | head -n1)"
    else
        print_error "Docker installation failed."
        echo "  Please install Docker manually:"
        echo "  https://docs.docker.com/engine/install/"
        exit 1
    fi
}

# Start Docker daemon if not running
start_docker_daemon() {
    if docker info >/dev/null 2>&1; then
        return 0
    fi

    print_step "Starting Docker daemon..."
    case "$OS" in
        linux|wsl)
            if command_exists systemctl; then
                sudo systemctl start docker
            elif command_exists service; then
                sudo service docker start
            else
                print_error "Could not start Docker daemon. No supported init system found."
                exit 1
            fi

            # Wait briefly for daemon to be ready
            local attempts=0
            while [ $attempts -lt 15 ]; do
                if docker info >/dev/null 2>&1; then
                    print_success "Docker daemon is running."
                    return 0
                fi
                sleep 2
                attempts=$((attempts + 1))
            done

            print_error "Docker daemon failed to start."
            echo "  Try starting it manually: sudo systemctl start docker"
            exit 1
            ;;
        macos)
            print_error "Docker daemon is not running. Please open Docker Desktop."
            exit 1
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    print_step "Checking prerequisites..."

    # Check and install curl
    if ! command_exists curl; then
        print_warning "curl is not installed."
        install_curl
    fi

    # Check and install git
    if ! command_exists git; then
        print_warning "git is not installed."
        install_git
    else
        print_success "Git: $(git --version)"
    fi

    # Check and install Docker
    if ! command_exists docker; then
        print_warning "Docker is not installed."
        install_docker
    else
        print_success "Docker: $(docker --version | head -n1)"
    fi

    # Check Docker Compose (v2 plugin — included by the official install script)
    if docker compose version >/dev/null 2>&1; then
        print_success "Docker Compose: $(docker compose version --short)"
    elif command_exists docker-compose; then
        print_warning "Found legacy docker-compose. Recommend upgrading to Docker Compose V2."
        print_success "Docker Compose: $(docker-compose --version)"
    else
        print_error "Docker Compose is not available."
        echo "  The Docker install script should have included Compose V2."
        echo "  Try: sudo apt-get install -y docker-compose-plugin"
        exit 1
    fi

    # Ensure Docker daemon is running
    start_docker_daemon

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
    echo -e "${BOLD}Open the Dashboard:${NC}"
    echo -e "  ${CYAN}http://localhost:3000${NC}"
    echo ""
    echo -e "${BOLD}Services:${NC}"
    echo -e "  Dashboard (React):  ${CYAN}http://localhost:3000${NC}"
    echo -e "  API / Health:       ${CYAN}http://localhost:8080${NC}"
    echo -e "  gRPC ingestion:     ${CYAN}localhost:4317${NC}"
    echo -e "  UDP  ingestion:     ${CYAN}localhost:4318${NC}"
    echo ""
    echo -e "${BOLD}TUI Dashboard (terminal):${NC}"
    echo "  cd $INSTALL_DIR"
    echo "  make dev-tui"
    echo ""
    echo -e "${BOLD}SDK Quick Start:${NC}"
    echo "  pip install agenttrace"
    echo ""
    cat << 'EOF'
  from agenttrace import AgentTrace

  tracer = AgentTrace()

  with tracer.trace("my-agent") as trace:
      with trace.span("llm-call", span_type="llm") as span:
          span.set_attribute("model", "gpt-4")
          span.set_attribute("tokens.input", 100)
          span.set_attribute("tokens.output", 50)
EOF
    echo ""
    echo -e "${BOLD}Management:${NC}"
    echo "  cd $INSTALL_DIR"
    echo "  make up          # Start all services"
    echo "  make down        # Stop all services"
    echo "  make restart     # Restart all services"
    echo "  make logs        # Follow all logs"
    echo "  make status      # Show service health"
    echo ""
    echo -e "${BOLD}Useful Commands:${NC}"
    echo "  make dev-collector   # Run collector with hot reload"
    echo "  make dev-dashboard   # Run React dashboard in dev mode"
    echo "  make dev-tui         # Launch the terminal UI"
    echo "  make db-shell        # Open a psql shell"
    echo "  make db-seed         # Load sample data"
    echo "  make help            # Show all available commands"
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
