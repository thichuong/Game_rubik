---
trigger: always_on
glob: "**/*.rs"
description: Quy tắc và lưu ý quan trọng khi làm việc với giao diện người dùng (UI) trong Bevy 0.18+.
---

# Bevy UI Guidelines (Quy tắc thiết kế UI trong Bevy 0.18+)

## 1. Hitbox & Interaction Blocking (Nhận diện tương tác và Chặn sự kiện)
- **Vấn đề "Xuyên thấu" (Transparent Hitboxes):** 
  - Trong Bevy, một Node có màu nền hoàn toàn trong suốt (`BackgroundColor(Color::NONE)`) có thể bị hệ thống Picking tối ưu bỏ qua và không nhận diện click chuột.
  - *Giải pháp:* Khi cần tạo một hitbox vô hình, hãy sử dụng màu nền gần như trong suốt thay vì NONE, ví dụ: `BackgroundColor(Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.01)))`.
- **Z-Index & Event Consuming (Lớp hiển thị và chặn sự kiện):**
  - Các phần tử con (children) hoặc phần tử sinh ra sau sẽ nằm đè lên các phần tử khác.
  - Nếu một phần tử con đè lên hitbox mà không có `Interaction` (không có Component `Button`), nó sẽ nuốt/chặn (consume) toàn bộ sự kiện click, làm cho phần tử bên dưới bị "liệt".
  - *Giải pháp:* Tách cấu trúc UI sao cho các phần tử hiển thị (Visuals) nằm bên dưới lớp Hitbox. Spawn lớp Hitbox **sau cùng** (sử dụng PositionType::Absolute với kích thước 100%) và có `Interaction` để nó hứng toàn bộ các sự kiện thay cho các con.

## 2. Hệ Tọa Độ & Transform của UI (UI Coordinate System)
- **Sự chia tách Transform trong Bevy 0.18+:**
  - **CỰC KỲ QUAN TRỌNG:** Ở các phiên bản mới của Bevy (như 0.18+ / bevy_ui 0.18+), hệ thống Transform của UI đã được tách hoàn toàn khỏi 3D Transform.
  - Các phần tử UI **KHÔNG CÒN** có `GlobalTransform` hay `Transform`. Thay vào đó, chúng sử dụng `UiGlobalTransform` và `UiTransform`.
  - Nếu bạn thực hiện Query chứa `&GlobalTransform` trên một entity UI, kết quả trả về sẽ luôn là `None`.
  - *Cách dùng đúng:* Import và sử dụng `bevy::ui::UiGlobalTransform`.
  - Khi cần lấy tọa độ X, Y của UI, hãy truy cập trực tiếp vào property `translation` của `UiGlobalTransform` (lưu ý: `translation` là một field, không phải method - ví dụ: `transform.translation.x`).

## 3. Kích Thước Node (Node Sizing)
- **Node vs ComputedNode:**
  - Để lấy kích thước thực tế sau khi layout (render) của một phần tử UI, bạn cần query `&ComputedNode`.
  - Method `.size()` trên `ComputedNode` (hoặc `Node` nếu áp dụng) trả về một `Vec2` chứa thông số width và height thực tế hiển thị trên màn hình.
