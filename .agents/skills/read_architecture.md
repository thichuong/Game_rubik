---
trigger: always_on
glob: "**/*.rs"
description: Yêu cầu bắt buộc đọc architecture.md để nắm vững cấu trúc hệ thống và thiết kế ECS trước khi chỉnh sửa hoặc phát triển mã nguồn.
---

# Project Architecture Guideline (Hướng dẫn Kiến trúc Dự án)

## Yêu cầu Bắt buộc
Mỗi khi bắt đầu một tác vụ liên quan đến refactor, thêm tính năng mới, hoặc thay đổi bất kỳ module nào trong dự án, agent **bắt buộc** phải đọc file `architecture.md` ở thư mục gốc của dự án.

## Lý do
- Dự án game Rubik này được thiết kế theo mô hình Entity Component System (ECS) chặt chẽ bằng Bevy Engine (v0.18).
- Các module như UI (`src/ui`), Input (`src/input`), Solver (`src/solver` & `rubik_solver`), Rubik Core (`src/rubik`), v.v., được tách biệt hoàn toàn để đảm bảo tính modular và dễ bảo trì.
- Việc đọc `architecture.md` giúp agent hiểu rõ:
  - Cấu trúc luồng tương tác giữa các hệ thống (System Architecture).
  - Các Component, Resource quan trọng và vị trí định nghĩa của chúng.
  - Các toán tử toán học và chuyển đổi được sử dụng cho Cube.

## Hành động Cần thực hiện
1. Trước khi viết code hoặc đề xuất giải pháp kiến trúc, hãy mở và đọc tệp `architecture.md` (sử dụng công cụ `view_file` hoặc tương đương).
2. Đối chiếu thiết kế hiện tại của bạn với sơ đồ kiến trúc để đảm bảo tính nhất quán, không phá vỡ tính modular và decoupled của hệ thống.
