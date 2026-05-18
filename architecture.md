# 🏗️ Rubik's Cube ECS - Architectural Design Document

This document outlines the architectural patterns, math models, entity relationships, and ECS (Entity Component System) design choices implemented in the 3D Rubik's Cube game using **Bevy Engine (v0.18)** and **Rust**.

---

## 🧭 System Architecture Overview

The application is structured into decoupled, specialized modules, each registered as a self-contained Bevy `Plugin`. This modularity ensures a high degree of maintainability, isolating graphics rendering, input handling, and mathematical solving.

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

    subgraph Solver ["🧠 Solver Module"]
        SolverPlugin["SolverPlugin"]
        SolveCubeForSize["solve_cube_for_size<br>(Unified Library Entry Point)"]
        Kewb["kewb Crate (Kociemba 2-Phase)"]
        StepByStep["StepByStepSolution Resource"]
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
    UiPlugin -->|Calls solve_cube_for_size| SolveCubeForSize
    SolveCubeForSize -->|Invokes| Kewb
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
| `RubikCube` | Marker component for the root transform containing all 27 cubies. | `rubik::components` |
| `Cubie` | Marker attached to individual 3D cubelets. | `rubik::components` |
| `GridCoord` | Contains logical `IVec3` coords in standard ranges `[-1, 0, 1]`. Used for state calculations. | `rubik::components` |
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
    *   [systems/creation.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/creation.rs): Spawns the central parent root, 27 cubies, colored facelets, and indicators.
    *   [systems/creation/voxel.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/creation/voxel.rs): Contains geometric 3D voxel coordinates representing letters (`U`, `D`, `L`, `R`, `F`, `B`) and face color mappings.
    *   [systems/rotation.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/rotation.rs): Manages the FIFO rotation queue and handles slice-rotation logic using Pivot entities.
    *   [systems/skin.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/skin.rs): Applies custom textures or patterns dynamically.
    *   [systems/label.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/label.rs): Matches the camera rotation to keep 3D face labels screen-aligned (billboard).
    *   [systems/interaction.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/rubik/systems/interaction.rs): Implements free 360-degree rotation (RMB) and orientation reset events.
*   **Cubie Generation**: Grid coordinate iteration spawns 27 cubies inside a central parent root entity. Colored stickers are spawned as children of the respective cubie transforms.
*   **Decoupled Slice Animation**:
    1.  When a slice move starts, a temporary `Pivot` entity is spawned at the center.
    2.  Affected cubies (sharing the target axis coordinate) are reparented to the `Pivot`.
    3.  A system interpolates the `Pivot`’s rotation `Quat` using an eased timer.
    4.  Once the animation finishes, the new positions and orientations are calculated, the cubies are reparented back to the root, and the `Pivot` is despawned.

### 2. Input & Picking Module (`src/input`)
Handles standard mouse clicking, camera control interception, dragging gestures, and **Camera Hand Tracking**.
*   **Manual Raycasting**: Decoupled from massive heavy picking libraries, the system manually translates the viewport cursor screen coordinates into a world-space ray utilizing Bevy's camera transform (`viewport_to_world`).
*   **Plane Intersection**:
    *   Iterates through face entities, performing plane intersection.
    *   Finds the nearest facelet bounded to a `0.5` radius box.
    *   Stores hit coordinates on mouse drag initialization (`DragState`).
*   **Drag Vector Calculation**: On cursor release, projects the current cursor ray onto the plane of the initially clicked face. The resulting swipe vector dictates the orientation cross-product to determine which 3D axis is rotated.
*   **Center Protection constraint**: Rotations with `index == 0` (center slices) are explicitly blocked from mouse interaction to prevent axis-shifting disorientation, keeping controls extremely intuitive.
*   **Camera Hand Tracking (`hand_tracker` & `hand_tracking.rs`)**:
    *   Utilizes a dedicated workspace crate (`hand_tracker`) communicating with a lightweight background Python subprocess via standard I/O streams (`std::process::Child` piping). This eliminates complex OpenCV C++ bindings, making compilation fast and robust.
    *   The Python worker runs **Google MediaPipe Hands** to detect 21 3D joint landmarks and perform ultra-fast gesture classification (Gesture 1: Open Hand for cube rotation, Gesture 2: Index Extended for hover selection, Gesture 3: Index Folded for swipe face rotation).
    *   In the Rust host side, landmarks data is skipped during packet parsing to avoid unnecessary heap allocations in the game loop.
    *   Calculates moving average smoothing using an EMA filter (`alpha = 0.65`) and filters micro-jitters with a dead-zone threshold (`dead_zone = 2.0`).
    *   Runs seamlessly on a separate background thread communicating via `std::sync::mpsc::channel`.
    *   Pushes `HandRotationEvent` mapping 2D hand movement deltas to 3D cube rotations.
    *   Pushes `CameraFrameEvent` carrying RGBA byte arrays to update the UI Camera view dynamically without blocking the main render loop.
    *   Employs a **Drain Channel** technique in the Bevy main thread to process only the latest tracking packet, completely eliminating I/O latency.

### 3. Solver Module (`src/solver`)
Bridges the physical 3D scene representation to the abstract mathematical two-phase algorithm.
*   **Unified Solver Interface**:
    *   The `rubik_solver` library exposes `solve_cube_for_size` as a single unified entry point for all supported sizes (2x2x2 and 3x3x3).
    *   The Bevy system `handle_solve_button` invokes only this library function, completely removing individual dimension checks and state-mapping boilerplate from the ECS systems layer.
