#!/bin/bash
set -euo pipefail
cd "$( dirname "${BASH_SOURCE[0]}" )"

cbindgen --lang c --output ../Quake/librust.h
