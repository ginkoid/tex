#!/bin/bash

mkdir -p /sys/fs/cgroup/{memory,cpu}/NSJAIL
chown nsjail /sys/fs/cgroup/{memory,cpu}/NSJAIL

exec setpriv --init-groups --reuid nsjail --regid nsjail --inh-caps=-all --bounding-set=-all /app/nsjail -C /app/nsjail.cfg
