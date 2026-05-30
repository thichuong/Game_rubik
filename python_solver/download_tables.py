#!/usr/bin/env python3
import sys
import os
import logging

# Configure basic logging to output progress updates directly to stdout
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger("download_tables")

# Retrieve the absolute path of the project root
PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(PROJECT_ROOT, "python_solver/rubiks-cube-NxNxN-solver"))

try:
    from rubikscubennnsolver.RubiksCube444 import RubiksCube444
    from rubikscubennnsolver.RubiksCube555 import RubiksCube555
except ImportError as e:
    logger.error(f"Failed to import rubikscubennnsolver modules: {e}")
    sys.exit(1)

def main():
    logger.info("Initializing pre-download sequence for NxN lookup tables...")

    # Change working directory to resolve LOOKUP_TABLES relative path to the correct folder
    target_cwd = os.path.join(PROJECT_ROOT, "python_solver/rubiks-cube-NxNxN-solver")
    os.chdir(target_cwd)
    logger.info(f"Changed working directory to: {target_cwd}")

    # Define dummy solved states to initialize the cube models
    dummy_state_444 = "U" * 16 + "L" * 16 + "F" * 16 + "R" * 16 + "B" * 16 + "D" * 16
    dummy_state_555 = "U" * 25 + "L" * 25 + "F" * 25 + "R" * 25 + "B" * 25 + "D" * 25

    # 1. Warm-up and pre-download 4x4x4 lookup tables
    logger.info("Initializing 4x4x4 Cube and pre-downloading lookup tables...")
    try:
        cube4 = RubiksCube444(dummy_state_444, "URFDLB")
        cube4.lt_init()
        logger.info("[SUCCESS] 4x4x4 lookup tables pre-downloaded successfully.")
    except Exception as e:
        logger.error(f"Failed to download 4x4x4 lookup tables: {e}")
        sys.exit(1)

    # 2. Warm-up and pre-download 5x5x5 lookup tables
    logger.info("Initializing 5x5x5 Cube and pre-downloading lookup tables...")
    try:
        cube5 = RubiksCube555(dummy_state_555, "URFDLB")
        cube5.lt_init()
        logger.info("[SUCCESS] 5x5x5 lookup tables pre-downloaded successfully.")
    except Exception as e:
        logger.error(f"Failed to download 5x5x5 lookup tables: {e}")
        sys.exit(1)

    logger.info("All Big Cube lookup tables successfully fetched and cached!")

if __name__ == "__main__":
    main()
