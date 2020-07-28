#!/bin/bash
set -e

source tests/scripts/lib.sh

if test -z "$1";
then
    echo "target dir is required!"
    exit 1
else
    target_dir=${PREFIX}/${1}
    if [ -f ${target_dir}/exec.sh ]
    then
        echo ==========================================
        echo
        echo "running ${target_dir}/exec.sh"
        echo
        echo ==========================================
        source ${target_dir}/exec.sh
    else
        echo ==========================================
        echo
        echo "running ${target_dir}/sql.sql"
        echo
        echo ==========================================
        mysql_exec "echo reset master" 2> /dev/null
        mysql_exec "cat ${target_dir}/sql.sql"
        mysql_exec "echo flush logs" 2> /dev/null
        dump_binlog ${target_dir}/dump.txt
        cp_binlog ${target_dir}/log.bin
    fi
fi