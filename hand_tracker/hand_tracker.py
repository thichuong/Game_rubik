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

def detect_gesture(landmarks, w, h):
    """
    Detect gesture using ultra-fast, direct Y-coordinate comparisons:
    - Gesture 1 (Open Hand): index, middle, ring, and pinky are all extended. Used to rotate the entire cube.
    - Gesture 2 (Index Extended, others closed): index is extended, middle, ring, pinky are closed. Used to hover select a cubie face.
    - Gesture 3 (Index Folded, others closed): index is folded, middle, ring, pinky are closed. Used to swipe drag.
    - Gesture 0 (Idle): fallback/idle, avoids overlapping gesture confusion.
    """
    # Direct Y comparison (extremely fast, zero allocation, zero math overhead)
    index_extended = landmarks[8].y < landmarks[6].y
    middle_extended = landmarks[12].y < landmarks[10].y
    ring_extended = landmarks[16].y < landmarks[14].y
    pinky_extended = landmarks[20].y < landmarks[18].y

    # Check if other fingers (excluding index) are completely closed/folded
    other_closed = (not middle_extended) and (not ring_extended) and (not pinky_extended)

    # 1. Gesture 1: Open Hand / Whole Hand (All 4 fingers are fully extended) -> Rotate entire Rubik's cube
    if index_extended and middle_extended and ring_extended and pinky_extended:
        # Open Hand - Palm center
        cx = (landmarks[0].x + landmarks[5].x + landmarks[9].x + landmarks[17].x) * 0.25 * w
        cy = (landmarks[0].y + landmarks[5].y + landmarks[9].y + landmarks[17].y) * 0.25 * h
        return 1, cx, cy

    # 2. Single finger controls for face rotation (Only index finger active, others must be closed)
    elif other_closed:
        if index_extended:
            # Gesture 2: Index Extended, others closed -> Hover select face
            return 2, landmarks[8].x * w, landmarks[8].y * h
        else:
            # Gesture 3: Index Folded, others closed -> Swipe drag rotation
            return 3, landmarks[8].x * w, landmarks[8].y * h

    # 3. Fallback / Idle (Gesture 0) - Disables overlapping "half-open, half-folded" confusions
    else:
        return 0, landmarks[0].x * w, landmarks[0].y * h

def main():
    # Initialize MediaPipe Hands with max_num_hands=2
    mp_hands = mp.solutions.hands
    mp_drawing = mp.solutions.drawing_utils

    hands = mp_hands.Hands(
        static_image_mode=False,
        max_num_hands=2,
        min_detection_confidence=0.5,
        min_tracking_confidence=0.5
    )

    # Open standard webcam index 0
    cap = cv2.VideoCapture(0)
    if not cap.isOpened():
        sys.stderr.write("Error: Could not open webcam\n")
        sys.stderr.flush()
        sys.exit(1)

    # Set camera resolution to an optimized, standard speed (640x480)
    cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
    cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)

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

            detected_hands = []

            if results.multi_hand_landmarks and results.multi_handedness:
                for idx, hand_landmarks in enumerate(results.multi_hand_landmarks):
                    # Get handedness: 'Left' or 'Right'
                    handedness_str = results.multi_handedness[idx].classification[0].label
                    # Map to integer: 0 = Left hand, 1 = Right hand
                    handedness_val = 0 if handedness_str == 'Left' else 1

                    # Detect gesture and cursor
                    gesture_type, cx, cy = detect_gesture(hand_landmarks.landmark, w, h)

                    # Flat landmarks array
                    flat_lms = []
                    for lm in hand_landmarks.landmark:
                        flat_lms.extend([lm.x, lm.y, lm.z])

                    detected_hands.append({
                        'handedness': handedness_val,
                        'gesture_type': gesture_type,
                        'cursor_x': cx,
                        'cursor_y': cy,
                        'landmarks': flat_lms
                    })

                    # Draw landmarks on frame: Pink/Purple for Left Hand, Cyan/Green for Right Hand
                    if handedness_val == 0:
                        joint_color = (76, 22, 121)    # Dark Purple
                        conn_color = (250, 44, 250)    # Bright Magenta
                    else:
                        joint_color = (121, 76, 22)    # Deep Cyan/Teal
                        conn_color = (250, 250, 44)    # Cyan/Yellow-Green

                    mp_drawing.draw_landmarks(
                        frame,
                        hand_landmarks,
                        mp_hands.HAND_CONNECTIONS,
                        mp_drawing.DrawingSpec(color=joint_color, thickness=2, circle_radius=4),
                        mp_drawing.DrawingSpec(color=conn_color, thickness=2, circle_radius=2)
                    )

            # Resize the preview frame to 320x240 to reduce IPC bandwidth and rendering load
            # This cuts down data size by 4x, drastically increasing performance
            preview_w, preview_h = 320, 240
            preview_frame = cv2.resize(frame, (preview_w, preview_h))

            # Convert to RGBA for direct Bevy UI rendering
            frame_rgba = cv2.cvtColor(preview_frame, cv2.COLOR_BGR2RGBA)
            frame_bytes = frame_rgba.tobytes()
            frame_len = len(frame_bytes)

            # Pack metadata:
            # 1. Global Header: b'HAND' (4 bytes) + W: u32 + H: u32 + Frame_len: u32 + detected_hands_count: u8 + reserved: 4s -> 21 bytes
            # Format: '<4sIIIB4s'
            global_header = b'HAND'
            hands_count = len(detected_hands)
            metadata_global = struct.pack('<4sIIIB4s', global_header, preview_w, preview_h, frame_len, hands_count, b'\x00' * 4)

            try:
                stdout.write(metadata_global)

                # 2. Hand Data Block: 268 bytes per hand
                # Format: '<BBff63f6s' -> handedness (u8), gesture_type (u8), cursor_x (f32), cursor_y (f32), landmarks (63f), reserved (6s)
                for hand in detected_hands:
                    # Map the cursor coordinates to the preview frame size for Rust consistency
                    mapped_cursor_x = (hand['cursor_x'] / w) * preview_w
                    mapped_cursor_y = (hand['cursor_y'] / h) * preview_h

                    hand_block = struct.pack(
                        '<BBff63f6s',
                        hand['handedness'],
                        hand['gesture_type'],
                        mapped_cursor_x,
                        mapped_cursor_y,
                        *hand['landmarks'],
                        b'\x00' * 6
                    )
                    stdout.write(hand_block)

                # 3. Write camera frame bytes
                stdout.write(frame_bytes)
                stdout.flush()
            except BrokenPipeError:
                # Rust application closed the pipe or exited
                break
    except (KeyboardInterrupt, SystemExit):
        pass
    except BrokenPipeError:
        pass
    finally:
        cap.release()
        hands.close()

if __name__ == '__main__':
    try:
        main()
    except (KeyboardInterrupt, SystemExit):
        pass

