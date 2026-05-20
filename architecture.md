# 🏗️ Rubik's Cube ECS - Architectural Design Document

This document outlines the architectural patterns, mathematical models, entity relationships, and ECS (Entity Component System) design choices implemented in the 3D Rubik's Cube game using **Bevy Engine (v0.18)** and **Rust**. It details the robust NxN modular solving engine ($N \ge 2$) which achieves a 100% verification success rate for up to 5x5x5 cubes and beyond.

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

    subgraph SolverModule ["🧠 Solver Module & Engine"]
        SolverPlugin["SolverPlugin"]
        StepByStep["StepByStepSolution Resource"]
        
        subgraph RubikSolverLib ["📚 rubik_solver Crate (NxN Engine)"]
            UnifiedEntry["solve_cube_for_size <br>(Unified Entry Point)"]
            NxNSolver["solve_nxn (N >= 4) <br>(Reduction Controller)"]
            
            subgraph NxNComponents ["🧩 NxN Reduction Components"]
                NxNState["NxNState <br>(Virtual State Representation)"]
                CenterSolver["solve_centers <br>(BFS Commutator Engine)"]
                EdgeSolver["pair_edges <br>(BFS Wing Pairing & Free Swap)"]
                L2ERecovery["L2E Parity Recovery <br>(Local OLL Parity)"]
                ParityCheck["Parity Correction <br>(OLL / PLL Multi-Combo Check)"]
            end
            
            KewbSolver["kewb Crate <br>(Kociemba 2-Phase 3x3)"]
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
    
    UnifiedEntry -->|Delegates (N >= 4)| NxNSolver
    UnifiedEntry -.->|Delegates (N < 4)| KewbSolver
    
    NxNSolver -->|1. Init & Sync Bevy| NxNState
    NxNSolver -->|2. Position BFS Centers| CenterSolver
    NxNSolver -->|3. Pair Wings| EdgeSolver
    NxNSolver -->|4. Resolve Parity| L2ERecovery
    NxNSolver -->|5. Verify & Correct| ParityCheck
    ParityCheck -->|6. Solve Core 3x3| KewbSolver
    
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
*   **Cubie Generation**: Grid coordinate iteration spawns $N^3$ cubies inside a central parent root entity. Colored stickers are spawned as children of the respective cubie transforms.
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
Bridges the physical 3D Bevy representation with the abstract mathematical modular solving engine (`rubik_solver` library).
*   **Unified Solver Interface**:
    *   The `rubik_solver` library exposes `solve_cube_for_size` as a single unified entry point for all supported sizes (from 2x2x2 up to NxNxN).
    *   The Bevy system `handle_solve_button` invokes this library function, completely removing individual dimension checks and state-mapping boilerplate from the ECS systems layer.
*   **3D-to-Facelet State Mapping**:
    *   Queries `GlobalTransform` and `CubieFace` components.
    *   Utilizes orthogonal vector configurations (`FaceMapping`) to project normal, right, and down vectors for each of the 6 core faces.
    *   Searches for the closest physical Bevy entity representing each virtual facelet location using geometric position mapping, feeding the result into the logical solver state (`NxNState`).

### 4. Camera & Environment Module (`src/camera` & `src/environment`)
Creates a high-end visual experience.
*   **Orbit Camera**: Tracks mouse movement when holding `Right-click`, modifying camera angles smoothly.
*   **Studio Light Tracking**: Features a dynamic lighting rig containing main, fill, and rim lights. The rig's rotation is updated relative to the camera vector, guaranteeing uniform illumination at any viewing angle.
*   **Reflection Plane & Shadows**: Renders a floor mesh configured to receive crisp soft shadows cast by the cube.

