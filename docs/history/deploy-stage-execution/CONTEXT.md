# Deploy-stage execution

## Objective

Make an approved deployment dispatch executable before release `2.37.3`.

The repair must preserve the deploy-role authorization contract. It must not weaken ordinary gather isolation.

## Observed failure

`bee dispatch prepare --runtime pi --kind gather --role deploy --stage deployment --release-version 2.37.3` accepted the role plan.

Its payload then produced two incompatible values:

- The worker brief prohibited all mutation.
- The herding command used the closed feature worktree as its current working directory.

The release script requires an authorized mutating worker on the main branch. The prepared worker could not satisfy that requirement.

## Locked behavior

Decision `38edea83` fixes the contract:

1. A deployment-stage dispatch gives the deploy worker an explicit execution brief.
2. The brief names the authorized version and the exact `scripts/release.sh` command.
3. The herding command runs from the main control root.
4. Ordinary gather dispatch remains read-only.
5. Ordinary feature work uses its granted worktree.
6. Existing authorization fields and one-use checks remain unchanged.

## Proof

Behavior tests must first reproduce both wrong payload values.

Pre-merge proof must show:

- deployment stdin permits mutation and names `scripts/release.sh 2.37.3`;
- deployment command cwd is the main root;
- ordinary gather stdin remains read-only;
- ordinary feature dispatch command cwd remains its worktree;
- the related Rust tests pass;
- one installed prepared payload executes through `agy-flash` against a harmless release-script stub and records cwd, argv, and authorization environment without publication.

Post-merge proof must run the real prepared payload from main. It must publish `2.37.3`, wait for green CI, and verify release assets.

## Boundaries

The worker cannot select a version. The leader supplies the version through `--release-version`.

The repair does not add a dispatch kind or a second role resolver.

Independent review remains user-invoked.
