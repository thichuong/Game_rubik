#!/usr/bin/env python3
import sys
import socket
import logging
from math import sqrt

# Add the solver path to sys.path to ensure absolute imports work
import os
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__) + "/rubiks-cube-NxNxN-solver"))

from rubikscubennnsolver import SolveError, configure_logging

# Configure logging
configure_logging()
logger = logging.getLogger("nxn_daemon")
logger.setLevel(logging.INFO)

# Import solver classes upfront to warm up Python interpreter
logger.info("Warming up NxN solver classes...")
try:
    from rubikscubennnsolver.RubiksCube222 import RubiksCube222
    from rubikscubennnsolver.RubiksCube333 import RubiksCube333
    from rubikscubennnsolver.RubiksCube444 import RubiksCube444
    from rubikscubennnsolver.RubiksCube555 import RubiksCube555
    from rubikscubennnsolver.RubiksCube666 import RubiksCube666
    from rubikscubennnsolver.RubiksCube777 import RubiksCube777
    from rubikscubennnsolver.RubiksCubeNNNEven import RubiksCubeNNNEven
    from rubikscubennnsolver.RubiksCubeNNNOdd import RubiksCubeNNNOdd
    logger.info("Warm-up completed successfully.")
except Exception as e:
    logger.error(f"Error during solver classes warm-up: {e}")

def solve_cube(state: str, order: str = "URFDLB") -> str:
    """Solve the rubik's cube for a given state string and return space-separated moves."""
    if "G" in state:
        state = state.replace("G", "F")
        state = state.replace("Y", "D")
        state = state.replace("O", "L")
        state = state.replace("W", "U")

    size = int(sqrt((len(state) / 6)))
    logger.info(f"Solving Rubik's Cube of size {size}x{size}x{size}...")

    if size == 2:
        cube = RubiksCube222(state, order)
    elif size == 3:
        cube = RubiksCube333(state, order)
    elif size == 4:
        cube = RubiksCube444(state, order)
    elif size == 5:
        cube = RubiksCube555(state, order)
    elif size == 6:
        cube = RubiksCube666(state, order)
    elif size == 7:
        cube = RubiksCube777(state, order)
    elif size % 2 == 0:
        cube = RubiksCubeNNNEven(state, order)
    else:
        cube = RubiksCubeNNNOdd(state, order)

    cube.sanity_check()
    cube.solve()

    if not cube.solved():
        raise SolveError("Cube should be solved but is not.")

    # Return space-separated moves, filtering out comments
    solution_minus_comments = [step for step in cube.solution if not step.startswith("COMMENT")]
    return " ".join(solution_minus_comments)

def main():
    # Record parent PID to detect if Rust parent exits
    parent_pid = os.getppid()

    # Set parent death signal on Linux to ensure immediate termination if parent dies
    if sys.platform.startswith("linux"):
        try:
            import ctypes
            import signal
            libc = ctypes.CDLL(None)
            # PR_SET_PDEATHSIG = 1; send SIGTERM to itself when parent dies
            libc.prctl(1, signal.SIGTERM)
            logger.info("Successfully set parent death signal (PR_SET_PDEATHSIG) to SIGTERM.")
        except Exception as e:
            logger.warning(f"Could not set parent death signal: {e}")
    
    port = 10023
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            logger.error("Invalid port specified. Using default 10023.")

    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    
    try:
        server_socket.bind(("127.0.0.1", port))
    except Exception as e:
        logger.error(f"Failed to bind to port {port}: {e}")
        sys.exit(1)

    server_socket.listen(5)
    server_socket.settimeout(5.0) # 5 seconds timeout to allow periodic parent check
    logger.info(f"NxN Solver Daemon started on 127.0.0.1:{port}")
    # Print a readiness marker so Rust can wait for it to be ready
    print(f"READY:{port}", flush=True)

    while True:
        try:
            try:
                client_socket, addr = server_socket.accept()
            except socket.timeout:
                # Periodic parent check: if parent PID has changed (reparented) or no longer exists
                if os.getppid() != parent_pid:
                    logger.info("Parent process died or changed. Exiting.")
                    break
                try:
                    os.kill(parent_pid, 0)
                except OSError:
                    logger.info("Parent process died (failed kill check). Exiting.")
                    break
                continue

            logger.info(f"Accepted connection from {addr}")
            
            data = b""
            while b"\n" not in data:
                chunk = client_socket.recv(4096)
                if not chunk:
                    break
                data += chunk
            
            if not data:
                client_socket.close()
                continue
                
            request_str = data.decode("utf-8").strip()
            if not request_str:
                client_socket.close()
                continue

            if request_str == "SHUTDOWN":
                logger.info("Received SHUTDOWN command. Exiting.")
                client_socket.sendall(b"OK\n")
                client_socket.close()
                break

            # Format: <state> or <order>:<state>
            if ":" in request_str:
                parts = request_str.split(":", 1)
                order = parts[0]
                state = parts[1]
            else:
                order = "URFDLB"
                state = request_str

            try:
                solution = solve_cube(state, order)
                response = f"SUCCESS:{solution}\n".encode("utf-8")
            except Exception as ex:
                logger.error(f"Solve error: {ex}")
                response = f"ERROR:{str(ex)}\n".encode("utf-8")

            client_socket.sendall(response)
            client_socket.close()
        except KeyboardInterrupt:
            logger.info("Daemon interrupted by user. Exiting.")
            break
        except Exception as e:
            logger.error(f"Unexpected error in daemon loop: {e}")

    server_socket.close()

if __name__ == "__main__":
    main()
