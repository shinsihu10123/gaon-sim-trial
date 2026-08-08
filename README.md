# Gaon Multi-Nation Simulation — TRIAL ARCHIVE

> **시험판 저장소 / NOT PRODUCTION**
>
> 이 저장소는 경제 동아리 「가온」의 AI 기반 다국가 통합 발전 시뮬레이션 **시험판 개발 기록을 보존하기 위한 저장소**입니다.
> 본판 개발과 혼동하지 않도록 2026-08-09 기준 시험판 개발선을 동결·분리했습니다.
> **본판 코드는 이 저장소에서 계속 개발하지 않습니다.**

## 보존 기준

| 구분 | 위치 | 상태 |
|---|---|---|
| 시험판 최종 안정본 | `archive/trial-stable-stage-3.1` | Stage 3.1까지 병합·검증된 기준선 |
| 시험판 마지막 실험본 | `archive/trial-stage-3.2-wip` | Stage 3.2 구현본, Core CI 실패 상태를 그대로 보존 |
| 기존 통합 개발선 | `develop` | 시험판 개발 이력 보존용 |
| 기본 진입 브랜치 | `main` | 저장소 안내용 |

## 시험판에서 검증된 핵심 기반

- Rust 기반 결정론적 고정 Tick 시뮬레이션 코어
- Save / Load / Replay 및 상태 해시
- 동적 Region 구조와 지형·자원 생성
- Stable Entity ID 기반 동적 Entity Architecture
- 초기 HumanGroup 생성 및 생태적 배치
- Three.js 기반 3D Viewer, 카메라·선택 시스템
- HumanGroup authoritative runtime state
- Save Format v7 / RenderSnapshot v7까지의 상태 연결

## 마지막 개발 상태

### Stage 3.1 — 완료

HumanGroup의 인구·영양·응집도·위험도·계보 상태를 authoritative runtime state로 승격하고 Save/Load/Replay 및 Viewer 경계까지 연결한 상태가 시험판의 **최종 안정 기준선**입니다.

### Stage 3.2 — 미완료 보존

다음 인구·생존 기능은 구현되었지만 Core CI 실패를 해결하지 않은 채 시험판 종료 시점의 WIP로 보존합니다.

- 일일 출생 처리
- 기본 사망
- 영양실조·기아 사망
- 지형 수용력 기반 인구압
- 수용력 초과 시 출생 억제
- Region/World 인구 집계
- 결정론적 사건 표본화

이 코드는 `archive/trial-stage-3.2-wip` 브랜치에서 확인할 수 있으며 **본판의 검증 완료 코드로 간주하지 않습니다.**

## 기존 디렉터리 구조

```text
crates/
  simulation-model/
  simulation-core/
  simulation-save/
  simulation-runner/
  simulation-protocol/
  simulation-wasm/
  simulation-worldgen/
viewer/
shared/contracts/
docs/
scripts/
.github/workflows/
```

## 본판 개발 원칙

본판은 별도 Production 개발선/저장소에서 진행합니다. 시험판 코드는 그대로 삭제하지 않고 참고·검증 자료로 보존하며, 본판으로 가져갈 때는 각 모듈을 `유지 / 수정 / 재설계 / 폐기`로 다시 판정합니다.

시험판의 과거 Feature Branch와 PR도 개발 이력으로 유지합니다.
