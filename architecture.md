# 🏗️ Rubik's Cube ECS - Architectural Design Document

This document outlines the architectural patterns, entity relationships, and ECS (Entity Component System) design choices implemented in the 3D Rubik's Cube game using **Bevy Engine (v0.18)** and **Rust**. It details the high-performance unified solving engine which bridges Bevy's physical entities with specialized Python background solvers (**kociemba** and **rubiks-cube-NxNxN-solver**) for 100% solving success rates on all dimensions.

---

## 🧭 System Architecture Overview

The application is structured into decoupled, specialized modules, each registered as a self-contained Bevy `Plugin`. This modularity ensures a high degree of maintainability, isolating graphics rendering, input handling, UI design, and background mathematical solving.

```mermaid
graph TD
    %% Module Plugins
    subgraph UI ["🎨 UI Module"]
        UiPlugin["UiPlugin"]
        Components["components.rs <br>(Marker Components)"]
        Interactions["interactions.rs <br>(Interactions & Input Sync)"]
        subgraph Layout ["layout/ (Modular Layout)"]
            LayoutMod["layout.rs <br>(setup_ui Orchestrator)"]
            Sidebar["sidebar.rs <br>(Left Sidebar Layout)"]
            Env["environment.rs <br>(3D Environment Controls)"]
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
        
        subgraph RubikSolverLib ["📚 rubik_solver Crate (Python Solver Bridge)"]
            UnifiedEntry["solve_cube_for_size <br>(Unified Entry Point)"]
            
            subgraph PythonSolvers ["🐍 Python Solver Background Worker"]
                Kociemba["python3 -c import kociemba <br>(Fast 3x3x3 solver)"]
                BigCubeSolver["rubiks-cube-solver.py <br>(General NxNxN solver)"]
            end
            
            NxNState["NxNState <br>(Bevy Entity Scraper & State Rep)"]
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
    
    UnifiedEntry -->|1. Scraping Bevy Entities| NxNState
    UnifiedEntry -->|2. Fast path (N=3)| Kociemba
    UnifiedEntry -->|3. General path (N!=3)| BigCubeSolver
    
    Kociemba -->|4. Return solution string| MoveParser
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

## 🧊 Core ECS Component & Resource Registry

The core design patterns of this application are represented cleanly in its components and resources.

### Key Components

| Component | Description | Location |
|:---|:---|:---|
| `RubikCube` | Marker component for the root transform containing all $N^3$ cubies. | `rubik::components` |
| `Cubie` | Marker attached to individual 3D cubelets. | `rubik::components` |
| `GridCoord` | Contains logical `IVec3` coords in standard ranges `[0..size-1]`. Used for state calculations. | `rubik::components` |
| `CubieFace` | Marker component containing a face direction (`Face`), attached to the individual 6-colored meshes of each cubie. | `rubik::components` |
| `Pivot` | Temporary parent entity spawned during slice rotation animations to rotate grouped cubies. | `rubik::components` |
| `TargetRotation` | Component carrying the target `Quat` to interpolate the pivot transformation smoothly. | `rubik::components` |
| `OrbitCamera` | Component carrying camera sphere orientation variables (`radius`, `alpha`, `beta`). | `camera::components` |

### Key Resources

| Resource | Description | Location |
|:---|:---|:---|
| `RotationQueue` | FIFO queue (`VecDeque<RotationMove>`) containing upcoming slice rotations. Decouples input/solver from animations. | `rubik::resources` |
| `CurrentlyRotating` | Active state of the animating slice (axis, index, timer progress, affected entity IDs). | `rubik::resources` |
| `MoveHistory` | Undo/Redo stacks (`done` and `undone` arrays) for keyboard shortcut commands (`Ctrl+Z`, `Ctrl+Y`). | `rubik::resources` |
| `RubikMaterials` | Standard matte materials for faces and loaded skin texture handles. | `rubik::resources` |
| `RubikSkin` | Current active skin selection (`Classic`, `Carbon`, `Geometric`, `Floral`). | `rubik::resources` |
| `EnvironmentSettings` | Real-time settings matching sliders (clear color, light intensities, warm/cool temperature). | `environment::resources` |
| `RubikSize` | Active dimension size of the Rubik's Cube (ranging from 2x2x2 up to 11x11x11, etc.). | `rubik::resources` |
| `StepByStepSolution` | Current step index and calculated move strings generated from the automated solver. | `solver::resources` |
| `HandTrackingEnabled` | Boolean toggle state for whether camera hand gestures should rotate the cube. | `input::hand_tracking` |

---

## 🔄 Module Breakdown

### 1. Rubik Core Module (`src/rubik`)
Manages structural rendering, mesh hierarchy, animation updates, and logical spatial tracking. Refactored from a large monolithic codebase into highly decoupled submodules under strict **Clippy standards**.
*   **Modular Architecture**:
    *   [mod.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/mod.rs): Registers `RubikPlugin` and handles resource initialization.
    *   [systems.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems.rs): The module entrypoint coordinating and re-exporting the systems.
    *   [systems/creation.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/creation.rs): Spawns the central parent root, $N^3$ cubies, colored facelets, and indicators.
    *   [systems/creation/voxel.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/creation/voxel.rs): Contains geometric 3D voxel coordinates representing letters (`U`, `D`, `L`, `R`, `F`, `B`) and face color mappings.
    *   [systems/rotation.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/rotation.rs): Manages the FIFO rotation queue and handles slice-rotation logic using Pivot entities.
    *   [systems/skin.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/skin.rs): Applies custom textures or patterns dynamically.
    *   [systems/label.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/label.rs): Matches the camera rotation to keep 3D face labels screen-aligned (billboard).
    *   [systems/interaction.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/interaction.rs): Implements free 360-degree rotation (RMB) and orientation reset events.

### 2. Input & Picking Module (`src/input`)
Handles standard mouse clicking, camera control interception, dragging gestures, and **Camera Hand Tracking**.
*   **Manual Raycasting**: Decoupled from massive heavy picking libraries, the system manually translates the viewport cursor screen coordinates into a world-space ray utilizing Bevy's camera transform (`viewport_to_world`).
*   **Plane Intersection**: Finds the nearest facelet bounded to a `0.5` radius box and stores hit coordinates on mouse drag initialization (`DragState`).
*   **Drag Vector Calculation**: Projects the current cursor ray onto the plane of the initially clicked face. The resulting swipe vector dictates the orientation cross-product to determine which 3D axis is rotated.
*   **Camera Hand Tracking (`hand_tracker` & `hand_tracking.rs`)**:
    *   Utilizes a dedicated workspace crate (`hand_tracker`) communicating with a lightweight background Python subprocess running **Google MediaPipe Hands**.
    *   Landmarks data is skipped during packet parsing to avoid heap allocations in the game loop.
    *   Calculates moving average smoothing using an EMA filter (`alpha = 0.65`) and filters micro-jitters with a dead-zone threshold (`dead_zone = 2.0`).
    *   Communicates via `std::sync::mpsc::channel` and updates the UI Camera view dynamically without blocking the main render loop.

### 3. Solver Module (`src/solver`)
Bridges the physical 3D Bevy representation with the automated Python solvers via the `rubik_solver` library.
*   **Unified Solver Interface**:
    *   The `rubik_solver` library exposes `solve_cube_for_size` as a single unified entry point for all supported sizes.
    *   The Bevy system `handle_solve_button` invokes this library function, completely removing individual dimension checks and state-mapping boilerplate from the ECS systems layer.
*   **3D-to-Facelet State Mapping**:
    *   Queries `GlobalTransform` and `CubieFace` components.
    *   Utilizes orthogonal vector configurations (`FaceMapping`) to project normal, right, and down vectors for each of the 6 core faces.
    *   Searches for the closest physical Bevy entity representing each virtual facelet location using geometric position mapping, feeding the result into the logical solver state (`NxNState`).

---

## 🐍 Unified Python Solver Integration

For any Rubik's Cube of size $N$, the Rust crate `rubik_solver` acts as a bridge executing specialized Python subprocesses.

```
[Bevy 3D Entities] -> [NxNState Scraping] -> [to_string_rep()] -> [Python Subprocess Launcher]
                                                                        |
                                  +-------------------------------------+
                                  |
                                  v
             +--------------------+--------------------+
             | (If size == 3)                          | (If size != 3)
             v                                         v
     [Python Kociemba]                       [rubiks-cube-solver.py]
  (Super-fast milliseconds solve)            (Unified NxN Reduction Solver)
             |                                         |
             +--------------------+--------------------+
                                  |
                                  v
                        [Solution stdout string]
                                  |
                                  v
                      [Smart Rust MoveParser]
       (Parses standard moves, wide layers, and slice indices)
                                  |
                                  v
                      [Vec<RotationMove> output]
