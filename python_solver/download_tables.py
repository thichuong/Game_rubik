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
    from rubikscubennnsolver.RubiksCube444 import RubiksCube444, solved_444
    from rubikscubennnsolver.RubiksCube555 import RubiksCube555, solved_555
    from rubikscubennnsolver.RubiksCube666 import RubiksCube666, solved_666
    from rubikscubennnsolver.RubiksCube777 import RubiksCube777, solved_777
except ImportError as e:
    logger.error(f"Failed to import rubikscubennnsolver modules: {e}")
    sys.exit(1)

def main():
    logger.info("Initializing pre-download sequence for NxN lookup tables...")

    # Change working directory to resolve LOOKUP_TABLES relative path to the correct folder
    target_cwd = os.path.join(PROJECT_ROOT, "python_solver/rubiks-cube-NxNxN-solver")
    os.chdir(target_cwd)
    logger.info(f"Changed working directory to: {target_cwd}")

    # 1. Warm-up and pre-download 4x4x4 lookup tables
    logger.info("Initializing 4x4x4 Cube and pre-downloading lookup tables...")
    try:
        cube4 = RubiksCube444(solved_444, "URFDLB")
        cube4.lt_init()
        logger.info("[SUCCESS] 4x4x4 lookup tables pre-downloaded successfully.")
    except Exception as e:
        logger.error(f"Failed to download 4x4x4 lookup tables: {e}")
        sys.exit(1)

    # 2. Warm-up and pre-download 5x5x5 lookup tables
    logger.info("Initializing 5x5x5 Cube and pre-downloading lookup tables...")
    try:
        cube5 = RubiksCube555(solved_555, "URFDLB")
        cube5.lt_init()
        logger.info("[SUCCESS] 5x5x5 lookup tables pre-downloaded successfully.")
    except Exception as e:
        logger.error(f"Failed to download 5x5x5 lookup tables: {e}")
        sys.exit(1)

    # 3. Warm-up and pre-download 6x6x6 lookup tables
    logger.info("Initializing 6x6x6 Cube and pre-downloading lookup tables...")
    try:
        cube6 = RubiksCube666(solved_666, "URFDLB")
        cube6.lt_init()
        logger.info("[SUCCESS] 6x6x6 lookup tables pre-downloaded successfully.")
    except Exception as e:
        logger.error(f"Failed to download 6x6x6 lookup tables: {e}")
        sys.exit(1)

    # 4. Warm-up and pre-download 7x7x7 lookup tables
    logger.info("Initializing 7x7x7 Cube and pre-downloading lookup tables...")
    try:
        cube7 = RubiksCube777(solved_777, "URFDLB")
        cube7.lt_init()
        logger.info("[SUCCESS] 7x7x7 lookup tables pre-downloaded successfully.")
    except Exception as e:
        logger.error(f"Failed to download 7x7x7 lookup tables: {e}")
        sys.exit(1)

    logger.info("All Big Cube lookup tables successfully fetched and cached!")

if __name__ == "__main__":
    main()
