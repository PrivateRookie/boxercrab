#!/bin/bash
set -e

source tests/scripts/lib.sh

if test -z "$1";
then
    echo "target dir is required!"
    exit 1
else
    mysql_exec "echo reset master"
    mysql_exec "cat ${PREFIX}/${1}/sql.sql"
    mysql_exec "echo flush logs"
    dump_binlog ${PREFIX}/${1}/dump.txt
    cp_binlog ${PREFIX}/${1}/log.bin
fi