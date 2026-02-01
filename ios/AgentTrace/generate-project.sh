#!/bin/bash

# AgentTrace iOS Project Generator
# This script generates the Xcode project using XcodeGen

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔧 AgentTrace iOS Project Generator"
echo "===================================="

# Check if XcodeGen is installed
if ! command -v xcodegen &> /dev/null; then
    echo "❌ XcodeGen is not installed."
    echo ""
    echo "Install with Homebrew:"
    echo "  brew install xcodegen"
    echo ""
    echo "Or with Mint:"
    echo "  mint install yonaskolb/XcodeGen"
    exit 1
fi

echo "✅ XcodeGen found: $(xcodegen --version)"
echo ""

# Generate the project
echo "📦 Generating Xcode project..."
xcodegen generate

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Project generated successfully!"
    echo ""
    echo "Next steps:"
    echo "  1. Open AgentTrace.xcodeproj in Xcode"
    echo "  2. Select your development team for signing"
    echo "  3. Build and run on your device or simulator"
    echo ""
    echo "To open the project:"
    echo "  open AgentTrace.xcodeproj"
else
    echo ""
    echo "❌ Project generation failed!"
    exit 1
fi
