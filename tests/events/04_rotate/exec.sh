#!/bin/bash

source tests/scripts/lib.sh

mysql_exec "echo reset master"
mysql_exec "echo flush logs"
dump_binlog ${PREFIX}/04_rotate/dump.txt
cp_binlog ${PREFIX}/04_rotate/log.bin

