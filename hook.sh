#!/bin/sh

cat << EOF >> "${nsjail_cfg}"
mount {
  dst: "/tmp"
  fstype: "tmpfs"
  rw: true
  options: "size=20m"
  nodev: true
  noexec: true
}
EOF
