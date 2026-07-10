#!/bin/bash
# SPDX-FileCopyrightText: Copyright 2026 LG Electronics Inc.
# SPDX-License-Identifier: MIT
#
# install.sh — One-step installer for Timpani
#
#   timpani-n → installed as a native .deb/.rpm package (systemd service)
#   timpani-o → loaded and run as a Podman container (systemd-managed)
#
# Usage:
#   sudo ./scripts/install.sh
#
# Prerequisite: run scripts/build.sh first so dist/ contains the artifacts.

set -euo pipefail

log()  { echo "[install.sh] $*"; }
die()  { echo "ERROR: $*" >&2; exit 1; }

if [[ "${EUID}" -ne 0 ]]; then
    die "Please run as root: sudo $0"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${REPO_ROOT}/dist"
TIMPANI_O_VERSION="0.1.0"

detect_os() {
    if [[ -f /etc/os-release ]]; then
        # shellcheck disable=SC1091
        source /etc/os-release
        echo "${ID}"
    else
        die "Cannot detect OS — /etc/os-release not found."
    fi
}

OS=$(detect_os)
log "Detected OS: ${OS}"

find_package() {
    local pattern="$1"
    local found
    found=$(find "${DIST_DIR}" -maxdepth 1 -name "${pattern}" 2>/dev/null | sort -V | tail -n1)
    if [[ -z "${found}" ]]; then
        die "Package matching '${pattern}' not found in ${DIST_DIR}. Run scripts/build.sh first."
    fi
    echo "${found}"
}

# ---------------------------------------------------------------------------
# timpani-n — install native package
# ---------------------------------------------------------------------------
install_timpani_n() {
    case "${OS}" in
        ubuntu|debian)
            local pkg
            pkg=$(find_package "timpani-n*.deb")
            log "Installing ${pkg}..."
            dpkg -i "${pkg}" || apt-get install -f -y
            ;;
        centos|rhel|fedora)
            local pkg
            pkg=$(find_package "timpani-n*.rpm")
            log "Installing ${pkg}..."
            dnf install -y "${pkg}"
            ;;
        *)
            die "Unsupported OS for timpani-n package install: ${OS}"
            ;;
    esac
}

# ---------------------------------------------------------------------------
# timpani-o — install podman + load/run the container
# ---------------------------------------------------------------------------
install_podman() {
    if command -v podman &>/dev/null; then
        log "podman already installed: $(podman --version)"
        return
    fi

    log "Installing podman..."
    case "${OS}" in
        ubuntu|debian)
            apt-get update -qq
            apt-get install -y podman
            ;;
        centos|rhel|fedora)
            dnf install -y podman
            ;;
        *)
            die "Unsupported OS '${OS}' for automatic podman install."
            ;;
    esac
}

install_timpani_o() {
    local image_tar="${DIST_DIR}/timpani-o-${TIMPANI_O_VERSION}.tar"

    if [[ -f "${image_tar}" ]]; then
        log "Loading timpani-o image from ${image_tar}..."
        podman load -i "${image_tar}"
    else
        log "No pre-built image tar found, building timpani-o image locally..."
        podman build \
            --build-arg "VERSION=${TIMPANI_O_VERSION}" \
            -f "${REPO_ROOT}/timpani-o/Containerfile" \
            -t "timpani-o:${TIMPANI_O_VERSION}" \
            "${REPO_ROOT}"
    fi

    log "Installing default config to /etc/timpani-o/..."
    mkdir -p /etc/timpani-o
    cp -n "${REPO_ROOT}/timpani-o/examples/node_configurations.yaml" /etc/timpani-o/node_configurations.yaml || true

    log "Installing timpani-o systemd service..."
    cp "${REPO_ROOT}/timpani-o/systemd/timpani-o.service" /etc/systemd/system/timpani-o.service
    systemctl daemon-reload
    systemctl enable timpani-o.service
    systemctl restart timpani-o.service
}

# ---------------------------------------------------------------------------
# Verify
# ---------------------------------------------------------------------------
verify() {
    log "Verifying installation..."

    if systemctl is-active --quiet timpani-n; then
        log "timpani-n service is running."
    else
        log "WARNING: timpani-n service is not active. Check: journalctl -u timpani-n"
    fi

    if systemctl is-active --quiet timpani-o; then
        log "timpani-o service is running."
    else
        log "WARNING: timpani-o service is not active. Check: journalctl -u timpani-o"
    fi
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
install_timpani_n
install_podman
install_timpani_o
verify
