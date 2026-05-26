# 🌟 Rubik's Cube ECS - Modular 3D Game & Multi-Size Solver

A premium, highly interactive 3D Rubik's Cube game and solver built from the ground up using **Rust** and the modern **Bevy Engine (v0.18)**. Featuring an elegant modular ECS architecture, studio-quality lighting, customizable skins, and a mathematically robust automated multi-size solver ($N \ge 2$) integrating fast Python solvers (**kociemba** for 3x3x3, and **rubiks-cube-NxNxN-solver** for big cubes) for a 100% solving success rate on all dimensions.

---

## ✨ Key Features

- **🎮 Highly Interactive Controls**:
  - Smooth **Orbit Camera** with right-click drag navigation.
  - Precise mouse **Raycast Pick & Slice Rotation** (left-click and drag to rotate the exact slice you touch).
  - **✨ Real-Time Hand Tracking**: Control the entire Rubik's cube rotation and rotate individual slices using intuitive hand gestures in front of your webcam! Powered by Google MediaPipe Hands running in a lightweight Python background worker, synchronized with Bevy via real-time standard I/O piping (eliminating complex OpenCV C++ installation and compilation steps).
- **🧠 Advanced Multi-Size Automated Solver ($N \ge 2$)**:
  - **Fast-Path 3x3x3 Core Solver**: Direct integration with the Python `kociemba` C-module library to solve the 3x3x3 core in milliseconds without requiring heavy optimization tables.
  - **Unified Big Cube Reduction Solver ($N \ge 4$)**: Fully interfaces with **rubiks-cube-NxNxN-solver** by dwalton76. Scrapes the 3D Bevy entity representation into a standard facelet notation, executes the Python reduction solver in the background, and parses advanced wide/slice moves into physical Bevy animations.
  - **Smart Move Parser Adapter**: Parses standard moves, wide turns (e.g. `Uw`, `Rw2`, `3Rw'`), and slice moves (e.g. `2R`, `3F'`) on arbitrary cube dimensions and translates them into physical 3D animations.
  - Full support for **Step-by-Step guided solving** with a modern overlay panel showing moves like `U`, `R'`, `F2`.
- **🎨 Premium Visual & Skin Customization**:
  - Real-time texture switching with diverse skin designs: **Classic**, **Carbon Fiber**, **Geometric Pattern**, and **Floral Texture**.
  - High-fidelity **Matte Materials** to avoid excessive glare and ensure visual comfort.
- **☀️ Real-time Environment & Studio Lighting**:
  - Advanced lighting rig that rotates with the camera to maintain optimal cube illumination.
  - Dynamic **shadows** cast onto a floor reflection plane.
  - Comprehensive UI configuration panel to customize background color, light intensity, ambient brightness, and color temperature.
- **⚡ High Performance**:
  - Completely designed using Bevy's ECS (Entity Component System) architecture.
  - Smooth animation interpolation using Bevy transforms and quaternions.
  - Fast vector icons using `bevy_resvg` rendering.

---

## 🚀 Installation & Environment Setup

To run the game and utilize the automated solver and hand tracking features, you simply need to execute our unified environment setup script. This script automatically builds the virtual environments, downloads the necessary big cube solver repository, and installs all python dependencies.

### 1. Run the Setup Script

From the project root directory, grant execution permissions and run the setup script:

```bash
chmod +x setup.sh
./setup.sh
```

This single script will configure:
- **Root Virtual Environment (`.venv/`)**: Compiles and installs `kociemba` and sets up the root Python engine for solver integration.
- **Big Cube Solver Repository (`python_solver/`)**: Clones dwalton76's `rubiks-cube-NxNxN-solver` repo into the project.
- **Hand Tracker Virtual Environment (`hand_tracker/.venv/`)**: Installs MediaPipe and OpenCV for the webcam gesture recognition engine.

> [!NOTE]  
> If you run the solver for size $N \ge 4$ for the first time, it will automatically download the required optimal lookup tables (approx. 200-300MB) from S3 and cache them locally in `python_solver/rubiks-cube-NxNxN-solver/lookup-tables/`. Subsequent runs will solve the cube instantly without any internet connection.

