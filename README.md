# 🌟 Rubik's Cube ECS - Modular 3D Game

A premium, highly interactive 3D Rubik's Cube game and solver built from the ground up using **Rust** and the modern **Bevy Engine (v0.18)**. Featuring an elegant modular ECS architecture, studio-quality lighting, customizable skins, and an automated Kociemba step-by-step solver.

---

## ✨ Key Features

- **🎮 Highly Interactive Controls**:
  - Smooth **Orbit Camera** with right-click drag navigation.
  - Precise mouse **Raycast Pick & Slice Rotation** (left-click and drag to rotate the exact slice you touch).
  - **✨ NEW: Real-Time Hand Tracking**: Control the entire Rubik's cube rotation and rotate individual slices using intuitive hand gestures in front of your webcam! Powered by Google MediaPipe Hands running in a lightweight Python background worker, synchronized with Bevy via real-time standard I/O piping (eliminating complex OpenCV C++ installation and compilation steps).
- **🧠 Intelligent Automated Solver**:
  - Integrated Kociemba's two-phase solver via the `kewb` library.
  - Full support for **Step-by-Step guided solving** with a modern overlay panel showing moves like `U`, `R'`, `F2`.
- **🎨 Premium Visual & Skin Customization**:
  - Real-time texture switching with diverse skin designs: **Classic**, **Carbon Fiber**, **Geometric Pattern**, and **Floral Texture**.
  - High-fidelity **Matte Materials** to avoid excessive glare and ensure visual comfort.
- **☀️ Real-time Environment & Studio Lighting**:
  - Advanced lighting rig that rotates with the camera to maintain optimal cube illumination.
  - Dynamic **shadows** cast onto a floor reflection plane.
  - Comprehensive UI configuration panel to customize:
    - **Background Color** (Clear Color & Floor base sync)
    - **Light Intensity**
    - **Ambient Brightness**
    - **Light Angle**
    - **Color Temperature** (Warmth/Coolness adjustment)
- **⚡ High Performance**:
  - Completely designed using Bevy's ECS (Entity Component System) architecture.
  - Smooth animation interpolation using Bevy transforms and quaternions.
  - Fast vector icons using `bevy_resvg` rendering.

---

## 🕹️ Controls Guide

| Action | Control | Description |
|:---|:---|:---|
| **Rotate Camera** | `Right-Click` + `Drag` | Orbit the camera around the Rubik's Cube. |
| **Hand Tracking** | `Camera Toggle` UI | Turn on the camera feed and rotate the entire cube by moving your hand left/right/up/down in front of your webcam. |
| **Zoom Camera** | `Scroll Wheel` | Zoom in and out on the cube. |
| **Reset Camera** | `Reset View Button` | Snaps the camera back to the default 45-degree angle. |
| **Rotate Rubik Slice**| `Left-Click` + `Drag` | Click on any cubelet face and drag in the desired direction to rotate that slice. |
| **Shuffle Cube** | `SHUFFLE` Button | Scrambles the cube randomly with smooth transition animations. |
| **Solve Cube** | `SOLVE` Button | Automatically calculates the fastest solution path and starts step-by-step solving. |
| **Next Step** | `NEXT STEP` Button | Executes the next rotation in the solution queue. |
| **Change Skin** | `Skins Dropdown` | Instantly switch between *Classic*, *Carbon*, *Geometric*, and *Floral* styles. |
| **Environment Settings**| `Gear/Control Icon` | Toggle environmental settings to customize colors, lights, and brightness. |

---

## 🚀 Installation & Run

### Prerequisites

Ensure you have the Rust toolchain installed. If not, get it from [rustup.rs](https://rustup.rs/).

**Hand Tracking Environment:**
The hand tracking feature uses a lightweight Python subprocess running **Google MediaPipe Hands** and **OpenCV-Python**. Since all computer vision calculations occur in Python, **no system-level OpenCV C++ development libraries or Clang packages are required to compile the Rust game**, ensuring 100% portable and instant Rust builds.

The Python virtual environment inside [hand_tracker](file:///home/tchuong/Màn hình nền/Game_rubik/hand_tracker) is pre-configured. If you ever need to set it up manually:
```bash
cd hand_tracker
python3 -m venv .venv
source .venv/bin/activate
pip install opencv-python mediapipe protobuf
```

```bash
# Clone the repository (if applicable)
cd Game_rubik
```

### Run the Application

To run the game in development mode with compiler optimizations for dependencies (configured in `Cargo.toml` for smooth Bevy performance):

```bash
cargo run --release
```

Using the `--release` flag is highly recommended for Bevy apps to ensure smooth 60+ FPS animations and instant solver calculations.

---

## 📂 Project Structure

```text
Game_rubik/
├── assets/                  # 3D Textures, Fonts, and UI SVG Icons
│   ├── fonts/               # UI fonts
│   └── textures/            # Skins and SVG/PNG UI icons
├── hand_tracker/            # Lightweight Rust workspace library and Python MediaPipe background worker for camera-based gesture control
├── src/                     # Rust Source Code (ECS Modules)
│   ├── camera/              # Orbit camera components and rotation systems
│   ├── environment/         # Studio lights, shadow cast floor, environmental adjustments
│   ├── input/               # Raycasting mouse interactions, slice selection & dragging, hand_tracking receiver
│   ├── rubik/               # Rubik cube spawn logic, material/skin application, rotation animations
│   ├── solver/              # State mapping to facelet notation & kewb solver interface
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
