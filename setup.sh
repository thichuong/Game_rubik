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

# -----------------------------------------------------------------------------
# 1. SETUP ROOT PYTHON VIRTUAL ENVIRONMENT (For Rubik Solver Crate)
# -----------------------------------------------------------------------------
echo "1. Creating root virtual environment (.venv)..."
if [ ! -d ".venv" ]; then
    python3 -m venv .venv
    echo "   [SUCCESS] Root virtual environment created."
else
    echo "   [INFO] Root virtual environment already exists."
fi

echo "   Upgrading pip and installing required root dependencies..."
.venv/bin/pip install --upgrade pip
.venv/bin/pip install opencv-python mediapipe protobuf

echo "   Compiling and installing dwalton76's Kociemba solver library..."
.venv/bin/pip install git+https://github.com/dwalton76/kociemba.git
echo "   [SUCCESS] Root Python environment configured successfully."
echo ""

# -----------------------------------------------------------------------------
# 2. SETUP BIG CUBE PYTHON SOLVER (rubiks-cube-NxNxN-solver)
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
echo ""

# -----------------------------------------------------------------------------
# 3. SETUP HAND TRACKER VIRTUAL ENVIRONMENT (For Bevy Webcam Integration)
# -----------------------------------------------------------------------------
echo "3. Setting up Hand Tracking Environment (hand_tracker/.venv)..."
if [ ! -d "hand_tracker/.venv" ]; then
    python3 -m venv hand_tracker/.venv
    echo "   [SUCCESS] Hand tracker virtual environment created."
else
    echo "   [INFO] Hand tracker virtual environment already exists."
fi

echo "   Upgrading pip and installing dependencies for MediaPipe tracker..."
hand_tracker/.venv/bin/pip install --upgrade pip
hand_tracker/.venv/bin/pip install opencv-python mediapipe protobuf
echo "   [SUCCESS] Hand Tracking Python environment configured successfully."
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
