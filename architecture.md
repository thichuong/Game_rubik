# 🏗️ Rubik's Cube ECS - Architecture & Design

This document outlines the architectural patterns, Entity Component System (ECS) design choices, and background solving engine integration implemented in the 3D Rubik's Cube game using **Bevy Engine (v0.18)** and **Rust**.

---

## 🧭 System Architecture

The application is structured into decoupled, self-contained Bevy `Plugin` modules.

```mermaid
graph TD
    %% Module Plugins
    subgraph UI ["🎨 UI Module"]
        UiPlugin["UiPlugin"]
        Components["components.rs <br>(Marker Components)"]
        subgraph Interactions ["interactions/ (Modular Systems)"]
            InteractionsMod["mod.rs <br>(Orchestrator & Re-exports)"]
            SolverInt["solver.rs <br>(Solve & Shuffle)"]
            SkinInt["skin.rs <br>(Skin Selection)"]
            EnvInt["environment.rs <br>(Env Settings)"]
            SizeInt["size.rs <br>(Cube Sizing)"]
            MappingInt["mapping.rs <br>(Face Mapping)"]
            SidebarInt["sidebar.rs <br>(Scrolling & Drag)"]
            CameraInt["camera.rs <br>(Camera & Gestures)"]
            AppInt["app.rs <br>(Exit Controls)"]
        end
        subgraph Layout ["layout/ (Modular Layout)"]
            LayoutMod["layout.rs <br>(setup_ui Orchestrator)"]
            Sidebar["sidebar.rs <br>(Left Sidebar Layout)"]
            Env["environment.rs <br>(3D Env Controls)"]
            HUD["hud.rs <br>(Bottom Steps HUD)"]
        end
    end

    subgraph Input ["🖱️ Input Module"]
        InputPlugin["InputPlugin"]
        Raycast["Manual Raycasting & Plane Intersect"]
        DragState["DragState Resource"]
    end

    subgraph SolverModule ["🧠 Solver Module & Engine"]
        SolverPlugin["SolverPlugin"]
        StepByStep["StepByStepSolution Resource"]
        
        subgraph RubikSolverLib ["📚 rubik_solver Crate (Python Daemon Bridge & Rust 3x3)"]
            UnifiedEntry["solve_cube_for_size <br>(Unified Entry Point)"]
            
            subgraph RustSolver ["🦀 Pure Rust Solver"]
                Kewb["kewb Crate <br>(Fast 3x3x3 optimal solver)"]
            end
            
            subgraph PythonSolvers ["🐍 Python Solver Background Daemon"]
                NxnDaemon["nxn_daemon.py <br>(Persistent TCP Socket Daemon)"]
                BigCubeSolver["rubiks-cube-NxNxN-solver <br>(General NxNxN solver)"]
            end
            
            MoveParser["MoveParser Adapter <br>(Advanced wide/slice string moves parsing)"]
        end
    end

    subgraph Camera ["🎥 Camera Module"]
        CameraPlugin["CameraPlugin"]
        Orbit["OrbitCamera Component"]
    end

    subgraph HandTracking ["✋ Hand Tracking (MediaPipe IPC)"]
        HandTrackerLib["hand_tracker (Pure Rust IPC Reader)"]
        HandTrackingPlugin["HandTrackingPlugin"]
        TrackerData["CameraFrameEvent & HandRotationEvent"]
    end

    subgraph Environment ["☀️ Environment Module"]
        EnvPlugin["EnvironmentPlugin"]
        Settings["EnvironmentSettings Resource"]
        LightRig["LightRig (Tracking Camera)"]
    end

    subgraph RubikCore ["🧊 Rubik Module"]
        RubikPlugin["RubikPlugin"]
        RotQueue["RotationQueue Resource"]
        subgraph RubikSystems ["systems/ (Modular Systems)"]
            SysOrch["systems.rs <br>(Module Orchestrator)"]
            subgraph RubikCreation ["creation/ (Cube Initialization)"]
                Creation["creation.rs <br>(Mesh & Materials Spawn)"]
                Voxel["voxel.rs <br>(3D Voxel Letters Geometric Art)"]
            end
            Rotation["rotation.rs <br>(Rotation Queue & Pivot Animate)"]
            Skin["skin.rs <br>(Skins Customization)"]
            Label["label.rs <br>(3D Face Labels Billboard)"]
            Interaction["interaction.rs <br>(RMB Orbit & Reset)"]
        end
    end

    %% Interactions
    UiPlugin -->|Triggers Solve/Steps| SolverPlugin
    UiPlugin -->|Calls solve_cube_for_size| UnifiedEntry
    
    UnifiedEntry -->|1. Scrapes Bevy Entities directly| StateScraping["helpers.rs Scraper"]
    UnifiedEntry -->|2. Fast path (N=3)| Kewb
    UnifiedEntry -->|3. NxN path (N>=4)| NxnDaemon
    NxnDaemon -->|Invokes| BigCubeSolver
    
    Kewb -->|4. Return solution string| MoveParser
    BigCubeSolver -->|4. Return solution string| MoveParser
    
    MoveParser -->|5. Translate to physical RotationMove vec| SolverPlugin
    
    UiPlugin -->|Applies Settings| EnvPlugin
    UiPlugin -->|Updates Skin State| RubikPlugin
    
    InputPlugin -->|Calculates Swipe Axis| RotQueue
    SolverPlugin -->|Pushes Solution Moves| RotQueue
    
    RotQueue -->|Feeds Moves| Rotation
    Rotation -->|Rotates Entities & Reparents| Creation
    
    CameraPlugin -.->|Orbit Reference| InputPlugin
    CameraPlugin -.->|Rig Rotation Target| EnvPlugin
    
    HandTrackerLib -->|Provides frame & delta| HandTrackingPlugin
    HandTrackingPlugin -->|Triggers| RotQueue
    HandTrackingPlugin -->|Sends UI Frames| UiPlugin
```