### 5. UI Module (`src/ui`)
Manages the graphical interface, widgets, customize sidebar, and step-by-step guidance HUD. Refactored from a monolithic codebase into a highly modular, decoupled structure satisfying strict **Clippy clean standards** (0 warnings, without using any `#[allow(clippy::too_many_lines)]` bypasses).
*   **Vector Rendering Excellence**: Uses the `bevy_resvg` framework to seamlessly render crystal-clear `.svg` vector icons into pixel-perfect Bevy UI meshes at runtime.
*   **Interactive Size Slider (Bevy 0.18+ Best Practices)**:
    *   **Invisible Picking Overlay**: In Bevy 0.18+, fully transparent nodes (`BackgroundColor(Color::NONE)`) are ignored by the picking engine. To create a highly responsive slider, the track implements a 1% opacity color overlay (`Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.01))`) rendered on top of the visuals to capture mouse drag gestures.
    *   **UI Coordinate Hierarchy (`UiGlobalTransform`)**: Bevy 0.18+ completely decouples UI rendering from 3D camera matrices by using `UiGlobalTransform` instead of standard `GlobalTransform`. The dragging calculations query `UiGlobalTransform` to resolve the layout center coordinates (`transform.translation.x`) dynamically and map cursor movements to percentages.
*   **Zero-Warning Clean Code**: Highly optimized visual spawning functions are aggressively divided into dedicated sub-functions (~30 to 60 lines each) to guarantee maximum code readability and strict Clippy pedantic compliance.

---

## 🧮 Advanced Mathematical Solving (NxN Reduction Engine)

For Rubik's Cubes of size $N \ge 4$, the mathematical complexity grows exponentially. Our `rubik_solver` implements a fully autonomous **Reduction Method** divided into distinct algorithmic stages:

```
[Bevy 3D Entities] -> [NxNState Scraping]
                             |
                             v
                    [1. Solve Centers]  <-- Setup + Commutator BFS Pathfinding
                             |
                             v
                    [2. Pair Edge Wings] <-- Setup + Edge-Flip BFS + Swap Slot
                             |
                             v
                    [3. Parity Resolution] <-- L2E local OLL Parity
                             |
                             v
                    [4. Solvability Check] <-- Try 4 OLL/PLL combinations
                             |
                             v
                    [5. Solve Core 3x3]  <-- Kociemba 2-Phase (kewb Crate)
                             |
                             v
                    [6. Move Optimization] <-- Merge redundant physical rotations
```

### 1. Virtual State Representation (`NxNState`)
To isolate complex pathfinding from graphics bottlenecks, all operations occur within a lightweight virtual state `NxNState`. It models $6 \times N^2$ facelets, tracking their logical 3D grid coordinates ($\vec{P} \in [0, N-1]^3$), normal orientations, and colors.
Applying physical moves onto the virtual state is implemented geometrically using quaternion math:

$$\vec{P}_{\text{new}} = \text{round}\left( \mathbf{Q} \cdot \left( \vec{P}_{\text{old}} - \vec{O} \right) + \vec{O} \right)$$

Where $\vec{O} = \frac{N - 1}{2} \cdot \vec{1}$ represents the cube center offset, and $\mathbf{Q}$ is the rotation quaternion (`Quat::from_axis_angle`) representing the $90^\circ$ physical slice rotation. Rounding back to integer coordinates preserves perfect grid alignment without accumulating floating-point drift.

### 2. BFS Center Solver with Commutators
Center pieces (which do not reside on the edges or corners) are solved face-by-face in an optimized sequence: **Left, Right, Down, Back, Up, Front**.
For each target center piece, the solver:
1. Performs a **Virtual Cube Rotation** so that the target face temporarily aligns with the `Up` face. It tests 4 different Y-axis orientations to find an orientation where the commutator's buffer piece does not clash with already solved centers.
2. Uses a **Breadth-First Search (BFS)** to discover setup moves ($S$) constructed from outer face turns and inner slice moves.
3. Applies a mathematical **Commutator** ($C$) designed to cycle exactly three center pieces (e.g., $F \rightarrow U \rightarrow$ Buffer $\rightarrow F$). We define different commutators for edge centers and corner centers:
   * **Left Corner Commutator**: $Lw' \cdot U' \cdot Lw \cdot U \cdot Lw' \cdot U' \cdot Lw$
   * **Right Corner Commutator**: $Rw \cdot U \cdot Rw' \cdot U' \cdot Rw \cdot U \cdot Rw'$
   * **Edge Center Commutator (Odd-sized)**: $Fw \cdot U \cdot Fw' \cdot U' \cdot Fw \cdot U \cdot Fw'$
4. Undo setup moves ($S^{-1}$). The full move sequence is:

$$\text{Moves} = S \cdot C \cdot S^{-1}$$

To guarantee that already solved center pieces remain untouched, the BFS rejects any setup moves that place a solved center piece into the active cycle or the buffer piece, represented in code by the `preserve_coords` filter.

