#!/bin/sh

cgroup_root=/app/cgroup

for controller in cpu,cpuacct memory; do
  mount -t cgroup -o $controller,rw,nosuid,nodev,noexec,relatime cgroup $cgroup_root/$controller
  chmod u+w $cgroup_root/$controller
  mkdir -p $cgroup_root/$controller/NSJAIL
  chown nsjail:nsjail $cgroup_root/$controller/NSJAIL
done

mount -t tmpfs tmpfs /tmp

exec setpriv --init-groups --reuid nsjail --regid nsjail \
  --inh-caps=-chown,-setuid,-setgid,-sys_admin,-setpcap --bounding-set=-chown,-setuid,-setgid,-sys_admin,-setpcap \
  /app/nsjail -C /app/nsjail.cfg