```

### 1. State Scraping and standard notation representation
The game uses `NxNState` inside `rubik_solver` to represent all $6 \times N^2$ facelets. The function `to_string_rep()` constructs a flat string representing the exact layout ordered by the standard Kociemba faces: **Up, Right, Front, Down, Left, Back**.

### 2. Fast-Path Kociemba Solver (Size = 3x3x3)
For standard 3x3x3 cubes, calling heavy reduction lookup scripts is redundant. The solver runs a fast inline Python statement invoking `kociemba.solve('<state>')`. This leverages Kociemba's compiled C-modules, yielding the shortest move sequence in under 5 milliseconds.

### 3. General Big-Cube Solver (Size != 3x3x3)
For other sizes (like 2x2x2, 4x4x4, 5x5x5, etc.), the crate spawns a child process running `rubiks-cube-solver.py` inside `python_solver/rubiks-cube-NxNxN-solver/`.
The script is launched with its working directory set to its library folder to ensure optimal resource and asset lookup (`lookup-tables/`).

### 4. Smart Move Parser Adapter
Since `rubiks-cube-NxNxN-solver` prints solution sequences containing specialized big-cube notation (such as wide moves and slice indices), we implement a smart parser in `rubik_solver/src/helpers.rs` to break them down into standard physical rotation steps:

- **Standard Face Turns** (e.g. `U`, `R'`, `F2`):
  Xoay lớp ngoài cùng của mặt tương ứng. Được ánh xạ sang `RotationMove` qua cấu hình `FaceMapping` của game.