---

## 🧊 Core ECS Components & Resources

### Key Components

| Component | Description | Location |
|:---|:---|:---|
| `RubikCube` | Marker component for the root transform containing all $N^3$ cubies. | `rubik::components` |
| `Cubie` | Marker attached to individual 3D cubelets (cubies). | `rubik::components` |
| `GridCoord` | Contains logical `IVec3` coords in standard ranges `[0..size-1]`. | `rubik::components` |
| `CubieFace` | Marker component carrying face direction (`Face`), attached to colored meshes. | `rubik::components` |
| `Pivot` | Temporary parent entity spawned during slice rotation animations. | `rubik::components` |
| `TargetRotation` | Target `Quat` to interpolate the pivot transformation smoothly. | `rubik::components` |
| `OrbitCamera` | Orbit configuration (`radius`, `alpha`, `beta`). | `camera::components` |

### Key Resources

| Resource | Description | Location |
|:---|:---|:---|
| `RotationQueue` | FIFO queue (`VecDeque<RotationMove>`) containing upcoming slice rotations. | `rubik::resources` |
| `CurrentlyRotating` | Active state of the animating slice (axis, index, timer progress, entities). | `rubik::resources` |
| `MoveHistory` | Undo/Redo stacks for manual slice turn history. | `rubik::resources` |
| `RubikSkin` | Current active skin selection (`Classic`, `Carbon`, `Geometric`, `Floral`). | `rubik::resources` |
| `EnvironmentSettings` | Sliders-backed settings (clear color, lights, temperature). | `environment::resources` |
| `RubikSize` | Active dimension size of the Rubik's Cube (ranges 2x2x2 up to 12x12x12). | `rubik::resources` |
| `StepByStepSolution` | Current step index and computed move strings. | `solver::resources` |
| `HandTrackingEnabled` | Toggle state for webcam hand gesture controls. | `input::hand_tracking` |

---

## 🔄 Module Breakdown

### 1. Rubik Core Module (`src/rubik`)
Manages structural rendering, mesh hierarchy, animation updates, and spatial mapping.
*   **Decoupled Systems**:
    *   `creation/`: Spawns the central parent root, $N^3$ cubies, color facelets, and indicators.
    *   `creation/voxel.rs`: Geometric 3D voxel letters representing `U`, `D`, `L`, `R`, `F`, `B` labels.
    *   `rotation.rs`: Animates slice rotations smoothly using Pivot entities and handles animation completion.
    *   `skin.rs`: Applies custom textures, patterns, and shaders dynamically on skin changes.
    *   `label.rs`: Dynamically aligns 3D face labels facing the camera (billboard effect).
    *   `interaction.rs`: Implements free 360-degree orbit (RMB) and camera reset events.

