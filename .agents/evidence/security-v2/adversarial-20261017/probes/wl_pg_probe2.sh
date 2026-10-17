#!/bin/sh
# Runs INSIDE the workload with brokered-role env only. Direct connection tests (fixed from wl_pg_probe.sh).
echo "peer_as_admin_os | rc=$("$PG_BIN/psql" -X -q -At -h "$PG_SOCK" -U sea_admin -d advtest -c 'SELECT 1' >./x1 2>&1; echo $?) | $(head -c 100 ./x1 | tr '\n' ' ')"
echo "guess_other_role | rc=$("$PG_BIN/psql" -X -q -At -h "$PG_SOCK" -U sfw_100002_000000 -d advtest -c 'SELECT 1' >./x2 2>&1; echo $?) | $(head -c 100 ./x2 | tr '\n' ' ')"
echo "tcp_connect | rc=$("$PG_BIN/psql" -X -q -At -h 127.0.0.1 -p 5432 -d advtest -c 'SELECT 1' >./x3 2>&1; echo $?) | $(head -c 100 ./x3 | tr '\n' ' ')"
echo "other_sessions_query_text | $("$PG_BIN/psql" -X -q -At -c "SELECT count(*) FILTER (WHERE query <> '<insufficient privilege>' AND usename <> current_user) || ' visible-other-query-texts of ' || count(*) FILTER (WHERE usename <> current_user) FROM pg_stat_activity" 2>&1 | head -c 120)"
n=0; for i in 1 2 3 4 5 6; do "$PG_BIN/psql" -X -q -At -c "SELECT pg_sleep(3)" >/dev/null 2>&1 && echo ok >> ./cl_ok & done; sleep 2
n=$(psql_count=$("$PG_BIN/psql" -X -q -At -c "SELECT count(*) FROM pg_stat_activity WHERE usename = current_user" 2>&1); echo $psql_count)
echo "conn_limit | own-role-sessions-while-6-spawned=$n (limit 4)"; wait
rm -f ./cl_ok ./x1 ./x2 ./x3
