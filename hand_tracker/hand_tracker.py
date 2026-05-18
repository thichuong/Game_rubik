import sys
import struct
import cv2

# Import MediaPipe and print friendly error if missing
try:
    import mediapipe as mp
except ImportError as e:
    sys.stderr.write(f"Error: Failed to import mediapipe: {e}\n")
    sys.stderr.flush()
    sys.exit(1)

def main():
    # Initialize MediaPipe Hands
    mp_hands = mp.solutions.hands
    mp_drawing = mp.solutions.drawing_utils

    hands = mp_hands.Hands(
        static_image_mode=False,
        max_num_hands=1,
        min_detection_confidence=0.5,
        min_tracking_confidence=0.5
    )

    # Open standard webcam index 0
    cap = cv2.VideoCapture(0)
    if not cap.isOpened():
        sys.stderr.write("Error: Could not open webcam\n")
        sys.stderr.flush()
        sys.exit(1)

    # Warm up camera
    cap.read()

    # Get binary stdout stream
    stdout = sys.stdout.buffer

    try:
        while True:
            ret, frame = cap.read()
            if not ret or frame is None:
                break

            # Mirror the frame horizontally (webcam mirror effect)
            frame = cv2.flip(frame, 1)
            h, w, c = frame.shape

            # Convert BGR to RGB for MediaPipe processing
            frame_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            results = hands.process(frame_rgb)

            has_center = 0
            cx = 0.0
            cy = 0.0

            if results.multi_hand_landmarks:
                has_center = 1
                landmarks = results.multi_hand_landmarks[0].landmark
                
                # Calculate stable palm center: average of WRIST (0), INDEX_MCP (5), MIDDLE_MCP (9), PINKY_MCP (17)
                cx = (landmarks[0].x + landmarks[5].x + landmarks[9].x + landmarks[17].x) / 4.0 * w
                cy = (landmarks[0].y + landmarks[5].y + landmarks[9].y + landmarks[17].y) / 4.0 * h

                # Draw landmarks and hand connections
                mp_drawing.draw_landmarks(
                    frame,
                    results.multi_hand_landmarks[0],
                    mp_hands.HAND_CONNECTIONS,
                    mp_drawing.DrawingSpec(color=(76, 22, 121), thickness=2, circle_radius=4), # Dark Purple landmarks
                    mp_drawing.DrawingSpec(color=(250, 44, 250), thickness=2, circle_radius=2) # Bright Magenta connections
                )

            # Convert to RGBA for direct Bevy UI rendering
            frame_rgba = cv2.cvtColor(frame, cv2.COLOR_BGR2RGBA)
            frame_bytes = frame_rgba.tobytes()
            frame_len = len(frame_bytes)

            # Pack metadata:
            # Header: 4 bytes b'HAND'
            # W: u32, H: u32, Has_center: u8, Cx: f32, Cy: f32, Frame_len: u32
            # Format: '<4sIIBffI' -> 25 bytes
            header = b'HAND'
            metadata = struct.pack('<4sIIBffI', header, w, h, has_center, cx, cy, frame_len)

            try:
                stdout.write(metadata)
                stdout.write(frame_bytes)
                stdout.flush()
            except BrokenPipeError:
                # Rust application closed the pipe or exited
                break

    finally:
        cap.release()
        hands.close()

if __name__ == '__main__':
    main()
