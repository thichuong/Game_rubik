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
        Buttons["Shuffle / Solve / Steps / Settings Controls"]
    end

    subgraph Input ["🖱️ Input Module"]
        InputPlugin["InputPlugin"]
        Raycast["Manual Raycasting & Plane Intersect"]
        DragState["DragState Resource"]
    end

    subgraph Solver ["🧠 Solver Module"]
        SolverPlugin["SolverPlugin"]
        Kewb["kewb Crate (Kociemba 2-Phase)"]
        StepByStep["StepByStepSolution Resource"]
    end

    subgraph Camera ["🎥 Camera Module"]
        CameraPlugin["CameraPlugin"]
        Orbit["OrbitCamera Component"]
    end

    subgraph Environment ["☀️ Environment Module"]
        EnvPlugin["EnvironmentPlugin"]
        Settings["EnvironmentSettings Resource"]
        LightRig["LightRig (Tracking Camera)"]
    end

    subgraph RubikCore ["🧊 Rubik Module"]
        RubikPlugin["RubikPlugin"]
        RotQueue["RotationQueue Resource"]
        PivotAnimate["Pivot Animation System"]
        GridCoord["GridCoord & Entity Re-parenting"]
    end

    %% Interactions
    UiPlugin -->|Triggers Solve/Steps| SolverPlugin
    UiPlugin -->|Applies Settings| EnvPlugin
    UiPlugin -->|Updates Skin State| RubikPlugin
    
    InputPlugin -->|Calculates Swipe Axis| RotQueue
    SolverPlugin -->|Pushes Solution Moves| RotQueue
    
    RotQueue -->|Feeds Moves| PivotAnimate
    PivotAnimate -->|Rotates Entities| GridCoord
    
    CameraPlugin -.->|Orbit Reference| InputPlugin
    CameraPlugin -.->|Rig Rotation Target| EnvPlugin
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
| `StepByStepSolution` | Current step index and calculated move strings generated from the automated solver. | `solver::resources` |

---

## 🔄 Module Breakdown

### 1. Rubik Core Module (`src/rubik`)
Manages structural rendering, mesh hierarchy, animation updates, and logical spatial tracking.
*   **Cubie Generation**: Grid coordinate iteration spawns 27 cubies inside a central parent root entity. Colored stickers are spawned as children of the respective cubie transforms.
*   **Decoupled Slice Animation**:
    1.  When a slice move starts, a temporary `Pivot` entity is spawned at the center.
    2.  Affected cubies (sharing the target axis coordinate) are reparented to the `Pivot`.
    3.  A system interpolates the `Pivot`’s rotation `Quat` using an eased timer.
    4.  Once the animation finishes, the new positions and orientations are calculated, the cubies are reparented back to the root, and the `Pivot` is despawned.

### 2. Input & Picking Module (`src/input`)
Handles standard mouse clicking, camera control interception, and dragging gestures.
*   **Manual Raycasting**: Decoupled from massive heavy picking libraries, the system manually translates the viewport cursor screen coordinates into a world-space ray utilizing Bevy's camera transform (`viewport_to_world`).
*   **Plane Intersection**:
    *   Iterates through face entities, performing plane intersection.
    *   Finds the nearest facelet bounded to a `0.5` radius box.
    *   Stores hit coordinates on mouse drag initialization (`DragState`).
*   **Drag Vector Calculation**: On cursor release, projects the current cursor ray onto the plane of the initially clicked face. The resulting swipe vector dictates the orientation cross-product to determine which 3D axis is rotated.
*   **Center Protection constraint**: Rotations with `index == 0` (center slices) are explicitly blocked from mouse interaction to prevent axis-shifting disorientation, keeping controls extremely intuitive.

### 3. Solver Module (`src/solver`)
Bridges the physical 3D scene representation to the abstract mathematical two-phase algorithm.
*   **3D-to-Facelet State Mapping**:
    *   Maps each of the 6 core faces using orthogonal vector combinations (e.g. face normal vector, a right vector, and a down vector).
    *   Iterates through the 9 positions of each face.
    *   For each position, it searches for a cubie sticker whose global transform aligns with that exact spatial coordinate.
    *   Extracts the logical color (`Face`) and maps it to a standard 54-char string representation (`U...R...F...D...L...B`).
*   **Solver Interface**: Passes the facelet string into the `kewb` library which runs Kociemba's two-phase algorithm.
*   **Solution Pipeline**: Converts generated steps (e.g. `R2 U' F`) into sequenced `RotationMove` structs and queues them directly inside `RotationQueue`.

### 4. Camera & Environment Module (`src/camera` & `src/environment`)
Creates a high-end visual experience.
*   **Orbit Camera**: Tracks mouse movement when holding `Right-click`, modifying camera angles smoothly.
*   **Studio Light Tracking**: Features a dynamic lighting rig containing main, fill, and rim lights. The rig's rotation is updated relative to the camera vector, guaranteeing uniform illumination at any viewing angle.
*   **Reflection Plane & Shadows**: Renders a floor mesh configured to receive crisp soft shadows cast by the cube.

### 5. UI Module (`src/ui`)
Xử lý all widgets including modular control buttons, skins selection, and dynamic configurations.
*   Built entirely in Bevy native UI nodes.
*   Resolves rendering quality through vector icons utilizing SVG to PNG rasterization with the `bevy_resvg` framework.
*   Integrates interactive state triggers for background colors, lighting parameters, and active step-by-step indexes.

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