### 2. Run the Game

Compile and run the Rust game in release mode to ensure smooth 60+ FPS animations and real-time hand-gesture polling:

```bash
cargo run --release
```

---

## 🕹️ Controls Guide

| Action | Control | Description |
|:---|:---|:---|
| **Rotate Camera** | `Right-Click` + `Drag` | Orbit the camera around the Rubik's Cube. |
| **Hand Tracking** | `Camera Toggle` UI | Turn on the camera feed and rotate the entire cube by moving your hand in front of your webcam. |
| **Zoom Camera** | `Scroll Wheel` | Zoom in and out on the cube. |
| **Reset Camera** | `Reset View Button` | Snaps the camera back to the default 45-degree angle. |
| **Rotate Rubik Slice**| `Left-Click` + `Drag` | Click on any cubelet face and drag in the desired direction to rotate that slice. |
| **Shuffle Cube** | `SHUFFLE` Button | Scrambles the cube randomly with smooth transition animations. |
| **Solve Cube** | `SOLVE` Button | Automatically calculates the solution path via the Python backend solver and starts step-by-step guidance. |
| **Next Step** | `NEXT STEP` Button | Executes the next rotation in the solution queue. |
| **Change Skin** | `Skins Dropdown` | Instantly switch between *Classic*, *Carbon*, *Geometric*, and *Floral* styles. |
| **Environment Settings**| `Gear/Control Icon` | Toggle environmental settings to customize colors, lights, and brightness. |

---

## 📂 Project Structure

```text
Game_rubik/
├── .venv/                   # Python virtual environment containing kociemba and MediaPipe libraries
├── assets/                  # 3D Textures, Fonts, and UI SVG Icons
│   ├── fonts/               # UI fonts
│   └── textures/            # Skins and SVG/PNG UI icons
├── examples/
│   └── solve_verification.rs# End-to-end integration test scrambling and solving 3x3, 4x4, and 5x5 cubes
├── hand_tracker/            # Rust workspace library communicating with MediaPipe Python worker
├── python_solver/           # Folder containing big cube solver scripts and lookup tables
│   └── rubiks-cube-NxNxN-solver/ # dwalton76's python reduction solver
├── rubik_solver/            # High-performance modular multi-size Rubik solver crate
│   ├── src/
│   │   ├── nxn/             
│   │   │   └── state.rs     # Virtual representation (NxNState) of the cubelets and Kociemba mapping
│   │   ├── core.rs          # Shared data structures (Face, RotationMove, Direction)
│   │   ├── helpers.rs       # Bevy physical entity scraping and advanced string-to-move parser
│   │   ├── lib.rs           # Library entries and exports
│   │   └── solver.rs        # Unified entrypoint interfacing with background Python subprocesses
│   └── Cargo.toml           
├── src/                     # Rust Source Code (ECS Bevy Modules)
│   ├── camera/              # Orbit camera components and rotation systems
│   ├── environment/         # Studio lights, shadow cast floor, environmental adjustments
│   ├── input/               # Raycasting mouse interactions, slice selection & dragging, hand_tracking receiver
│   ├── rubik/               # Rubik cube spawn logic, material/skin application, rotation animations
│   ├── solver/              # State mapping to facelet notation & rubik_solver integration
│   ├── ui/                  # Advanced Bevy UI buttons, camera feed panel, dropdowns and settings
│   ├── events.rs            # Custom ECS event messages (CameraFrameEvent, HandRotationEvent, etc.)
│   └── main.rs              # Application initialization and plugin assembly
├── Cargo.toml               # Cargo package configuration and Bevy 0.18 dependencies
└── architecture.md          # In-depth architectural details
```

---

## 🏗️ Architecture & Technical Design

To explore the mathematical model of the cube, the coordinate transformation systems, raycasting implementation, and ECS data flows, please refer to the detailed [Architecture Documentation](architecture.md).

---

## 📄 License

This project is open-source. Feel free to explore, modify, and expand upon this codebase!