### 3. Edge Pairing with Free Swap Slots
Edge wing pieces are paired so that all winglets along a composite edge share the same two-color combination.
1. The solver loops through all unpaired composite edges, matching winglets at coordinate $idx$ against a reference winglet (`target_colors`).
2. A BFS Setup pathfinder searches for outer face moves ($S$) to place the destination winglet at **Front-Right (FR)** and the source winglet at **Front-Left (FL)** on the same horizontal level (`slice_idx`).
3. Depending on the winglet orientation, the solver executes one of two algorithms:
   * **Parallel / Matching Colors**: Requires flipping the Front-Right edge first ($F$), then slicing, flipping, and slicing back ($S_y \cdot F \cdot S_y^{-1}$).
   * **Skewed / Opposite Colors**: Performs a direct standard edge-pairing slice and flip ($S_y \cdot F \cdot S_y^{-1}$).
   where the Edge-Flip algorithm is:

$$\text{Flip} = R \cdot F' \cdot U \cdot R' \cdot F$$

4. **Free Swap Protection**: To prevent ruining previously paired edges during the slice operations, the solver scans the `Up` and `Down` faces for *unpaired* composite edges. It generates a setup sequence to swap the active top-right slot with an unpaired edge ("Free Swap"), ensuring that when the slice is restored ($S_y^{-1}$), no completed edges are split.
5. If the solver stalls (zero progress), it introduces a random face turn (outer scramble) to break cyclical deadlocks.

### 4. Last Two Edges (L2E) & Local OLL Parity
When only 2 composite edges remain unpaired, it is mathematically impossible to find a third "Free Swap" slot.
1. The solver aligns the last two edges at **FL** and **FR** using a designated L2E setup.
2. It identifies which wing levels (`slice_idx`) are misaligned.
3. It resolves the misalignment by applying a localized **OLL Parity** algorithm targeting only the specific slice:

$$\text{L2E Parity} = Z \cdot \text{OLLParity}(\text{slice\_idx}) \cdot Z^{-1}$$

Where $Z$ is a rotation alignment, and $\text{OLLParity}(y)$ operates only on slice $y$:

$$\text{OLLParity}(y) = Rw_y^2 \cdot B^2 \cdot U^2 \cdot Lw_y \cdot U^2 \cdot Rw_y' \cdot U^2 \cdot Rw_y \cdot U^2 \cdot F^2 \cdot Rw_y \cdot F^2 \cdot Lw_y' \cdot B^2 \cdot Rw_y^2$$

### 5. Parity Verification & Correction
Once all centers are solved and edges are paired, the NxN cube behaves exactly like a 3x3x3 cube. However, because edge wings can be paired upside down, the cube may suffer from **OLL Parity** (one flipped composite edge) or **PLL Parity** (two swapped composite edges), which are mathematically impossible to solve on a standard 3x3x3.
Our solver handles this seamlessly:
1. Maps the `NxNState` into a virtual 54-char 3x3x3 facelet string.
2. Evaluates the 4 mathematical combinations of parity states:
   * **Combo 0**: `(false, false)` - No parity.
   * **Combo 1**: `(true, false)` - OLL Parity only.
   * **Combo 2**: `(false, true)` - PLL Parity only.
   * **Combo 3**: `(true, true)` - OLL + PLL Parity.
3. For each combo, it simulates the parity-flip on the virtual string and uses `kewb::CubieCube::try_from` to check if the resulting 3x3x3 state is mathematically solvable.
4. Once the correct combination is identified, the corresponding physical $N\times N$ parity correction moves are applied:
   * **OLL Parity Correction**:
     $$Rw^2 \cdot B^2 \cdot U^2 \cdot Lw \cdot U^2 \cdot Rw' \cdot U^2 \cdot Rw \cdot U^2 \cdot F^2 \cdot Rw \cdot F^2 \cdot Lw' \cdot B^2 \cdot Rw^2$$
   * **PLL Parity Correction**:
     $$r^2 \cdot U^2 \cdot r^2 \cdot Uw^2 \cdot r^2 \cdot Uw^2$$
5. Finally, the solved virtual state is fed into the Kociemba 2-Phase solver to calculate the final 3x3 core solution.

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
