#!/bin/bash

mkdir /sys/fs/cgroup/{memory,cpu}/NSJAIL
chown nsjail /sys/fs/cgroup/{memory,cpu}/NSJAIL

exec setpriv --init-groups --reuid nsjail --regid nsjail --ambient-caps=-all --inh-caps=-all --bounding-set=-all /app/nsjail -C /app/nsjail.cfg
