#!/usr/bin/env bash

# This is a convenience script for setting up the example CouchDB cluster
# It expects the three nodes to be available on ports 5985, 5986 and 5987
# exposed via Kubernetes port-forwarding

COUCHDB_USER=enseada
COUCHDB_PASSWORD=enseada
curl -X PUT "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5985/_users"
curl -X PUT "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5985/_replicator"
curl -X PUT "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5985/_global_changes"

curl -X PUT "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5986/_users"
curl -X PUT "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5986/_replicator"
curl -X PUT "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5986/_global_changes"

curl -X PUT "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5987/_users"
curl -X PUT "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5987/_replicator"
curl -X PUT "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5987/_global_changes"

curl -X POST -H "Content-Type: application/json" "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5985/_cluster_setup" -d '{"action": "enable_cluster", "bind_address":"0.0.0.0", "username": "enseada", "password": "enseada", "node_count":"3"}'

curl -X POST -H "Content-Type: application/json" "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5985/_cluster_setup" -d '{"action": "enable_cluster", "bind_address":"0.0.0.0", "username": "enseada", "password": "enseada", "port": 5984, "node_count": "3", "remote_node": "couchdb-1.couchdb-headless.enseada.svc.cluster.local", "remote_current_user": "admin", "remote_current_password": "password" }'
curl -X POST -H "Content-Type: application/json" "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5985/_cluster_setup" -d '{"action": "add_node", "host":"couchdb-1.couchdb-headless.enseada.svc.cluster.local", "port": 5984, "username": "enseada", "password": "enseada"}'

curl -X POST -H "Content-Type: application/json" "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5985/_cluster_setup" -d '{"action": "enable_cluster", "bind_address":"0.0.0.0", "username": "enseada", "password": "enseada", "port": 5984, "node_count": "3", "remote_node": "couchdb-2.couchdb-headless.enseada.svc.cluster.local", "remote_current_user": "admin", "remote_current_password": "password" }'
curl -X POST -H "Content-Type: application/json" "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5985/_cluster_setup" -d '{"action": "add_node", "host":"couchdb-2.couchdb-headless.enseada.svc.cluster.local", "port": 5984, "username": "enseada", "password": "enseada"}'

curl -X POST -H "Content-Type: application/json" "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5985/_cluster_setup" -d '{"action": "finish_cluster"}'

curl "http://$COUCHDB_USER:$COUCHDB_PASSWORD@127.0.0.1:5985/_membership"
