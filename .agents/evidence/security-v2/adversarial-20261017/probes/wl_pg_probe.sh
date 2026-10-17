#!/bin/sh
# Runs INSIDE the isolated workload with only the brokered session conninfo in env.
# Each line: label | rc | first 100 chars of output/error. No secrets are printed.
PSQL="$PG_BIN/psql"
q() { label="$1"; shift; out=$("$PSQL" -X -q -At -c "$1" 2>&1); rc=$?; echo "$label | rc=$rc | $(echo "$out" | tr '\n' ' ' | cut -c1-100)"; }
q whoami "SELECT current_user, session_user"
q read_granted_items "SELECT count(*) FROM public.items"
q read_admin_secret_table "SELECT v FROM public.admin_secret"
q insert_items "INSERT INTO public.items VALUES (9,'planted')"
q create_table_public "CREATE TABLE public.exfil_t(x text)"
q create_temp_table "CREATE TEMP TABLE tt(x text); INSERT INTO tt VALUES ('a'); SELECT count(*) FROM tt"
q copy_to_file "COPY public.items TO '/tmp/adv_copy_out'"
q copy_to_program "COPY (SELECT 1) TO PROGRAM 'id > /tmp/adv_prog_out'"
q pg_read_file_gitconfig "SELECT length(pg_read_file('/home/sprime01/.gitconfig'))"
q lo_import_hostname "SELECT lo_import('/etc/hostname')"
q set_role_admin "SET ROLE sea_admin"
q alter_self_superuser "ALTER ROLE current_user SUPERUSER"
q create_extension_dblink "CREATE EXTENSION dblink"
q list_roles "SELECT count(*) FROM pg_roles"
q list_other_sessions "SELECT count(*) FROM pg_stat_activity"
q listen_notify "LISTEN adv_chan; NOTIFY adv_chan, 'x'; SELECT 1"
q peer_as_admin_os "$PG_BIN/psql -X -q -At -h $PG_SOCK -U sea_admin -d advtest -c 'SELECT 1'"
q guess_other_role "$PG_BIN/psql -X -q -At -h $PG_SOCK -U sfw_100002_000000 -d advtest -c 'SELECT 1'"
q tcp_connect "$PG_BIN/psql -X -q -At -h 127.0.0.1 -p 5432 -U $PGUSER -d advtest -c 'SELECT 1'"
i=0; while [ $i -lt 6 ]; do "$PSQL" -X -q -At -c "SELECT pg_sleep(3)" >/dev/null 2>&1 & i=$((i+1)); done; sleep 1
echo "conn_limit_probe: started 6 concurrent sessions (limit 4)"; wait