*   **3D-to-Facelet State Mapping**:
    *   Maps each of the 6 core faces using orthogonal vector combinations (e.g. face normal vector, a right vector, and a down vector).
    *   Iterates through the 9 positions of each face. For a 2x2x2 cube, it dynamically maps corners into a virtual 3x3x3 facelet representation.
    *   For each position, it searches for a cubie sticker whose global transform aligns with that exact spatial coordinate.
    *   Extracts the logical color (`Face`) and maps it to a standard 54-char string representation (`U...R...F...D...L...B`).
*   **Solver Interface**: Passes the generated facelet string into the Kociemba 2-phase solver (`kewb` library).
*   **Solution Pipeline**: Converts generated steps (e.g. `R2 U' F`) into sequenced `RotationMove` structs and queues them directly inside `RotationQueue`.

### 4. Camera & Environment Module (`src/camera` & `src/environment`)
Creates a high-end visual experience.
*   **Orbit Camera**: Tracks mouse movement when holding `Right-click`, modifying camera angles smoothly.
*   **Studio Light Tracking**: Features a dynamic lighting rig containing main, fill, and rim lights. The rig's rotation is updated relative to the camera vector, guaranteeing uniform illumination at any viewing angle.
*   **Reflection Plane & Shadows**: Renders a floor mesh configured to receive crisp soft shadows cast by the cube.

### 5. UI Module (`src/ui`)
Manages the graphical interface, widgets, customize sidebar, and step-by-step guidance HUD. Refactored from a monolithic codebase into a highly modular, decoupled structure satisfying strict **Clippy clean standards** (0 warnings, without using any `#[allow(clippy::too_many_lines)]` bypasses).
*   **Modular Architecture**:
    *   [mod.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/ui/mod.rs): Registers `UiPlugin` and sets up the resource flows.
    *   [components.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/ui/components.rs): Houses all ECS Marker Components (e.g. `CloseButton`, `ShuffleButton`, `EnvControl`) used to query and identify UI elements.
    *   [interactions.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/ui/interactions.rs): Contains systems that respond to user interactions (Hover, Click) by mutating internal resources (e.g., trigger solve pipeline, update background clear colors, or toggle skin materials).
    *   [layout.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/ui/layout.rs): The main setup entrypoint containing the `setup_ui` orchestrator which loads assets and builds the outer UI frame.
    *   [layout/sidebar.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/ui/layout/sidebar.rs): Spawns the modular Left Sidebar (including logo headers, basic button layouts, and collapsible skin options).
    *   [layout/environment.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/ui/layout/environment.rs): Spawns detailed panels for real-time 3D lighting, temperature presets, angle controls, and scene backdrops.
    *   [layout/hud.rs](file:///home/tchuong/M%C3%A0n%20h%C3%ACnh%20n%E1%BB%81n/Game_rubik/src/ui/layout/hud.rs): Spawns the bottom-anchored step-by-step guidance dashboard.
*   **Vector Rendering Excellence**: Uses the `bevy_resvg` framework to seamlessly render crystal-clear `.svg` vector icons into pixel-perfect Bevy UI meshes at runtime.
*   **Interactive Size Slider (Bevy 0.18+ Best Practices)**:
    *   **Invisible Picking Overlay**: In Bevy 0.18+, fully transparent nodes (`BackgroundColor(Color::NONE)`) are ignored by the picking engine. To create a highly responsive slider, the track implements a 1% opacity color overlay (`Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.01))`) rendered on top of the visuals to capture mouse drag gestures.
    *   **UI Coordinate Hierarchy (`UiGlobalTransform`)**: Bevy 0.18+ completely decouples UI rendering from 3D camera matrices by using `UiGlobalTransform` instead of standard `GlobalTransform`. The dragging calculations query `UiGlobalTransform` to resolve the layout center coordinates (`transform.translation.x`) dynamically and map cursor movements to percentages.
*   **Zero-Warning Clean Code**: Highly optimized visual spawning functions are aggressively divided into dedicated sub-functions (~30 to 60 lines each) to guarantee maximum code readability and strict Clippy pedantic compliance.

---

## 🧮 Mathematical Model & Transformations

To ensure consistent grid alignment throughout continuous rotations, the system updates entity coordinates mathematically using the following algorithms:

### 1. Slice Grid Coordination
Since 3D floating-point rotations accumulate precision errors over time, spatial locations are updated discretely on animation completion:

$$\vec{P}_{\text{new}} = \text{round}(R \cdot \vec{P}_{\text{old}})$$

Where $R$ is the $90^\circ$ rotation quaternion (`Quat::from_axis_angle`) and $\vec{P}$ is the logical `GridCoord` vector. Rounding guarantees that the coordinates are kept as precise integers (`-1`, `0`, or `1`).

### 2. Relative Face Orientation
Sticker rotation is applied dynamically to the individual mesh transformation matrices to ensure textures, skins, and face labels look visually authentic:

$$Q_{\text{new}} = Q_{\text{step}} \cdot Q_{\text{old}}$$

This allows children meshes to preserve their relative rotation histories correctly without drifting.
