#!/bin/bash
set -e

export PREFIX="$(git rev-parse --show-toplevel)/tests/events"

function mysql_exec() {
    $1 | mysql -u root -p1234TttT -h 127.0.0.1 default
}

function dump_binlog() {
    docker container exec -it mysql_db_1 mysqlbinlog -H /var/lib/mysql/mysql_bin.000001 > $1
}

function cp_binlog () {
    docker container cp mysql_db_1:/var/lib/mysql/mysql_bin.000001 $1
}
