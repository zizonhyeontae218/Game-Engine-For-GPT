# First-run task packets (proposals, pending actual repo discovery)

Use these as work-order starting points, **not** permission to modify unknown files.

## Packet P0-S — Survey (scout, read-only)

Goal: establish the verified GE4G 0.2 → 0.3 migration baseline. Discover root build manifests, source directories, engine entrypoints, scenes/entities, event loop, renderer/input, plugin hooks, test setup and current limitations. Output verified paths and three strongest boundary risks. **Do not edit.**

## Packet P0-A — Core boundary (architect, docs-only)

Prerequisite: P0-S. Output a proposal mapping existing structures to candidate plugin registration/lifecycle, view-independent entity/event/resource/input contracts, serialization format compatibility, error handling and version policy. Mark unimplemented areas. **No invented API signatures presented as existing.**

## Packet P0-R — Independent review (auditor, read-only)

Prerequisite: P0-A. Try to refute the proposed dependency directions. Identify unsafe generic registries, hidden renderer imports, unbounded singleton state and API breaking changes. Require concrete tests for plugin unload and view replacement. No code editing.

## Packet P1-C — First implementation vertical slice (core)

Prerequisite: approved P0-A/P0-R; source tree known. Implement the smallest demonstrable plugin lifecycle slice, with one toy plugin and one unload/isolation test. **No game genre or camera dependency**. No additional mechanic until G1/G2 evidence exists. Owned paths must be assigned from the actual repo by the orchestrator.

## Packet P2-V — View format trial (view, after P1-C)

A single removable view adapter against the approved contract, preferably the least invasive of Top-down or Classic 2D according to actual repo capabilities. Confirm coordinate, layers, camera and plugin independence before extending to remaining views. Use public API and document install/uninstall. Mixed 2D + 3D support is a required eventual view capability, not an excuse to overbuild Core.
