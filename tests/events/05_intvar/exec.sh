#!/bin/bash

source tests/scripts/lib.sh

target_dir=${PREFIX}/05_intvar

mysql_exec "echo reset master" 2> /dev/null
mysql_exec "cat ${target_dir}/sql.sql"
dump_binlog ${target_dir}/dump.txt
cp_binlog ${target_dir}/log.bin
