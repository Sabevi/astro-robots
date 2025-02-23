# 🚀 EREEA: Robot Swarm Exploration Simulation

This project is part of the **"Essaim de Robots pour l'Exploration et l'Étude Astrobiologique (EREEA)"** initiative.  
It simulates a **2D procedural map** where robots will later explore and interact with their environment.

---

## 🏗 **Implemented Features (ZL's Responsibilities)**
For detailed information on ZL's responsibilities, see the [Ziad Lahrouni Responsibility](ziad-lahrouni-responsibility.md) document.

For a detailed breakdown of tasks, see the [Tasks](tasks.md) document.

---

## 🛠 **How to Run the Project**
### 📦 **1. Install Dependencies**
Ensure you have **Rust** installed:
```sh
rustup update
```

### 🚀 **2. Run the Simulation**
To start the map generation and visualization:
```sh
cargo run
```
The map will appear in your terminal!

### 🧪 **3. Run Tests**
To verify the correctness of the map generation:
```sh
cargo test
```

## 🔍 **Project Structure**
```perl
📂 src/
 ├── 📜 main.rs          # Entry point, initializes and renders the map
 ├── 📂 map/
 │    ├── 📜 mod.rs      # Core map logic: generation & obstacle placement
 │    ├── 📜 map_widget.rs # Handles terminal rendering of the map
 ├── 📜 Cargo.toml       # Project dependencies
```