- **Wide Turns / Multi-layer rotations** (e.g. `Rw`, `3Uw2`, `Lw'`):
  Đại diện cho việc xoay đồng thời nhiều lớp từ ngoài vào trong. Cú pháp tổng quát: `<depth><Face>w[modifiers]` (nếu không có `<depth>`, mặc định là 2).
  Bộ phân tích của Rust sẽ tính toán `depth` lớp ngoài cùng tương ứng trên trục quay và tạo ra danh sách nhiều thực thể `RotationMove` xoay đồng thời cùng một chiều để Bevy thực thi.
- **Slice Turns / Single inner layer rotations** (e.g. `2R`, `3F'`):
  Đại diện cho việc xoay duy nhất lớp thứ `index` nằm ở giữa tính từ mặt chỉ định. Cú pháp tổng quát: `<index><Face>[modifiers]`.
  Bộ phân tích của Rust sẽ tìm chính xác lớp thứ `index` (0-indexed từ ngoài vào trong) và tạo duy nhất 1 `RotationMove` tác động lên lớp đó.

---

## 🧮 Mathematical Model & Transformations

To ensure consistent grid alignment throughout continuous rotations, the system updates entity coordinates mathematically using the following algorithms:

### 1. Slice Grid Coordination
Since 3D floating-point rotations accumulate precision errors over time, spatial locations are updated discretely on animation completion:

$$\vec{P}_{\text{new}} = \text{round}(R \cdot \vec{P}_{\text{old}})$$

Where $R$ is the $90^\circ$ rotation quaternion (`Quat::from_axis_angle`) and $\vec{P}$ is the logical `GridCoord` vector. Rounding guarantees that the coordinates are kept as precise integers (`-1`, `0`, or `1` for 3x3x3; or `0` to `size-1` for $N\times N\times N$).

### 2. Relative Face Orientation
Sticker rotation is applied dynamically to the individual mesh transformation matrices to ensure textures, skins, and face labels look visually authentic:

$$Q_{\text{new}} = Q_{\text{step}} \cdot Q_{\text{old}}$$

This allows children meshes to preserve their relative rotation histories correctly without drifting.
