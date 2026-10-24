# Competitive intelligence in roadmap and upgrade work

**Policy date:** 2026-10-19

Competitive findings are an input to product decisions, not a one-off report.
Every pending roadmap item, new feature, and routine dependency upgrade should
check the repository's [`AUDIT_FINDINGS.md`](AUDIT_FINDINGS.md) before work
starts, and record the decision in the PR or changelog.

## Required check before building

1. **Read `AUDIT_FINDINGS.md`.** Find the relevant competitor row and the
   ranked next actions. If the finding is stale, update it with a dated source.
2. **Name the competitor and the user problem.** A feature is not justified by
   "competitor has it". State the workflow it unlocks, who needs it, and how
   often. Mark unverified demand as a hypothesis.
3. **Prefer reuse over new code.** Check whether an existing Mullassery repo
   already provides the capability (see `BUNDLES.md` in the profile repo and
   the bundle role file in this repo, when present). Typical reuse:
   - crawling/fetching/robots/SSRF → **MudFish**
   - architecture maps / PR diffs → **CodeAtlas**
   - data-quality contracts / validation → **StatGuardian**
   - token counting / cost tables → **PyTokenCalc**
   - dependency graph / advisories / SBOM → **PyDeepMesh**
   - segmentation / CLV / churn → **ClusterAudienceKit**
   - activation / lineage / PII → **PyReverseETL**
   - fleet state sync → **SyncFleet** (only after conflict resolution)
4. **Choose one disposition:** build, integrate, defer, or reject. Write it
   down. A rejection with a reason is a successful outcome.
5. **For routine upgrades**, check whether the upgrade closes a known
   competitor gap or breaks a differentiated capability. Run the tests that
   cover the affected path; do not merge a green build that silently changes a
   user-visible contract.

## PR / changelog checklist

- [ ] Competitor and source URL/date recorded (or "no direct competitor").
- [ ] User impact and frequency stated; unverified demand labelled.
- [ ] Reuse check done against sibling repos/bundles.
- [ ] Failure mode and rollback considered (especially auth, PII, cost).
- [ ] Docs/roadmap status updated honestly.
- [ ] Tests or a manual verification command included.

## Review cadence

- Refresh the competitor row when a release changes the capability.
- Re-check the full `AUDIT_FINDINGS.md` at least quarterly and before a major
  release.
- If a finding is wrong or stale, correct it in the same PR that relies on it.

## Non-goals

- Copying features for parity.
- Adding a dependency just because a competitor uses it.
- Treating star counts or download counts as proof of user need.

## OSS gap analysis — 2026-10-24

**What matters most here:** turnkey images and Linux/CI reach. Manual kernel/rootfs placement and macOS-only hosts cap adoption; finish the image pipeline and Intel artifact before GUI or Docker parity, which are already correctly deferred.

Scope: TinyBridge 0.8.0 compared with four actively maintained OSS Linux VM managers on Apple hosts, covering image acquisition, host integration, lifecycle features, and distribution. TinyBridge claims come from its README/roadmap; competitor claims come from their repository documentation.

### Comparable OSS projects

| Project | GitHub | License | Last activity | Why it is the comparison |
| --- | --- | --- | --- | --- |
| Lima | https://github.com/lima-vm/lima | Apache-2.0 | pushed 2026-10-24 (~22.2k stars) | Closest technical peer: macOS Linux VMs with Virtualization.framework, cloud-init, mounts, port forwarding |
| Colima | https://github.com/abiosoft/colima | MIT | pushed 2026-10-13 (~31.4k stars) | Minimal-setup container runtime on macOS/Linux built on Lima |
| UTM | https://github.com/utmapp/UTM | Apache-2.0 | pushed 2026-10-20 (~36.5k stars) | Full GUI VM host using QEMU and Apple Virtualization.framework |
| Multipass | https://github.com/canonical/multipass | GPL-3.0 | pushed 2026-10-24 (~9.3k stars) | Cross-platform Ubuntu VM manager with cloud-init and one-command launch |

### Capability comparison

| Capability | Ours | Lima | Colima | UTM | Multipass | Gap |
| --- | --- | --- | --- | --- | --- | --- |
| Linux VM on Apple Silicon | Yes | Yes | Yes | Yes | Yes | None |
| Apple Virtualization.framework backend | Yes | Yes | Partial | Yes | Yes | Colima inherits Lima's backend |
| Cross-platform host | No | Partial | Partial | Partial | Yes | Real gap |
| Turnkey OS images | No | Yes | Yes | Yes | Yes | Real gap |
| One-command launch | Partial | Yes | Yes | Yes | Yes | Ours needs guest assets placed first |
| Automated image build pipeline | No | Yes | Partial | Yes | Yes | Real gap |
| Cloud-init provisioning | Yes | Yes | Partial | Partial | Yes | Ours is comparable |
| Host directory sharing | Yes | Yes | Yes | Yes | Yes | Ours is VirtioFS with path allowlisting |
| First-class port forwarding | Partial | Yes | Yes | Yes | Partial | Ours is an SSH `-L` convenience |
| Snapshots | Yes | Unverified | No | Yes | Partial | Ours has a snapshot command |
| Suspend and resume | Yes | Yes | Partial | Yes | Yes | None |
| GUI | No | No | No | Yes | No | Deliberately deferred |
| Container/Docker workflow | No | Yes | Yes | Partial | No | Deliberately out of scope |
| Prebuilt signed distribution | Partial | Yes | Yes | Yes | Yes | Ours is Homebrew plus source build |
| Published boot/performance benchmarks | No | Partial | Partial | Partial | Partial | Real evidence gap |

