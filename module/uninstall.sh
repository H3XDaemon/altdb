#!/system/bin/sh
MODDIR=${0%/*}
"$MODDIR/bin/altdb" ctl stop >/dev/null 2>&1
# The directory is fixed, module-owned state. Never follow a substituted symlink.
if [ -d /data/adb/altdb ] && [ ! -L /data/adb/altdb ]; then
  rm -rf /data/adb/altdb
fi
