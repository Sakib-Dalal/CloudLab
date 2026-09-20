#!/bin/sh
set -eu
export VIRTUAL_ENV=/home/lab/.venv
export PATH="$VIRTUAL_ENV/bin:$PATH"
if [ ! -x "$VIRTUAL_ENV/bin/python" ]; then
  uv venv --offline --system-site-packages --python "$CLOUDLAB_BASE_PYTHON" "$VIRTUAL_ENV"
fi
# Register the persistent interpreter for notebooks in every template.
"$VIRTUAL_ENV/bin/python" -m ipykernel install --user --name python3 --display-name 'Python (CloudLab · uv)' >/dev/null
if [ "${CLOUDLAB_TEMPLATE:-}" = code ]; then
  exec /usr/bin/code-server "$@"
fi
exec "$@"
