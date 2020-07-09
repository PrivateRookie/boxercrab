#!/bin/bash

source tests/scripts/lib.sh

mysql_exec "echo reset master"
docker container restart mysql_db_1
sleep 5
mysql_exec "echo flush logs"
dump_binlog ${PREFIX}/03_stop/dump.txt
cp_binlog ${PREFIX}/03_stop/log.bin

