# CloudLab 1.x archive

The Go CLI, Python server, dashboard, and original installers are preserved here
only as migration reference. They are **not** used by CloudLab 2.0. The original
tools could execute commands on the host and publicly expose services. Do not
run them as part of the new container-isolated lab.

CloudLab 2.0 does not automatically import 1.x credentials or host environments.
Stop the old services and tunnels before starting the new coordinator. Copy
project files into a workspace explicitly after reviewing what should be shared.
