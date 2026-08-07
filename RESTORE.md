# gaon-sim-trial 복구 지침

이 저장소는 경제 동아리 「가온」의 AI 기반 다국가 통합 발전 시뮬레이션 시험판 개발용 저장소입니다.

## v1.2 전체 Git 저장소 백업

`backups/gaon-sim-trial-v1.2.bundle.b64`에는 시험판 v1.2 시점의 Git bundle 전체가 Base64 형식으로 보관되어 있습니다. 소스 파일뿐 아니라 당시의 Git 커밋과 브랜치 이력을 함께 복구할 수 있습니다.

원본 bundle SHA-256:

`2eb951e31f4989bb9b0984ebc64e16da165ba0f340ea2454f8cf29e9b239ffcd`

## 복구 절차

```bash
base64 --decode backups/gaon-sim-trial-v1.2.bundle.b64 > gaon-sim-trial-v1.2.bundle
sha256sum gaon-sim-trial-v1.2.bundle
git clone gaon-sim-trial-v1.2.bundle gaon-sim-trial-restored
cd gaon-sim-trial-restored
git branch -a
git log --oneline --decorate --all
```

macOS에서 `base64 --decode`가 지원되지 않는 경우 다음을 사용할 수 있습니다.

```bash
base64 -D backups/gaon-sim-trial-v1.2.bundle.b64 > gaon-sim-trial-v1.2.bundle
```

## 브랜치 원칙

- `main`: 검증된 기준 상태
- `develop`: 통합 개발 브랜치
- `feature/<name>`: 개별 기능 개발
- `fix/<name>`: 오류 수정

기능은 구현·검증 후 `develop`에 통합하고, 단계 기준을 통과한 상태만 `main` 기준점으로 올립니다.

## 중요

현재 bundle은 v1.2까지의 로컬 개발 이력을 보존하기 위한 안전 백업입니다. 이후 개발은 GitHub 저장소 자체를 원격 기준점으로 사용하고, 주요 단계 완료 시 별도 태그 또는 bundle 백업을 추가합니다.