### 2. Input & Picking Module (`src/input`)
Processes mouse picking, swipe detection, and MediaPipe-based hand tracking.
*   **Manual Raycasting & Plane Intersection**: Translates viewport screen coordinates into a world-space ray (`viewport_to_world`) to manually compute box intersections without external heavyweight picking engines.
*   **Drag Vector Calculation**: Projects swipe directions onto the active face plane. Cross-product math determines which slice index and axis to rotate.
*   **Hand Tracking (`hand_tracker` crate)**: Reads MediaPipe webcam coordinate streams via Rust-Python IPC. Performs moving-average smoothing (EMA) and dead-zone filtering to eliminate physical micro-jitters without blocking Bevy's main loop.

### 3. UI Module (`src/ui`)
Provides the control HUD, settings sidebars, camera feeds, and modular components.
*   **Modular Architecture**:
    *   `layout/`: Assembles the beautiful glassmorphism-themed UI (left Sidebar, bottom HUD, top overlays).
    *   `interactions/`: The core interaction logic is fully split into decoupled, dedicated submodules:
        *   `solver.rs`: Controls background solver async task polling and steps execution.
        *   `skin.rs`: Handles custom skin customization panels.
        *   `environment.rs`: Real-time light intensities and warm/cool temperature settings.
        *   `size.rs`: Manages slider track dragging and fast increment/decrement size buttons.
        *   `mapping.rs`: Face mapping preferences (U/D/F first priority choices).
        *   `sidebar.rs`: Custom viewport-aware sidebar scrolling and scrollbar dragging.
        *   `camera.rs`: Webcam feed rendering and gesture controls toggle.
        *   `app.rs`: General system integrations (like exit button events).

### 4. Solver Module & Unified Engine (`src/solver` & `rubik_solver`)
Integrates physical 3D ECS entities with background high-performance solvers (Rust 3x3 solver and Python daemon NxN solver).

```
[Bevy 3D Entities] -> [Direct Raycast Scraper] -> [Flat color string representation]
                                                              |
                                  +---------------------------+
                                  |
                                  v
            +--------------------+--------------------+
            | (If size == 3)                          | (If size >= 4)
            v                                         v
     [Pure Rust kewb Crate]                  [Python TCP Daemon]
    (Super-fast optimal 3x3)              (Persistent Socket Connection)
            |                                         |
            +--------------------+--------------------+
                                 |
                                 v
                       [Solution move string]
                                 |
                                 v
                     [Smart Rust MoveParser]
         (Translates wide moves, depths, and slice indices)
                                 |
                                 v
                     [Vec<RotationMove> output]
```

*   **State Scraping**: Iterates over physical Bevy entities (`CubieFace`), projects coordinate positions using 3D normals, and constructs standard flat color string representations dynamically for any size.
*   **Solver Orchestration**:
    *   **3x3 Path (N=3)**: Invokes the pure-Rust `kewb` solver crate, which resolves optimal moves in milliseconds without any external subprocess overhead.
    *   **NxN Path (N>=4)**: Connects to a persistent background Python daemon (`python_solver/nxn_daemon.py`) via local TCP sockets to solve large cubes. This bypasses subprocess startup lag and ensures sub-second response times.
*   **Smart Move Parser Adapter (`rubik_solver::helpers`)**: Parses solved sequence strings and breaks them down into standard physical animations:
    *   *Standard turns* (e.g. `U`, `R'`, `F2`): Rotates the outer face layer.
    *   *Wide turns* (e.g. `Rw`, `3Uw2`): Rotates multiple outer-to-inner layers simultaneously.
    *   *Slice turns* (e.g. `2R`, `3F'`): Rotates a single inner layer indexed from the specified face.

---

## 🧮 Mathematical Model & Transformations

To ensure absolute grid alignment and prevent numerical precision drift over continuous rotations, spatial locations are updated discretely:

### 1. Slice Grid Coordination
On animation completion, cubies are mathematically aligned back to their exact integer grids:

$$\vec{P}_{\text{new}} = \text{round}(R \cdot \vec{P}_{\text{old}})$$

Where $R$ is the $90^\circ$ rotation quaternion (`Quat::from_axis_angle`) and $\vec{P}$ is the logical `GridCoord` vector.

### 2. Relative Face Orientation
Sticker mesh orientations are multiplied to maintain correct relative rotation histories:

$$Q_{\text{new}} = Q_{\text{step}} \cdot Q_{\text{old}}$$