Nuance: "Partial" for Lima snapshots means snapshot support was not verified in this pass, not that it is absent. Colima is a container-runtime manager rather than a general VM manager, so GUI and snapshot rows are not its product goal. Multipass is Ubuntu-centric by design; it is a fair comparison for the provisioning and cross-platform workflows TinyBridge lacks, not for arbitrary guest images. UTM is the only GUI in this set and the only one targeting iOS/visionOS.

### Gaps that matter (ranked)

1. **No turnkey images or automated asset pipeline.** TinyBridge requires the operator to place `kernel`, `initrd`, `disk.raw`, and an optional seed under the cache directory, and the README says there is no automated image pipeline. Lima, UTM, and Multipass each offer one-command launch from a catalog. The existing Lima-compatible catalog pull is a good start, but it does not close the end-to-end "boot a VM from nothing" gap. This is the single largest adoption blocker. Effort L, partly because macOS cannot create ext4 without Linux tooling.
2. **macOS-only host.** Linux and Windows adapters are unimplemented scaffolding. Lima and Colima run on macOS and Linux; Multipass covers Linux, Windows, and macOS. For teams that want the same VM workflow on CI or a Linux workstation, TinyBridge has no answer. Effort L and explicitly deferred.
3. **No first-class networking or port forwarding.** TinyBridge's `forward` is an SSH local forward, and host-to-guest SSH depends on the macOS Local Network privacy grant on the verified machine. Lima and Colima provide automatic port forwarding and file sharing as core behaviour. This makes service-oriented development (web server in guest, browser on host) less reliable than the alternatives. Effort M.
4. **Distribution is source-first.** There is no crates.io publication, guest assets are manual, and the Homebrew formula can skew from the workspace. UTM ships an app, Multipass ships installers, and Lima/Colima ship easy package-manager installs. Effort M.
5. **No Intel artifact and no published boot benchmarks.** The Intel x86_64 build/artifact is listed as blocked, while UTM and Multipass support Intel Macs. There are also no boot-to-shell benchmarks or independent security review, so the efficiency and hardening claims are unquantified. Effort M for the artifact, M for benchmarks.
6. **No GUI, by design.** UTM owns the GUI workflow. TinyBridge's README explicitly defers GUI and Docker Desktop parity to Lima/OrbStack. This is a documented scope decision rather than an accidental omission, and TinyBridge-UI is the companion answer.

### Where we are differentiated

- **Minimal, auditable Apple stack:** a real Rust to C ABI to Swift Virtualization.framework call chain, with no paid Apple Developer account and no embedded browser or Electron.
- **Process and privilege separation:** each VM runs in its own codesigned `tinybridge-vmhost`, supervised by `tinybridged` over a `0600` Unix-socket JSON-RPC control plane.
- **Security-minded host sharing:** VirtioFS host paths are canonicalized, allowlisted, reject `..` escapes, and default to read-only.
- **Operational honesty:** fail-fast asset validation, orphan detection, hypervisor checks in `doctor`, and a roadmap that distinguishes built, verified, broken, and deferred.

### Disposition

- **Build:** an automated image acquisition/build pipeline with checksum verification (P1, L); Intel artifact (P1, M); first-class TCP port forwarding and more reliable guest networking (P1, M); crates.io/packaging and release automation (P2, M).
- **Integrate/reuse:** keep the Lima-compatible image catalog format and cloud-init templates; reuse Lima's catalog metadata and Ubuntu cloud images rather than inventing another image format; keep Virtualization.framework as the only backend until a Linux host backend can be developed and verified on real hardware.
- **Defer/reject:** GUI, Docker/container workflow, and Windows/Linux backends now. The README already rejects GUI and Docker parity; those decisions are sound given Lima/OrbStack/Colima and TinyBridge-UI.

### Sources

- https://github.com/lima-vm/lima (Apache-2.0, pushed 2026-10-24, accessed 2026-10-24)
- https://github.com/abiosoft/colima (MIT, pushed 2026-10-13, accessed 2026-10-24)
- https://github.com/utmapp/UTM (Apache-2.0, pushed 2026-10-20, accessed 2026-10-24)
- https://github.com/canonical/multipass (GPL-3.0, pushed 2026-10-24, accessed 2026-10-24)
- Repo files inspected: `README.md`, `COMPETITIVE_INTELLIGENCE.md`, `ROADMAP_HONEST.md`, `QUEUE_STATUS.md`, `docs/ASSETS.md`, `docs/BUILD_ASSETS_GUIDE.md`
