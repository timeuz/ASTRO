#!/bin/bash
set -e

VERSION="1.0.0-beta.1"
ARCH="amd64"
PKG_NAME="astro-agent-companion"
BUILD_DIR="build/${PKG_NAME}_${VERSION}_${ARCH}"

echo "Building ASTRO $VERSION for $ARCH..."

# Clean old build
rm -rf build/
mkdir -p "$BUILD_DIR/DEBIAN"
mkdir -p "$BUILD_DIR/usr/bin"
mkdir -p "$BUILD_DIR/usr/lib/systemd/user"
mkdir -p "$BUILD_DIR/usr/share/gnome-shell/extensions/astro-spike@astro.project.org"
mkdir -p "$BUILD_DIR/usr/share/astro-agent/scripts"
mkdir -p "$BUILD_DIR/usr/share/astro-agent/docs"

# Control file
cat <<CTRL > "$BUILD_DIR/DEBIAN/control"
Package: $PKG_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Depends: jq
Maintainer: DevOps Automator <devops@astro.project>
Description: ASTRO Agent Companion Beta
 Observador de sessões para agentes AI.
 Beta Release Candidate para Ubuntu 26.04.
CTRL

# Build Rust Daemon
cd daemon/agent-companiond
source $HOME/.cargo/env
cargo build --release
cd ../..

# Copy binaries
cp daemon/agent-companiond/target/release/agent-companiond "$BUILD_DIR/usr/bin/astro-companiond"

# Copy extension
cp -r extension/* "$BUILD_DIR/usr/share/gnome-shell/extensions/astro-spike@astro.project.org/"

# Copy systemd unit
cp packaging/systemd/astro-companion.service "$BUILD_DIR/usr/lib/systemd/user/"

# Copy scripts and docs
cp -r scripts/* "$BUILD_DIR/usr/share/astro-agent/scripts/"
cp -r docs/* "$BUILD_DIR/usr/share/astro-agent/docs/"

# Post-install script
cat <<POSTINST > "$BUILD_DIR/DEBIAN/postinst"
#!/bin/bash
systemctl --user daemon-reload || true
# Orientacao de onboarding CLI
echo "ASTRO instalado. Para iniciar, execute:"
echo "python3 /usr/share/astro-agent/scripts/onboarding.py"
POSTINST
chmod +x "$BUILD_DIR/DEBIAN/postinst"

# Build deb
dpkg-deb --build "$BUILD_DIR"
echo "Package built at build/${PKG_NAME}_${VERSION}_${ARCH}.deb"
sha256sum "build/${PKG_NAME}_${VERSION}_${ARCH}.deb" > "build/${PKG_NAME}_${VERSION}_${ARCH}.deb.sha256"
echo "Checksum saved."
