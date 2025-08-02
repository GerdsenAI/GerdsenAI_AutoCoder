#!/bin/bash

# GerdsenAI Socrates - Universal Installer Wrapper
# Copyright (c) 2025 GerdsenAI. All rights reserved.
# This script detects the platform and runs the appropriate installer

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to display header
show_header() {
    echo -e "${GREEN}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                                                              ║"
    echo "║                   🚀 GerdsenAI Socrates                     ║"
    echo "║                 Advanced AI Coding Assistant                ║"
    echo "║                                                              ║"
    echo "║                    Universal Installer                      ║"
    echo "║                                                              ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo
}

# Function to detect platform
detect_platform() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        PLATFORM="linux"
        DISTRO="Unknown"
        if command -v lsb_release &> /dev/null; then
            DISTRO=$(lsb_release -si)
        elif [ -f /etc/os-release ]; then
            . /etc/os-release
            DISTRO=$NAME
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        PLATFORM="macos"
        DISTRO="macOS $(sw_vers -productVersion)"
    elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        PLATFORM="windows"
        DISTRO="Windows"
    else
        echo -e "${RED}❌ Unsupported platform: $OSTYPE${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}🔍 Detected platform: $PLATFORM ($DISTRO)${NC}"
}

# Function to check for required tools
check_requirements() {
    echo -e "${BLUE}🔍 Checking requirements...${NC}"
    
    # Check for curl
    if ! command -v curl &> /dev/null; then
        echo -e "${RED}❌ curl is required but not installed${NC}"
        echo "Please install curl and try again"
        exit 1
    fi
    
    # Platform-specific checks
    case "$PLATFORM" in
        "linux")
            # Check for bash version
            if [ "${BASH_VERSION%%.*}" -lt 4 ]; then
                echo -e "${YELLOW}⚠️  Bash 4+ recommended. Current: $BASH_VERSION${NC}"
            fi
            ;;
        "macos")
            # Check for Xcode command line tools
            if ! xcode-select -p &> /dev/null; then
                echo -e "${YELLOW}⚠️  Xcode command line tools not installed${NC}"
                echo "Install with: xcode-select --install"
                read -p "Continue anyway? (y/N) " -n 1 -r
                echo
                if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                    exit 1
                fi
            fi
            ;;
        "windows")
            echo -e "${YELLOW}⚠️  For Windows, please run install-simplified.ps1 with PowerShell as Administrator${NC}"
            echo "This script is for Unix-like systems only"
            exit 1
            ;;
    esac
    
    echo -e "${GREEN}✅ Requirements check passed${NC}"
}

# Function to show installation options
show_options() {
    echo -e "${BLUE}📋 Installation Options:${NC}"
    echo
    echo "1. 🚀 Full Installation (Recommended)"
    echo "   • Installs all dependencies automatically"
    echo "   • Downloads and configures AI models"
    echo "   • Sets up optional services (Docker, SearXNG)"
    echo "   • Ready to use immediately"
    echo
    echo "2. 🔧 Custom Installation"
    echo "   • Choose which components to install"
    echo "   • Skip optional dependencies"
    echo "   • More control over the process"
    echo
    echo "3. 📦 Development Installation"
    echo "   • For developers and contributors"
    echo "   • Includes development tools and dependencies"
    echo "   • Source code compilation"
    echo
    echo "4. 🔄 Update Existing Installation"
    echo "   • Update to latest version"
    echo "   • Preserve existing configuration"
    echo "   • Backup previous version"
    echo
    echo -n "Choose an option (1-4): "
    read -r OPTION
    echo
}

# Function to run the appropriate installer
run_installer() {
    local script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
    local installer_script="$script_dir/scripts/install-simplified.sh"
    
    # Check if installer script exists
    if [ ! -f "$installer_script" ]; then
        echo -e "${RED}❌ Installer script not found: $installer_script${NC}"
        exit 1
    fi
    
    # Make installer executable
    chmod +x "$installer_script"
    
    case "$OPTION" in
        "1")
            echo -e "${GREEN}🚀 Starting full installation...${NC}"
            "$installer_script"
            ;;
        "2")
            echo -e "${BLUE}🔧 Starting custom installation...${NC}"
            echo "Custom installation options:"
            echo
            
            read -p "Skip prerequisites installation? (y/N) " -n 1 -r
            echo
            SKIP_PREREQ=""
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                SKIP_PREREQ="--skip-prerequisites"
            fi
            
            read -p "Skip AI model download? (y/N) " -n 1 -r
            echo
            SKIP_MODELS=""
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                SKIP_MODELS="--skip-models"
            fi
            
            read -p "Skip Docker installation? (y/N) " -n 1 -r
            echo
            SKIP_DOCKER=""
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                SKIP_DOCKER="--skip-docker"
            fi
            
            "$installer_script" $SKIP_PREREQ $SKIP_MODELS $SKIP_DOCKER
            ;;
        "3")
            echo -e "${BLUE}📦 Starting development installation...${NC}"
            "$installer_script" --development
            ;;
        "4")
            echo -e "${BLUE}🔄 Starting update installation...${NC}"
            "$installer_script" --update
            ;;
        *)
            echo -e "${RED}❌ Invalid option: $OPTION${NC}"
            exit 1
            ;;
    esac
}

# Function to show post-installation info
show_completion() {
    echo
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}║                    🎉 Installation Complete!                ║${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo
    echo -e "${BLUE}🚀 Getting Started:${NC}"
    
    case "$PLATFORM" in
        "macos")
            echo "  • Open Applications folder and launch 'GerdsenAI Socrates'"
            echo "  • Or use Spotlight search (⌘+Space) and type 'GerdsenAI'"
            ;;
        "linux")
            echo "  • Run: gerdsenai-socrates"
            echo "  • Or find 'GerdsenAI Socrates' in your applications menu"
            ;;
    esac
    
    echo
    echo -e "${BLUE}📚 Resources:${NC}"
    echo "  • User Manual: COMPREHENSIVE_USER_MANUAL.md"
    echo "  • Troubleshooting: TROUBLESHOOTING_GUIDE.md"
    echo "  • GitHub: https://github.com/GerdsenAI/GerdsenAI_AutoCoder"
    echo
    echo -e "${BLUE}💬 Need Help?${NC}"
    echo "  • Documentation: Check the user manual first"
    echo "  • Issues: GitHub Issues for bug reports"
    echo "  • Community: Join our Discord/discussions"
    echo
    echo -e "${GREEN}Happy coding with GerdsenAI Socrates! 🚀${NC}"
}

# Main installation flow
main() {
    # Show header
    show_header
    
    # Detect platform
    detect_platform
    
    # Check requirements
    check_requirements
    
    # Show options
    show_options
    
    # Confirm installation
    echo -e "${YELLOW}⚠️  This installer will modify your system and install dependencies.${NC}"
    echo -n "Do you want to continue? (y/N) "
    read -r CONFIRM
    echo
    
    if [[ ! $CONFIRM =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Installation cancelled by user${NC}"
        exit 0
    fi
    
    # Run installer
    run_installer
    
    # Show completion message
    show_completion
}

# Trap to handle interruption
trap 'echo -e "\n${RED}Installation interrupted by user${NC}"; exit 1' INT

# Run main function
main "$@"