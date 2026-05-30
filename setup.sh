#!/bin/bash

# Exit immediately if any command fails
set -e

echo "=================================================="
echo "   🚀 RUBIK'S CUBE ECS ENVIRONMENT SETUP SCRIPT"
echo "=================================================="
echo "This script automatically sets up the environment for the Rubik Solver"
echo "and Hand Tracking systems, including all Python dependencies."
echo ""

# Get the absolute path to the project root directory
PROJECT_ROOT="$(pwd)"

# Find a compatible Python version (MediaPipe solutions work best on 3.9 - 3.12)
COMPATIBLE_PYTHON=""
for py_bin in python3.12 python3.11 python3.10 python3.9 python3; do
    if command -v "$py_bin" &> /dev/null; then
        # Check if the version is not 3.13 or 3.14 (where MediaPipe solutions is broken)
        py_ver=$("$py_bin" -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" 2>/dev/null)
        if [ "$py_ver" != "3.13" ] && [ "$py_ver" != "3.14" ] && [ "$py_ver" != "3.15" ] && [ ! -z "$py_ver" ]; then
            COMPATIBLE_PYTHON="$py_bin"
            break
        fi
    fi
done

if [ -z "$COMPATIBLE_PYTHON" ]; then
    echo "⚠️  WARNING: No highly compatible Python version (3.9 - 3.12) found in your PATH!"
    echo "   MediaPipe Hands legacy solutions are known to be broken on Python 3.13+ (detected: $(python3 --version))."
    echo "   We will fallback to '$(command -v python3)' but hand tracking may fail to import."
    echo "   👉 TO FIX THIS: Please install Python 3.12 by running:"
    echo "      Fedora:  sudo dnf install python3.12"
    echo "      Ubuntu:  sudo apt install python3.12"
    echo "   Then delete the existing .venv folders and re-run this setup script."
    echo ""
    COMPATIBLE_PYTHON="python3"
else
    echo "   [INFO] Found compatible Python interpreter: $(command -v $COMPATIBLE_PYTHON) (version $($COMPATIBLE_PYTHON --version))"
    echo ""
fi

# -----------------------------------------------------------------------------
# 1. SETUP UNIFIED PYTHON VIRTUAL ENVIRONMENT (For Solver & Hand Tracker)
# -----------------------------------------------------------------------------
echo "1. Creating unified virtual environment (.venv)..."
if [ ! -d ".venv" ]; then
    $COMPATIBLE_PYTHON -m venv .venv
    echo "   [SUCCESS] Unified virtual environment created."
else
    echo "   [INFO] Unified virtual environment already exists."
fi

echo "   Upgrading pip and installing required Python dependencies..."
.venv/bin/pip install --upgrade pip
.venv/bin/pip install opencv-python "mediapipe==0.10.14" protobuf

echo "   Compiling and installing dwalton76's Kociemba solver library..."
if ! .venv/bin/pip install git+https://github.com/dwalton76/kociemba.git; then
    echo ""
    echo "❌ ERROR: Failed to compile and install Kociemba solver library!"
    echo "   This usually happens because Python development headers (pyconfig.h) are missing."
    echo "   👉 TO FIX THIS, please run the following command to install headers:"
    echo "      Fedora:  sudo dnf install python3.12-devel"
    echo "      Ubuntu:  sudo apt install python3.12-dev"
    echo "   Then re-run this setup script: ./setup.sh"
    echo ""
    exit 1
fi
echo "   [SUCCESS] Unified Python environment configured successfully."
echo ""

# -----------------------------------------------------------------------------
# 2. SETUP BIG CUBE PYTHON SOLVER (python_solver/)
# -----------------------------------------------------------------------------
echo "2. Setting up Big Cube Solver (python_solver/)..."
mkdir -p python_solver

if [ ! -d "python_solver/rubiks-cube-NxNxN-solver" ]; then
    echo "   Cloning dwalton76's rubiks-cube-NxNxN-solver repository..."
    git clone https://github.com/dwalton76/rubiks-cube-NxNxN-solver.git python_solver/rubiks-cube-NxNxN-solver
    echo "   [SUCCESS] Big cube solver repository cloned."
else
    echo "   [INFO] Big cube solver repository already exists."
fi

echo "   Compiling C-based solver executable (ida_search_via_graph)..."
if [ -f "python_solver/rubiks-cube-NxNxN-solver/rubikscubennnsolver/ida_search_via_graph.c" ]; then
    cd python_solver/rubiks-cube-NxNxN-solver
    gcc -O3 -o ida_search_via_graph rubikscubennnsolver/ida_search_core.c rubikscubennnsolver/rotate_xxx.c rubikscubennnsolver/ida_search_666.c rubikscubennnsolver/ida_search_777.c rubikscubennnsolver/ida_search_via_graph.c -lm
    cd "$PROJECT_ROOT"
    echo "   [SUCCESS] C-based solver compiled successfully."
else
    echo "   [ERROR] C-based solver source files not found!"
    exit 1
fi
echo ""

# -----------------------------------------------------------------------------
# 3. PRE-DOWNLOAD SOLVER LOOKUP TABLES
# -----------------------------------------------------------------------------
echo "3. Pre-downloading Big Cube Solver Lookup Tables from S3..."
echo "   This ensures the Bevy game can solve 4x4 and 5x5 cubes instantly without timeout errors."
echo "   Downloading ~350MB of data, this may take a few minutes depending on your internet connection..."

# Run the python pre-download script inside virtual environment
if .venv/bin/python python_solver/download_tables.py; then
    echo "   [SUCCESS] Lookup tables successfully downloaded and cached."
else
    echo "   [WARNING] Failed to pre-download some lookup tables."
    echo "             The game will still attempt to download them lazily on first solve."
fi
echo ""

echo "=================================================="
echo "   🎉 ALL SYSTEMS CONFIGURED SUCCESSFULLY!"
echo "=================================================="
echo "You are ready to run the Bevy game and verify its solvers."
echo ""
echo "To run the end-to-end Rust verification tests (3x3, 4x4, 5x5):"
echo "   cargo run --example solve_verification"
echo ""
echo "To launch the premium 3D Game with Webcam Hand Tracking:"
echo "   cargo run --release"
echo ""
echo "Note: The first time you solve a 4x4 or 5x5 cube, the solver will"
echo "automatically download lookup tables (~200-300MB) from S3 and cache"
echo "them. Subsequent runs will solve the cube instantly."
echo "=================================================="
