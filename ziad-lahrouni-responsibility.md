# 🏗 Implemented Features (ZL's Responsibilities)

### ✅ **1. Procedural Map Generation**
- The map is **generated using noise-based terrain generation**.
- It ensures **reproducibility** using a **random seed**.

### ✅ **2. Obstacle Placement**
- Obstacles are placed **procedurally** based on **noise values**.
- A threshold defines which tiles become obstacles.

### ✅ **3. Efficient Data Structure**
- The map is stored as a **1D `Vec<Tile>`**, representing a **2D grid**.
- Each tile is either:
  - **`Tile::Empty`** → Walkable space (`.`)
  - **`Tile::Obstacle`** → Blocks movement (`#`)

### ✅ **4. Adaptive Terminal Rendering**
- The map **automatically resizes** based on the terminal.
- It always fits within **the available space**.
- Obstacles and empty spaces are **color-coded** for better visibility.

### ✅ **5. Unit Testing**
- Tests ensure:
  - The **map is correctly generated**.
  - **Obstacles are present** but do not cover the entire map.
  - **Using the same seed always produces the same map**.
