# AerogelOS

**현대적인 오픈소스 운영체제 - Rust + C# 기반**

## 🎯 비전

확장자 기반 파일 관리, YAML 설정, C# 앱 개발이 가능한 사용자 친화적 OS

## 🚀 현재 상태

**Phase 1 - 기초 구현 중** (v0.1.0)

✅ UEFI/BIOS 부트로더  
✅ Rust 커널 (인터럽트, GDT)  
✅ 키보드 입력 (폴링 모드)  
✅ VGA 텍스트 출력  

## 🛠 빌드 방법

### 필요 도구
```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add x86_64-unknown-none
cargo install bootimage

# 기타
sudo apt install nasm qemu-system-x86 build-essential
```

### 빌드 & 실행
```bash
make run
```

## 📁 프로젝트 구조

```
AerogelOS/
├── bootloader/       # Assembly 부트로더
├── kernel/           # Rust 커널
├── drivers/          # 하드웨어 드라이버
│   └── vga/         # VGA 드라이버
├── apps/            # 사용자 앱 (향후)
└── tools/           # 빌드 도구
```

## 🗺 로드맵

- [x] **Phase 1**: 부트로더 + 기본 커널 (진행 중)
- [ ] **Phase 2**: 파일 시스템 (YAML, JSON, VFS)
- [ ] **Phase 3**: 그래픽 드라이버 & GUI
- [ ] **Phase 4**: .NET 런타임
- [ ] **Phase 5**: 앱 마켓플레이스

자세한 내용: [설계 문서](myos_design_doc.md)

## 💡 핵심 특징 (계획)

- **확장자 기반 파일 관리**: 직관적인 타입 인식
- **YAML 설정**: 사람이 읽기 쉬운 설정 파일
- **C# 앱 개발**: .NET 기반 현대적 개발 환경
- **가상 파일 시스템**: 실시간 하드웨어 정보 (`/hardware/cpu`)
- **마켓플레이스**: 15% 수수료 (Apple 30%보다 저렴)

## 📝 라이선스

MIT License

## 🤝 기여하기

이슈, PR 환영합니다!