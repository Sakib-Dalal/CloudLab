# Analytics and GPU support

Compute nodes and Workspaces have visual dashboards with a resource selector, 5-minute/15-minute/1-hour time windows, a pause-display button, and an optional GPU section. The chart legend toggles individual series; the keyboard-accessible time slider inspects historical values. Every measurement comes from an agent, except the explicitly labeled `?demo=1` preview.

## Measurements

- Node CPU is a percentage of total host CPU capacity. Workspace CPU is a percentage of its allocated CPU budget: Docker's 200% on two allocated cores is displayed as 100%.
- Memory charts compare measured usage against host capacity or the workspace limit. Docker container memory excludes cache according to Docker stats semantics. It can differ from host totals.
- Network charts show received plus transmitted bytes per second, calculated from consecutive counters. Node totals include the interfaces reported by the host, including virtual interfaces; they are not billed external traffic. Isolated containers normally have little or no network I/O.
- Workspace disk I/O measures block reads and writes. It does not measure free disk space or persistent-volume size. Process counts are available in workspace telemetry.
- GPU utilization, memory, temperature and power are optional sensors. Missing measurements display `—`; detected hardware does not imply every sensor is supported. Apple GPU memory is shared system memory and is not presented as dedicated VRAM.
- GPU charts are device-wide, including host applications. A workspace's GPU readings refer to its assigned devices, not usage attributed exclusively to its container.

The dashboard requests updates every five seconds. Hardware and Docker sampling run separately from agent heartbeats and lifecycle jobs; slow or missing sensor tools do not block work. A sample older than 30 seconds is stale; nodes become offline after 45 seconds without a heartbeat. Charts break across gaps and counter resets rather than inventing readings. Keep agent and coordinator clocks synchronized.

The coordinator keeps up to 360 samples per resource at a minimum ten-second interval, covering at most one hour. History is in memory and resets when the coordinator restarts. The latest measurement and GPU inventory persist with node metadata but are subject to the same freshness checks. This is a live dashboard, not long-term telemetry storage or a Grafana/Prometheus server.

Opened consoles, JupyterLab and VS Code show CPU/RAM/status metrics and expandable history without interrupting the terminal or editor. The workspace gateway exposes only that workspace's metrics and its assigned GPUs; lab credentials and other workspaces' telemetry are excluded.

## GPU capability

| Host/device | Monitoring | Workspace access |
| --- | --- | --- |
| NVIDIA | `nvidia-smi`: utilization, VRAM, temperature and power when available | Local Linux Docker with NVIDIA Container Toolkit; supported NVIDIA WSL2 environments |
| Apple Silicon/macOS | IOAccelerator statistics when exposed; display inventory fallback | Monitoring only; these Docker Desktop Linux workspaces cannot use Metal |
| Intel/AMD on macOS | IOAccelerator or display inventory; driver-dependent sensors | Monitoring only |
| AMD on Linux | DRM inventory and exported GPU busy/VRAM/temperature/power sensors | Selected `/dev/dri/renderD*`, plus `/dev/kfd` when present |
| Intel on Linux | DRM inventory; any exported compatible sensors (utilization often unavailable) | Selected `/dev/dri/renderD*` |
| Other Linux GPUs | DRM inventory and available compatible sensors | Monitoring only |
| Windows | NVIDIA sensors through `nvidia-smi`; other adapters through Windows display inventory | Run the agent inside a supported Linux/WSL2 GPU environment |

Install the hardware's driver and configure Docker first. NVIDIA requires a configured GPU runtime; AMD/Intel require accessible render devices. Remote Docker contexts are excluded from GPU allocation, because local hardware identifiers cannot safely describe a remote engine. The node must run beside the Docker engine (or in supported WSL2).

Choose a compatible node in **New workspace**, then select one or more available GPUs. Selection is optional and off by default. GPUs are reserved until they are unassigned or the workspace is removed, including while it is stopped. For an existing workspace, stop it and choose **Manage → Edit workspace** to change its GPU allocation. Reservations do not prevent host applications from using the same GPU and do not enforce a GPU memory quota.

Workspace images still need compatible CUDA, ROCm, oneAPI, or graphics libraries for the intended application. The fixed CloudLab images are general-purpose and do not automatically install ML frameworks or GPU runtimes. Device access alone does not turn an existing Python package into a GPU-enabled build. CloudLab does not install drivers or change host security settings automatically.

References: [Docker GPU access](https://docs.docker.com/engine/containers/gpu/), [Docker Desktop GPU support](https://docs.docker.com/desktop/features/gpu/), [NVIDIA Container Toolkit](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html), [AMD GPU sysfs sensors](https://docs.kernel.org/gpu/amdgpu/thermal.html).


## GPU not available in a notebook or editor

The GPU note in workspace controls, the console, and browser workspace windows distinguishes detected hardware from assigned compute access. An Apple M-series GPU is **monitoring only** with the Docker backend; installing PyTorch, CUDA, or a VS Code extension cannot expose Metal to a Linux container. Use a supported Linux/WSL2 GPU node for these workspaces. Native macOS GPU execution would require a separate native backend.

For supported nodes, stop the workspace, assign the GPU in Edit workspace, resume, and install a compatible framework through **Python packages**. The uv environment is `/home/lab/.venv`; Jupyter’s CloudLab kernel uses it automatically, and VS Code’s Python extension should select that interpreter. GPU utilization charts alone do not mean a workspace has GPU access.
